use crate::utils::is_option_wrapped;
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
use syn::{Attribute, DeriveInput, Ident, Type};

/// Controls whether the primary key is generated (default) or manually managed.
#[derive(Debug, Copy, Clone, Eq, PartialEq, darling::FromMeta)]
#[darling(rename_all = "lowercase")]
pub enum PrimaryKeyType {
    Generated,
    Manual,
}

fn default_pk_type() -> PrimaryKeyType {
    PrimaryKeyType::Generated
}

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(lorm), supports(struct_named))]
pub(crate) struct TableAttributes {
    #[darling(rename = "rename")]
    table_name_override: Option<String>,

    #[darling(default = "default_pk_type")]
    pub(crate) pk_type: PrimaryKeyType,

    #[darling(rename = "pk_selector")]
    pub(crate) pk_selector: Option<String>,

    #[darling(rename = "has_many", multiple, default, with = "parse_has_many_spec")]
    pub(crate) has_many_specs: Vec<HasRelSpec>,

    #[darling(rename = "has_one", multiple, default, with = "parse_has_one_spec")]
    pub(crate) has_one_specs: Vec<HasRelSpec>,
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

    /// Returns the method name for the composite pk selector.
    ///
    /// - If pk_selector is provided, use it directly.
    /// - If single-field manual pk: use `by_<field_name>`.
    /// - If composite pk (2+ fields): use `by_key`.
    ///
    /// Caller provides pk_fields to determine the name when no override.
    pub fn pk_selector_name(&self, pk_fields: &[&str]) -> String {
        if let Some(ident) = &self.pk_selector {
            return ident.clone();
        }
        if pk_fields.len() == 1 {
            format!("by_{}", pk_fields[0])
        } else {
            "by_key".to_string()
        }
    }

    pub fn has_relations(&self) -> impl Iterator<Item = &HasRelSpec> {
        self.has_many_specs.iter().chain(self.has_one_specs.iter())
    }
}

/// Represents one field entry in `#[lorm(flattened(field: Type, field2: Type = "renamed_col"))]`
#[derive(Debug, Clone)]
pub(crate) struct FlattenedField {
    pub(crate) ident: Ident,
    pub(crate) ty: Type,
    pub(crate) column_name: String,
}

impl syn::parse::Parse for FlattenedField {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        input.parse::<syn::Token![:]>()?;
        let ty: Type = input.parse()?;
        let column_name = if input.peek(syn::Token![=]) {
            input.parse::<syn::Token![=]>()?;
            let s: syn::LitStr = input.parse()?;
            s.value()
        } else {
            ident.to_string()
        };
        Ok(FlattenedField {
            ident,
            ty,
            column_name,
        })
    }
}

/// Parsed form of `#[lorm(flattened(field1: Type1, field2: Type2 = "col_name", ...))]`
#[derive(Debug, Clone)]
pub(crate) struct FlattenedFields {
    pub(crate) fields: Vec<FlattenedField>,
}

impl darling::FromMeta for FlattenedFields {
    fn from_meta(item: &syn::Meta) -> darling::Result<Self> {
        if let syn::Meta::List(list) = item {
            let fields: syn::punctuated::Punctuated<FlattenedField, syn::Token![,]> = list
                .parse_args_with(syn::punctuated::Punctuated::parse_terminated)
                .map_err(|e| darling::Error::custom(e.to_string()))?;
            Ok(FlattenedFields {
                fields: fields.into_iter().collect(),
            })
        } else {
            Err(darling::Error::custom(
                "expected list: #[lorm(flattened(field: Type, field2: Type = \"col_name\"))]",
            ))
        }
    }
}

/// Target type for a relation — either a specific type path or the current struct itself (Self).
#[derive(Debug, Clone)]
pub(crate) enum RelationTarget {
    /// A specific model type, e.g., `User`, `crate::models::User`
    Path(syn::Path),
    /// Self-referential: `belongs_to = Self`
    SelfRef,
}

