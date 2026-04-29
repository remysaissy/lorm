# Lorm - A light ORM for SQLx

Lorm is an async and lightweight ORM for SQLx that uses derive macros to generate type-safe database operations at compile time.

## Features

- **Zero-cost abstractions** - All code is generated at compile time
- **Type-safe queries** - Leverages Rust's type system for compile-time query validation
- **Async-first** - Built on tokio and async/await
- **Automatic CRUD** - Generate create, read, update, and delete operations
- **Flexible querying** - Builder pattern for complex queries with filtering, ordering, grouping, aggregation, and pagination
- **Pool and Transaction support** - Works seamlessly with both connection pools and transactions
- **Timestamp management** - Automatic handling of `created_at` and `updated_at` fields
- **Custom field generation** - Support for UUID, custom types, and database-generated values

## Quickstart

### Installation
Add Lorm to your `Cargo.toml`:

```toml
[dependencies]
lorm = "0.2"
sqlx = { version = "0.8", features = ["runtime-tokio", "sqlite"] }
tokio = { version = "1", features = ["full"] }
uuid = { version = "1", features = ["v4"] }
chrono = "0.4"
```

To use a different database backend, change **both** the `lorm` and `sqlx` features:

```toml
# PostgreSQL
lorm = { version = "0.2", default-features = false, features = ["postgres"] }
sqlx = { version = "0.8", features = ["runtime-tokio", "postgres"] }

# MySQL / MariaDB
lorm = { version = "0.2", default-features = false, features = ["mysql"] }
sqlx = { version = "0.8", features = ["runtime-tokio", "mysql"] }
```

### Supported Databases

Lorm supports the same database backends as SQLx:

- **SQLite** (default) - `features = ["sqlite"]`
- **PostgreSQL** - `features = ["postgres"]`
- **MySQL / MariaDB** - `features = ["mysql"]`

The `sqlite`, `postgres`, and `mysql` features are **mutually exclusive** — only one database backend can be active at a time.

