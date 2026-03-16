use crate::attributes::{
    FieldAttributes, FieldProperties, FlattenedField, PrimaryKeyType, TableAttributes,
};
use crate::orm::logical_field::LogicalField;
use darling::{FromDeriveInput, FromField};
use proc_macro_error2::emit_error;
use quote::ToTokens;
use std::slice;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::{DeriveInput, Field, Ident, Visibility, parse};

pub(crate) enum PrimaryKey<'a> {
    Generated(Box<LogicalField<'a>>),
    Manual(Vec<LogicalField<'a>>),
}

impl<'a> PrimaryKey<'a> {
    pub fn is_generated(&self) -> bool {
        matches!(self, PrimaryKey::Generated(_))
    }

    pub fn fields(&'a self) -> &'a [LogicalField<'a>] {
        match self {
            PrimaryKey::Generated(field) => slice::from_ref(field),
            PrimaryKey::Manual(fields) => fields,
        }
    }

    pub fn column_names(&self) -> impl Iterator<Item = &str> {
        self.fields().iter().map(|f| f.column_name.as_str())
    }

    fn from_type_and_fields(
        input: &'a DeriveInput,
        key_type: PrimaryKeyType,
        mut fields: Vec<LogicalField<'a>>,
    ) -> syn::Result<Self> {
        match key_type {
            PrimaryKeyType::Generated => {
                let error = "For generated primary keys, exactly one field must have the #[lorm(pk)] attribute.";
                let field = fields
                    .pop()
                    .ok_or_else(|| syn::Error::new(input.ident.span(), error))?;
                if !fields.is_empty() {
                    return Err(syn::Error::new(field.base_field.span(), error));
                }

                Ok(PrimaryKey::Generated(Box::new(field)))
            }
            PrimaryKeyType::Manual => {
                if fields.is_empty() {
                    emit_error!(input.span(), "At least one field must be marked with the #[lorm(pk)] attribute to form a manual primary key.");
                }
                Ok(PrimaryKey::Manual(fields))
            },
        }
    }
}

pub(crate) struct OrmModel<'a> {
    pub(crate) struct_name: &'a Ident,
    pub(crate) struct_visibility: &'a Visibility,
    pub(crate) table_name: String,
    pub(crate) fields: Vec<LogicalField<'a>>,
    pub(crate) primary_key: PrimaryKey<'a>,
    pub(crate) primary_key_selector: Ident,
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
        let mut logical_fields = vec![];
        let pk_type = top_level_attributes.pk_type;

        for field in fields.iter() {
            logical_fields = process_struct_field(field, logical_fields, pk_type)?;
        }

        let created_at_fields = logical_fields
            .iter()
            .filter(|field| field.column_properties.created_at)
            .collect::<Vec<_>>();
        if created_at_fields.len() > 1 {
            let first = created_at_fields[0].base_field.span();
            let second = created_at_fields[1].base_field.span();
            let joined = first.join(second).unwrap();
            emit_error!(
                joined,
                "Only one field can hold the #[lorm(created_at)] attribute."
            );
        }

        let updated_at_fields = logical_fields
            .iter()
            .filter(|field| field.column_properties.updated_at)
            .collect::<Vec<_>>();
        if updated_at_fields.len() > 1 {
            let first = updated_at_fields[0].base_field.span();
            let second = updated_at_fields[1].base_field.span();
            let joined = first.join(second).unwrap();
            emit_error!(
                joined,
                "Only one field can hold the #[lorm(created_at)] attribute."
            );
        }

        let pk_fields = logical_fields
            .iter_mut()
            .filter(|f| f.column_properties.primary_key)
            .collect::<Vec<_>>();
        let pk_fields = pk_fields.into_iter().map(|f| f.clone()).collect();
        let primary_key = PrimaryKey::from_type_and_fields(input, pk_type, pk_fields)?;

        let primary_key_selector = top_level_attributes.manual_primary_key_selector(&primary_key);

        Ok(Self {
            struct_name,
            struct_visibility,
            table_name,
            fields: logical_fields,
            primary_key,
            primary_key_selector,
        })
    }

    pub(crate) fn full_select_columns(&self) -> String {
        self.fields
            .iter()
            .map(|f| f.column_name.as_str())
            .collect::<Vec<_>>()
            .join(", ")
    }
}

fn process_struct_field<'a>(
    field: &'a Field,
    mut fields: Vec<LogicalField<'a>>,
    pk_type: PrimaryKeyType,
) -> syn::Result<Vec<LogicalField<'a>>> {
    let properties = FieldProperties::from(field, FieldAttributes::from_field(field)?, pk_type);

    if properties.column_properties.skip {
        return Ok(fields);
    }

    let logical_fields: Box<dyn Iterator<Item = LogicalField<'a>>> =
        match properties.flattened_fields {
            Some(flattened) => Box::new(flattened.into_iter().map(
                |FlattenedField {
                     field_name,
                     ty,
                     column_name,
                 }| LogicalField {
                    base_field: field,
                    field: field_name,
                    ty,
                    column_name,
                    is_flattened: true,
                    column_properties: properties.column_properties.clone(),
                },
            )),
            None => {
                let column_name = properties.column_name;

                let logical_field = LogicalField {
                    base_field: field,
                    field: field.ident.clone().unwrap(),
                    ty: parse((&field.ty).into_token_stream().into())?,
                    is_flattened: false,
                    column_name,
                    column_properties: properties.column_properties,
                };

                Box::new(Some(logical_field).into_iter())
            }
        };

    for logical_field in logical_fields {
        fields.push(logical_field);
    }

    Ok(fields)
}
