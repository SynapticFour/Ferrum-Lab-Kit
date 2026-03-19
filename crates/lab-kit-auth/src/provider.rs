use async_trait::async_trait;
use serde_json::Value;

use crate::AuthError;

/// Pluggable authentication for gateways: LS Login, Keycloak, or local LDAP facade.
#[async_trait]
pub trait AuthProvider: Send + Sync {
    /// Human-readable provider id (`ls-login`, `keycloak`, ...).
    fn name(&self) -> &'static str;

    /// Validate an OIDC ID token and return decoded claims JSON.
    async fn validate_id_token(&self, token: &str) -> Result<Value, AuthError>;
}
