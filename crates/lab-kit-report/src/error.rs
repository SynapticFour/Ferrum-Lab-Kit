use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReportError {
    #[error("IO: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("PDF: {0}")]
    Pdf(String),
    #[error("license: {0}")]
    License(String),
    #[error("conformance report is empty: {0}")]
    EmptyConformance(String),
}
