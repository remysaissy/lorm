//! A zero cost and lightweight ORM operations for SQLx.
//!
//! Lorm generates type-safe database operations at compile time using derive macros.
//!
//! # Installation
//!
//! Add the following to your project's `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! lorm = { version = "0.1" }
//! sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite"] }
//! ```
//!
//! # Quick Example
//!
//! ```ignore
//! use lorm::ToLOrm;
//! use lorm::predicates::Where;
//! use sqlx::{FromRow, SqlitePool};
//!
//! #[derive(Debug, Default, Clone, FromRow, ToLOrm)]
//! struct User {
//!     #[lorm(pk)]
//!     #[lorm(readonly)]
//!     pub id: i32,
//!
//!     #[lorm(by)]
//!     pub email: String,
//! }
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let pool = SqlitePool::connect("sqlite::memory:").await?;
//!
//!     // Create a user
//!     let mut user = User::default();
//!     user.email = "alice@example.com".to_string();
//!     let user = user.save(&pool).await?;
//!
//!     // Find by field (generated from #[lorm(by)])
//!     let found = User::by_email(&pool, "alice@example.com").await?;
//!
//!     // Query with filtering and pagination
//!     let users = User::select()
//!         .where_email(Where::Eq, "alice@example.com")
//!         .order_by_email()
//!         .desc()
//!         .limit(10)
//!         .build(&pool)
//!         .await?;
//!
//!     // Delete the user
//!     user.delete(&pool).await?;
//!
//!     Ok(())
//! }
//! ```
//!
//! # Generated Methods
//!
//! For a struct with `#[derive(ToLOrm)]`, Lorm generates:
//!
//! - `save(&executor)` - Insert or update (upsert)
//! - `delete(&executor)` - Delete by primary key
//! - `by_{field}(&executor, value)` - Find one by field (for `#[lorm(by)]` fields)
//! - `with_{field}(&executor, value)` - Find all by field (for `#[lorm(by)]` fields)
//! - `select()` - Start a query builder
//!
//! # Query Builder
//!
//! The `select()` method returns a builder with these methods:
//!
//! - `where_{field}(Where::Eq, value)` - Filter by comparison
//! - `where_between_{field}(start, end)` - Filter by range
//! - `order_by_{field}()` - Add ordering (chain with `.asc()` or `.desc()`)
//! - `group_by_{field}()` - Group results
//! - `limit(n)` / `offset(n)` - Pagination
//! - `build(&executor)` - Execute and return results

pub mod errors;
pub mod predicates;

pub use lorm_macros::ToLOrm;
