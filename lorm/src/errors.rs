use std::fmt::Debug;
use thiserror::Error;

/// Error types that can occur when using Lorm.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// An error occurred in the underlying SQLx database layer.
    #[error("{0}")]
    DatabaseError(#[from] sqlx::Error),

    /// An error occurred while preparing a query.
    #[error("{0}")]
    QueryPreparationError(String),
}

/// A specialized `Result` type for Lorm operations.
pub type Result<T> = std::result::Result<T, Error>;
