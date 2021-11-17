use thiserror::Error;

#[derive(Error, Debug)]
pub enum InternalError {
    #[error(transparent)]
    Error(#[from] crate::BoxErr),
    #[error("Feature `{0}` is not supported")]
    Unsupported(String),
    #[error("{0}")]
    Other(String),
}

#[derive(Error, Debug)]
pub enum TestError {
    #[error(transparent)]
    Error(#[from] crate::BoxErr),
    // #[error("Feature `{0}` is not supported")]
    // Assertion(String),
    // #[error("{0}")]
    // Other(String),
    /// Internal error
    #[error(transparent)]
    Internal(#[from] InternalError),
}
