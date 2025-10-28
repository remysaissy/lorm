# Lorm Examples

This directory contains runnable examples demonstrating various Lorm features.

## Running Examples

To run an example:

```bash
cargo run --example <example_name>
```

For instance:
```bash
cargo run --example basic_crud
```

## Available Examples

### basic_crud.rs
Demonstrates fundamental CRUD operations:
- Creating records
- Reading records by primary key
- Updating records
- Deleting records

```bash
cargo run --example basic_crud
```

### query_builder.rs
Shows the query builder API:
- Filtering with where clauses
- Ordering results
- Pagination with limit and offset
- Combining multiple conditions

```bash
cargo run --example query_builder
```

### transactions.rs
Illustrates transaction handling:
- Using Lorm with database transactions
- Atomic operations
- Rollback on errors
- Commit successful transactions

```bash
cargo run --example transactions
```

## Requirements

All examples use an in-memory SQLite database for simplicity and require no additional setup. They will:
1. Create an in-memory database
2. Set up the schema
3. Demonstrate the features
4. Clean up automatically

## Learning Path

If you're new to Lorm, we recommend running the examples in this order:

1. **basic_crud.rs** - Start here to understand fundamental operations
2. **query_builder.rs** - Learn advanced querying capabilities
3. **transactions.rs** - Understand transaction handling

## Adapting Examples

All examples use SQLite for simplicity. To adapt them for other databases:

1. Change the connection string:
   ```rust
   // PostgreSQL
   let pool = PgPool::connect("postgres://user:pass@localhost/db").await?;

   // MySQL
   let pool = MySqlPool::connect("mysql://user:pass@localhost/db").await?;
   ```

2. Adjust SQL types in CREATE TABLE statements to match your database

3. Update your Cargo.toml dependencies to include the appropriate database driver
