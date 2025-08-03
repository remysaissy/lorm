use crate::helpers::{db_placeholder, get_field_name};
use crate::models::OrmModel;
use crate::util::get_type_as_reference;
use quote::{__private::TokenStream, format_ident, quote};

pub fn generate_with(executor_type: &TokenStream, model: &OrmModel) -> syn::Result<TokenStream> {
    let trait_ident = format_ident!("{}WithTrait", model.struct_name);
    let struct_name = model.struct_name;
    let struct_visibility = model.struct_visibility;
    let table_name = &model.table_name;
    let table_columns = &model.table_columns;

    let stream: Vec<(TokenStream, TokenStream)> = model.by_fields.iter().map(|field| {
        let field_ident = field.ident.as_ref().unwrap();
        let field_type = get_type_as_reference(&field.ty).unwrap();
        let field_name = get_field_name(field);
        let with_fn = format_ident!("with_{}",field_ident);
        let placeholder = db_placeholder(field, 1).unwrap();
        let trait_code = quote! {
            async fn #with_fn(executor: E, value: #field_type) -> lorm::errors::Result<Vec<#struct_name>>;
        };
        let sql_ident = format!("SELECT {} FROM {} WHERE {} = {}", table_columns, table_name, field_name, placeholder);

        let impl_code = quote! {
            async fn #with_fn(executor: E, value: #field_type) -> lorm::errors::Result<Vec<#struct_name>> {
                let r = sqlx::query_as::<_, Self>(#sql_ident)
                    .bind(value)
                    .fetch_all(executor).await?;
                Ok(r)
            }
        };
        (trait_code, impl_code)
    }).collect::<Vec<(_, _)>>();
    let (trait_tokens, impl_tokens): (Vec<TokenStream>, Vec<TokenStream>) =
        stream.into_iter().unzip();

    Ok(quote! {
        #struct_visibility trait #trait_ident<'e, E: #executor_type>: Sized {
            #(#trait_tokens)*
        }

        impl<'e, E: #executor_type> #trait_ident<'e, E> for #struct_name {
            #(#impl_tokens)*
        }
    })
}
