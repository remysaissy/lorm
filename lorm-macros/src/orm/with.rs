use crate::models::OrmModel;
use crate::utils::{db_placeholder, get_bind_type_constraint};
use quote::{__private::TokenStream, format_ident, quote};

pub fn generate_with(
    executor_type: &TokenStream,
    database_type: &TokenStream,
    model: &OrmModel,
) -> syn::Result<TokenStream> {
    let trait_ident = format_ident!("{}WithTrait", model.struct_name);
    let struct_name = model.struct_name;
    let struct_visibility = model.struct_visibility;
    let table_name = &model.table_name;
    let table_columns = model.full_column_select();

    let stream: Vec<(TokenStream, TokenStream)> = model.query_columns().map(|column| {
        let field_name = &column.field;
        let column_name = &column.column_name;
        let field_type_constraints = get_bind_type_constraint(column.base_field, database_type).unwrap();
        let with_fn = format_ident!("with_{}",field_name);
        let placeholder = db_placeholder(column.base_field, 1).unwrap();
        let trait_code = quote! {
            async fn #with_fn<T: #field_type_constraints>(executor: E, value: T) -> lorm::errors::Result<Vec<#struct_name>>;
        };
        let sql_ident = format!("SELECT {table_columns} FROM {table_name} WHERE {column_name} = {placeholder}");

        let impl_code = quote! {
            async fn #with_fn<T: #field_type_constraints>(executor: E, value: T) -> lorm::errors::Result<Vec<#struct_name>> {
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
