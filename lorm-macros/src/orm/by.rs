use crate::helpers::{db_placeholder, get_field_name};
use crate::models::OrmModel;
use crate::util::get_option_type;
use quote::{__private::TokenStream, format_ident, quote};

pub fn generate_by(db_pool_type: &TokenStream, model: &OrmModel) -> syn::Result<TokenStream> {
    let trait_ident = format_ident!("{}ByTrait", model.struct_name);
    let struct_name = model.struct_name;
    let struct_visibility = model.struct_visibility;
    let table_name = &model.table_name;
    let table_columns = &model.table_columns;

    let stream: Vec<(TokenStream, TokenStream)> = model.by_fields.iter().map(|field| {
        let field_ident = field.ident.as_ref().unwrap();
        let (_, field_type) = get_option_type(&field.ty);
        let field_name = get_field_name(field);
        let by_fn = format_ident!("by_{}",field_ident);
        let placeholder = db_placeholder(field, 1).unwrap();

        let trait_code = quote! {
            async fn #by_fn(pool: &#db_pool_type, value: #field_type) -> lorm::errors::Result<Option<#struct_name>>;
        };

        let impl_code = quote! {
            async fn #by_fn(pool: &#db_pool_type, value: #field_type) -> lorm::errors::Result<Option<#struct_name>> {
                let sql = format!("SELECT {} FROM {} WHERE {} = {}",#table_columns, #table_name, #field_name, #placeholder);
                let r = sqlx::query_as::<_, Self>(&sql)
                .bind(value)
                .fetch_optional(pool).await?;
                Ok(r)
            }
        };
        (trait_code, impl_code)
    }).collect::<Vec<(_, _)>>();
    let (trait_tokens, impl_tokens): (Vec<TokenStream>, Vec<TokenStream>) =
        stream.into_iter().unzip();

    Ok(quote! {
        #struct_visibility trait #trait_ident {
            #(#trait_tokens)*
        }

        impl #trait_ident for #struct_name {
            #(#impl_tokens)*
        }
    })
}
