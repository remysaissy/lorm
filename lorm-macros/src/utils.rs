use inflector::Inflector;
use quote::{__private::TokenStream, ToTokens, quote};
use syn::spanned::Spanned;
use syn::{DeriveInput, Expr, Field, LitStr, PathArguments, Type, parse};

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

/// Gets the value of an attribute by its key.
///
/// For example, extracts `"users"` from `#[lorm(rename="users")]` when called with
/// `name="lorm"` and `key="rename"`.
pub(crate) fn get_attribute_by_key(
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

/// Returns the type without reference, unwrapping it from an `Option<>` if present.
///
/// For example, `Option<&String>` becomes `String`, and `&i32` becomes `i32`.
pub(crate) fn get_type_without_reference(ty: &Type) -> syn::Result<Type> {
    match ty {
        Type::Path(type_path) => {
            let last_segment = type_path
                .path
                .segments
                .last()
                .expect("Type path should have at least one segment");
            let ident = &last_segment.ident;

            // Check for Option types and recurse
            if ident == "Option" {
                if let PathArguments::AngleBracketed(angle_bracketed) = &last_segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_type)) =
                        angle_bracketed.args.first()
                    {
                        return get_type_without_reference(inner_type);
                    }
                }
            }

            // Always return the type without reference
            parse(ty.into_token_stream().into())
        }
        _ => parse(ty.into_token_stream().into()),
    }
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
    match get_attribute_by_key(&field.attrs, "lorm", "new") {
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
    match get_attribute_by_key(&field.attrs, "lorm", "is_set") {
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
    let table_name = get_attribute_by_key(&input.attrs, "lorm", "rename");
    match table_name {
        None => {
            let table_name = input.ident.to_string().to_table_case();
            pluralizer::pluralize(table_name.as_str(), 2, false)
        }
        Some(table_name) => table_name,
    }
}

/// Gets the database column name for a field.
///
/// Uses the `#[lorm(rename="...")]` attribute if specified, otherwise converts the field name
/// to snake_case (e.g., `userId` becomes `user_id`).
pub fn get_field_name(field: &Field) -> String {
    get_attribute_by_key(&field.attrs, "lorm", "rename")
        .unwrap_or_else(|| field.ident.as_ref().unwrap().to_string().to_snake_case())
}

/// Creates SQL placeholders for INSERT statements.
///
/// Generates database-specific placeholders: `"$1, $2, $3"` for PostgreSQL/SQLite or `"?, ?, ?"` for MySQL.
pub(crate) fn create_insert_placeholders(fields: &[&Field]) -> String {
    fields
        .iter()
        .enumerate()
        .map(|(i, f)| db_placeholder(f, i + 1).unwrap())
        .collect::<Vec<_>>()
        .join(",")
}

/// Creates SQL placeholders for UPDATE statements.
///
/// Generates database-specific SET clauses: `"name = $1, email = $2"` for PostgreSQL/SQLite
/// or `"name = ?, email = ?"` for MySQL.
pub(crate) fn create_update_placeholders(fields: &[&Field]) -> String {
    fields
        .iter()
        .enumerate()
        .map(|(i, f)| {
            format!(
                "{} = {}",
                get_field_name(f),
                db_placeholder(f, i + 1).unwrap()
            )
        })
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

/// Generates the type constraints needed for binding a field value to SQLx queries.
///
/// Returns different trait bounds depending on whether the field is a primitive type or complex type.
/// For primitives, uses `Encode` and `Type` traits. For complex types, also includes `Into` or `AsRef`.
pub(crate) fn get_bind_type_constraint(
    field: &Field,
    database_type: &TokenStream,
) -> syn::Result<TokenStream> {
    let field_type = get_type_without_reference(&field.ty)?;
    if is_primitive_type(&field.ty) {
        Ok(quote! { 'static + sqlx::Encode<'static, #database_type> + sqlx::Type<#database_type> })
    } else {
        let as_ref = if is_string_type(&field_type) || is_primitive_type(&field_type) {
            quote! { std::convert::Into<#field_type> }
        } else {
            quote! { std::convert::AsRef<#field_type> }
        };
        Ok(
            quote! { 'static + sqlx::Encode<'static, #database_type> + sqlx::Type<#database_type> + #as_ref },
        )
    }
}