**MySQL-specific notes:**
- Timestamp fields must use `chrono::DateTime<chrono::Utc>` instead of `DateTime<FixedOffset>` (MySQL's sqlx driver does not support `FixedOffset`)
- The `save()` method requires the executor to implement `Copy` (works with `&Pool`, not with `&mut Transaction`)

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

Lorm works seamlessly with both `Pool` and `Transaction` connections. Check the [tests directory](tests) for more examples.

### Attribute Reference

Lorm provides several attributes to customize code generation. Attributes can be applied at struct level or field level.

#### Field-Level Attributes

| Attribute | Description | Example | Generated Methods |
|-----------|-------------|---------|-------------------|
| `#[lorm(pk)]` | Marks field as primary key. Automatically includes `by` functionality. Can only be set at creation time unless combined with `readonly`. | `#[lorm(pk)]`<br>`pub id: Uuid` | `by_id()`, `delete()`, `save()` |
| `#[lorm(by)]` | Generates query and utility methods for this field | `#[lorm(by)]`<br>`pub email: String` | `by_<field>()`, `with_<field>()`, `where_<field>()`, `order_by_<field>()`, `group_by_<field>()` |
| `#[lorm(readonly)]` | Field cannot be updated by application code. Database handles the value. | `#[lorm(readonly)]`<br>`pub count: i32` | Excluded from UPDATE queries |
| `#[lorm(skip)]` | Field is ignored for all persistence operations. Use with `#[sqlx(skip)]` | `#[lorm(skip)]`<br>`#[sqlx(skip)]`<br>`pub tmp: String` | Excluded from all queries |
| `#[lorm(created_at)]` | Marks field as creation timestamp | `#[lorm(created_at)]`<br>`pub created_at: DateTime` | Auto-set on INSERT |
| `#[lorm(updated_at)]` | Marks field as update timestamp | `#[lorm(updated_at)]`<br>`pub updated_at: DateTime` | Auto-set on INSERT and UPDATE |
| `#[lorm(new="expr")]` | Custom expression to generate field value | `#[lorm(new="Uuid::new_v4()")]` | Used in INSERT queries |
| `#[lorm(is_set="path")]` | Callable path to check if field has a value — invoked as `(path)(&field)`, must return `bool` | `#[lorm(is_set="Uuid::is_nil")]` | Used to determine INSERT vs UPDATE |
| `#[lorm(rename="name")]` | Renames field to specific column name | `#[lorm(rename="user_email")]` | Uses custom column name |
| `#[sqlx(json)]` | Serialises the field as JSON when writing and deserialises it when reading. Lorm wraps bind values with `sqlx::types::Json` automatically. Cannot be combined with `#[lorm(pk)]`. | `#[sqlx(json)]`<br>`pub preferences: serde_json::Value` | Field stored as JSON/JSONB/TEXT depending on backend |
| `#[sqlx(flatten)]` + `#[lorm(flattened(...))]` | Flattens a nested struct field into multiple SQL columns. Requires both attributes. For optional nested structs, use `Option<Nested>`. | `#[sqlx(flatten)]`<br>`#[lorm(flattened(street: String, zip: String = "zip_code"))]`<br>`pub address: Address` | Nested field is expanded into multiple columns |

#### Flattened Nested Structs

Use `#[sqlx(flatten)]` together with `#[lorm(flattened(...))]` to map a nested struct field to multiple SQL columns.

```rust
#[derive(Debug, Default, Clone, sqlx::FromRow)]
pub struct Address {
    pub street: String,
    #[sqlx(rename = "zip_code")]
    pub zip: String,
}

#[derive(Debug, Default, Clone, sqlx::FromRow, lorm::ToLOrm)]
pub struct Customer {
    #[lorm(pk, new = "Uuid::new_v4()", is_set = "Uuid::is_nil")]
    pub id: Uuid,
    #[lorm(by)]
    pub email: String,
    #[sqlx(flatten)]
    #[lorm(flattened(street: String, zip: String = "zip_code"))]
    pub address: Address,
}
```

To represent nullable flattened columns, use `Option<NestedStruct>`:

```rust
#[derive(Debug, Default, Clone, sqlx::FromRow, lorm::ToLOrm)]
pub struct OptCustomer {
    #[lorm(pk, new = "Uuid::new_v4()", is_set = "Uuid::is_nil")]
    pub id: Uuid,
    #[lorm(by)]
    pub email: String,
    #[sqlx(flatten)]
    #[lorm(flattened(street: String, zip: String = "zip_code"))]
    pub address: Option<Address>,
}
```

#### Struct-Level Attributes

| Attribute | Description | Example |
|-----------|-------------|---------|
| `#[lorm(rename="name")]` | Sets custom table name | `#[lorm(rename="app_users")]`<br>`struct User` |
| `#[lorm(pk_type="manual")]` | Enables composite/manual primary key mode. All `#[lorm(pk)]` fields form the composite key. | `#[lorm(pk_type="manual")]`<br>`struct UserRole` |
| `#[lorm(pk_selector="name")]` | Custom selector method name for composite pk (default: `by_key` for 2+ fields, `by_<field>` for 1 field) | `#[lorm(pk_type="manual", pk_selector="find_by_ids")]` |

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
#[lorm(pk, new = "Uuid::new_v4()")]
pub id: Uuid

// Timestamp managed by application
#[lorm(created_at, new = "chrono::Utc::now().fixed_offset()")]
pub created_at: DateTime<FixedOffset>

// Timestamp managed by database
#[lorm(created_at, readonly)]
pub created_at: DateTime<FixedOffset>
```

### Query Builder API

Lorm generates a fluent query builder using `::select()`. The builder supports filtering, ordering, grouping, aggregation, and pagination.

#### Available Methods

**Filtering** (available for `#[lorm(by)]` fields):
- `where_{field}(Where::Eq, value)` - Equals comparison
- `where_{field}(Where::NotEq, value)` - Not equals comparison
- `where_{field}(Where::GreaterThan, value)` - Greater than
- `where_{field}(Where::GreaterOrEqualTo, value)` - Greater than or equal
- `where_{field}(Where::LesserThan, value)` - Less than
- `where_{field}(Where::LesserOrEqualTo, value)` - Less than or equal
- `where_{field}(Where::Like, value)` - Search for a specified pattern
- `where_between_{field}(start, end)` - Between two values (inclusive)

**Aggregation & Having** (available for `#[lorm(by)]` fields):
- `having_{field}(Having::Op, Function::Type, value)` - Filter grouped results
- `having_all_count(Having::Op, value)` - Filter by COUNT(*) on grouped results

**Aggregate Functions** (used with HAVING clauses):
- `Function::Count { is_distinct: bool }` - Count rows (with optional DISTINCT)
- `Function::Sum` - Sum of values
- `Function::Avg` - Average of values
- `Function::Min` - Minimum value
- `Function::Max` - Maximum value

**Ordering** (available for `#[lorm(by)]` fields):
- `order_by_{field}().asc()` - Ascending order
- `order_by_{field}().desc()` - Descending order

**Grouping** (available for `#[lorm(by)]` fields):
- `group_by_{field}()` - Group results by field. All remaining SELECT columns are automatically added to the GROUP BY clause for SQL standard compliance across all backends.

**Pagination**:
- `limit(n)` - Limit number of results
- `offset(n)` - Skip first n results

#### Query Examples

```rust
use lorm::predicates::{Where, Having, Function};

// Simple query with exact match
let users = User::select()
    .where_email(Where::Eq, "alice@example.com")
    .build(&pool)
    .await?;

// Filtering and ordering
let recent_users = User::select()
    .where_created_at(Where::GreaterOrEqualTo, yesterday)
    .order_by_created_at()
    .desc()
    .build(&pool)
    .await?;

// Pagination
let page_2 = User::select()
    .order_by_email()
    .asc()
    .limit(10)
    .offset(10)
    .build(&pool)
    .await?;

// Complex query combining multiple conditions
let results = User::select()
    .where_between_id(100, 200)
    .where_email(Where::NotEq, "banned@example.com")
    .order_by_created_at()
    .desc()
    .limit(20)
    .build(&pool)
    .await?;

// Grouping with ordering
let grouped = User::select()
    .group_by_email()
    .group_by_id()
    .order_by_email()
    .asc()
    .build(&pool)
    .await?;

// Aggregation with HAVING clause
let high_value_groups = Product::select()
    .group_by_category()
    .having_price(Having::GreaterThan, Function::Avg, 100.0)
    .build(&pool)
    .await?;

// Count aggregation with HAVING
let popular_categories = Product::select()
    .group_by_category()
    .having_all_count(Having::GreaterOrEqualTo, 10)
    .build(&pool)
    .await?;

// Complex aggregation query
let stats = Order::select()
    .where_created_at(Where::GreaterOrEqualTo, last_month)
    .group_by_customer_id()
    .having_amount(Having::GreaterThan, Function::Sum, 1000.0)
    .having_all_count(Having::GreaterOrEqualTo, 5)
    .order_by_customer_id()
    .asc()
    .build(&pool)
    .await?;
```

#### Direct Field Queries

For fields marked with `#[lorm(by)]`, convenience methods are generated:

```rust
// Find single record by field (returns first match)
let user = User::by_email(&pool, "alice@example.com").await?;

// Find all records matching field value
let users = User::with_email(&pool, "alice@example.com").await?;

// Delete a specific record (by primary key)
user.delete(&pool).await?;
```

### Examples

Complete, runnable examples are available in the [`examples/`](examples/) directory:

- **[basic_crud.rs](examples/basic_crud.rs)** - Create, read, update, and delete operations
- **[query_builder.rs](examples/query_builder.rs)** - Advanced querying with filtering, ordering, and pagination
- **[transactions.rs](examples/transactions.rs)** - Transaction handling and atomic operations

Run an example with:
```bash
cargo run --example basic_crud -p lorm
```

Additional examples are documented in the test cases at `tests/main.rs`.

## FAQ

### How does Lorm differ from Diesel?

Lorm is significantly lighter and simpler than Diesel:
- **Lorm**: Focused on CRUD operations with SQLx integration, minimal features
- **Diesel**: Full-featured ORM with query builder, migrations, and advanced relationship handling

Choose Lorm for simple CRUD with SQLx, choose Diesel for comprehensive ORM features.

### How does Lorm compare to SeaORM?

- **Lorm**: Lightweight macro-based code generation, no runtime overhead, limited to CRUD
- **SeaORM**: Full async ORM with entities, relations, migrations, and active record pattern

Lorm is better for simple use cases; SeaORM is better for complex applications with relationships.

### Can I use raw SQL with Lorm?

Yes! Lorm is built on SQLx, so you can mix Lorm-generated methods with raw SQLx queries:

```rust
// Use Lorm for simple operations
let user = User::by_email(&pool, "alice@example.com").await?;

// Use SQLx for complex queries
let results = sqlx::query_as::<_, User>(
    "SELECT * FROM users WHERE created_at > ? AND status = ?"
)
.bind(yesterday)
.bind("active")
.fetch_all(&pool)
.await?;
```

### Does Lorm support relationships/joins?

No. Lorm focuses on single-table CRUD operations. For relationships and joins, use SQLx directly.

### How do I handle migrations?

Lorm doesn't provide migrations. Use:
- **SQLx migrations**: Built-in support with `sqlx migrate`
- **Refinery**: Alternative migration tool
- **Custom scripts**: SQL files or custom tooling

### What's the compile-time impact?

Lorm uses proc macros which add to compile time, but the impact is minimal for small to medium projects. The generated code is optimized and adds no runtime overhead.

### Can I inspect the generated code?

Yes! Use `cargo expand` to see exactly what code Lorm generates:

```bash
cargo install cargo-expand
cargo expand --test main
```

### Does Lorm work with connection pools?

Yes! Lorm works with SQLx connection pools (`&Pool`) on all backends. On SQLite and PostgreSQL, it also works with transactions (`&mut Transaction`). On MySQL, the `save()` method requires a `Copy` executor, so it only works with `&Pool`.

### How do I store JSON data?

Use `#[sqlx(json)]` on a field typed as `serde_json::Value` (or any `serde::Serialize + serde::Deserialize` type). Lorm wraps the bind value with `sqlx::types::Json` on write and SQLx's `FromRow` derive deserialises it on read.

```rust
#[derive(Debug, Default, FromRow, ToLOrm)]
struct Profile {
    #[lorm(pk, new = "Uuid::new_v4()")]
    pub id: Uuid,
    #[sqlx(json)]
    pub preferences: serde_json::Value,
}
```

The underlying column type should be `TEXT` for SQLite, `JSONB` for PostgreSQL, and `JSON` for MySQL. A field with `#[sqlx(json)]` cannot be the primary key.

### How do I handle composite primary keys?

Use `#[lorm(pk_type = "manual")]` on the struct and mark each pk field with `#[lorm(pk)]`. Lorm generates `save()`, `delete()`, and a composite selector method (`by_key()` by default, or a custom name via `pk_selector`):

```rust
#[derive(Debug, Default, sqlx::FromRow, lorm::ToLOrm)]
#[lorm(pk_type = "manual")]
pub struct UserRole {
    #[lorm(pk)]
    pub user_id: Uuid,
    #[lorm(pk)]
    pub role_id: Uuid,
    pub assigned_at: String,
}

// Usage
let ur = UserRole { user_id, role_id, assigned_at: "2024-01-01".into() };
let saved = ur.save(&pool).await?;
let found = UserRole::by_key(&pool, &saved.user_id, &saved.role_id).await?;
saved.delete(&pool).await?;
```

For a custom selector name, use `#[lorm(pk_type = "manual", pk_selector = "find_by_ids")]`.

### Can I customize the SQL queries Lorm generates?

No. Lorm generates standard CRUD operations. For custom queries, use SQLx alongside Lorm.

### Is Lorm production-ready?

Lorm is in early development (0.x.y versions). The API may change. Use with caution in production and pin your version.

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

## Contributing

We welcome contributions! Please read [CONTRIBUTING.md](CONTRIBUTING.md) for details on the development setup, testing, and the pull request process.

**Signed commits are required.** See the [Signing Your Commits](CONTRIBUTING.md#signing-your-commits) section in CONTRIBUTING.md for setup instructions using GPG, SSH, or S/MIME.

### Verifying Release Artifacts

Published crate artifacts include [GitHub Artifact Attestations](https://docs.github.com/en/actions/security-for-github-actions/using-artifact-attestations) for supply chain verification:

```bash
gh attestation verify ./lorm-*.crate --repo remysaissy/lorm
```

## License
Licensed under Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE) or http://www.apache.org/licenses/LICENSE-2.0)

Unless you explicitly state otherwise, any Contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be licensed as above, without any additional terms or conditions.
