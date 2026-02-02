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

/// Comparison operators for HAVING clauses in select queries.
///
/// Used with the generated `having_{field}()` methods to specify how to compare values.
///
/// # Example
///
/// ```ignore
/// use lorm::predicates::Having;
///
/// // Find products with average price greater than 100
/// let expensive = Product::select()
///     .group_by_price()
///     .having_price(Having::GreaterThan, Function::Avg, 100)
///     .build(&pool)
///     .await?;
/// ```
#[derive(Default, Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum Having {
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

impl Display for Having {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Having::Eq => write!(f, "="),
            Having::NotEq => write!(f, "!="),
            Having::GreaterThan => write!(f, ">"),
            Having::GreaterOrEqualTo => write!(f, ">="),
            Having::LesserThan => write!(f, "<"),
            Having::LesserOrEqualTo => write!(f, "<="),
            Having::Like => write!(f, "LIKE"),
        }
    }
}

/// Function for queries.
#[derive(Default, Debug, Clone, PartialEq, Deserialize, Serialize)]
pub enum Function {
    /// No function applied
    #[default]
    Null,

    /// Number of rows (`COUNT`) function
    Count { is_distinct: bool },

    /// Total of values (`SUM`) function
    Sum,

    /// Average value (`AVG`) function
    Avg,

    /// Minimum value (`MIN`) function
    Min,

    /// Maximum value (`MAX`) function
    Max,
}

impl Display for Function {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Function::Null => write!(f, ""),
            Function::Count { .. } => write!(f, "COUNT"),
            Function::Sum => write!(f, "SUM"),
            Function::Avg => write!(f, "AVG"),
            Function::Min => write!(f, "MIN"),
            Function::Max => write!(f, "MAX"),
        }
    }
}
