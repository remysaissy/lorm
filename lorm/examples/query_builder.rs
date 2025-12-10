//! Query builder example
//!
//! This example demonstrates:
//! - Filtering with where clauses
//! - Ordering results
//! - Pagination with limit and offset
//! - Combining multiple query conditions

use anyhow::Result;
use chrono::FixedOffset;
use lorm::ToLOrm;
use lorm::predicates::Where;
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

#[derive(Debug, Default, Clone, FromRow, ToLOrm)]
struct Product {
    #[lorm(pk)]
    #[lorm(new = "Uuid::new_v4()")]
    pub id: Uuid,

    #[lorm(by)]
    pub name: String,

    #[lorm(by)]
    pub price: i32,

    #[lorm(by)]
    pub category: String,

    #[lorm(created_at)]
    #[lorm(new = "chrono::Utc::now().fixed_offset()")]
    pub created_at: chrono::DateTime<FixedOffset>,

    #[lorm(updated_at)]
    #[lorm(new = "chrono::Utc::now().fixed_offset()")]
    pub updated_at: chrono::DateTime<FixedOffset>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;

    // Create schema
    sqlx::query(
        r#"
        CREATE TABLE products (
            id TEXT PRIMARY KEY NOT NULL,
            name TEXT NOT NULL,
            price INTEGER NOT NULL,
            category TEXT NOT NULL,
            created_at TEXT NOT NULL,
            updated_at TEXT NOT NULL
        )
        "#,
    )
    .execute(&pool)
    .await?;

    println!("=== Query Builder Example ===\n");

    // Create sample data
    println!("Creating sample products...");
    let products = vec![
        ("Laptop", 1200, "Electronics"),
        ("Mouse", 25, "Electronics"),
        ("Keyboard", 75, "Electronics"),
        ("Desk", 300, "Furniture"),
        ("Chair", 150, "Furniture"),
        ("Monitor", 400, "Electronics"),
        ("Lamp", 45, "Furniture"),
        ("Notebook", 5, "Stationery"),
        ("Pen", 2, "Stationery"),
        ("Backpack", 60, "Accessories"),
    ];

    for (name, price, category) in products {
        let mut product = Product::default();
        product.name = name.to_string();
        product.price = price;
        product.category = category.to_string();
        product.save(&pool).await?;
    }
    println!("Created 10 products\n");

    // Example 1: Simple filtering
    println!("1. Find all Electronics:");
    let electronics = Product::with_category(&pool, "Electronics").await?;
    for p in &electronics {
        println!("   - {} (${}/100)", p.name, p.price);
    }
    println!();

    // Example 2: Price range filtering
    println!("2. Products between $50 and $500:");
    let mid_range = Product::select()
        .where_between_price(50, 500)
        .order_by_price()
        .asc()
        .build(&pool)
        .await?;
    for p in &mid_range {
        println!("   - {} (${}/100)", p.name, p.price);
    }
    println!();

    // Example 3: Price filtering
    println!("3. Products under $100:");
    let affordable = Product::select()
        .where_price(Where::LesserThan, 100)
        .order_by_price()
        .desc()
        .build(&pool)
        .await?;
    for p in &affordable {
        println!("   - {} (${}/100)", p.name, p.price);
    }
    println!();

    // Example 4: Pagination
    println!("4. Products page 1 (limit 3):");
    let page1 = Product::select()
        .order_by_name()
        .asc()
        .limit(3)
        .build(&pool)
        .await?;
    for p in &page1 {
        println!("   - {}", p.name);
    }
    println!();

    println!("5. Products page 2 (limit 3, offset 3):");
    let page2 = Product::select()
        .order_by_name()
        .asc()
        .limit(3)
        .offset(3)
        .build(&pool)
        .await?;
    for p in &page2 {
        println!("   - {}", p.name);
    }
    println!();

    // Example 5: Price filtering with ordering
    println!("6. Expensive items (price > $100):");
    let expensive = Product::select()
        .where_price(Where::GreaterThan, 100)
        .order_by_price()
        .desc()
        .build(&pool)
        .await?;
    for p in &expensive {
        println!("   - {} - {} (${}/100)", p.category, p.name, p.price);
    }

    Ok(())
}
