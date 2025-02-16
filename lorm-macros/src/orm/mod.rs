mod by;
mod delete;
mod fk;
mod save;
mod select;

use crate::helpers::*;
use crate::models::OrmModel;
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
    let db_pool_type = db_pool_type(input)?;

    let by_code = by::generate_by(&db_pool_type, &model)?;
    let fk_code = fk::generate_fk(&db_pool_type, &model)?;
    let select_code = select::generate_select(&db_pool_type, &model)?;
    let delete_code = delete::generate_delete(&db_pool_type, &model)?;
    let save_code = save::generate_save(&db_pool_type, &model)?;

    Ok(TokenStream::from(quote! {
        #by_code
        #fk_code
        #select_code
        #delete_code
        #save_code
    }))
}
