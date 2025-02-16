use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod helpers;
mod models;
mod orm;
mod util;

/// `#[derive(ToLOrm)]`
/// generate methods for Object Relational Mapping.
///
/// attributes:
///
/// `#[lorm(pk)]`
///  Annotated field is marked as being the primary key and can only be generated at insertion time.
///  This field is also automatically considered as a `#[lorm(by)]` field.
///  - If `#[lorm(new)]` is specified, it will use its struct method to generate a new pk at insertion time
///  - If `#[lorm(is_set)]` is specified, it will use its instance method against `self` to check if the pk is set. Otherwise it compares the pk value with its <struct>::default() (assuming the Default trait is set)
///  - If `#[lorm(readonly)]` is specified, it will ignore is_set `#[lorm(new)]` and `#[lorm(is_set)]` and let the database handles the field
///
/// `#[lorm(rename="name")]`
///   - at struct level to rename at table name
///   - at field level to rename at column name
///
///   by default, a table name is the struct name pluralized and converted to table case: UserDetail => user_details.
///   by default, a field name is converted to snake_case: UserDetail => user_detail.
///
/// `#[lorm(transient)]`
///  ignore field. using sqlx::FromRow, skip need `#[lorm(transient)]` and `#[sqlx(skip)]`
///
/// `#[lorm(readonly)]`
///  readonly attribute. Cannot be updated not inserted.
///  Special cases to consider:
///   - If applied to the primary key, key generation is left to the database. No update possible as it is the primary key.
///   - If applied to create_at or updated_at field, timestamp generation is left to the database. No update possible.
///
/// `#[lorm(by)]`
///  generate by_<field>, delete_by_<field> and select with its order_by_<field>, group_by_<field>,
///  limit and offset methods.
///
/// `#[lorm(fk="module::path::class")]`
///  Add the `#[lorm(fk="module::path::class")]` annotation to a foreign key field to generate the get_<field>() method which returns an instance of `module::path::class`.
///  The generated method removes the trailing _id if present in the field name.
///
/// `#[lorm(created_at)]`
///  Add the `#[lorm(created_at)]` annotation to mark the field as the `created_at` field.
///  - If `#[lorm(new)]` is specified, it will use its method to update the time upon insertion
///  - If `#[lorm(readonly)]` is specified, it will ignore is_set `#[lorm(new)]` and let the database handles the field
///
/// `#[lorm(updated_at)]`
///  Add the `#[lorm(updated_at)]` annotation to mark the field as the `updated_at` field.
///  - If `#[lorm(new)]` is specified, it will use its method to update the time upon insertion and update
///  - If `#[lorm(readonly)]` is specified, it will ignore is_set `#[lorm(new)]` and let the database handles the field
///
/// `#[lorm(new="module::path::class::new_custom()")]`
///  Add the `#[lorm(new="module::path::class::new_custom()")]` annotation to use a custom creation method.
///  - The function call is expected to return an instance
///  - When not provided, the type::new() method is called
///
/// `#[lorm(is_set="is_nil()")]`
///  Uses a specific function call to check if the returned value if the default value.
///  The function call is expected to return bool.
///  Defaults to class_type::default() which assumes both the Default and PartialEq trait are implemented.
///
#[proc_macro_derive(ToLOrm,
    attributes(
        lorm,
        // lorm(pk),
        // lorm(by),
        // lorm(transient),
        // lorm(readonly),
        // lorm(fk="module::path::class"),
        // lorm(new="module::path::class::new_custom()"),
        // lorm(is_set="is_nil()"),
        // lorm(rename="name"),
        // lorm(created_at),
        // lorm(updated_at),
    )
)]
pub fn sql_derive_to_orm(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match orm::expand_derive_to_orm(&input) {
        Ok(ts) => ts,
        Err(e) => e.to_compile_error().into(),
    }
}
