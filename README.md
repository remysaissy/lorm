# Lorm - A light ORM for SQLx

Lorm is an async and lightweight ORM for SQLx.

## Quickstart

### Installation
Add Lorm to your `Cargo.toml`:

```toml
[dependencies]
lorm = "0.0.9"
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite"] }
tokio = { version = "1", features = ["full"] }
uuid = { version = "1", features = ["v4"] }
chrono = "0.4"
```

**Note**: Replace `sqlite` with your preferred database driver (`postgres`, `mysql`, `mssql`).

### Usage

Define your model by adding `#[derive(ToLOrm)]` alongside SQLx's `#[derive(FromRow)]`:

```rust
use sqlx::{FromRow, SqlitePool};
use lorm::ToLOrm;
use uuid::Uuid;

#[derive(Debug, Default, FromRow, ToLOrm)]
struct User {
    #[lorm(pk, new = "Uuid::new_v4()")]
    pub id: Uuid,

    #[lorm(by)]
    pub email: String,

    #[lorm(created_at, new = "chrono::Utc::now().fixed_offset()")]
    pub created_at: chrono::DateTime<chrono::FixedOffset>,

    #[lorm(updated_at, new = "chrono::Utc::now().fixed_offset()")]
    pub updated_at: chrono::DateTime<chrono::FixedOffset>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let pool = SqlitePool::connect("sqlite::memory:").await?;

    // Create a user
    let mut user = User::default();
    user.email = "alice@example.com".to_string();
    user.save(&pool).await?;

    // Find by email (generated from #[lorm(by)])
    let found = User::by_email(&pool, "alice@example.com").await?;
    println!("Found user: {}", found.email);

    // Update the user
    user.email = "alice.updated@example.com".to_string();
    user.save(&pool).await?;

    // Delete the user
    user.delete(&pool).await?;

    Ok(())
}
```

Lorm works seamlessly with both `Pool` and `Transaction` connections. Check the [tests directory](lorm/tests/main.rs) for more examples.

### Attribute Reference

Lorm provides several attributes to customize code generation. Attributes can be applied at struct level or field level.

#### Field-Level Attributes

| Attribute | Description | Example | Generated Methods |
|-----------|-------------|---------|-------------------|
| `#[lorm(pk)]` | Marks field as primary key. Automatically includes `by` functionality. Can only be set at creation time unless combined with `readonly`. | `#[lorm(pk)]`<br>`pub id: Uuid` | `by_id()`, `delete()`, `save()` |
| `#[lorm(by)]` | Generates query and utility methods for this field | `#[lorm(by)]`<br>`pub email: String` | `by_<field>()`, `with_<field>()`, `delete_by_<field>()`, `order_by_<field>()`, `group_by_<field>()` |
| `#[lorm(readonly)]` | Field cannot be updated by application code. Database handles the value. | `#[lorm(readonly)]`<br>`pub count: i32` | Excluded from UPDATE queries |
| `#[lorm(transient)]` | Field is ignored for all persistence operations. Use with `#[sqlx(skip)]` | `#[lorm(transient)]`<br>`#[sqlx(skip)]`<br>`pub tmp: String` | Excluded from all queries |
| `#[lorm(created_at)]` | Marks field as creation timestamp | `#[lorm(created_at)]`<br>`pub created_at: DateTime` | Auto-set on INSERT |
| `#[lorm(updated_at)]` | Marks field as update timestamp | `#[lorm(updated_at)]`<br>`pub updated_at: DateTime` | Auto-set on INSERT and UPDATE |
| `#[lorm(new="expr")]` | Custom expression to generate field value | `#[lorm(new="Uuid::new_v4()")]` | Used in INSERT queries |
| `#[lorm(is_set="fn()")]` | Custom function to check if field has a value | `#[lorm(is_set="is_nil()")]` | Used to determine INSERT vs UPDATE |
| `#[lorm(rename="name")]` | Renames field to specific column name | `#[lorm(rename="user_email")]` | Uses custom column name |

#### Struct-Level Attributes

| Attribute | Description | Example |
|-----------|-------------|---------|
| `#[lorm(rename="name")]` | Sets custom table name | `#[lorm(rename="app_users")]`<br>`struct User` |

#### Naming Conventions

- **Table names**: Struct name pluralized and converted to snake_case
  - `User` → `users`
  - `UserDetail` → `user_details`
- **Column names**: Field name converted to snake_case
  - `userId` → `user_id`
  - `createdAt` → `created_at`

#### Attribute Combinations

Common attribute combinations:

```rust
// Auto-generated UUID primary key
#[lorm(pk, new = "Uuid::new_v4()", is_set = "is_nil()")]
pub id: Uuid

// Database-generated integer primary key
#[lorm(pk, readonly)]
pub id: i32

// Timestamp managed by application
#[lorm(created_at, new = "chrono::Utc::now().fixed_offset()")]
pub created_at: DateTime<FixedOffset>

// Timestamp managed by database
#[lorm(created_at, readonly)]
pub created_at: DateTime<FixedOffset>
```

### Select methods
Queries are run using the Class::select() method.
This method returns a builder to configure the select.

- where_between_{field}(value)
- where_equal_{field}(value)
- where_not_equal_{field}(value)
- where_less_{field}(value)
- where_less_equal_{field}(value)
- where_more_{field}(value)
- where_more_equal_{field}(value)
- order_by_{field}(OrderBy::Asc)
- group_by_{field}()

### Examples
Usage examples are documented in the test cases. Please refer to `lorm/tests/main.rs` for a concrete example of how to use each feature.  

## License
Licensed under Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0)

## Contribution
Unless you explicitly state otherwise, any Contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be  licensed as above, without any additional terms or conditions.
