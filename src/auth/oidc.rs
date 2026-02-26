// auth/oidc.rs — OpenID Connect discovery and token validation.
//
// This module handles the OIDC authorization code flow with Microsoft Entra ID:
//
//   1. User clicks "Login" → we redirect them to Microsoft's authorize endpoint.
//   2. User signs in with their org account.
//   3. Microsoft redirects back to our callback URL with an authorization code.
//   4. We exchange the code for tokens (ID token + access token).
//   5. We validate the ID token and extract the user's info.
//   6. We create a signed JWT session cookie so subsequent requests are authenticated.
//
// Why OIDC?
//   OIDC (OpenID Connect) is a standard identity protocol built on top of OAuth 2.0.
//   Microsoft Entra ID (formerly Azure AD) supports it, which means we don't need
//   Microsoft-specific code — just a standard OIDC client library.

use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

use crate::config::AppConfig;

/// Claims we store in our session JWT cookie.
///
/// This is NOT the Microsoft ID token — it's our own JWT that we create after
/// validating the Microsoft ID token. We store just the info we need so we
/// don't have to call Microsoft on every request.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionClaims {
    /// Subject: the user's unique ID from Microsoft (OID claim).
    pub sub: String,
    /// The user's display name.
    pub name: String,
    /// The user's email address.
    pub email: String,
    /// Expiration time (as Unix timestamp).
    pub exp: i64,
    /// Issued at (as Unix timestamp).
    pub iat: i64,
}

/// Create a signed session JWT for a user.
///
/// This JWT is stored as an HTTP-only cookie and validated on each API request
/// by the auth middleware (see middleware.rs).
pub fn create_session_token(
    config: &AppConfig,
    user_id: &str,
    name: &str,
    email: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    // Sessions last 24 hours. After that, the user must re-authenticate.
    let exp = now + Duration::hours(24);

    let claims = SessionClaims {
        sub: user_id.to_string(),
        name: name.to_string(),
        email: email.to_string(),
        exp: exp.timestamp(),
        iat: now.timestamp(),
    };

    // Sign the JWT with our session secret using HMAC-SHA256.
    let key = EncodingKey::from_secret(config.session_secret.as_bytes());
    encode(&Header::default(), &claims, &key)
}

/// Validate a session JWT and extract the claims.
///
/// Returns None if the token is invalid, expired, or tampered with.
pub fn validate_session_token(
    config: &AppConfig,
    token: &str,
) -> Option<SessionClaims> {
    let key = DecodingKey::from_secret(config.session_secret.as_bytes());
    let mut validation = Validation::default();
    // We don't set an issuer or audience because this is our own JWT,
    // not a Microsoft token.
    validation.validate_aud = false;

    decode::<SessionClaims>(token, &key, &validation)
        .ok()
        .map(|data| data.claims)
}

/// The OIDC discovery URL for Microsoft Entra ID.
///
/// This endpoint returns the OpenID Provider Configuration, which tells us
/// where to send the user to log in, where to exchange codes for tokens, etc.
pub fn discovery_url(tenant_id: &str) -> String {
    format!(
        "https://login.microsoftonline.com/{tenant_id}/v2.0/.well-known/openid-configuration"
    )
}

/// Build the authorization URL that redirects the user to Microsoft login.
///
/// Query parameters:
/// - client_id: our app registration's ID
/// - response_type: "code" for authorization code flow
/// - redirect_uri: where Microsoft sends the user back after login
/// - scope: what we're requesting access to (openid + profile + email)
/// - response_mode: "query" means the code comes as a URL query parameter
pub fn authorize_url(config: &AppConfig) -> String {
    let redirect_uri = format!("{}/api/auth/callback", config.app_url);
    format!(
        "https://login.microsoftonline.com/{}/oauth2/v2.0/authorize?\
         client_id={}&\
         response_type=code&\
         redirect_uri={}&\
         scope=openid%20profile%20email&\
         response_mode=query",
        config.azure_tenant_id,
        config.azure_client_id,
        urlencoding::encode(&redirect_uri),
    )
}

/// Exchange an authorization code for tokens by calling Microsoft's token endpoint.
///
/// This is the server-side part of the authorization code flow. The code was
/// received in the callback URL, and we exchange it for an ID token (which
/// contains the user's info) and an access token.
pub async fn exchange_code(
    config: &AppConfig,
    code: &str,
) -> Result<TokenResponse, Box<dyn std::error::Error>> {
    let redirect_uri = format!("{}/api/auth/callback", config.app_url);
    let token_url = format!(
        "https://login.microsoftonline.com/{}/oauth2/v2.0/token",
        config.azure_tenant_id,
    );

    // POST the code to Microsoft's token endpoint.
    let client = reqwest::Client::new();
    let response = client
        .post(&token_url)
        .form(&[
            ("client_id", config.azure_client_id.as_str()),
            ("client_secret", config.azure_client_secret.as_str()),
            ("code", code),
            ("redirect_uri", redirect_uri.as_str()),
            ("grant_type", "authorization_code"),
            ("scope", "openid profile email"),
        ])
        .send()
        .await?;

    if !response.status().is_success() {
        let body = response.text().await.unwrap_or_default();
        return Err(format!("Token exchange failed: {body}").into());
    }

    let token_response: TokenResponse = response.json().await?;
    Ok(token_response)
}

/// Response from Microsoft's token endpoint.
#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub id_token: Option<String>,
    pub access_token: Option<String>,
}

/// Decoded claims from a Microsoft ID token.
///
/// We only decode the payload (without full signature validation against JWKS)
/// because we trust the token came directly from Microsoft via HTTPS in the
/// server-side code exchange. For a public-facing API that accepts tokens from
/// clients directly, you'd want full JWKS validation.
#[derive(Debug, Deserialize)]
pub struct MicrosoftIdClaims {
    /// User's unique object ID in the tenant.
    pub oid: Option<String>,
    /// Subject claim (fallback if oid is missing).
    pub sub: Option<String>,
    /// Display name.
    pub name: Option<String>,
    /// Email address.
    #[serde(rename = "preferred_username")]
    pub preferred_username: Option<String>,
}

/// Extract user info from a Microsoft ID token (JWT).
///
/// This does a simple base64 decode of the JWT payload — we trust it because
/// we received it directly from Microsoft's token endpoint over HTTPS.
pub fn decode_id_token_claims(id_token: &str) -> Option<MicrosoftIdClaims> {
    // A JWT has three parts separated by dots: header.payload.signature
    let parts: Vec<&str> = id_token.split('.').collect();
    if parts.len() != 3 {
        return None;
    }

    // Decode the payload (second part) from base64.
    use base64::Engine;
    let payload_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(parts[1])
        .ok()?;

    serde_json::from_slice(&payload_bytes).ok()
}
