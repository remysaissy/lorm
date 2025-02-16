use syn::{LitStr, Type};

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

/// whether `Option<inner_type>` returns (whether Option, inner_type).
pub(crate) fn get_option_type(ty: &Type) -> (bool, &Type) {
    get_inner_type(ty, "Option")
}

/// whether inner_type,such as: Option<String>,Vec<String>
/// returns (whether, inner_type).
pub(crate) fn get_inner_type<'a>(ty: &'a Type, name: &str) -> (bool, &'a Type) {
    if let syn::Type::Path(ref path) = ty {
        if let Some(segment) = path.path.segments.first() {
            if segment.ident == name {
                if let syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                    args,
                    ..
                }) = &segment.arguments
                {
                    if let Some(syn::GenericArgument::Type(ty)) = args.first() {
                        return (true, ty);
                    }
                }
            }
        }
    }
    (false, ty)
}
