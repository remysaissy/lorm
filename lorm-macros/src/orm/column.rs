use crate::attributes::ColumnProperties;
use quote::ToTokens;
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
    /// Whether a `by_*`, `with_*` or selector function should be generated for this column.
    ///
    /// Such a selector should be generated if any of the
    /// `#[lorm(by)]`, `#[lorm(created_at)]` and `#[lorm(updated_at)]` attributes are present
    /// or if it is the only field making up the primary key.
    pub(crate) fn should_generate_query_function(&self) -> bool {
        self.column_properties.generate_by
            || self.column_properties.created_at
            || self.column_properties.updated_at
            || self.column_properties.primary_key
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
