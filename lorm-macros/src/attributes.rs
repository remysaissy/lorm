use darling::ast::NestedMeta;
use darling::util::{Callable, Flag};
use darling::{FromAttributes, FromDeriveInput, FromField, FromMeta};
use proc_macro_error2::emit_error;
use proc_macro2::Ident;
use quote::{ToTokens, format_ident};
use std::vec;
use syn::parse::{ParseStream, Parser};
use syn::spanned::Spanned;
use syn::{Attribute, Expr, Field, LitStr, Token, Type, parse};

#[derive(Debug, Copy, Clone, Eq, PartialEq, FromMeta)]
pub enum PrimaryKeyType {
    Generated,
    Manual,
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(lorm), supports(struct_named))]
pub struct TableAttributes {
    #[darling(rename = "rename")]
    pub table_name_override: Option<String>,
    #[darling(default = default_pk_type)]
    pub pk_type: PrimaryKeyType,
    #[darling(rename = "pk_selector", default = default_primary_key_selector)]
    pub primary_key_selector: Ident,
}

fn default_pk_type() -> PrimaryKeyType {
    PrimaryKeyType::Generated
}

fn default_primary_key_selector() -> Ident {
    format_ident!("by_key")
}

#[derive(Debug, FromAttributes)]
#[darling(attributes(sqlx))]
pub struct SqlxColumnAttributes {
    #[darling(rename = "json")]
    pub is_json: Option<JsonOptions>,
    pub flatten: Flag,
    pub skip: Flag,
    pub rename: Option<String>,
}

#[derive(Debug, Default)]
pub struct JsonOptions {
    pub nullable: bool,
}

impl FromMeta for JsonOptions {
    // #[sqlx(json)]
    fn from_word() -> darling::Result<Self> {
        Ok(JsonOptions { nullable: false })
    }

    // #[sqlx(json(nullable))]
    fn from_list(items: &[NestedMeta]) -> darling::Result<Self> {
        let nullable = items.iter().any(
            |item| matches!(item, NestedMeta::Meta(syn::Meta::Path(p)) if p.is_ident("nullable")),
        );
        Ok(JsonOptions { nullable })
    }
}

#[derive(Debug, FromMeta)]
struct ColumnPropertyAttrs {
    #[darling(rename = "pk")]
    pub is_primary_key: Flag,
    #[darling(rename = "by")]
    pub generate_by: Flag,
    #[darling(rename = "readonly")]
    pub readonly: Flag,
    #[darling(rename = "created_at")]
    pub is_created_at: Flag,
    #[darling(rename = "updated_at")]
    pub is_updated_at: Flag,
    #[darling(rename = "new")]
    pub new_expression: Option<Expr>,
    #[darling(rename = "is_set")]
    pub is_set_expression: Option<Callable>,
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

    /// Whether the field should be stored as JSON in the database. Specified by `#[sqlx(json)]`.
    pub use_json: bool,

    /// The expression to use to generate a new value for the field. Used when generating a new primary key or the `created_at` and `updated_at` fields.
    pub new_expression: Expr,
    /// The expression to use to check if a field is already populated with a value when deciding whether to generate a new primary key.
    /// The user can supply this with the `#[lorm(is_set = callable)]` attribute
    pub is_set_expression: Expr,
}

#[derive(Debug, FromField)]
#[darling(attributes(lorm), forward_attrs(sqlx))]
pub struct FieldAttributes {
    #[darling(flatten)]
    pub field_properties: ColumnPropertyAttrs,

    #[darling(with = "parse_sqlx_attrs")]
    pub attrs: SqlxColumnAttributes,
    pub flattened: Option<FlattenedFields>,
}

fn default_new_expression() -> Expr {
    syn::parse_str("Default::default()").unwrap()
}

fn default_is_set_expression(field: &Field) -> Expr {
    let instance_field = field.ident.as_ref().unwrap();
    let ty = &field.ty;
    syn::parse_quote! {
        #instance_field == <#ty as Default>::default()
    }
}

fn parse_sqlx_attrs(attrs: Vec<Attribute>) -> Result<SqlxColumnAttributes, darling::Error> {
    SqlxColumnAttributes::from_attributes(&attrs)
}

