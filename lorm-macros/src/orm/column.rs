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
        }
    }
}
