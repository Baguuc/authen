/// Enum modelling every error that can happen during email client construction.
#[derive(Debug, thiserror::Error)]
pub enum EmailClientConstructionError {
    #[error("Invalid method: {0}")]
    InvalidMethod(String),
    #[error("Invalid header: {0}")]
    InvalidHeaderName(String),
    #[error("Invalid header: {0}")]
    InvalidHeaderValue(String),
}