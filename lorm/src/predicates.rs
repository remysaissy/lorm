use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};

/// Comparison operators for WHERE clauses in select queries.
///
/// Used with the generated `where_{field}()` methods to specify how to compare values.
///
/// # Example
///
/// ```ignore
/// use lorm::predicates::Where;
///
/// // Find users with id equal to 1
/// let users = User::select()
///     .where_id(Where::Eq, 1)
///     .build(&pool)
///     .await?;
///
/// // Find products with price greater than 100
/// let expensive = Product::select()
///     .where_price(Where::GreaterThan, 100)
///     .build(&pool)
///     .await?;
/// ```
#[derive(Default, Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum Where {
    /// Equals (`=`) comparison
    #[default]
    Eq,

    /// Not equals (`!=`) comparison
    NotEq,

    /// Greater than (`>`) comparison
    GreaterThan,

    /// Greater than or equal to (`>=`) comparison
    GreaterOrEqualTo,

    /// Less than (`<`) comparison
    LesserThan,

    /// Less than or equal to (`<=`) comparison
    LesserOrEqualTo,

    /// Like (`LIKE`) to search for a specified pattern
    Like,
}

impl Display for Where {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Where::Eq => write!(f, "="),
            Where::NotEq => write!(f, "!="),
            Where::GreaterThan => write!(f, ">"),
            Where::GreaterOrEqualTo => write!(f, ">="),
            Where::LesserThan => write!(f, "<"),
            Where::LesserOrEqualTo => write!(f, "<="),
            Where::Like => write!(f, "LIKE"),
        }
    }
}