impl darling::FromMeta for RelationTarget {
    fn from_meta(item: &syn::Meta) -> darling::Result<Self> {
        match item {
            syn::Meta::NameValue(nv) => match &nv.value {
                syn::Expr::Path(expr_path) => {
                    let path = &expr_path.path;
                    if path.is_ident("Self") || path.is_ident("self") {
                        Ok(RelationTarget::SelfRef)
                    } else {
                        Ok(RelationTarget::Path(path.clone()))
                    }
                }
                _ => Err(darling::Error::custom(
                    "belongs_to requires a type name, e.g. #[lorm(belongs_to = User)]",
                )
                .with_span(item)),
            },
            syn::Meta::Path(path) => {
                // Handle bare `belongs_to = Self` case when darling passes just the path
                if path.is_ident("Self") || path.is_ident("self") {
                    Ok(RelationTarget::SelfRef)
                } else {
                    Ok(RelationTarget::Path(path.clone()))
                }
            }
            _ => Err(darling::Error::custom(
                "belongs_to requires a type, e.g. #[lorm(belongs_to = User)] or #[lorm(belongs_to = Self)]",
            )
            .with_span(item)),
        }
    }

    fn from_expr(expr: &syn::Expr) -> darling::Result<Self> {
        match expr {
            syn::Expr::Path(expr_path) => {
                let path = &expr_path.path;
                if path.is_ident("Self") || path.is_ident("self") {
                    Ok(RelationTarget::SelfRef)
                } else {
                    Ok(RelationTarget::Path(path.clone()))
                }
            }
            _ => Err(darling::Error::custom(
                "belongs_to requires a type name, e.g. #[lorm(belongs_to = User)]",
            )),
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub(crate) enum Cardinality {
    HasMany,
    HasOne,
    BelongsTo,
}

/// Parsed spec for `#[lorm(has_many(...))]` / `#[lorm(has_one(...))]`.
///
/// Supports the following forms:
/// - `#[lorm(has_many = Post)]`
/// - `#[lorm(has_many(Post, fk = "user_id"))]`
/// - `#[lorm(has_many(Post, fk = "user_id", as = "authored_posts"))]`
/// - `#[lorm(has_one = Profile)]`
/// - `#[lorm(has_many = Self)]`
#[derive(Debug, Clone)]
pub(crate) struct HasRelSpec {
    pub target: RelationTarget,
    pub fk: Option<String>,
    pub method_name: Option<String>,
    pub cardinality: Cardinality,
}

#[derive(Debug)]
struct HasRelSpecArgs {
    target: RelationTarget,
    fk: Option<String>,
    method_name: Option<String>,
}

impl syn::parse::Parse for HasRelSpecArgs {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let target_path: syn::Path = input.parse()?;
        let target = if target_path.is_ident("Self") || target_path.is_ident("self") {
            RelationTarget::SelfRef
        } else {
            RelationTarget::Path(target_path)
        };

        let mut fk: Option<String> = None;
        let mut method_name: Option<String> = None;

        while input.peek(syn::Token![,]) {
            let _comma: syn::Token![,] = input.parse()?;
            if input.is_empty() {
                break;
            }

            // `as` is a keyword; it cannot be parsed as an Ident.
            if input.peek(syn::Token![as]) {
                let _: syn::Token![as] = input.parse()?;
                let _eq: syn::Token![=] = input.parse()?;
                let val: syn::LitStr = input.parse()?;
                method_name = Some(val.value());
                continue;
            }

            let key: syn::Ident = input.parse()?;
            let _eq: syn::Token![=] = input.parse()?;
            let val: syn::LitStr = input.parse()?;

            match key.to_string().as_str() {
                "fk" => fk = Some(val.value()),
                other => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!("unknown key `{}` (supported: fk, as)", other),
                    ));
                }
            }
        }

        Ok(Self {
            target,
            fk,
            method_name,
        })
    }
}

impl HasRelSpec {
    fn with_cardinality(mut self, cardinality: Cardinality) -> Self {
        self.cardinality = cardinality;
        self
    }

    fn from_meta_without_cardinality(item: &syn::Meta) -> darling::Result<Self> {
        match item {
            // `has_many = Post`
            syn::Meta::NameValue(nv) => {
                let target = RelationTarget::from_expr(&nv.value)?;
                Ok(Self {
                    target,
                    fk: None,
                    method_name: None,
                    // Caller sets it.
                    cardinality: Cardinality::HasMany,
                })
            }
            // `has_many(Post, fk = "col", as = "name")`
            syn::Meta::List(list) => {
                let args: HasRelSpecArgs = syn::parse2(list.tokens.clone())
                    .map_err(|e| darling::Error::custom(e.to_string()).with_span(item))?;
                Ok(Self {
                    target: args.target,
                    fk: args.fk,
                    method_name: args.method_name,
                    // Caller sets it.
                    cardinality: Cardinality::HasMany,
                })
            }
            _ => Err(darling::Error::custom(
                "expected name-value or list, e.g. #[lorm(has_many = Post)] or #[lorm(has_many(Post, fk = \"col\"))]",
            )
            .with_span(item)),
        }
    }
}

