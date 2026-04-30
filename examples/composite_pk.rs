//! Composite primary key example
//!
//! This example demonstrates:
//! - Composite primary keys with `#[lorm(pk_type = "manual")]`
//! - Upsert behavior: save() performs INSERT ... ON CONFLICT DO UPDATE for manual pk
//! - Full-key model (Tag): all columns are pk → save() uses DO NOTHING semantics

use anyhow::Result;
use lorm::ToLOrm;
use sqlx::{FromRow, SqlitePool};

#[derive(Debug, Clone, FromRow, ToLOrm)]
#[lorm(pk_type = "manual")]
struct UserRole {
    #[lorm(pk)]
    pub user_id: String,
    #[lorm(pk)]
    pub role_id: String,
    pub assigned_at: String,
}

#[derive(Debug, Clone, FromRow, ToLOrm)]
#[lorm(pk_type = "manual")]
struct Tag {
    #[lorm(pk)]
    pub name: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;

    // Create tables
    sqlx::query(
        "CREATE TABLE user_roles (
            user_id TEXT NOT NULL,
            role_id TEXT NOT NULL,
            assigned_at TEXT NOT NULL,
            PRIMARY KEY (user_id, role_id)
        )",
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        "CREATE TABLE tags (
            name TEXT NOT NULL PRIMARY KEY
        )",
    )
    .execute(&pool)
    .await?;

    println!("=== Composite Primary Key Example ===\n");

    // 1. Create a user role
    println!("1. Saving a user role (INSERT)...");
    let ur = UserRole {
        user_id: "alice".to_string(),
        role_id: "admin".to_string(),
        assigned_at: "2024-01-01".to_string(),
    };
    let saved = ur.save(&pool).await?;
    println!("   Saved: {:?}", saved);

    // 2. Look up by composite pk
    println!("\n2. Looking up by composite key...");
    let found = UserRole::by_key(&pool, &saved.user_id, &saved.role_id).await?;
    println!("   Found: {:?}", found);
    assert_eq!(found.assigned_at, "2024-01-01");

    // 3. Upsert: update assigned_at
    println!("\n3. Upserting (same pk, different assigned_at)...");
    let ur2 = UserRole {
        user_id: "alice".to_string(),
        role_id: "admin".to_string(),
        assigned_at: "2024-06-15".to_string(),
    };
    let upserted = ur2.save(&pool).await?;
    println!("   Upserted: {:?}", upserted);
    assert_eq!(upserted.assigned_at, "2024-06-15");

    // 4. Delete
    println!("\n4. Deleting the user role...");
    upserted.delete(&pool).await?;
    match UserRole::by_key(&pool, &ur.user_id, &ur.role_id).await {
        Ok(_) => println!("   ERROR: record still exists!"),
        Err(_) => println!("   Confirmed: user role deleted"),
    }

    // 5. Tag (full-key model — all columns are pk)
    println!("\n5. Saving a tag (full-key model, INSERT OR IGNORE)...");
    let tag = Tag {
        name: "rust".to_string(),
    };
    let saved_tag = tag.save(&pool).await?;
    println!("   Saved tag: {:?}", saved_tag);

    // Save again — idempotent (DO NOTHING semantics)
    println!("\n6. Saving the same tag again (idempotent)...");
    let saved_tag2 = tag.save(&pool).await?;
    println!("   Saved tag again (idempotent): {:?}", saved_tag2);
    assert_eq!(saved_tag.name, saved_tag2.name);

    // Look up tag by pk
    println!("\n7. Looking up tag by name...");
    let found_tag = Tag::by_name(&pool, &tag.name).await?;
    println!("   Found tag: {:?}", found_tag);

    println!("\n=== Done ===");
    Ok(())
}
