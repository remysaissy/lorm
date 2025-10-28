use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// Specifies the sort order for query results.
#[derive(Default, Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum OrderBy {
    /// Sort in ascending order (A-Z, 0-9, oldest to newest).
    #[default]
    Asc,
    /// Sort in descending order (Z-A, 9-0, newest to oldest).
    Desc,
}

impl Display for OrderBy {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderBy::Asc => write!(f, "ASC"),
            OrderBy::Desc => write!(f, "DESC"),
        }
    }
}