impl ColumnProperties {
    fn from(
        field: &Field,
        value: ColumnPropertyAttrs,
        sqlx_attrs: &SqlxColumnAttributes,
        pk_type: PrimaryKeyType,
    ) -> Self {
        // updated_at and created_at or being the field of a generated primary key imply generate_by
        let generate_by = value.generate_by.is_present()
            || value.is_updated_at.is_present()
            || value.is_created_at.is_present()
            || pk_type == PrimaryKeyType::Generated && value.is_primary_key.is_present();

        // new_expression only makes sense on generated primary key fields or the created_at and updated_at fields
        if (pk_type != PrimaryKeyType::Generated || !value.is_primary_key.is_present())
            && !value.is_updated_at.is_present()
            && !value.is_created_at.is_present()
        {
            if value.new_expression.is_some() {
                emit_error!(
                    field.span(),
                    "The `is_set` attribute only makes sense on generated primary key fields."
                );
            }
        }
        // is_set_expression only makes sense on generated primary key fields
        if pk_type != PrimaryKeyType::Generated || !value.is_primary_key.is_present() {
            if value.is_set_expression.is_some() {
                emit_error!(
                    field.span(),
                    "The `is_set` attribute only makes sense on generated primary key fields."
                );
            }
        }

        ColumnProperties {
            skip: sqlx_attrs.skip.is_present(),
            readonly: value.readonly.is_present(),
            primary_key: value.is_primary_key.is_present(),
            generate_by,
            created_at: value.is_created_at.is_present(),
            updated_at: value.is_updated_at.is_present(),
            use_json: sqlx_attrs.is_json.is_some(),
            new_expression: value
                .new_expression
                .unwrap_or_else(|| default_new_expression()),
            is_set_expression: value.is_set_expression.map_or_else(
                || default_is_set_expression(field),
                |c| {
                    let field = field.ident.as_ref().unwrap();
                    syn::parse_quote!(#field.#c())
                },
            ),
        }
    }
}

#[derive(Debug)]
pub struct FieldProperties {
    pub column_properties: ColumnProperties,
    pub column_name: String,
    pub flattened_fields: Option<Vec<FlattenedField>>,
}

impl FieldProperties {
    pub fn from(field: &Field, attributes: FieldAttributes, pk_type: PrimaryKeyType) -> Self {
        let column_properties = ColumnProperties::from(
            field,
            attributes.field_properties,
            &attributes.attrs,
            pk_type,
        );

        // Rename is only possible on non-flattened fields
        if attributes.attrs.flatten.is_present() && attributes.attrs.rename.is_some() {
            emit_error!(
                field.span(),
                "The `rename` attribute is only supported on non-flattened fields."
            );
        }

        let column_name = attributes
            .attrs
            .rename
            .unwrap_or_else(|| field.ident.as_ref().unwrap().to_string());

        Self {
            column_properties,
            column_name,
            flattened_fields: attributes.flattened.map(|f| f.0),
        }
    }
}

#[derive(Debug)]
pub struct FlattenedField {
    pub field_name: Ident,
    pub ty: Type,
    pub column_name: String,
}

impl parse::Parse for FlattenedField {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let field: Ident = input.parse()?;

        input.parse::<Token![:]>()?;
        let ty = input.parse::<Type>()?;

        let column = if input.peek(Token![=]) {
            input.parse::<Token![=]>()?;
            input.parse::<LitStr>()?.value()
        } else {
            field.to_string()
        };

        Ok(FlattenedField {
            field_name: field,
            ty,
            column_name: column,
        })
    }
}

#[derive(Debug)]
pub struct FlattenedFields(Vec<FlattenedField>);

impl FromMeta for FlattenedFields {
    fn from_meta(meta: &syn::Meta) -> darling::Result<Self> {
        let syn::Meta::List(list) = meta else {
            return Err(darling::Error::unsupported_format(
                "expected `flattened(...)`",
            ));
        };

        let fields = syn::punctuated::Punctuated::<FlattenedField, Token![,]>::parse_terminated
            .parse2(list.tokens.clone())
            .map_err(darling::Error::from)?;

        Ok(FlattenedFields(fields.into_iter().collect()))
    }
}
