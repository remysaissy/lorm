//! Transaction example
//!
//! This example demonstrates:
//! - Using Lorm with database transactions
//! - Atomic operations
//! - Rollback on errors
//! - Commit successful transactions

use anyhow::Result;
use chrono::FixedOffset;
use lorm::ToLOrm;
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

#[derive(Debug, Default, Clone, FromRow, ToLOrm)]
struct Account {
    #[lorm(pk)]
    #[lorm(new = "Uuid::new_v4()")]
    #[lorm(is_set = "is_nil()")]
    pub id: Uuid,

    #[lorm(by)]
    pub name: String,

    pub balance: i64,

    #[lorm(created_at)]
    #[lorm(new = "chrono::Utc::now().fixed_offset()")]
    pub created_at: chrono::DateTime<FixedOffset>,

    #[lorm(updated_at)]
    #[lorm(new = "chrono::Utc::now().fixed_offset()")]
    pub updated_at: chrono::DateTime<FixedOffset>,
}

#[allow(dead_code)]
fn is_nil(id: &Uuid) -> bool {
    *id == Uuid::default()
}

async fn transfer(
    pool: &SqlitePool,
    from_id: &Uuid,
    to_id: &Uuid,
    amount: i64,
) -> Result<()> {
    // Start a transaction
    let mut tx = pool.begin().await?;

    // Get accounts within transaction
    let mut from_account = Account::by_id(&mut *tx, from_id).await?;
    let mut to_account = Account::by_id(&mut *tx, to_id).await?;

    // Check balance
    if from_account.balance < amount {
        anyhow::bail!("Insufficient funds");
    }

    // Perform transfer
    from_account.balance -= amount;
    to_account.balance += amount;

    // Save within transaction
    from_account.save(&mut *tx).await?;
    to_account.save(&mut *tx).await?;

    // Commit transaction
    tx.commit().await?;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;

    // Create schema
    sqlx::query(
        r#"
        CREATE TABLE accounts (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            balance INTEGER NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    println!("=== Transaction Example ===\n");

    // Create accounts
    println!("1. Creating accounts...");
    let mut alice = Account::default();
    alice.name = "Alice".to_string();
    alice.balance = 1000;
    alice = alice.save(&pool).await?;
    println!("   Alice's account: ${}", alice.balance);

    let mut bob = Account::default();
    bob.name = "Bob".to_string();
    bob.balance = 500;
    bob = bob.save(&pool).await?;
    println!("   Bob's account: ${}\n", bob.balance);

    // Successful transfer
    println!("2. Transfer $200 from Alice to Bob...");
    transfer(&pool, &alice.id, &bob.id, 200).await?;

    // Verify balances
    let alice_updated = Account::by_id(&pool, &alice.id).await?;
    let bob_updated = Account::by_id(&pool, &bob.id).await?;
    println!("   Alice's balance: ${}", alice_updated.balance);
    println!("   Bob's balance: ${}\n", bob_updated.balance);

    // Failed transfer (insufficient funds)
    println!("3. Attempt to transfer $1000 from Bob to Alice...");
    match transfer(&pool, &bob_updated.id, &alice_updated.id, 1000).await {
        Ok(_) => println!("   ERROR: Should have failed!"),
        Err(e) => println!("   Failed as expected: {}", e),
    }

    // Verify balances unchanged
    let alice_final = Account::by_id(&pool, &alice.id).await?;
    let bob_final = Account::by_id(&pool, &bob.id).await?;
    println!("   Alice's balance (unchanged): ${}", alice_final.balance);
    println!("   Bob's balance (unchanged): ${}\n", bob_final.balance);

    // Demonstrate explicit transaction usage
    println!("4. Explicit transaction with rollback...");
    let mut tx = pool.begin().await?;

    let mut charlie = Account::default();
    charlie.name = "Charlie".to_string();
    charlie.balance = 2000;
    charlie = charlie.save(&mut *tx).await?;
    println!("   Created Charlie with balance ${}", charlie.balance);

    // Rollback instead of commit
    tx.rollback().await?;
    println!("   Transaction rolled back");

    // Verify Charlie doesn't exist
    match Account::by_name(&pool, "Charlie").await {
        Ok(_) => println!("   ERROR: Charlie should not exist!"),
        Err(_) => println!("   Confirmed: Charlie's account was not created"),
    }

    Ok(())
}
