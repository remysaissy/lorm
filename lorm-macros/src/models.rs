use crate::attributes::ColumnProperties;
use crate::attributes::FieldAttributes;
use crate::attributes::FieldProperties;
use crate::attributes::TableAttributes;
use crate::orm::column::Column;
use crate::utils::is_option_wrapped;
use darling::FromDeriveInput;
use darling::FromField;
use quote::ToTokens;
use quote::quote;
use syn::parse;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
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

        let created_at_columns = columns
            .iter()
            .filter(|c| c.column_properties.created_at)
            .count();
        if created_at_columns > 1 {
            return Err(syn::Error::new(
                input.ident.span(),
                "Only one field can hold the #[lorm(created_at)] attribute",
            ));
        }
        let updated_at_columns = columns
            .iter()
            .filter(|c| c.column_properties.updated_at)
            .count();
        if updated_at_columns > 1 {
            return Err(syn::Error::new(
                input.ident.span(),
                "Only one field can hold the #[lorm(updated_at)] attribute",
            ));
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
    let field_attrs = FieldAttributes::from_field(field)?;

    let has_sqlx_flatten = field_attrs.has_sqlx_flatten();
    let has_lorm_flattened = field_attrs.has_lorm_flattened();

    if has_sqlx_flatten || has_lorm_flattened {
        // Both attributes must be present together
        if has_sqlx_flatten && !has_lorm_flattened {
            return Err(syn::Error::new(
                field.span(),
                "#[sqlx(flatten)] requires a matching #[lorm(flattened(field: Type, ...))] attribute",
            ));
        }
        if !has_sqlx_flatten && has_lorm_flattened {
            return Err(syn::Error::new(
                field.span(),
                "#[lorm(flattened(...))] requires a matching #[sqlx(flatten)] attribute",
            ));
        }

        // Reject incompatible parent attributes
        if field_attrs.is_primary_key() {
            return Err(syn::Error::new(
                field.span(),
                "A flattened field cannot be the primary key.",
            ));
        }
        if field_attrs.is_created_at_field() {
            return Err(syn::Error::new(
                field.span(),
                "A flattened field cannot be #[lorm(created_at)].",
            ));
        }
        if field_attrs.is_updated_at_field() {
            return Err(syn::Error::new(
                field.span(),
                "A flattened field cannot be #[lorm(updated_at)].",
            ));
        }

        if field_attrs.is_skip() {
            return Ok(()); // Parent skipped → skip all nested fields
        }

        let generate_by = field_attrs.flatten_generate_by();
        let readonly = field_attrs.flatten_readonly();
        let flattened_fields = field_attrs.take_flattened_fields();
        let parent_is_option = is_option_wrapped(&field.ty);
        for entry in flattened_fields.fields {
            let ty = if parent_is_option {
                let inner = entry.ty;
                syn::parse2(quote! { Option<#inner> })?
            } else {
                entry.ty
            };
            let col_props = ColumnProperties {
                skip: false,
                readonly,
                primary_key: false,
                generate_by,
                created_at: false,
                updated_at: false,
                new_expression: syn::parse_str("Default::default()").unwrap(),
                is_set_expression: None,
                use_json: false,
            };

            columns.push(Column {
                base_field: field,
                field: entry.ident,
                ty,
                is_flattened: true,
                column_name: entry.column_name,
                column_properties: col_props,
            });
        }

        return Ok(());
    }

    let properties = FieldProperties::from(field, field_attrs)?;

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