impl darling::FromMeta for HasRelSpec {
    fn from_meta(item: &syn::Meta) -> darling::Result<Self> {
        Self::from_meta_without_cardinality(item)
    }

    fn from_expr(expr: &syn::Expr) -> darling::Result<Self> {
        let target = RelationTarget::from_expr(expr)?;
        Ok(Self {
            target,
            fk: None,
            method_name: None,
            // Caller sets it.
            cardinality: Cardinality::HasMany,
        })
    }
}

fn parse_has_many_spec(meta: &syn::Meta) -> darling::Result<HasRelSpec> {
    HasRelSpec::from_meta_without_cardinality(meta)
        .map(|s| s.with_cardinality(Cardinality::HasMany))
}

fn parse_has_one_spec(meta: &syn::Meta) -> darling::Result<HasRelSpec> {
    HasRelSpec::from_meta_without_cardinality(meta).map(|s| s.with_cardinality(Cardinality::HasOne))
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

    #[darling(rename = "flattened")]
    flattened_fields: Option<FlattenedFields>,

    #[darling(rename = "belongs_to")]
    belongs_to_target: Option<RelationTarget>,
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

    pub use_json: bool,

    #[allow(dead_code)]
    pub belongs_to_target: Option<RelationTarget>,
}

#[derive(Debug, FromAttributes)]
#[darling(attributes(sqlx), allow_unknown_fields)]
pub struct SqlxColumnAttributes {
    pub skip: Flag,
    pub rename: Option<String>,
    #[darling(rename = "json")]
    pub is_json: Flag,

    pub flatten: Flag,
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

impl FieldAttributes {
    pub(crate) fn has_sqlx_flatten(&self) -> bool {
        self.attrs.flatten.is_present()
    }

    pub(crate) fn has_lorm_flattened(&self) -> bool {
        self.field_properties.flattened_fields.is_some()
    }

    pub(crate) fn is_skip(&self) -> bool {
        self.attrs.skip.is_present()
    }

    pub(crate) fn is_primary_key(&self) -> bool {
        self.field_properties.is_primary_key.is_present()
    }

    pub(crate) fn is_created_at_field(&self) -> bool {
        self.field_properties.is_created_at.is_present()
    }

    pub(crate) fn is_updated_at_field(&self) -> bool {
        self.field_properties.is_updated_at.is_present()
    }

    pub(crate) fn flatten_generate_by(&self) -> bool {
        self.field_properties.generate_by.is_present()
    }

    pub(crate) fn flatten_readonly(&self) -> bool {
        self.field_properties.readonly.is_present()
    }

    /// Consumes self and returns the FlattenedFields. Only call if `has_lorm_flattened()` is true.
    pub(crate) fn take_flattened_fields(self) -> FlattenedFields {
        self.field_properties.flattened_fields.unwrap()
    }
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
        if value.is_primary_key.is_present() && sqlx.is_json.is_present() {
            return Err(syn::Error::new(
                field.span(),
                "A field annotated with #[sqlx(json)] cannot be the primary key.",
            ));
        }

        if value.belongs_to_target.is_some() && value.is_primary_key.is_present() {
            return Err(syn::Error::new(
                field.span(),
                "The `belongs_to` attribute is incompatible with `#[lorm(pk)]`. A primary key field cannot be a foreign key reference.",
            ));
        }

        if value.belongs_to_target.is_some()
            && (sqlx.flatten.is_present() || value.flattened_fields.is_some())
        {
            return Err(darling::Error::custom(
                "The `belongs_to` attribute is incompatible with `#[sqlx(flatten)]` / `#[lorm(flattened)]`. A flattened field expands into multiple columns and cannot be used as a foreign key.",
            )
            .with_span(field)
            .into());
        }

