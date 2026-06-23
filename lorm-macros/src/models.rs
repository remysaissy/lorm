use crate::attributes::ColumnProperties;
use crate::attributes::FieldAttributes;
use crate::attributes::FieldProperties;
use crate::attributes::PrimaryKeyType;
use crate::attributes::TableAttributes;
use crate::orm::column::Column;
use crate::orm::relations::RelationInfo;
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

pub(crate) enum PrimaryKey<'a> {
    /// Single generated pk (current behavior)
    Generated(Box<Column<'a>>),
    /// One or more manual pk columns
    Manual(Vec<Column<'a>>),
}

impl<'a> PrimaryKey<'a> {
    pub(crate) fn is_generated(&self) -> bool {
        matches!(self, PrimaryKey::Generated(_))
    }

    pub(crate) fn fields(&self) -> &[Column<'a>] {
        match self {
            PrimaryKey::Generated(col) => std::slice::from_ref(col.as_ref()),
            PrimaryKey::Manual(cols) => cols,
        }
    }

    /// For Generated pk, returns the single column.
    /// For Manual pk, returns the first column (should not be used for composite-only logic).
    pub(crate) fn generated_column(&self) -> &Column<'a> {
        match self {
            PrimaryKey::Generated(col) => col.as_ref(),
            PrimaryKey::Manual(cols) => &cols[0],
        }
    }
}

pub(crate) struct OrmModel<'a> {
    pub(crate) struct_name: &'a Ident,
    pub(crate) struct_visibility: &'a Visibility,
    pub(crate) table_name: String,
    pub(crate) columns: Vec<Column<'a>>,

    pub(crate) primary_key: PrimaryKey<'a>,
    pub(crate) pk_selector_name: String,
    pub(crate) relations: Vec<RelationInfo>,
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

        let mut pk_columns = columns
            .iter()
            .filter(|c| c.column_properties.primary_key)
            .cloned()
            .collect::<Vec<_>>();

        match top_level_attributes.pk_type {
            PrimaryKeyType::Generated => {
                if pk_columns.len() != 1 {
                    return Err(syn::Error::new(
                        input.ident.span(),
                        "expected exactly one primary key when pk_type is Generated",
                    ));
                }
            }
            PrimaryKeyType::Manual => {
                if pk_columns.is_empty() {
                    return Err(syn::Error::new(
                        input.ident.span(),
                        "at least one #[lorm(pk)] field required when pk_type is Manual",
                    ));
                }
            }
        }

        let pk_field_names = pk_columns
            .iter()
            .map(|c| c.field.to_string())
            .collect::<Vec<_>>();
        let pk_field_names_ref = pk_field_names
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<_>>();
        let pk_selector_name = top_level_attributes.pk_selector_name(&pk_field_names_ref);

        let primary_key = match top_level_attributes.pk_type {
            PrimaryKeyType::Generated => PrimaryKey::Generated(Box::new(pk_columns.remove(0))),
            PrimaryKeyType::Manual => PrimaryKey::Manual(pk_columns),
        };

        // Build relations: first from column-level `belongs_to`, then from table-level has_many/has_one specs
        let relations_from_columns = columns
            .iter()
            .filter_map(|col| {
                col.belongs_to.as_ref().map(|target| RelationInfo {
                    target: target.clone(),
                    fk_column: col.column_name.clone(),
                    method_name: String::new(),
                    cardinality: crate::attributes::Cardinality::BelongsTo,
                })
            })
            .collect::<Vec<_>>();

        let relations_from_table = top_level_attributes
            .has_relations()
            .map(|spec| RelationInfo {
                target: spec.target.clone(),
                fk_column: spec.fk.clone().unwrap_or_default(),
                method_name: spec.method_name.clone().unwrap_or_default(),
                cardinality: spec.cardinality,
            })
            .collect::<Vec<_>>();

        let mut relations = relations_from_columns;
        relations.extend(relations_from_table);

        Ok(Self {
            struct_name,
            struct_visibility,
            table_name,
            columns,
            primary_key,
            pk_selector_name,
            relations,
        })
    }

    pub(crate) fn query_columns(&self) -> impl Iterator<Item = &Column<'a>> {
        self.columns
            .iter()
            .filter(|c| c.should_generate_query_function(self.primary_key.is_generated()))
    }

    pub(crate) fn full_column_select(&self) -> String {
        self.columns
            .iter()
            .map(|c| c.column_name.clone())
            .collect::<Vec<_>>()
            .join(", ")
    }

    pub(crate) fn primary_key(&self) -> &PrimaryKey<'a> {
        &self.primary_key
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
        let pk_columns: Box<dyn Iterator<Item = &Column<'a>>> = match &self.primary_key {
            PrimaryKey::Generated(col) => {
                if col.column_properties.readonly {
                    Box::new(std::iter::empty())
                } else {
                    Box::new(std::iter::once(col.as_ref()))
                }
            }
            PrimaryKey::Manual(cols) => Box::new(cols.iter()),
        };

        pk_columns.chain(self.update_columns())
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
                belongs_to_target: None,
            };

            columns.push(Column {
                base_field: field,
                field: entry.ident,
                ty,
                column_name: entry.column_name,
                is_flattened: true,
                column_properties: col_props,
                belongs_to: None,
            });
        }

        return Ok(());
    }

    let properties = FieldProperties::from(field, field_attrs)?;

    if properties.column_properties.skip {
        return Ok(());
    }

    let logical_fields: Box<dyn Iterator<Item = Column<'a>>> = {
        let column_name = properties.column_name.clone();
        // move out column_properties once, then extract belongs_to from it
        let col_props = properties.column_properties;
        let belongs_to = col_props.belongs_to_target.clone();

        let logical_field = Column {
            base_field: field,
            field: field.ident.clone().unwrap(),
            ty: parse((&field.ty).into_token_stream().into())?,
            is_flattened: false,
            column_name,
            column_properties: col_props,
            belongs_to,
        };

        Box::new(Some(logical_field).into_iter())
    };

    for logical_field in logical_fields {
        columns.push(logical_field);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use syn::parse_str;

    fn parse_model(src: &str) -> (syn::DeriveInput, syn::punctuated::Punctuated<syn::Field, syn::token::Comma>) {
        let input: syn::DeriveInput = parse_str(src).unwrap();
        let fields = match &input.data {
            syn::Data::Struct(s) => match &s.fields {
                syn::Fields::Named(n) => n.named.clone(),
                _ => panic!("named fields required"),
            },
            _ => panic!("struct required"),
        };
        (input, fields)
    }

    #[test]
    fn test_parse_simple() {
        let (input, fields) = parse_model(r#"
            struct User {
                pub id: u32,
            }
        "#);
        assert_eq!(input.ident, "User");
        assert_eq!(fields.len(), 1);
    }

    #[test]
    fn test_table_attributes_parsing() {
        use crate::attributes::TableAttributes;
        use darling::FromDeriveInput;
        
        let (input, _) = parse_model(r#"
            struct User {
                pub id: u32,
            }
        "#);
        let attrs = TableAttributes::from_derive_input(&input).unwrap();
        assert_eq!(attrs.table_name(&input), "users");
    }

    #[test]
    fn test_field_attributes_parsing() {
        use crate::attributes::FieldAttributes;
        use darling::FromField;
        
        let (_input, fields) = parse_model(r#"
            struct User {
                #[lorm(pk)]
                pub id: u32,
            }
        "#);
        let field = fields.first().unwrap();
        let fa = FieldAttributes::from_field(field).unwrap();
        assert!(fa.is_primary_key());
    }

    
}
