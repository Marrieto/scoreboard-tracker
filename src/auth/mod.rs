// auth/mod.rs — Authentication module.
//
// Handles OIDC login via Microsoft Entra ID (Azure AD) and session management
// using signed JWT cookies.

pub mod middleware;
pub mod oidc;
