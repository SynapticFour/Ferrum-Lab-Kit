use async_trait::async_trait;
use serde_json::Value;

use crate::provider::AuthProvider;
use crate::AuthError;

/// Placeholder Keycloak adapter — validate via Ferrum gateway or extend with realm JWKS.
pub struct KeycloakAuthAdapter;

#[async_trait]
impl AuthProvider for KeycloakAuthAdapter {
    fn name(&self) -> &'static str {
        "keycloak"
    }

    async fn validate_id_token(&self, _token: &str) -> Result<Value, AuthError> {
        Err(AuthError::Config(
            "Keycloak token validation not implemented in Lab Kit — use Ferrum IdP integration or extend lab-kit-auth::KeycloakAuthAdapter".into(),
        ))
    }
}
