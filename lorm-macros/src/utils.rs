use crate::models::UpsertField;
use inflector::Inflector;
use quote::{__private::TokenStream, ToTokens, format_ident, quote};
use syn::spanned::Spanned;
use syn::{DeriveInput, Expr, Field, Ident, LitStr, PathArguments, Type, parse};

/// Checks if an attribute with the given name and value exists on the field.
///
/// For example, checks if `#[lorm(pk)]` exists when called with `name="lorm"` and `value="pk"`.
pub(crate) fn has_attribute_value(attrs: &[syn::Attribute], name: &str, value: &str) -> bool {
    for attr in attrs.iter() {
        if !attr.path().is_ident(name) {
            continue;
        }

        let f = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident(value) {
                return Ok(());
            }
            Err(meta.error("attribute value not found"))
        });
        if f.is_ok() {
            return true;
        }
    }
    false
}

/// Gets the value of a String-type attribute by its key.
///
/// For example, extracts `"users"` from `#[lorm(rename="users")]` when called with
/// `name="lorm"` and `key="rename"`.
pub(crate) fn get_string_attribute_by_key(
    attrs: &[syn::Attribute],
    name: &str,
    key: &str,
) -> Option<String> {
    let mut val: Option<String> = None;
    for attr in attrs.iter() {
        if !attr.path().is_ident(name) {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident(key) {
                let value = meta.value()?; // this parses the `=`
                let v: LitStr = value.parse()?; // this parses `"val"`
                val = Some(v.value());
                return Ok(());
            }
            Err(meta.error("attribute value not found"))
        })
        .ok();
    }
    val
}

/// Gets the value of an attribute with an identifier value by its key.
///
/// For example, extracts `by_primary_key` from `#[lorm(pk_by=by_primary_key)]` when called with
/// `name="lorm"` and `key="pk_by"`.
pub(crate) fn get_ident_attribute_by_key(
    attrs: &[syn::Attribute],
    name: &str,
    key: &str,
) -> Option<Ident> {
    let mut val: Option<Ident> = None;
    for attr in attrs.iter() {
        if !attr.path().is_ident(name) {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident(key) {
                let value = meta.value()?; // this parses the `=`
                let v: Ident = value.parse()?; // this parses `"val"`
                val = Some(v);
                return Ok(());
            }
            Err(meta.error("attribute value not found"))
        })
        .ok();
    }
    val
}

/// Checks whether a type is a Rust primitive type.
///
/// Returns `true` for types like `i32`, `u64`, `bool`, `char`, etc.
pub(crate) fn is_primitive_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            let type_name = segment.ident.to_string();
            matches!(
                type_name.as_str(),
                "i8" | "i16"
                    | "i32"
                    | "i64"
                    | "i128"
                    | "isize"
                    | "u8"
                    | "u16"
                    | "u32"
                    | "u64"
                    | "u128"
                    | "usize"
                    | "f32"
                    | "f64"
                    | "bool"
                    | "char"
            )
        } else {
            false
        }
    } else {
        false
    }
}

/// Convert the type into the type that the db columns have. This does two things:
///
/// - Returns the type without its `Option<>` wrapper if present.
/// - Converts `String` to `&str`.
///
/// For example, `Option<String>` becomes `String`, and `Option<i32>` becomes `i32`.
pub(crate) fn to_column_type(ty: &Type) -> syn::Result<Type> {
    let res = match ty {
        Type::Path(type_path) => {
            let last_segment = type_path
                .path
                .segments
                .last()
                .expect("Type path should have at least one segment");
            let ident = &last_segment.ident;

            // Check for Option types and recurse
            if ident == "Option"
                && let PathArguments::AngleBracketed(angle_bracketed) = &last_segment.arguments
                && let Some(syn::GenericArgument::Type(inner_type)) = angle_bracketed.args.first()
            {
                inner_type
            } else {
                ty
            }
        }
        _ => ty,
    };

    if let Type::Path(type_path) = res
        && let Some(last_segment) = type_path.path.segments.last()
        && last_segment.ident == "String"
    {
        return parse(quote::quote! { str }.into());
    }

    // This is a clone in disguise as `Type` doesn't implement `Clone`
    parse(res.into_token_stream().into())
}

/// Checks if a field is marked to be skipped with `#[lorm(skip)]` or `#[sqlx(skip)]`.
///
/// Skipped fields are excluded from database operations.
pub(crate) fn is_skip(field: &Field) -> bool {
    has_attribute_value(&field.attrs, "lorm", "skip")
        | has_attribute_value(&field.attrs, "sqlx", "skip")
}

