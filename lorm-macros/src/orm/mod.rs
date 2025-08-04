mod by;
mod delete;
mod save;
mod select;
mod with;

use crate::models::OrmModel;
use crate::utils::executor_type;
use crate::utils::*;
use proc_macro::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;
use syn::token::Comma;
use syn::{Data, DataStruct, DeriveInput, Field, Fields, FieldsNamed, FieldsUnnamed};

pub fn expand_derive_to_orm(input: &DeriveInput) -> syn::Result<TokenStream> {
    match &input.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(FieldsNamed { named, .. }),
            ..
        }) => expand_derive_to_orm_struct(input, named),

        Data::Struct(DataStruct {
            fields: Fields::Unnamed(FieldsUnnamed { .. }),
            ..
        }) => Err(syn::Error::new(
            input.ident.span(),
            "unnamed structs are not supported",
        )),

        Data::Struct(DataStruct {
            fields: Fields::Unit,
            ..
        }) => Err(syn::Error::new(
            input.ident.span(),
            "unit structs are not supported",
        )),

        Data::Enum(_) => Err(syn::Error::new(
            input.ident.span(),
            "enums are not supported",
        )),

        Data::Union(_) => Err(syn::Error::new(
            input.ident.span(),
            "unions are not supported",
        )),
    }
}

pub fn expand_derive_to_orm_struct(
    input: &DeriveInput,
    fields: &Punctuated<Field, Comma>,
) -> syn::Result<TokenStream> {
    let model = OrmModel::from_fields(input, fields)?;
    let executor_type = executor_type(input)?;
    let database_type = database_type(input)?;

    let with_code = with::generate_with(&executor_type, &database_type, &model)?;
    let by_code = by::generate_by(&executor_type, &database_type, &model)?;
    let select_code = select::generate_select(&executor_type, &model)?;
    let delete_code = delete::generate_delete(&executor_type, &model)?;
    let save_code = save::generate_save(&executor_type, &model)?;

    Ok(TokenStream::from(quote! {
        #with_code
        #by_code
        #select_code
        #delete_code
        #save_code
    }))
}
