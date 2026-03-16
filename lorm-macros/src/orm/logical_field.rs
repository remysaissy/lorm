use crate::attributes::ColumnProperties;
use crate::models::PrimaryKey;
use crate::utils::is_option_wrapped;
use proc_macro2::{Ident, TokenStream};
use quote::{ToTokens, quote};
use syn::{Field, Type, parse};

pub(crate) struct LogicalField<'a> {
    pub(crate) base_field: &'a Field,
    pub(crate) field: Ident,
    pub(crate) ty: Type,
    pub(crate) column_name: String,
    pub(crate) is_flattened: bool,
    pub(crate) column_properties: ColumnProperties,
}

impl<'a> LogicalField<'a> {
    pub(crate) fn self_accessor(&self) -> TokenStream {
        let base_ident = self.base_field.ident.as_ref().unwrap();
        if self.is_flattened {
            let field_ident = &self.field;
            if is_option_wrapped(&self.base_field.ty) {
                quote! {#base_ident.map(|base| &base.#field_ident)}
            } else {
                quote! {#base_ident.#field_ident}
            }
        } else {
            quote! {#base_ident}
        }
    }

    /// Whether a `by_*`, `with_*` or selector function should be generated for this column.
    ///
    /// Such a selector should be generated if any of the
    /// `#[lorm(by)]`, `#[lorm(created_at)]` and `#[lorm(updated_at)]` attributes are present
    /// or if it is the only field making up the primary key.
    pub(crate) fn should_generate_selector(&self, pk: &PrimaryKey) -> bool {
        if self.column_properties.generate_by
            || self.column_properties.created_at
            || self.column_properties.updated_at
        {
            true
        } else {
            pk.fields().len() == 1 && self.column_properties.primary_key
        }
    }
}

impl<'a> Clone for LogicalField<'a> {
    fn clone(&self) -> Self {
        LogicalField {
            base_field: self.base_field,
            field: self.field.clone(),
            ty: parse((&self.ty).into_token_stream().into()).unwrap(),
            column_name: self.column_name.clone(),
            is_flattened: self.is_flattened,
            column_properties: self.column_properties.clone(),
        }
    }
}
