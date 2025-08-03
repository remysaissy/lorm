use crate::util::*;
use inflector::Inflector;
use quote::{__private::TokenStream, ToTokens, quote};
use syn::spanned::Spanned;
use syn::{DeriveInput, Expr, Field};

/// transient field
pub(crate) fn is_transient(field: &Field) -> bool {
    has_attribute_value(&field.attrs, "lorm", "transient")
        | has_attribute_value(&field.attrs, "sqlx", "skip")
}

/// readonly field
pub(crate) fn is_readonly(field: &Field) -> bool {
    has_attribute_value(&field.attrs, "lorm", "readonly")
}

/// primary key field
pub(crate) fn is_pk(field: &Field) -> bool {
    has_attribute_value(&field.attrs, "lorm", "pk")
}

/// by field
pub(crate) fn is_by(field: &Field) -> bool {
    has_attribute_value(&field.attrs, "lorm", "by")
}

/// created_at field
pub(crate) fn is_created_at(field: &Field) -> bool {
    has_attribute_value(&field.attrs, "lorm", "created_at")
}

/// updated_at field
pub(crate) fn is_updated_at(field: &Field) -> bool {
    has_attribute_value(&field.attrs, "lorm", "updated_at")
}

/// new_method
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

/// is_set check
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

/// table_name
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

/// field_name if rename
pub(crate) fn get_field_name(field: &Field) -> String {
    get_attribute_by_key(&field.attrs, "lorm", "rename")
        .unwrap_or_else(|| field.ident.as_ref().unwrap().to_string().to_snake_case())
}

// make string "?, ?, ?" or "$1, $2, $3"
pub(crate) fn create_insert_placeholders(fields: &[&Field]) -> String {
    fields
        .iter()
        .enumerate()
        .map(|(i, f)| db_placeholder(f, i + 1).unwrap())
        .collect::<Vec<_>>()
        .join(",")
}

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

/// Emits the pool type
pub(crate) fn executor_type(input: &DeriveInput) -> syn::Result<TokenStream> {
    if cfg!(feature = "postgres") {
        Ok(quote!(sqlx::PgExecutor<'e>))
        // Ok(quote!(sqlx::PgPool))
    } else if cfg!(feature = "sqlite") {
        Ok(quote!(sqlx::SqliteExecutor<'e>))
        // Ok(quote!(sqlx::SqlitePool))
    } else if cfg!(feature = "mysql") {
        Ok(quote!(sqlx::MysqlExecutor<'e>))
        // Ok(quote!(sqlx::MysqlPool))
    } else {
        Err(syn::Error::new(
            input.span(),
            "Unsupported database type. Valid databases are: postgres, mysql, sqlite.",
        ))
    }
}
