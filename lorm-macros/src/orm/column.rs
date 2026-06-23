use crate::attributes::ColumnProperties;
use crate::utils::is_option_wrapped;
use quote::__private::TokenStream;
use quote::{ToTokens, quote};
use syn::Field;
use syn::Ident;
use syn::Type;
use syn::parse;

pub(crate) struct Column<'a> {
    pub(crate) base_field: &'a Field,
    pub(crate) field: Ident,
    pub(crate) ty: Type,
    pub(crate) column_name: String,
    pub(crate) is_flattened: bool,
    pub(crate) column_properties: ColumnProperties,
    pub(crate) belongs_to: Option<crate::attributes::RelationTarget>,
}

impl<'a> Column<'a> {
    /// Generate the token stream to access the field on `self`.
    pub(crate) fn self_accessor(&self) -> TokenStream {
        let base_ident = self.base_field.ident.as_ref().unwrap();
        if self.is_flattened {
            let field_ident = &self.field;
            if is_option_wrapped(&self.base_field.ty) {
                quote! {self.#base_ident.as_ref().map(|base| &base.#field_ident)}
            } else {
                quote! {&self.#base_ident.#field_ident}
            }
        } else {
            quote! {&self.#base_ident}
        }
    }

    /// Whether a `by_*`, `with_*` or selector function should be generated for this column.
    ///
    /// Such a selector should be generated if any of the
    /// `#[lorm(by)]`, `#[lorm(created_at)]` and `#[lorm(updated_at)]` attributes are present,
    /// or if it is part of a generated primary key.
    ///
    /// For manual primary keys (including composite), we do not auto-generate `by_<field>` methods
    /// for the pk fields unless they also have explicit `#[lorm(by)]`.
    pub(crate) fn should_generate_query_function(&self, pk_is_generated: bool) -> bool {
        self.column_properties.generate_by
            || self.column_properties.created_at
            || self.column_properties.updated_at
            || (self.column_properties.primary_key && pk_is_generated)
    }
}

impl<'a> Clone for Column<'a> {
    fn clone(&self) -> Self {
        Column {
            base_field: self.base_field,
            field: self.field.clone(),
            ty: parse((&self.ty).into_token_stream().into()).unwrap(),
            column_name: self.column_name.clone(),
            is_flattened: self.is_flattened,
            column_properties: self.column_properties.clone(),
            belongs_to: self.belongs_to.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attributes::ColumnProperties;
    use syn::parse_str;

    fn make_col_with_props(props: ColumnProperties) -> Column<'static> {
        let item: syn::ItemStruct = parse_str("struct S { pub f: i32 }").unwrap();
        let field = item.fields.iter().next().unwrap().clone();
        let field_ref: &'static syn::Field = Box::leak(Box::new(field));
        Column {
            base_field: field_ref,
            field: parse_str::<syn::Ident>("f").unwrap(),
            ty: parse_str("i32").unwrap(),
            column_name: "f".to_string(),
            is_flattened: false,
            column_properties: props,
            belongs_to: None,
        }
    }

    fn default_props() -> ColumnProperties {
        ColumnProperties {
            skip: false,
            readonly: false,
            primary_key: false,
            generate_by: false,
            created_at: false,
            updated_at: false,
            new_expression: parse_str("Default::default()").unwrap(),
            is_set_expression: None,
            use_json: false,
            belongs_to_target: None,
        }
    }

    #[test]
    fn should_generate_query_function_false_for_plain_field() {
        let col = make_col_with_props(default_props());
        assert!(!col.should_generate_query_function(true)); // kills true mutation
        assert!(!col.should_generate_query_function(false));
    }

    #[test]
    fn should_generate_query_function_true_for_generate_by() {
        let mut p = default_props();
        p.generate_by = true;
        let col = make_col_with_props(p);
        assert!(col.should_generate_query_function(false)); // kills false mutation, kills || → &&
    }

    #[test]
    fn should_generate_query_function_true_for_created_at() {
        let mut p = default_props();
        p.created_at = true;
        let col = make_col_with_props(p);
        assert!(col.should_generate_query_function(false)); // kills || → && at line 47
    }

    #[test]
    fn should_generate_query_function_true_for_updated_at() {
        let mut p = default_props();
        p.updated_at = true;
        let col = make_col_with_props(p);
        assert!(col.should_generate_query_function(false)); // kills || → && at line 48
    }

    #[test]
    fn should_generate_query_function_true_for_generated_pk_only_when_flag_set() {
        let mut p = default_props();
        p.primary_key = true;
        let col = make_col_with_props(p);
        assert!(col.should_generate_query_function(true)); // pk + generated → true
        assert!(!col.should_generate_query_function(false)); // pk but manual → false — kills && → || at line 48
    }
}
