use proc_macro::TokenStream;
use proc_macro_error2::proc_macro_error;
use syn::{DeriveInput, parse_macro_input};

mod attributes;
mod models;
mod orm;
mod utils;

/// `#[derive(ToLOrm)]`
///
/// Generates Object Relational Model traits and methods for a named struct.
///
/// ## Private Key Handling
///
/// Lorm supports two types of private keys.
///
/// ### `generated` (default)
/// `[#[lorm(pk_type = "generated")]`
///
/// This is the default mode. Exactly one field must be marked with `#[lorm(pk)]`.
/// When saving an instance, lorm will determine whether the private key is already set or not and generate one if needed.
///
/// If not specified otherwise using the `#[lorm(is_set = "method_name")]` attribute,
/// lorm considers the private key not set if its value is equal to the default value
/// of its type (requiring the primary key type to implement `Default` and `Eq`).
///
/// A new key value is generated using [Default::default()] or by the expression specified with `#[lorm(new = "expr")]`.
///
/// If the primary key field is marked as `#[lorm(readonly)]`, key generation is skipped and the database is expected to provide the value (for example, with an auto-incrementing column).
///
/// If the fields are also marked as `#[lorm(readonly)]`, key generation is skipped and the database is expected to provide the value.
///
/// ### `manual`
/// `#[lorm(pk_type = "manual")]`
///
/// In this mode, lorm does not touch your primary key ever and assumes that it always holds a correct value.
/// The primary key can also be a composite primary key if multiple fields are marked with `#[lorm(pk)]`.
///
/// ## `created_at` and `updated_at`
///
/// Lorm can manage fields that are specified to hold times that an instance was first or last saved to the database. These fields are marked with `#[lorm(created_at)]` and `#[lorm(updated_at)]`, respectively.
///
/// Their value is constructed at insert/update time using [Default::default()] or the expression specified with `#[lorm(new = "expr")]`.
///
/// ## Query Methods
///
/// Lorm generates the following standalone query methods:
/// - `by_<field>(executor, value)`: Find a single record by field value
/// - `with_<field>(executor, value)`: Find all records matching a field value
/// and the following for the `::select()` query builder:
/// - `where_<field>(Where, value)`: Filter by field value
/// - `where_between_<field>(start, end)`: Filter by field value range (SQL `BETWEEN`)
/// - `order_by_<field>()`: Order results by filed (chain with `.asc()` or `.desc()`)
/// - `group_by_<field>()`: Group results by this field
///
/// These methods are generated for a field if it fulfills any of the following conditions:
/// - It is annotated with `#[lorm(by)]`
/// - It is annotated with `#[lorm(created_at)]` or `#[lorm(updated_at)]`
/// - It is a non-composite primary key (the only field annotated with `#[lorm(pk)]`)
///
/// Additionally, a `by_key` method is generated for `manual` composite primary keys that takes
/// each column of the primary key as an argument. The name of this method can be customized with `#[lorm(pk_selector = "method_name")]`.
///
/// ## Flattening
///
/// Lorm supports fields annotated with `#[sqlx(flatten)]`. However, users must specify by hand the fields this flattened to.
/// This is done using the `#[lorm(flattened(field: Type = "column", field2: Type2, ...))]` attribute.
/// The optional `= "column` can be used for fields of the flattened type annotated with `#[sqlx(rename = "column_name")]`.
///
/// # Supported Attributes
///
/// Struct-level:
/// - `#[lorm(rename = "table_name")]`
///   Overrides the SQL table name. By default, the struct name is converted to table_case and pluralized
///   (for example: `UserDetail` -> `user_details`).
/// - `#[lorm(pk_type = "generated" | "manual")]`
///   Primary key mode. Default is `generated`.
///   - `generated`: exactly one field must be marked `#[lorm(pk)]`. Lorm will determine whether to generate a new key or not.
///   - `manual`: one or more fields must be marked `#[lorm(pk)]`.
/// - `#[lorm(pk_selector = "method_name")]`
///   Only used for `pk_type = "manual"` with multiple `#[lorm(pk)]` fields.
///   Renames the generated composite-key selector method (default: `by_key`).
///
/// Field-level:
/// - `#[lorm(pk)]`
///   Marks a primary-key field.
/// - `#[lorm(by)]`
///   Generate query methods for this field.
/// - `#[lorm(readonly)]`
///   Excludes the field from insert/update in `save()`.
/// - `#[lorm(created_at)]`
///   Marks the created-at field (at most one field can use this).
/// - `#[lorm(updated_at)]`
///   Marks the updated-at field (at most one field can use this).
/// - `#[lorm(new = "expr")]
///   Custom value expression used by `save()` when generating a new primary key or the `created_at` or `updated_at` values.
///   If omitted, defaults to `Default::default()`.
/// - `#[lorm(is_set = "method_name")]
///   If omitted, defaults to a comparison with `<FieldType as Default>::default()`.
///   Can only be used with `#[lorm(pk)]` and `generated` primary key mode.
/// - `#[lorm(flattened(field_a: TypeA = "column_a", field_b: TypeB, ...))]`
///   Expands one Rust field into multiple logical SQL columns in generated SQL.
///
/// Attributes from `sqlx`'s `FromRow` derive macro also consumed by lorm:
/// - `#[sqlx(skip)]`: skips the field entirely in generated ORM SQL.
/// - `#[sqlx(rename = "column_name")]`: overrides SQL column name.
/// - `#[sqlx(json)]`: binds field values as `sqlx::types::Json(...)`.
/// - `#[sqlx(flatten)]`: supported alongside `#[lorm(flattened(...))]` for flattened fields.
///
#[proc_macro_error]
#[proc_macro_derive(ToLOrm,
    attributes(
        lorm,
        // lorm(pk),
        // lorm(pk_type = "generated" | "manual"),
        // lorm(pk_selector = "by_key"),
        // lorm(by),
        // lorm(readonly),
        // lorm(created_at),
        // lorm(updated_at),
        // lorm(new = "expr"),
        // lorm(is_set = "method_name"),
        // lorm(flattened(field: Type = "column")),
        // lorm(rename = "table_name"),

        // additionally consumed on fields:
        // sqlx(rename = "column_name")
        // sqlx(skip)
        // sqlx(json)
        // sqlx(json(nullable))
        // sqlx(flatten)
    )
)]
pub fn sql_derive_to_orm(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    orm::expand_derive_to_orm(&input).unwrap_or_else(|e| e.to_compile_error().into())
}
