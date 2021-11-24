use thiserror::Error;
use client::subxt::Error as SubxtError;

#[derive(Error, Debug)]
pub enum InternalError {
    #[error(transparent)]
    Error(#[from] crate::BoxErr),

    #[error(transparent)]
    ClientError(#[from] SubxtError),

    #[error("Feature `{0}` is not supported")]
    Unsupported(String),

    #[error("{0}")]
    Other(String),
}

#[derive(Error, Debug)]
pub enum TestError {
    #[error(transparent)]
    Error(#[from] crate::BoxErr),

    // #[error("`{0}`")]
    // Assertion(String),
    #[error("{0}")]
    Other(&'static str),

    /// Internal error
    #[error(transparent)]
    Internal(#[from] InternalError),
}


impl From<SubxtError> for TestError {
    fn from(err: SubxtError) -> Self { Self::Internal(err.into()) }
}
