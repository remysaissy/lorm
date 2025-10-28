# Lorm - A light ORM for SQLx

Lorm is an async and lightweight ORM for SQLx that uses derive macros to generate type-safe database operations at compile time.

## Features

- **Zero-cost abstractions** - All code is generated at compile time
- **Type-safe queries** - Leverages Rust's type system for compile-time query validation
- **Async-first** - Built on tokio and async/await
- **Automatic CRUD** - Generate create, read, update, and delete operations
- **Flexible querying** - Builder pattern for complex queries with filtering, ordering, and pagination
- **Pool and Transaction support** - Works seamlessly with both connection pools and transactions
- **Timestamp management** - Automatic handling of `created_at` and `updated_at` fields
- **Custom field generation** - Support for UUID, custom types, and database-generated values

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

### Supported Databases

Lorm supports all databases that SQLx supports:

- **PostgreSQL** - Use `features = ["postgres"]` in sqlx
- **MySQL / MariaDB** - Use `features = ["mysql"]` in sqlx
- **SQLite** - Use `features = ["sqlite"]` in sqlx
- **Microsoft SQL Server** - Use `features = ["mssql"]` in sqlx

All features work consistently across database backends.

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

### Query Builder API

Lorm generates a fluent query builder using `::select()`. The builder supports filtering, ordering, grouping, and pagination.

#### Available Methods

**Filtering** (available for all fields):
- `where_equal_{field}(value)` - Exact match
- `where_not_equal_{field}(value)` - Not equal
- `where_less_{field}(value)` - Less than
- `where_less_equal_{field}(value)` - Less than or equal
- `where_more_{field}(value)` - Greater than
- `where_more_equal_{field}(value)` - Greater than or equal
- `where_between_{field}(start, end)` - Between two values (inclusive)

**Ordering** (available for `#[lorm(by)]` fields):
- `order_by_{field}(OrderBy::Asc)` - Ascending order
- `order_by_{field}(OrderBy::Desc)` - Descending order

**Grouping** (available for `#[lorm(by)]` fields):
- `group_by_{field}()` - Group results by field

**Pagination**:
- `limit(n)` - Limit number of results
- `offset(n)` - Skip first n results

#### Query Examples

```rust
use lorm::predicates::OrderBy;

// Simple query
let users = User::select()
    .where_equal_email("alice@example.com")
    .build(&pool)
    .await?;

// Filtering and ordering
let recent_users = User::select()
    .where_more_equal_created_at(yesterday)
    .order_by_created_at(OrderBy::Desc)
    .build(&pool)
    .await?;

// Pagination
let page_2 = User::select()
    .order_by_email(OrderBy::Asc)
    .limit(10)
    .offset(10)
    .build(&pool)
    .await?;

// Complex query combining multiple conditions
let results = User::select()
    .where_between_id(100, 200)
    .where_not_equal_email("banned@example.com")
    .order_by_created_at(OrderBy::Desc)
    .limit(20)
    .build(&pool)
    .await?;

// Grouping
let grouped = User::select()
    .group_by_email()
    .build(&pool)
    .await?;
```

#### Direct Field Queries

For fields marked with `#[lorm(by)]`, additional convenience methods are generated:

```rust
// Find single record by field
let user = User::by_email(&pool, "alice@example.com").await?;

// Find multiple records with same field value
let users = User::with_email(&pool, "alice@example.com").await?;

// Delete by field
User::delete_by_email(&pool, "alice@example.com").await?;
```

### Examples
Usage examples are documented in the test cases. Please refer to `lorm/tests/main.rs` for a concrete example of how to use each feature.

## Design Philosophy

Lorm is designed to be:
- **Lightweight** - Minimal runtime overhead with compile-time code generation
- **Pragmatic** - Cover 80% of common use cases without complexity
- **Composable** - Works alongside raw SQLx queries when needed
- **Transparent** - Generated code can be inspected with `cargo expand`

## Requirements

- **Rust Edition**: 2024 or later
- **SQLx**: 0.8 or later
- **Tokio**: 1.0 or later (for async runtime)

## Limitations

- No automatic schema migrations (use SQLx migrations or other tools)
- No relationships/joins (use SQLx for complex queries)
- Requires `Default` trait on structs for most operations
- Primary key field name detection is attribute-based, not convention-based

## When to Use Lorm

**Use Lorm when:**
- You need simple CRUD operations
- You want type safety with minimal boilerplate
- You're building on top of SQLx
- You want explicit control over your database schema

**Consider alternatives when:**
- You need complex relationships and joins
- You want automatic schema migrations
- You need an Active Record pattern
- You require ORM-managed relationships

## License
Licensed under Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0)

## Contribution
Unless you explicitly state otherwise, any Contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be  licensed as above, without any additional terms or conditions.
