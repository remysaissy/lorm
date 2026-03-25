use crate::attributes::FieldAttributes;
use crate::attributes::FieldProperties;
use crate::attributes::TableAttributes;
use crate::orm::column::Column;
use darling::FromDeriveInput;
use darling::FromField;
use quote::ToTokens;
use syn::parse;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{DeriveInput, Field, Ident, Visibility};

pub(crate) struct OrmModel<'a> {
    pub(crate) struct_name: &'a Ident,
    pub(crate) struct_visibility: &'a Visibility,
    pub(crate) table_name: String,
    pub(crate) columns: Vec<Column<'a>>,
}

impl<'a> OrmModel<'a> {
    pub(crate) fn from_fields(
        input: &'a DeriveInput,
        fields: &'a Punctuated<Field, Comma>,
    ) -> syn::Result<Self> {
        let top_level_attributes = TableAttributes::from_derive_input(input)?;

        let struct_name = &input.ident;
        let struct_visibility = &input.vis;
        let table_name = top_level_attributes.table_name(input);

        let mut columns = Vec::new();

        for field in fields.iter() {
            process_struct_field(field, &mut columns)?;
        }

        let pk_columns = columns
            .iter()
            .filter(|c| c.column_properties.primary_key)
            .collect::<Vec<_>>();
        if pk_columns.len() != 1 {
            return Err(syn::Error::new(
                input.ident.span(),
                "expected exactly one primary key using #[lorm(pk)] attribute",
            ));
        }

        Ok(Self {
            struct_name,
            struct_visibility,
            table_name,
            columns,
        })
    }

    pub(crate) fn query_columns(&self) -> impl Iterator<Item = &Column<'a>> {
        self.columns
            .iter()
            .filter(|c| c.should_generate_query_function())
    }

    pub(crate) fn full_column_select(&self) -> String {
        self.columns
            .iter()
            .map(|c| c.column_name.clone())
            .collect::<Vec<_>>()
            .join(", ")
    }

    pub(crate) fn primary_key(&self) -> &Column<'a> {
        self.columns
            .iter()
            .find(|c| c.column_properties.primary_key)
            .unwrap()
    }

    pub(crate) fn created_at(&self) -> Option<&Column<'a>> {
        self.columns.iter().find(|c| c.column_properties.created_at)
    }

    pub(crate) fn updated_at(&self) -> Option<&Column<'a>> {
        self.columns.iter().find(|c| c.column_properties.updated_at)
    }

    pub(crate) fn update_columns(&self) -> impl Iterator<Item = &Column<'a>> {
        self.columns
            .iter()
            .filter(|c| !c.column_properties.readonly && !c.column_properties.primary_key)
    }

    pub(crate) fn insert_columns(&self) -> impl Iterator<Item = &Column<'a>> {
        let primary_key = self.primary_key();
        let pk_column = if primary_key.column_properties.readonly {
            None
        } else {
            Some(primary_key)
        };

        pk_column.into_iter().chain(self.update_columns())
    }
}

fn process_struct_field<'a>(field: &'a Field, columns: &mut Vec<Column<'a>>) -> syn::Result<()> {
    let properties = FieldProperties::from(field, FieldAttributes::from_field(field)?)?;

    if properties.column_properties.skip {
        return Ok(());
    }

    let logical_fields: Box<dyn Iterator<Item = Column<'a>>> = {
        let column_name = properties.column_name;

        let logical_field = Column {
            base_field: field,
            field: field.ident.clone().unwrap(),
            ty: parse((&field.ty).into_token_stream().into())?,
            is_flattened: false,
            column_name,
            column_properties: properties.column_properties,
        };

        Box::new(Some(logical_field).into_iter())
    };

    for logical_field in logical_fields {
        columns.push(logical_field);
    }

    Ok(())
}
