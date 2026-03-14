use crate::models::OrmModel;
use crate::utils::{
    db_placeholder, get_bind_param_type_and_usage, get_bind_type_where_constraint, get_column_name,
};
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
    let table_columns = &model.table_columns;

    let stream: Vec<(TokenStream, TokenStream)> = model.by_fields.iter().map(|field| (|| -> syn::Result<_> {
        let field_ident = field.ident.as_ref().unwrap();
        let field_name = get_column_name(field);

        let lifetime = quote! {'a};

        let constraints = get_bind_type_where_constraint(field, database_type, &lifetime).unwrap();
        let param = quote! {value};
        let (param_type, param_value) = get_bind_param_type_and_usage(&param, field, &lifetime)?;

        let with_fn = format_ident!("with_{}",field_ident);
        let placeholder = db_placeholder(field, 1).unwrap();

        let signature = quote! {
            async fn #with_fn<#lifetime>(executor: E, #param: #param_type) -> lorm::errors::Result<Vec<#struct_name>> where #constraints
        };
        let trait_code = quote! {
            #signature;
        };
        let sql_ident = format!("SELECT {} FROM {} WHERE {} = {}", table_columns, table_name, field_name, placeholder);

        let impl_code = quote! {
            #signature {
                let r = sqlx::query_as::<_, Self>(#sql_ident)
                    .bind(#param_value)
                    .fetch_all(executor).await?;
                Ok(r)
            }
        };
        Ok((trait_code, impl_code))
    })()).collect::<Result<_, _>>()?;
    let (trait_tokens, impl_tokens): (Vec<TokenStream>, Vec<TokenStream>) =
        stream.into_iter().unzip();

    Ok(quote! {
        #struct_visibility trait #trait_ident<'e, E: #executor_type>: Sized {
            #(#trait_tokens)*
        }

        #[automatically_derived]
        impl<'e, E: #executor_type> #trait_ident<'e, E> for #struct_name {
            #(#impl_tokens)*
        }
    })
}
