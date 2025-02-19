use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

#[derive(Default, Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum OrderBy {
    #[default]
    Asc,
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

