use crate::attributes::{Cardinality, RelationTarget};
use crate::models::OrmModel;
use crate::utils::{
    default_belongs_to_method, default_has_many_method, default_has_one_method, infer_fk_column,
    is_option_wrapped,
};
use quote::{__private::TokenStream, format_ident, quote};

// RelationInfo: central representation of relationships for OrmModel
pub(crate) struct RelationInfo {
    pub(crate) target: crate::attributes::RelationTarget,
    /// resolved FK column name
    pub(crate) fk_column: String,
    /// resolved method name
    pub(crate) method_name: String,
    pub(crate) cardinality: crate::attributes::Cardinality,
}

/// Generate `belongs_to` relation methods for the given model.
///
/// For each `belongs_to` relation on the model, emits a method like:
/// - Non-nullable FK:  `pub fn user(&self) -> UserSelectBuilder<'_>`
/// - Nullable FK:      `pub fn user(&self) -> Option<UserSelectBuilder<'_>>`
pub(crate) fn generate_belongs_to(model: &OrmModel) -> TokenStream {
    let struct_name = model.struct_name;
    let mut impl_tokens = TokenStream::new();

    for relation in &model.relations {
        if relation.cardinality != Cardinality::BelongsTo {
            continue;
        }

        let fk_col = model
            .columns
            .iter()
            .find(|c| c.column_name == relation.fk_column);
        let fk_col = match fk_col {
            Some(c) => c,
            None => continue,
        };
        let fk_field_ident = &fk_col.field;

        let method_name_str: String = match &relation.target {
            RelationTarget::Path(path) => {
                if relation.method_name.is_empty() {
                    default_belongs_to_method(path)
                } else {
                    relation.method_name.clone()
                }
            }
            RelationTarget::SelfRef => {
                if relation.method_name.is_empty() {
                    "parent".to_string()
                } else {
                    relation.method_name.clone()
                }
            }
        };
        let method_ident = format_ident!("{}", method_name_str);

        let builder_tokens: TokenStream = match &relation.target {
            RelationTarget::Path(path) => {
                let mut builder_path = path.clone();
                if let Some(last) = builder_path.segments.last_mut() {
                    last.ident = format_ident!("{}SelectBuilder", last.ident);
                }
                quote! { #builder_path }
            }
            RelationTarget::SelfRef => {
                let builder_ident = format_ident!("{}SelectBuilder", struct_name);
                quote! { #builder_ident }
            }
        };

        let is_nullable = is_option_wrapped(&fk_col.ty);
        let method_tokens = if is_nullable {
            quote! {
                pub fn #method_ident(&self) -> Option<#builder_tokens<'_>> {
                    self.#fk_field_ident.as_ref().map(|v| #builder_tokens::with_initial_where("id", v))
                }
            }
        } else {
            quote! {
                pub fn #method_ident(&self) -> #builder_tokens<'_> {
                    #builder_tokens::with_initial_where("id", &self.#fk_field_ident)
                }
            }
        };

        impl_tokens.extend(method_tokens);
    }

    if impl_tokens.is_empty() {
        return TokenStream::new();
    }

    quote! {
        #[automatically_derived]
        impl #struct_name {
            #impl_tokens
        }
    }
}

pub(crate) fn generate_has_relations(model: &OrmModel) -> TokenStream {
    let struct_name = model.struct_name;
    let mut impl_tokens = TokenStream::new();

    let parent_path: syn::Path = syn::parse_quote!(#struct_name);

    for relation in &model.relations {
        match relation.cardinality {
            Cardinality::HasMany | Cardinality::HasOne => {}
            _ => continue,
        }

        let fk_col: String = match &relation.target {
            RelationTarget::Path(_path) => {
                if relation.fk_column.is_empty() {
                    infer_fk_column(&parent_path)
                } else {
                    relation.fk_column.clone()
                }
            }
            RelationTarget::SelfRef => {
                if relation.fk_column.is_empty() {
                    continue;
                } else {
                    relation.fk_column.clone()
                }
            }
        };

        let method_name_str: String = match &relation.target {
            RelationTarget::Path(path) => {
                if relation.method_name.is_empty() {
                    match relation.cardinality {
                        Cardinality::HasMany => default_has_many_method(path),
                        Cardinality::HasOne => default_has_one_method(path),
                        _ => unreachable!(),
                    }
                } else {
                    relation.method_name.clone()
                }
            }
            RelationTarget::SelfRef => {
                if relation.method_name.is_empty() {
                    continue;
                } else {
                    relation.method_name.clone()
                }
            }
        };
        let method_ident = format_ident!("{}", method_name_str);

        let builder_tokens: TokenStream = match &relation.target {
            RelationTarget::Path(path) => {
                let mut builder_path = path.clone();
                if let Some(last) = builder_path.segments.last_mut() {
                    last.ident = format_ident!("{}SelectBuilder", last.ident);
                }
                quote! { #builder_path }
            }
            RelationTarget::SelfRef => {
                let builder_ident = format_ident!("{}SelectBuilder", struct_name);
                quote! { #builder_ident }
            }
        };

        let pk_access = match model.primary_key() {
            crate::models::PrimaryKey::Generated(col) => {
                let pk_field = &col.field;
                quote! { &self.#pk_field }
            }
            crate::models::PrimaryKey::Manual(_) => {
                quote! { compile_error!("has_many/has_one requires a Generated primary key (not manual/composite)") }
            }
        };

        impl_tokens.extend(quote! {
            pub fn #method_ident(&self) -> #builder_tokens<'_> {
                #builder_tokens::with_initial_where(#fk_col, #pk_access)
            }
        });
    }

    if impl_tokens.is_empty() {
        return TokenStream::new();
    }

    quote! {
        #[automatically_derived]
        impl #struct_name {
            #impl_tokens
        }
    }
}

