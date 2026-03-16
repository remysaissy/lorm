use crate::models::OrmModel;
use crate::utils::{db_placeholder, get_bind_param_type_and_usage, get_bind_type_where_constraint};
use quote::{__private::TokenStream, format_ident, quote};
use syn::spanned::Spanned;

pub fn generate_with(
    executor_type: &TokenStream,
    database_type: &TokenStream,
    model: &OrmModel,
) -> syn::Result<TokenStream> {
    let trait_ident = format_ident!("{}WithTrait", model.struct_name);
    let struct_name = model.struct_name;
    let struct_visibility = model.struct_visibility;
    let table_name = &model.table_name;
    let table_columns = &model.full_select_columns();

    let stream: Vec<(TokenStream, TokenStream)> = model.fields.iter().filter(|f| f.should_generate_selector(&model.primary_key)).map(|field| (|| -> syn::Result<_> {
        let field_ident = &field.field;
        let column_name = &field.column_name;

        let lifetime = quote! {'a};

        let constraints = get_bind_type_where_constraint(&field.ty, database_type, &lifetime).unwrap();
        let param = quote! {value};
        let (param_type, param_value) = get_bind_param_type_and_usage(&param, &field.ty, &lifetime)?;

        let with_fn = format_ident!("with_{}",field_ident);
        let placeholder = db_placeholder(field.base_field.span(), 1).unwrap();

        let signature = quote! {
            async fn #with_fn<#lifetime>(executor: E, #param: #param_type) -> lorm::errors::Result<Vec<#struct_name>> where #constraints
        };
        let trait_code = quote! {
            #signature;
        };
        let sql_ident = format!("SELECT {} FROM {} WHERE {} = {}", table_columns, table_name, column_name, placeholder);

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
