//! Basic CRUD operations example
//!
//! This example demonstrates:
//! - Creating records
//! - Reading records by primary key
//! - Updating records
//! - Deleting records

use anyhow::Result;
use chrono::FixedOffset;
use lorm::ToLOrm;
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

#[derive(Debug, Default, Clone, FromRow, ToLOrm)]
struct User {
    #[lorm(pk)]
    #[lorm(new = "Uuid::new_v4()")]
    pub id: Uuid,

    #[lorm(by)]
    pub email: String,

    pub name: String,

    #[lorm(created_at)]
    #[lorm(new = "chrono::Utc::now().fixed_offset()")]
    pub created_at: chrono::DateTime<FixedOffset>,

    #[lorm(updated_at)]
    #[lorm(new = "chrono::Utc::now().fixed_offset()")]
    pub updated_at: chrono::DateTime<FixedOffset>,
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
            email TEXT NOT NULL,
            name TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    println!("=== Basic CRUD Example ===\n");

    // CREATE
    println!("1. Creating a new user...");
    let mut user = User::default();
    user.email = "alice@example.com".to_string();
    user.name = "Alice".to_string();
    user = user.save(&pool).await?;
    println!("   Created user: {} ({})\n", user.name, user.email);

    // READ
    println!("2. Reading user by ID...");
    let found_user = User::by_id(&pool, user.id).await?;
    println!(
        "   Found user: {} ({})\n",
        found_user.name, found_user.email
    );

    // UPDATE
    println!("3. Updating user...");
    user.name = "Alice Smith".to_string();
    user = user.save(&pool).await?;
    println!("   Updated name to: {}\n", user.name);

    // DELETE
    println!("4. Deleting user...");
    let user_id = user.id;
    user.delete(&pool).await?;
    println!("   User deleted\n");

    // Verify deletion
    println!("5. Verifying deletion...");
    match User::by_id(&pool, user_id).await {
        Ok(_) => println!("   ERROR: User still exists!"),
        Err(_) => println!("   Confirmed: User no longer exists"),
    }

    Ok(())
}
