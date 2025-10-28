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

It all starts by adding the `#[derive(ToLOrm)]` to a structure with SQLx's `#[derive(FromRow)]`.
This will instrument the structure by generating traits and methods according to your needs.

It is then possible to call lorm generated methods using a database connection, whether it comes from
a Pool or a Transaction.
For eg.
```rust
let mut conn = pool.acquire().await.unwrap();
    _test(&mut *conn).await;

    // Recreate a DB or the second test fails as existing users conflict.
    let pool = get_pool().await.expect("Failed to create pool");
    let mut tx = pool.begin().await.unwrap();
    _test(&mut *tx).await;
    tx.commit().await.unwrap();

    async fn _test(conn: &mut SqliteConnection) {
        let id = Uuid::new_v4();
        User::by_id(&mut *conn, &id).await;
    }
```
Check out the lorm/tests directory for more examples.

### Supported attributes

**Primary key**
Add the `#[lorm(pk)]` annotation to the primary key field of your structure.
- the field is marked as being the primary key and can only be generated at insertion time
- this field is also automatically considered as a `#[lorm(by)]` field.
- If `#[lorm(new)]` is specified, it will use its struct method to generate a new pk at insertion time
- If `#[lorm(is_set)]` is specified, it will use its instance method against `self` to check if the pk is set. Otherwise it compares the pk value with its <struct>::default() (assuming the Default trait is set)
- If `#[lorm(readonly)]` is specified, it will ignore is_set `#[lorm(new)]` and `#[lorm(is_set)]` and let the database handles the field 

**Field of table renaming**
Add the `#[lorm(rename="name")]` annotation
 - at struct level to rename at table name
 - at field level to rename at column name
By default, a table name is the struct name pluralized and converted to table case: UserDetail => user_details.
By default, a field name is converted to snake_case: UserDetail => user_detail.

**Field to be ignored for persistence**
Add the `#[lorm(transient)]` annotation to ignore the field for all lorm methods.
It is recommended to use `#[lorm(transient)]` and `#[sqlx(skip)]` together as transient does not forcibly insert the sqlx skip annotation.

**Field that can't be modified under any condition**
Add the `#[lorm(readonly)]` annotation to indicate a field is provided but never updated by your code.
Special cases to consider:
 - If applied to the primary key, key generation is left to the database. No update possible as it is the primary key
 - If applied to create_at or updated_at field, timestamp generation is left to the database. No update is possible

**CRUD operation using a specific field**
Add the `#[lorm(by)]` annotation to generate with_<field>, by_<field>, delete_by_<field> and select with order_by_<field>, group_by_<field>, limit and offset methods.

**created_at support**
Add the `#[lorm(created_at)]` annotation to mark the field as the `created_at` field. 
- If `#[lorm(new)]` is specified, it will use its method to update the time upon insertion
- If `#[lorm(readonly)]` is specified, it will ignore is_set `#[lorm(new)]` and let the database handles the field

**updated_at support**
Add the `#[lorm(updated_at)]` annotation to mark the field as the `updated_at` field.
- If `#[lorm(new)]` is specified, it will use its method to update the time upon insertion and update
- If `#[lorm(readonly)]` is specified, it will ignore is_set `#[lorm(new)]` and let the database handles the field

**Custom new method**
Add the `#[lorm(new="module::path::class::new_custom()")]` annotation to use a custom creation method.
- The function call is expected to return an instance
- When not provided, the type::new() method is called

**Custom check**
Add the `#[lorm(is_set="is_nil()")]` annotation to use a custom check.
It uses a specific function call to check if the returned value if the default value.
The function call is expected to return bool.
Defaults to class_type::default() which assumes both the Default and PartialEq trait are implemented.

### Select methods
Queries are run using the Class::select() method.
This method returns a builder to configure the select.

- where_between_{field}(value)
- where_equal_{field}(value)
- where_not_equal_{field}(value)
- where_less_{field}(value)
- where_less_equal_{field}(value)
- where_more{field}(value)
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
