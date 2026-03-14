use crate::utils::{
    PrimaryKeyType, get_field_name, get_primary_key_by_ident, get_primary_key_type, get_table_name,
    is_by, is_created_at, is_pk, is_readonly, is_skip, is_updated_at,
};
use proc_macro_error2::emit_warning;
use std::slice;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::{DeriveInput, Field, Ident, Visibility};

pub(crate) enum PrimaryKey<'a> {
    Generated(&'a Field),
    Manual(Vec<&'a Field>),
}

impl<'a> PrimaryKey<'a> {
    pub fn fields(&self) -> &[&'a Field] {
        match self {
            PrimaryKey::Generated(field) => slice::from_ref(field),
            PrimaryKey::Manual(fields) => fields,
        }
    }

    pub fn columns(&self) -> syn::Result<Vec<&'a Ident>> {
        Ok(self
            .fields()
            .iter()
            .map(|field| {
                field.ident.as_ref().ok_or_else(|| {
                    syn::Error::new(field.span(), "Primary key field must have an identifier.")
                })
            })
            .collect::<Result<Vec<_>, _>>()?)
    }

    pub fn column_names(&self) -> String {
        self.fields()
            .iter()
            .map(|f| get_field_name(f))
            .collect::<Vec<_>>()
            .join(",")
    }

    fn from_type_and_fields(
        input: &'a DeriveInput,
        key_type: PrimaryKeyType,
        mut fields: Vec<&'a Field>,
    ) -> syn::Result<Self> {
        match key_type {
            PrimaryKeyType::Generated => {
                let error = "For generated primary keys, exactly one field must have the #[lorm(pk)] attribute.";
                let field = fields
                    .pop()
                    .ok_or_else(|| syn::Error::new(input.ident.span(), error))?;
                if fields.len() > 0 {
                    return Err(syn::Error::new(field.span(), error));
                }

                Ok(PrimaryKey::Generated(field))
            }
            PrimaryKeyType::Manual => Ok(PrimaryKey::Manual(fields)),
        }
    }
}

pub(crate) struct OrmModel<'a> {
    pub(crate) struct_name: &'a Ident,
    pub(crate) struct_visibility: &'a Visibility,
    pub(crate) table_name: String,
    pub(crate) by_fields: Vec<&'a Field>,
    pub(crate) upsert_fields: Vec<&'a Field>,
    pub(crate) table_columns: String,
    pub(crate) primary_key: PrimaryKey<'a>,
    pub(crate) primary_key_by_name: Ident,
    pub(crate) created_at_field: Option<&'a Field>,
    pub(crate) updated_at_field: Option<&'a Field>,
}

impl<'a> OrmModel<'a> {
    pub(crate) fn from_fields(
        input: &'a DeriveInput,
        fields: &'a Punctuated<Field, Comma>,
    ) -> syn::Result<Self> {
        let struct_name = &input.ident;
        let struct_visibility = &input.vis;
        let table_name = get_table_name(input);
        let mut by_fields: Vec<&Field> = vec![];
        let mut upsert_fields: Vec<&Field> = vec![];
        let mut table_columns: Vec<String> = vec![];
        let pk_type = get_primary_key_type(input);
        let primary_key_by_name = get_primary_key_by_ident(input);
        let mut pk_fields: Vec<&Field> = vec![];
        let mut created_at_field: Option<&Field> = None;
        let mut updated_at_field: Option<&Field> = None;

        for field in fields.iter() {
            process_field(
                field,
                &mut table_columns,
                &mut pk_fields,
                &mut created_at_field,
                &mut updated_at_field,
                &mut by_fields,
                &mut upsert_fields,
            );
        }
        let primary_key = PrimaryKey::from_type_and_fields(input, pk_type, pk_fields)?;
        if primary_key.fields().len() == 1 {
            let field = primary_key.fields().first().unwrap();
            by_fields.push(field);
        }

        Ok(Self {
            struct_name,
            struct_visibility,
            table_name,
            by_fields,
            upsert_fields,
            table_columns: table_columns.join(","),
            primary_key,
            primary_key_by_name,
            created_at_field,
            updated_at_field,
        })
    }
}

fn process_field<'a>(
    field: &'a Field,
    table_columns: &mut Vec<String>,
    primary_key_fields: &mut Vec<&'a Field>,
    created_at: &mut Option<&'a Field>,
    updated_at: &mut Option<&'a Field>,
    by_fields: &mut Vec<&'a Field>,
    upsert_fields: &mut Vec<&'a Field>,
) {
    if is_skip(field) {
        return;
    }

    table_columns.push(get_field_name(field));
    if is_pk(field) {
        primary_key_fields.push(field);
    }
    if is_created_at(field) {
        let previous = created_at.replace(field);
        if previous.is_some() {
            emit_warning!(
                field.span(),
                "Multiple fields with #[lorm(created_at)] attribute found. Only the last one will be used as the created_at field."
            );
        }
    }
    if is_updated_at(field) {
        let previous = updated_at.replace(field);
        if previous.is_some() {
            emit_warning!(
                field.span(),
                "Multiple fields with #[lorm(updated_at)] attribute found. Only the last one will be used as the updated_at field."
            );
        }
    }
    if is_by(field) || is_created_at(field) || is_updated_at(field) {
        by_fields.push(field);
    }
    if !is_readonly(field) {
        upsert_fields.push(field);
    }
}
