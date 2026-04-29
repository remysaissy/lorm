use darling::FromField;
use darling::FromMeta;
use darling::util::Callable;
use darling::util::Flag;
use darling::{FromAttributes, FromDeriveInput};
use heck::ToSnakeCase;
use quote::__private::TokenStream;
use quote::quote;
use syn::Expr;
use syn::Field;
use syn::spanned::Spanned;
use syn::{Attribute, DeriveInput, Type};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(lorm), supports(struct_named))]
pub struct TableAttributes {
    #[darling(rename = "rename")]
    table_name_override: Option<String>,
}

impl TableAttributes {
    /// Gets the specified table name from the `#[lorm(rename="...")]` attribute if specified, otherwise converts the struct name
    /// to table_case and pluralizes it (e.g., `UserDetail` becomes `user_details`).
    pub fn table_name(&self, input: &DeriveInput) -> String {
        self.table_name_override.clone().unwrap_or_else(|| {
            let table_case = input.ident.to_string().to_snake_case();
            pluralizer::pluralize(table_case.as_str(), 2, false)
        })
    }
}

#[derive(Debug, FromMeta)]
struct ColumnPropertyAttrs {
    #[darling(rename = "pk")]
    is_primary_key: Flag,
    #[darling(rename = "by")]
    generate_by: Flag,
    #[darling(rename = "readonly")]
    readonly: Flag,
    #[darling(rename = "created_at")]
    is_created_at: Flag,
    #[darling(rename = "updated_at")]
    is_updated_at: Flag,
    #[darling(rename = "new")]
    new_expression: Option<Expr>,
    #[darling(rename = "is_set")]
    is_set_expression: Option<Callable>,
}

#[derive(Debug, Clone)]
pub struct ColumnProperties {
    /// Whether the field should be entirely skipped.
    /// Specified by `#[sqlx(skip)]`.
    pub skip: bool,
    /// Whether the field is readonly in the database.
    /// Readonly fields will not be included in insert or update statements, but will be deserialized when selecting.
    pub readonly: bool,

    /// Whether the field is (part of) the primary key.
    pub primary_key: bool,

    /// Whether `by_*`, `with_*` and selector methods should be generated for this field.
    pub generate_by: bool,

    /// Whether the field is the `created_at` field.
    pub created_at: bool,
    /// Whether the field is the `updated_at` field.
    pub updated_at: bool,

    /// The expression to use to generate a new value for the field. Used when generating a new primary key or the `created_at` and `updated_at` fields.
    pub new_expression: Expr,
    /// A callable path used to determine whether the field is set (e.g. `Uuid::is_nil`).
    /// Invoked as `(callable)(&field_value)` and must return `bool`.
    /// If not set, the field's value will be compared with [Default::default].
    ///
    /// Used to determine whether the instance is in the database or not
    pub is_set_expression: Option<Callable>,
}

#[derive(Debug, FromAttributes)]
#[darling(attributes(sqlx), allow_unknown_fields)]
pub struct SqlxColumnAttributes {
    pub skip: Flag,
    pub rename: Option<String>,
}

fn parse_sqlx_attrs(attrs: Vec<Attribute>) -> Result<SqlxColumnAttributes, darling::Error> {
    SqlxColumnAttributes::from_attributes(&attrs)
}

#[derive(Debug, FromField)]
#[darling(attributes(lorm), forward_attrs(sqlx))]
pub struct FieldAttributes {
    #[darling(flatten)]
    field_properties: ColumnPropertyAttrs,
    #[darling(with = "parse_sqlx_attrs")]
    attrs: SqlxColumnAttributes,
}

fn default_new_expression() -> Expr {
    syn::parse_str("Default::default()").unwrap()
}

impl ColumnProperties {
    fn from(
        field: &Field,
        value: ColumnPropertyAttrs,
        sqlx: SqlxColumnAttributes,
    ) -> syn::Result<Self> {
        // new_expression only makes sense on the primary key field or the created_at and updated_at fields
        if (!value.is_primary_key.is_present())
            && !value.is_updated_at.is_present()
            && !value.is_created_at.is_present()
            && value.new_expression.is_some()
        {
            return Err(syn::Error::new(
                field.span(),
                "The `new` attribute only makes sense on primary key, created_at or updated_at fields.",
            ));
        }
        // is_set_expression only makes sense on the primary key field
        if (!value.is_primary_key.is_present()) && value.is_set_expression.is_some() {
            return Err(syn::Error::new(
                field.span(),
                "The `is_set` attribute only makes sense on generated primary key fields.",
            ));
        }

        Ok(ColumnProperties {
            skip: sqlx.skip.is_present(),
            readonly: value.readonly.is_present(),
            primary_key: value.is_primary_key.is_present(),
            generate_by: value.generate_by.is_present(),
            created_at: value.is_created_at.is_present(),
            updated_at: value.is_updated_at.is_present(),
            new_expression: value.new_expression.unwrap_or_else(default_new_expression),
            is_set_expression: value.is_set_expression,
        })
    }

    pub fn is_set(&self, base: TokenStream, ty: &Type) -> TokenStream {
        match &self.is_set_expression {
            Some(callable) => quote! { (#callable)(#base) },
            None => quote! { (|val: &#ty| val == &<#ty as Default>::default())(#base) },
        }
    }
}

#[derive(Debug)]
pub struct FieldProperties {
    pub column_properties: ColumnProperties,
    pub column_name: String,
}

impl FieldProperties {
    pub fn from(field: &Field, attributes: FieldAttributes) -> syn::Result<Self> {
        let column_name = attributes
            .attrs
            .rename
            .clone()
            .unwrap_or_else(|| field.ident.as_ref().unwrap().to_string());

        let column_properties =
            ColumnProperties::from(field, attributes.field_properties, attributes.attrs);

        Ok(Self {
            column_properties: column_properties?,
            column_name,
        })
    }
}