/// Checks if a field is marked as readonly with `#[lorm(readonly)]`.
///
/// Readonly fields cannot be updated or inserted manually.
pub(crate) fn is_readonly(field: &Field) -> bool {
    has_attribute_value(&field.attrs, "lorm", "readonly")
}

/// Checks if a field is marked as the primary key with `#[lorm(pk)]`.
pub(crate) fn is_pk(field: &Field) -> bool {
    has_attribute_value(&field.attrs, "lorm", "pk")
}

/// Checks if a field is marked for query generation with `#[lorm(by)]`.
///
/// This generates helper methods like `by_<field>`, `delete_by_<field>`, etc.
pub(crate) fn is_by(field: &Field) -> bool {
    has_attribute_value(&field.attrs, "lorm", "by")
}

/// Checks if a field is marked as the creation timestamp with `#[lorm(created_at)]`.
pub(crate) fn is_created_at(field: &Field) -> bool {
    has_attribute_value(&field.attrs, "lorm", "created_at")
}

/// Checks if a field is marked as the update timestamp with `#[lorm(updated_at)]`.
pub(crate) fn is_updated_at(field: &Field) -> bool {
    has_attribute_value(&field.attrs, "lorm", "updated_at")
}

/// Gets the method call to initialize a new value for a field.
///
/// Uses the `#[lorm(new="...")]` attribute if specified, otherwise defaults to `Type::new()`.
pub(crate) fn get_new_method(field: &Field) -> TokenStream {
    match get_string_attribute_by_key(&field.attrs, "lorm", "new") {
        None => {
            let class_token = field.ty.to_token_stream();
            quote! {
                #class_token::new()
            }
        }
        Some(method_name) => {
            let method_name: Expr =
                syn::parse_str(&method_name).expect("Failed to parse new method name");
            quote! {
                #method_name
            }
        }
    }
}

/// Gets the expression to check if a field is set (non-default).
///
/// Uses the `#[lorm(is_set="...")]` attribute if specified, otherwise compares against `Type::default()`.
pub(crate) fn get_is_set(field: &Field) -> TokenStream {
    let instance_field = field.ident.as_ref().unwrap();
    match get_string_attribute_by_key(&field.attrs, "lorm", "is_set") {
        None => {
            let class_token = field.ty.to_token_stream();
            quote! {
                #instance_field == #class_token::default()
            }
        }
        Some(method_name) => {
            let method_name: Expr =
                syn::parse_str(&method_name).expect("Failed to parse is_set method name");
            quote! {
                #instance_field.#method_name
            }
        }
    }
}

