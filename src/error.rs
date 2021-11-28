use thiserror::Error;
use client::subxt::Error as SubxtError;

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum InternalError {
    #[error(transparent)]
    Error(#[from] crate::BoxErr),

    #[error(transparent)]
    IoError(#[from] std::io::Error),

    #[error(transparent)]
    ClientError(#[from] SubxtError),

    #[error(transparent)]
    ProcError(#[from] subprocess::PopenError),

    #[error("Feature `{0}` is not supported")]
    Unsupported(String),

    #[error("{0}")]
    Other(String),
}

#[non_exhaustive]
#[derive(Error, Debug)]
pub enum TestError {
    /// Success doesn't happen becouse time is out of specified limit.
    #[error("Conditions out of time")]
    Timeout(#[from] async_std::future::TimeoutError),

    /// Failure conditions are met.
    #[error("Feature: `{0}`")]
    Feature(String),

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
