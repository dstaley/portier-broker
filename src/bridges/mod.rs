use crate::crypto;
use crate::error::BrokerError;
use crate::web::{return_to_relier, Context, HandlerResult};
use serde_derive::{Deserialize, Serialize};

/// Session data stored by bridges.
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum BridgeData {
    Email(email::EmailBridgeData),
    Oidc(oidc::OidcBridgeData),
}

/// Once a bridge has authenticated the user, this function can be used to finished up the redirect
/// to the relying party with a token generated by us.
pub async fn complete_auth(ctx: &mut Context) -> HandlerResult {
    let data = ctx
        .session_data
        .as_ref()
        .expect("complete_auth called without a session");
    ctx.app.store.remove_session(&ctx.session_id).await?;

    let aud = data
        .return_params
        .redirect_uri
        .origin()
        .ascii_serialization();
    let jwt = crypto::create_jwt(
        &ctx.app,
        &data.email,
        &data.email_addr,
        &aud,
        &data.nonce,
        data.signing_alg,
    )
    .map_err(|err| {
        // Currently, the only possible failure here is that we accepted a signing algorithm
        // from the RP that suddenly disappeared from our config. Treat as an internal error.
        BrokerError::Internal(format!("Could not create a JWT: {:?}", err))
    })?;

    Ok(return_to_relier(
        ctx,
        &[("id_token", &jwt), ("state", &data.return_params.state)],
    ))
}

pub mod email;
pub mod oidc;
