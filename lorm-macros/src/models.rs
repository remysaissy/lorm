use crate::utils::{
    PrimaryKeyType, get_column_name, get_primary_key_by_ident, get_primary_key_type,
    get_table_name, has_attribute_value, is_by, is_created_at, is_pk, is_readonly, is_skip,
    is_updated_at,
};
use proc_macro_error2::{emit_error, emit_warning};
use std::slice;
use syn::parse::ParseStream;
use syn::punctuated::Punctuated;
use syn::spanned::Spanned;
use syn::token::Comma;
use syn::{DeriveInput, Field, Ident, LitStr, Token, Visibility};

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
        self.fields()
            .iter()
            .map(|field| {
                field.ident.as_ref().ok_or_else(|| {
                    syn::Error::new(field.span(), "Primary key field must have an identifier.")
                })
            })
            .collect::<Result<Vec<_>, _>>()
    }

    pub fn column_names(&self) -> String {
        self.fields()
            .iter()
            .map(|f| get_column_name(f))
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
                if !fields.is_empty() {
                    return Err(syn::Error::new(field.span(), error));
                }

                Ok(PrimaryKey::Generated(field))
            }
            PrimaryKeyType::Manual => Ok(PrimaryKey::Manual(fields)),
        }
    }
}

pub(crate) struct FlattenedField {
    pub(crate) field: Ident,
    pub(crate) column: String,
}

pub(crate) enum UpsertField<'a> {
    Field { field: &'a Field, use_json: bool },
    Flattened(&'a Field, Vec<FlattenedField>),
}

impl<'a> UpsertField<'a> {
    pub(crate) fn base(&self) -> &'a Field {
        match self {
            UpsertField::Field { field, .. } => field,
            UpsertField::Flattened(f, _) => f,
        }
    }

    pub(crate) fn column_names(&self) -> Vec<String> {
        match self {
            Self::Field { field, .. } => vec![get_column_name(field)],
            Self::Flattened(_, flattened_fields) => flattened_fields
                .iter()
                .map(|flattened| flattened.column.clone())
                .collect(),
        }
    }
}

pub(crate) struct OrmModel<'a> {
    pub(crate) struct_name: &'a Ident,
    pub(crate) struct_visibility: &'a Visibility,
    pub(crate) table_name: String,
    pub(crate) by_fields: Vec<&'a Field>,
    pub(crate) upsert_fields: Vec<UpsertField<'a>>,
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
        let mut by_fields = vec![];
        let mut upsert_fields = vec![];
        let mut table_columns = vec![];
        let pk_type = get_primary_key_type(input);
        let primary_key_by_name = get_primary_key_by_ident(input);
        let mut pk_fields = vec![];
        let mut created_at_field = None;
        let mut updated_at_field = None;

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
    upsert_fields: &mut Vec<UpsertField<'a>>,
) {
    if is_skip(field) {
        return;
    }

    if has_attribute_value(&field.attrs, "sqlx", "flatten") {
        let flattened_fields = get_flattened_names(&field.attrs);

        let Some(flattened_fields) = flattened_fields else {
            emit_error!(
                field.span(),
                "On structs deriving ToLOrm, fields with the #[sqlx(flatten)] attribute require the #[lorm(flattened = ...)] attribute to specify what colums the field gets flattened to.",
            );
            return;
        };

        table_columns.extend(
            flattened_fields
                .iter()
                .map(|flattened| flattened.column.to_string()),
        );
        upsert_fields.push(UpsertField::Flattened(field, flattened_fields));

        for value in ["pk", "created_at", "updated_at", "by", "readonly"] {
            if has_attribute_value(&field.attrs, "lorm", value) {
                emit_warning!(
                    field.span(),
                    "The #[lorm({value})] attribute has no effect on fields with the #[sqlx(flatten)] attribute. Remove the #[lorm({value})] attribute from this field.",
                );
            }
        }

        return;
    } else if has_attribute_value(&field.attrs, "lorm", "flattened") {
        emit_error!(
            field.span(),
            "#[lorm(flattened = ...)] should only be used on fields with the #[sqlx(flatten)] attribute. Remove the #[lorm(flattened = ...)] attribute from this field.",
        );
    }

    table_columns.push(get_column_name(field));
    if is_pk(field) {
        primary_key_fields.push(field);
    }
    if is_created_at(field) {
        let previous = created_at.replace(field);
        if let Some(previous) = previous {
            emit_error!(
                field.span(),
                "Only one field can hold the #[lorm(created_at)] attribute. Also present on {}.",
                previous
                    .ident
                    .as_ref()
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| "unnamed field".to_string())
            );
        }
    }
    if is_updated_at(field) {
        let previous = updated_at.replace(field);
        if let Some(previous) = previous {
            emit_error!(
                field.span(),
                "Only one field can hold the #[lorm(updated_at)] attribute. Also present on {}.",
                previous
                    .ident
                    .as_ref()
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| "unnamed field".to_string())
            );
        }
    }
    if is_by(field) || is_created_at(field) || is_updated_at(field) {
        by_fields.push(field);
    }
    if !is_readonly(field) {
        let use_json = has_attribute_value(&field.attrs, "sqlx", "json");

        upsert_fields.push(UpsertField::Field { field, use_json });
    }
}

impl syn::parse::Parse for FlattenedField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let field: Ident = input.parse()?;

        let column = if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            input.parse::<LitStr>()?.value()
        } else {
            field.to_string()
        };

        Ok(FlattenedField { field, column })
    }
}

fn get_flattened_names(attrs: &[syn::Attribute]) -> Option<Vec<FlattenedField>> {
    let mut val: Option<_> = None;
    for attr in attrs.iter() {
        if !attr.path().is_ident("lorm") {
            continue;
        }

        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("flattened") {
                let value = meta.value()?; // this parses the `=`

                let names;
                syn::parenthesized!(names in value);
                let idents = Punctuated::<FlattenedField, Token![,]>::parse_terminated(&names)?;
                val = Some(idents.into_iter().collect());
            }
            Err(meta.error("attribute value not found"))
        })
        .ok();
    }
    val
}
