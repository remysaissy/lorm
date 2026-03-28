use quote::{__private::TokenStream, ToTokens, quote};
use syn::spanned::Spanned;
use syn::{DeriveInput, Field, PathArguments, Type, parse};

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
            if ident == "Option"
                && let PathArguments::AngleBracketed(angle_bracketed) = &last_segment.arguments
                && let Some(syn::GenericArgument::Type(inner_type)) = angle_bracketed.args.first()
            {
                return get_type_without_reference(inner_type);
            }

            // Always return the type without reference
            parse(ty.into_token_stream().into())
        }
        _ => parse(ty.into_token_stream().into()),
    }
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
