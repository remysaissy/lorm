//! Basic CRUD operations example
//!
//! This example demonstrates:
//! - Creating records
//! - Reading records by primary key
//! - Updating records
//! - Deleting records

use anyhow::Result;
use lorm::ToLOrm;
use lorm::predicates::Where;
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

#[derive(Debug, Default, Clone, FromRow, ToLOrm)]
struct User {
    #[lorm(pk)]
    #[lorm(new = "Uuid::new_v4()")]
    pub id: Uuid,

    #[lorm(by)]
    pub email: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Create an in-memory database
    let pool = SqlitePool::connect("sqlite::memory:").await?;

    // Create schema
    sqlx::query(
        r#"
        CREATE TABLE users (
            id TEXT PRIMARY KEY NOT NULL,
            email TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    let mut user = User::default();
    user.email = "alice@example.com".to_string();
    user = user.save(&pool).await?;
    let id = user.id;
    println!("   Created user: {} {}\n", user.id, user.email);

    let s = User::select()
        .where_id(Where::Eq, id)
        .where_email(Where::Eq, user.email)
        .order_by_email()
        .asc()
        .build(&pool)
        .await?;

    let found_user = s.first().unwrap();
    println!("   Found user: {}\n", found_user.email);
    Ok(())
}
