use crate::orm::logical_field::LogicalField;
use proc_macro2::Span;
use quote::{__private::TokenStream, ToTokens, quote};
use syn::spanned::Spanned;
use syn::{DeriveInput, PathArguments, Type, parse};

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

/// Check whether the provided type is an [Option]
pub(crate) fn is_option_wrapped(ty: &Type) -> bool {
    match ty {
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
                true
            } else {
                false
            }
        }
        _ => false,
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

/// Creates SQL placeholders for INSERT statements.
///
/// Generates database-specific placeholders: `"$1, $2, $3"` for PostgreSQL/SQLite or `"?, ?, ?"` for MySQL.
pub(crate) fn create_insert_placeholders(fields: &[&LogicalField]) -> String {
    fields
        .iter()
        .enumerate()
        .map(|(i, f)| db_placeholder(f.base_field.span(), i + 1).unwrap())
        .collect::<Vec<_>>()
        .join(",")
}

/// Generates a database-specific placeholder for a single field.
///
/// Returns `"$n"` for PostgreSQL/SQLite or `"?"` for MySQL, where n is the index.
pub(crate) fn db_placeholder(span: Span, index: usize) -> syn::Result<String> {
    if cfg!(feature = "postgres") || cfg!(feature = "sqlite") {
        Ok(format!("${}", index))
    } else if cfg!(feature = "mysql") {
        Ok("?".to_string())
    } else {
        Err(syn::Error::new(
            span,
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

/// Generates the type constraints for the `where` clause needed for binding a field value to SQLx queries.
///
/// These constraints are that the field type implements `sqlx::Encode` and `sqlx::Type` for the specific database type.
/// Field types that are wrapped with an [Option] are stripped of it.
pub(crate) fn get_bind_type_where_constraint(
    ty: &Type,
    database_type: &TokenStream,
    encode_lifetime: &TokenStream,
) -> syn::Result<TokenStream> {
    let base_type = to_column_type(ty)?;
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
    ty: &Type,
    encode_lifetime: &TokenStream,
) -> syn::Result<(TokenStream, TokenStream)> {
    let base_type = to_column_type(ty)?;
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
