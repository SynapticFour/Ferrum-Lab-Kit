use thiserror::Error;

#[derive(Debug, Error)]
pub enum AuthError {
    #[error("HTTP: {0}")]
    Http(#[from] reqwest::Error),
    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("JWT: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),
    #[error("OIDC: {0}")]
    Oidc(String),
    #[error("configuration: {0}")]
    Config(String),
}