/// Gets the database table name for a struct.
///
/// Uses the `#[lorm(rename="...")]` attribute if specified, otherwise converts the struct name
/// to table_case and pluralizes it (e.g., `UserDetail` becomes `user_details`).
pub(crate) fn get_table_name(input: &DeriveInput) -> String {
    let table_name = get_string_attribute_by_key(&input.attrs, "lorm", "rename");
    match table_name {
        None => {
            let table_name = input.ident.to_string().to_table_case();
            pluralizer::pluralize(table_name.as_str(), 2, false)
        }
        Some(table_name) => table_name,
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum PrimaryKeyType {
    Generated,
    Manual,
}

/// Gets the [PrimaryKeyType] for a struct.
///
/// Uses `#[lorm(pk_type = "...")]` is specified, otherwise ith defaults to [PrimaryKeyType::Generated].
pub(crate) fn get_primary_key_type(input: &DeriveInput) -> PrimaryKeyType {
    let primary_key_type = get_string_attribute_by_key(&input.attrs, "lorm", "pk_type");
    if primary_key_type.is_none() {
        return PrimaryKeyType::Generated;
    }
    match primary_key_type.unwrap().as_str() {
        "generated" => PrimaryKeyType::Generated,
        "manual" => PrimaryKeyType::Manual,
        other => panic!(
            "Invalid primary key type: {}. Valid types are: generated, manual.",
            other
        ),
    }
}

pub(crate) fn get_primary_key_by_ident(input: &DeriveInput) -> Ident {
    let ident = get_ident_attribute_by_key(&input.attrs, "lorm", "pk_by");
    ident.unwrap_or(format_ident!("by_key"))
}

/// Gets the database column name for a field.
///
/// Uses the `#[sqlx(rename="...")]` attribute if specified, otherwise keeps the field name as is (sqlx default).
/// It does not support `#[sqlx(rename_all = "...")]`.
pub fn get_column_name(field: &Field) -> String {
    get_string_attribute_by_key(&field.attrs, "sqlx", "rename")
        .unwrap_or_else(|| field.ident.as_ref().unwrap().to_string())
}

/// Creates SQL placeholders for INSERT statements.
///
/// Generates database-specific placeholders: `"$1, $2, $3"` for PostgreSQL/SQLite or `"?, ?, ?"` for MySQL.
pub(crate) fn create_insert_placeholders(fields: &[UpsertField]) -> String {
    fields
        .iter()
        .flat_map(|f| f.column_names().into_iter().map(|_| f.base()))
        .enumerate()
        .map(|(i, f)| db_placeholder(f, i + 1).unwrap())
        .collect::<Vec<_>>()
        .join(",")
}

/// Generates a database-specific placeholder for a single field.
///
/// Returns `"$n"` for PostgreSQL/SQLite or `"?"` for MySQL, where n is the index.
pub(crate) fn db_placeholder(field: &Field, index: usize) -> syn::Result<String> {
    if cfg!(feature = "postgres") || cfg!(feature = "sqlite") {
        Ok(format!("${}", index))
    } else if cfg!(feature = "mysql") {
        Ok("?".to_string())
    } else {
        Err(syn::Error::new(
            field.span(),
            "Unsupported database type. Valid databases are: postgres, mysql, sqlite.",
        ))
    }
}

/// Generates the SQLx executor type token based on the enabled database feature.
///
/// Returns `PgExecutor`, `SqliteExecutor`, or `MysqlExecutor` depending on which feature is enabled.
pub(crate) fn executor_type(input: &DeriveInput) -> syn::Result<TokenStream> {
    if cfg!(feature = "postgres") {
        Ok(quote!(sqlx::PgExecutor<'e>))
    } else if cfg!(feature = "sqlite") {
        Ok(quote!(sqlx::SqliteExecutor<'e>))
    } else if cfg!(feature = "mysql") {
        Ok(quote!(sqlx::MysqlExecutor<'e>))
    } else {
        Err(syn::Error::new(
            input.span(),
            "Unsupported database type. Valid databases are: postgres, mysql, sqlite.",
        ))
    }
}

/// Generates the SQLx database type token based on the enabled database feature.
///
/// Returns `Postgres`, `Sqlite`, or `Mysql` depending on which feature is enabled.
pub(crate) fn database_type(input: &DeriveInput) -> syn::Result<TokenStream> {
    if cfg!(feature = "postgres") {
        Ok(quote!(sqlx::Postgres))
    } else if cfg!(feature = "sqlite") {
        Ok(quote!(sqlx::Sqlite))
    } else if cfg!(feature = "mysql") {
        Ok(quote!(sqlx::Mysql))
    } else {
        Err(syn::Error::new(
            input.span(),
            "Unsupported database type. Valid databases are: postgres, mysql, sqlite.",
        ))
    }
}

/// Checks whether a type is `String` or `str`.
fn is_string_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            let type_name = segment.ident.to_string();
            matches!(type_name.as_str(), "String" | "str")
        } else {
            false
        }
    } else {
        false
    }
}

/// Generates the type constraints for the `where` clause needed for binding a field value to SQLx queries.
///
/// These constraints are that the field type implements `sqlx::Encode` and `sqlx::Type` for the specific database type.
/// Field types that are wrapped with an [Option] are stripped of it.
pub(crate) fn get_bind_type_where_constraint(
    field: &Field,
    database_type: &TokenStream,
    encode_lifetime: &TokenStream,
) -> syn::Result<TokenStream> {
    let base_type = to_column_type(&field.ty)?;
    let constraints = vec![
        quote! {sqlx::Encode<#encode_lifetime, #database_type>},
        quote! {sqlx::Type<#database_type>},
    ];
    let col_type = if is_primitive_type(&base_type) {
        base_type
    } else {
        parse(quote::quote! { &#encode_lifetime #base_type }.into())?
    };
    let x = quote! {#col_type: #(#constraints)+*};
    Ok(x)
}

/// Generates the type for the bind parameter and its usage.
///
/// For primitive types and stringy types (String and str), the parameter is of type `impl Into<...>`, for all other types it uses `impl Borrow<...>`.
pub(crate) fn get_bind_param_type_and_usage(
    param: &TokenStream,
    field: &Field,
    encode_lifetime: &TokenStream,
) -> syn::Result<(TokenStream, TokenStream)> {
    let base_type = to_column_type(&field.ty)?;
    let res = if is_primitive_type(&base_type) {
        (
            quote! {impl std::convert::Into<#base_type>},
            quote! {#param.into()},
        )
    } else {
        (
            quote! {&#encode_lifetime (impl std::borrow::Borrow<#base_type> + ?Sized)},
            quote! {#param.borrow()},
        )
    };
    Ok(res)
}
