use inflector::Inflector;
use quote::{__private::TokenStream, ToTokens, quote};
use syn::spanned::Spanned;
use syn::{DeriveInput, Expr, Field, LitStr, PathArguments, Type, TypeReference, parse};

/// `#[name(value)]` attribute value exist or not
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

/// `#[name(key="val")]` Get the value of the name attribute by key
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

/// whether a type is a primitive one.
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

/// returns the type without reference, stripping it from an eventual Option<>
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

/// returns the type as a reference, stripping it from an eventual Option<>
pub(crate) fn get_type_as_reference(ty: &Type) -> syn::Result<Type> {
    let base_type = get_type_without_reference(ty)?;

    if is_primitive_type(&base_type) {
        Ok(base_type)
    } else {
        Ok(Type::Reference(TypeReference {
            and_token: Default::default(),
            lifetime: None,
            mutability: None,
            elem: Box::new(base_type),
        }))
    }
}

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
pub fn get_field_name(field: &Field) -> String {
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

/// Emits the executor type
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

/// Emits the database type
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

pub(crate) fn get_bind_type_constraint(
    field: &Field,
    database_type: &TokenStream,
) -> syn::Result<TokenStream> {
    let field_type = get_type_without_reference(&field.ty)?;
    if is_primitive_type(&field.ty) {
        Ok(quote! { 'e + sqlx::Encode<'e, #database_type> + sqlx::Type<#database_type> })
    } else {
        let as_ref = if is_string_type(&field_type) || is_primitive_type(&field_type) {
            quote! { std::convert::Into<#field_type> }
        } else {
            quote! { std::convert::AsRef<#field_type> }
        };
        Ok(quote! { 'e + sqlx::Encode<'e, #database_type> + sqlx::Type<#database_type> + #as_ref })
    }
}
