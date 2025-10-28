//! Query builder example
//!
//! This example demonstrates:
//! - Filtering with where clauses
//! - Ordering results
//! - Pagination with limit and offset
//! - Combining multiple query conditions

use anyhow::Result;
use chrono::FixedOffset;
use lorm::predicates::OrderBy;
use lorm::ToLOrm;
use sqlx::{FromRow, SqlitePool};
use uuid::Uuid;

#[derive(Debug, Default, Clone, FromRow, ToLOrm)]
struct Product {
    #[lorm(pk, new = "Uuid::new_v4()", is_set = "is_nil()")]
    pub id: Uuid,

    #[lorm(by)]
    pub name: String,

    #[lorm(by)]
    pub price: i32,

    #[lorm(by)]
    pub category: String,

    #[lorm(created_at, new = "chrono::Utc::now().fixed_offset()")]
    pub created_at: chrono::DateTime<FixedOffset>,

    #[lorm(updated_at, new = "chrono::Utc::now().fixed_offset()")]
    pub updated_at: chrono::DateTime<FixedOffset>,
}

fn is_nil(id: &Uuid) -> bool {
    *id == Uuid::default()
}

#[tokio::main]
async fn main() -> Result<()> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;

    // Create schema
    sqlx::query(
        r#"
        CREATE TABLE products (
            id BLOB PRIMARY KEY NOT NULL,
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
        .order_by_price(OrderBy::Asc)
        .build(&pool)
        .await?;
    for p in &mid_range {
        println!("   - {} (${}/100)", p.name, p.price);
    }
    println!();

    // Example 3: Complex filtering
    println!("3. Electronics under $100:");
    let affordable_electronics = Product::select()
        .where_equal_category("Electronics")
        .where_less_price(100)
        .order_by_price(OrderBy::Desc)
        .build(&pool)
        .await?;
    for p in &affordable_electronics {
        println!("   - {} (${}/100)", p.name, p.price);
    }
    println!();

    // Example 4: Pagination
    println!("4. Products page 1 (limit 3):");
    let page1 = Product::select()
        .order_by_name(OrderBy::Asc)
        .limit(3)
        .build(&pool)
        .await?;
    for p in &page1 {
        println!("   - {}", p.name);
    }
    println!();

    println!("5. Products page 2 (limit 3, offset 3):");
    let page2 = Product::select()
        .order_by_name(OrderBy::Asc)
        .limit(3)
        .offset(3)
        .build(&pool)
        .await?;
    for p in &page2 {
        println!("   - {}", p.name);
    }
    println!();

    // Example 5: Multiple conditions
    println!("6. Expensive Furniture (price > $100):");
    let expensive_furniture = Product::select()
        .where_equal_category("Furniture")
        .where_more_price(100)
        .order_by_price(OrderBy::Desc)
        .build(&pool)
        .await?;
    for p in &expensive_furniture {
        println!("   - {} (${}/100)", p.name, p.price);
    }

    Ok(())
}
