// config.rs — Application configuration loaded from environment variables.
//
// We use a simple struct + `from_env()` pattern rather than a config crate,
// keeping dependencies minimal. Every field maps to an environment variable
// documented in `.env.example`.

use std::env;

/// All configuration the app needs at runtime.
/// Clone-able so we can share it via Axum's State extractor.
#[derive(Clone, Debug)]
pub struct AppConfig {
    // ── Azure Table Storage ──────────────────────────────────────────
    /// The name of the Azure Storage account (e.g. "myscoreboard").
    pub azure_storage_account: String,
    /// Access key for the storage account. Used to authenticate table operations.
    pub azure_storage_access_key: String,

    // ── Azure AD / Entra ID (OIDC) ──────────────────────────────────
    /// The Azure AD tenant ID (a GUID).
    pub azure_tenant_id: String,
    /// The OAuth2 client ID from the Azure app registration.
    pub azure_client_id: String,
    /// The OAuth2 client secret from the Azure app registration.
    pub azure_client_secret: String,

    // ── App settings ─────────────────────────────────────────────────
    /// The public-facing URL of this application (used for OIDC redirect URIs).
    pub app_url: String,
    /// Secret used to sign session JWT cookies.
    pub session_secret: String,
    /// Port to listen on. Defaults to 3000.
    pub port: u16,
}

impl AppConfig {
    /// Load configuration from environment variables.
    ///
    /// Panics with a clear message if a required variable is missing — this is
    /// intentional because the app can't function without these values, and we
    /// want to fail fast at startup rather than later at runtime.
    pub fn from_env() -> Self {
        Self {
            azure_storage_account: required("AZURE_STORAGE_ACCOUNT"),
            azure_storage_access_key: required("AZURE_STORAGE_ACCESS_KEY"),
            azure_tenant_id: required("AZURE_TENANT_ID"),
            azure_client_id: required("AZURE_CLIENT_ID"),
            azure_client_secret: required("AZURE_CLIENT_SECRET"),
            app_url: required("APP_URL"),
            session_secret: required("SESSION_SECRET"),
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .expect("PORT must be a valid u16"),
        }
    }
}

/// Helper: read a required env var or panic with a helpful message.
fn required(name: &str) -> String {
    env::var(name).unwrap_or_else(|_| {
        panic!(
            "Missing required environment variable: {name}. See .env.example for documentation."
        )
    })
}