        if let Some(RelationTarget::SelfRef) = &value.belongs_to_target
            && !is_option_wrapped(&field.ty)
        {
            return Err(syn::Error::new(
                field.span(),
                "Self-referential `belongs_to = Self` requires the field to be `Option<T>` (nullable FK). Use `Option<Uuid>` instead of `Uuid`.",
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
            use_json: sqlx.is_json.is_present(),
            belongs_to_target: value.belongs_to_target,
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

#[cfg(test)]
mod tests {
    use super::*;
    use darling::FromDeriveInput;
    use syn::parse_str;

    #[test]
    fn table_name_defaults_to_snake_case_plural() {
        let input: syn::DeriveInput = parse_str("struct UserDetail { }").unwrap();
        let attrs = TableAttributes::from_derive_input(&input).unwrap();
        assert_eq!(attrs.table_name(&input), "user_details");
    }

    #[test]
    fn table_name_uses_rename_override() {
        let input: syn::DeriveInput = parse_str(r#"#[lorm(rename = "accounts")] struct User { }"#).unwrap();
        let attrs = TableAttributes::from_derive_input(&input).unwrap();
        assert_eq!(attrs.table_name(&input), "accounts");
        assert_ne!(attrs.table_name(&input), ""); // mutant: String::new()
        assert_ne!(attrs.table_name(&input), "xyzzy"); // mutant: "xyzzy"
    }

    #[test]
    fn pk_selector_name_single_field_defaults_to_by_field() {
        let input: syn::DeriveInput = parse_str("struct User { }").unwrap();
        let attrs = TableAttributes::from_derive_input(&input).unwrap();
        assert_eq!(attrs.pk_selector_name(&["id"]), "by_id");
        assert_ne!(attrs.pk_selector_name(&["id"]), ""); // mutant: String::new()
        assert_ne!(attrs.pk_selector_name(&["id"]), "xyzzy"); // mutant: "xyzzy"
    }

    #[test]
    fn pk_selector_name_composite_uses_by_key() {
        let input: syn::DeriveInput = parse_str("struct UserRole { }").unwrap();
        let attrs = TableAttributes::from_derive_input(&input).unwrap();
        assert_eq!(attrs.pk_selector_name(&["user_id", "role_id"]), "by_key");
        assert_ne!(attrs.pk_selector_name(&["user_id", "role_id"]), "by_user_id"); // mutant: == → !=
    }

    #[test]
    fn pk_selector_name_single_vs_composite_differ() {
        let input: syn::DeriveInput = parse_str("struct T { }").unwrap();
        let attrs = TableAttributes::from_derive_input(&input).unwrap();
        assert_eq!(attrs.pk_selector_name(&["email"]), "by_email");
        assert_eq!(attrs.pk_selector_name(&["a", "b"]), "by_key");
    }

    #[test]
    fn pk_selector_name_uses_override() {
        let input: syn::DeriveInput = parse_str(r#"#[lorm(pk_type = "manual", pk_selector = "find_by_ids")] struct T { }"#).unwrap();
        let attrs = TableAttributes::from_derive_input(&input).unwrap();
        assert_eq!(attrs.pk_selector_name(&["user_id", "role_id"]), "find_by_ids");
    }

    #[test]
    fn has_relations_returns_all_specs() {
        let input: syn::DeriveInput = parse_str(r#"
            #[lorm(has_many = Post)]
            #[lorm(has_one = Profile)]
            struct User { }
        "#).unwrap();
        let attrs = TableAttributes::from_derive_input(&input).unwrap();
        let relations: Vec<_> = attrs.has_relations().collect();
        assert_eq!(relations.len(), 2); // mutant: empty()
    }

    #[test]
    fn field_attributes_boolean_accessors_with_annotations() {
        // Test that annotated fields report true
        use darling::FromField;
        let s: syn::ItemStruct = parse_str(r#"
            struct S {
                #[lorm(pk)]
                pub id: u32,
            }
        "#).unwrap();
        let field = s.fields.iter().next().unwrap();
        let fa = FieldAttributes::from_field(field).unwrap();
        assert!(fa.is_primary_key());
        assert!(!fa.is_skip());
        assert!(!fa.is_created_at_field());
        assert!(!fa.is_updated_at_field());
        assert!(!fa.flatten_generate_by());
        assert!(!fa.flatten_readonly());
    }

    #[test]
    fn field_attributes_created_at_annotation() {
        use darling::FromField;
        let s: syn::ItemStruct = parse_str(r#"
            struct S {
                #[lorm(created_at)]
                pub created_at: String,
            }
        "#).unwrap();
        let field = s.fields.iter().next().unwrap();
        let fa = FieldAttributes::from_field(field).unwrap();
        assert!(fa.is_created_at_field()); // mutant: false
        assert!(!fa.is_updated_at_field());
        assert!(!fa.is_primary_key());
    }

    #[test]
    fn field_attributes_updated_at_annotation() {
        use darling::FromField;
        let s: syn::ItemStruct = parse_str(r#"
            struct S {
                #[lorm(updated_at)]
                pub updated_at: String,
            }
        "#).unwrap();
        let field = s.fields.iter().next().unwrap();
        let fa = FieldAttributes::from_field(field).unwrap();
        assert!(fa.is_updated_at_field()); // mutant: false
        assert!(!fa.is_created_at_field());
    }

    #[test]
    fn field_attributes_skip_annotation() {
        use darling::FromField;
        let s: syn::ItemStruct = parse_str(r#"
            struct S {
                #[sqlx(skip)]
                pub tmp: String,
            }
        "#).unwrap();
        let field = s.fields.iter().next().unwrap();
        let fa = FieldAttributes::from_field(field).unwrap();
        assert!(fa.is_skip()); // mutant: false
    }

    #[test]
    fn field_attributes_by_annotation() {
        use darling::FromField;
        let s: syn::ItemStruct = parse_str(r#"
            struct S {
                #[lorm(by)]
                pub email: String,
            }
        "#).unwrap();
        let field = s.fields.iter().next().unwrap();
        let fa = FieldAttributes::from_field(field).unwrap();
        assert!(fa.flatten_generate_by()); // mutant: false
    }

    #[test]
    fn field_attributes_readonly_annotation() {
        use darling::FromField;
        let s: syn::ItemStruct = parse_str(r#"
            struct S {
                #[lorm(readonly)]
                pub count: i32,
            }
        "#).unwrap();
        let field = s.fields.iter().next().unwrap();
        let fa = FieldAttributes::from_field(field).unwrap();
        assert!(fa.flatten_readonly()); // mutant: false
    }

    #[test]
    fn column_properties_rejects_new_on_non_pk_field() {
        // Kills the ! deletions at lines 466-468: if logic is inverted, this should NOT return Err
        use darling::FromField;
        let s: syn::ItemStruct = parse_str(r#"
            struct S {
                #[lorm(new = "String::new()")]
                pub email: String,
            }
        "#).unwrap();
        let field = s.fields.iter().next().unwrap();
        let fa = FieldAttributes::from_field(field).unwrap();
        let result = FieldProperties::from(field, fa);
        assert!(result.is_err(), "new on non-pk/ts field must be rejected");
    }

    #[test]
    fn column_properties_rejects_is_set_on_non_pk_field() {
        use darling::FromField;
        let s: syn::ItemStruct = parse_str(r#"
            struct S {
                #[lorm(is_set = "String::is_empty")]
                pub email: String,
            }
        "#).unwrap();
        let field = s.fields.iter().next().unwrap();
        let fa = FieldAttributes::from_field(field).unwrap();
        let result = FieldProperties::from(field, fa);
        assert!(result.is_err(), "is_set on non-pk field must be rejected");
    }

    #[test]
    fn column_properties_allows_new_on_created_at() {
        // Ensures the || logic is correct (not && mutation): new is allowed on created_at
        use darling::FromField;
        let s: syn::ItemStruct = parse_str(r#"
            struct S {
                #[lorm(created_at, new = "String::new()")]
                pub created_at: String,
            }
        "#).unwrap();
        let field = s.fields.iter().next().unwrap();
        let fa = FieldAttributes::from_field(field).unwrap();
        let result = FieldProperties::from(field, fa);
        assert!(result.is_ok(), "new on created_at field must be allowed");
    }

    #[test]
    fn column_properties_rejects_belongs_to_with_flatten() {
        // Kills the || → && mutation at line 498
        use darling::FromField;
        let s: syn::ItemStruct = parse_str(r#"
            struct S {
                #[sqlx(flatten)]
                #[lorm(belongs_to = User)]
                pub user: String,
            }
        "#).unwrap();
        let field = s.fields.iter().next().unwrap();
        let fa = FieldAttributes::from_field(field);
        // This might fail at darling level or ColumnProperties::from level; either way it must not succeed
        if let Ok(fa) = fa {
            let result = FieldProperties::from(field, fa);
            assert!(result.is_err(), "belongs_to with flatten must be rejected");
        }
    }
}
