use std::fmt::Debug;
use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    #[error("{0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("{0}")]
    QueryPreparationError(String),
}

pub type Result<T> = std::result::Result<T, Error>;
