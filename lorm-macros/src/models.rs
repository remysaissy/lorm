use crate::utils::{
    get_field_name, get_table_name, is_by, is_created_at, is_pk, is_readonly, is_skip,
    is_updated_at,
};
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{DeriveInput, Field, Ident, Visibility};

pub(crate) struct OrmModel<'a> {
    pub(crate) struct_name: &'a Ident,
    pub(crate) struct_visibility: &'a Visibility,
    pub(crate) table_name: String,
    pub(crate) by_fields: Vec<&'a Field>,
    pub(crate) update_fields: Vec<&'a Field>,
    pub(crate) insert_fields: Vec<&'a Field>,
    pub(crate) table_columns: String,
    pub(crate) pk_field: &'a Field,
    pub(crate) is_pk_readonly: bool,
    pub(crate) created_at_field: Option<&'a Field>,
    pub(crate) is_created_at_readonly: bool,
    pub(crate) updated_at_field: Option<&'a Field>,
    pub(crate) is_updated_at_readonly: bool,
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
        let mut update_fields: Vec<&Field> = vec![];
        let mut insert_fields: Vec<&Field> = vec![];
        let mut table_columns_vec: Vec<String> = vec![];
        let mut pk_field: Option<&Field> = None;
        let mut is_pk_readonly = false;
        let mut created_at_field: Option<&Field> = None;
        let mut is_created_at_readonly = false;
        let mut updated_at_field: Option<&Field> = None;
        let mut is_updated_at_readonly = false;

        for field in fields.iter() {
            if !is_skip(field) {
                table_columns_vec.push(get_field_name(field));
                if is_pk(field) {
                    pk_field = Some(field);
                    if is_readonly(field) {
                        is_pk_readonly = true;
                    }
                }
                if is_created_at(field) {
                    created_at_field = Some(field);
                    if is_readonly(field) {
                        is_created_at_readonly = true;
                    }
                }
                if is_updated_at(field) {
                    updated_at_field = Some(field);
                    if is_readonly(field) {
                        is_updated_at_readonly = true;
                    }
                }
                if is_by(field) || is_pk(field) || is_created_at(field) || is_updated_at(field) {
                    by_fields.push(field);
                }
                if !is_readonly(field) {
                    insert_fields.push(field);
                    update_fields.push(field);
                }
            }
        }
        let pk_field = match pk_field {
            Some(field) => field,
            None => {
                return Err(syn::Error::new(
                    input.ident.span(),
                    "expected a primary key using #[lorm(pk)] attribute on a field",
                ));
            }
        };

        Ok(Self {
            struct_name,
            struct_visibility,
            table_name,
            by_fields,
            update_fields,
            insert_fields,
            table_columns: table_columns_vec.join(","),
            pk_field,
            is_pk_readonly,
            created_at_field,
            is_created_at_readonly,
            updated_at_field,
            is_updated_at_readonly,
        })
    }
}
