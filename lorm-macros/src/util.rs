use quote::ToTokens;
use syn::{LitStr, PathArguments, Type, TypeReference, parse};

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

/// whether a type can be passed as reference or not.
pub fn get_type_as_reference(ty: &Type) -> syn::Result<Type> {
    match ty {
        Type::Path(type_path) => {
            let last_segment = type_path
                .path
                .segments
                .last()
                .expect("Type path should have at least one segment");
            let ident = &last_segment.ident;

            // Check for primitive types that should not be referenced
            if matches!(
                ident.to_string().as_str(),
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
            ) {
                return parse(ty.into_token_stream().into());
            }

            // Check for Option types
            if ident == "Option" {
                if let PathArguments::AngleBracketed(angle_bracketed) = &last_segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_type)) =
                        angle_bracketed.args.first()
                    {
                        return get_type_as_reference(inner_type);
                    }
                }
            }

            // Default to returning the type with a reference
            let elem = parse(ty.into_token_stream().into()).unwrap();
            Ok(Type::Reference(TypeReference {
                and_token: Default::default(),
                lifetime: None,
                mutability: None,
                elem: Box::new(elem),
            }))
        }
        _ => parse(ty.into_token_stream().into()),
    }
}
