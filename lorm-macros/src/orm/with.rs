use crate::models::OrmModel;
use crate::utils::{
    db_placeholder, get_bind_param_type_and_usage, get_bind_type_where_constraint, to_column_type,
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
    let table_columns = model.full_column_select();

    let stream: Vec<(TokenStream, TokenStream)> = model.query_columns().map(|column| (|| -> syn::Result<_> {
        let field_name = &column.field;
        let column_name = &column.column_name;

        let lifetime = quote! {'a};
        let param = quote! {value};
        let (param_type, param_value) = get_bind_param_type_and_usage(&param, &column.ty, &lifetime)?;

        let constraints = if column.column_properties.use_json {
            let base_type = to_column_type(&column.ty).unwrap();
            quote! { #base_type: serde::Serialize }
        } else {
            get_bind_type_where_constraint(&column.ty, database_type, &lifetime).unwrap()
        };

        let bind_value = if column.column_properties.use_json {
            quote! { sqlx::types::Json(#param_value) }
        } else {
            param_value.clone()
        };

        let with_fn = format_ident!("with_{}",field_name);
        let placeholder = db_placeholder(column.base_field, 1).unwrap();

        let signature = quote! {
            async fn #with_fn<#lifetime>(executor: E, #param: #param_type) -> lorm::errors::Result<Vec<#struct_name>> where #constraints
        };
        let trait_code = quote! {
            #signature;
        };
        let sql_ident = format!("SELECT {table_columns} FROM {table_name} WHERE {column_name} = {placeholder}");

        let impl_code = quote! {
            #signature {
                let r = sqlx::query_as::<_, Self>(#sql_ident)
                    .bind(#bind_value)
                    .fetch_all(executor).await?;
                Ok(r)
            }
        };
        Ok((trait_code, impl_code))
    })()).collect::<Result<Vec<(_, _)>, _>>()?;
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
