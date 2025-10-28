//! A zero cost and lightweight ORM operations for SQLx.
//! # Use
//! adding the following to your project's Cargo.toml:
//! ```toml
//! [dependencies]
//! lorm = { version = "0" }
//! sqlx = { version = "0.8" }
//! ```
//!
//! # Examples
//! ```ignore
//! #[derive(Debug, Default, Clone, FromRow, ToLOrm)]
//! struct AltUser {
//!     #[lorm(pk)]
//!     #[lorm(readonly)]
//!     pub id: i32,
//!
//!     #[lorm(by)]
//!     pub email: String,
//!
//!     #[allow(unused)]
//!     #[lorm(created_at)]
//!     #[lorm(readonly)]
//!     pub created_at: chrono::DateTime<FixedOffset>,
//!
//!     #[lorm(updated_at)]
//!     #[lorm(new = "chrono::Utc::now().fixed_offset()")]
//!     pub updated_at: chrono::DateTime<FixedOffset>,
//! }
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not set");
//!     let pool = SqlitePool::connect(&database_url).await?;
//!
//!     // Create users
//!     for i in 0..10 {
//!         let mut u = AltUser::default();
//!         u.email = format!("user{}@example.com", i);
//!         u.save(&pool).await?;
//!     }
//!
//!     // Query with pagination
//!     let users = AltUser::select()
//!         .order_by_email(OrderBy::Desc)
//!         .limit(2)
//!         .offset(2)
//!         .build(&pool)
//!         .await?;
//!
//!     println!("Found {} users", users.len());
//!     Ok(())
//! }
//! ```

pub mod errors;
pub mod predicates;

pub use lorm_macros::ToLOrm;
