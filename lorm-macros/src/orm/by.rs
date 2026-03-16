use crate::models::{OrmModel, PrimaryKey};
use crate::utils::{
    db_placeholder, get_bind_param_type_and_usage, get_bind_type_where_constraint,
};
use quote::{__private::TokenStream, format_ident, quote};
use syn::spanned::Spanned;

pub fn generate_by(
    executor_type: &TokenStream,
    database_type: &TokenStream,
    model: &OrmModel,
) -> syn::Result<TokenStream> {
    let trait_ident = format_ident!("{}ByTrait", model.struct_name);
    let struct_name = model.struct_name;
    let struct_visibility = model.struct_visibility;
    let table_name = &model.table_name;
    let all_columns = model.full_select_columns();

    let stream: Vec<(TokenStream, TokenStream)> = model.fields.iter().filter(|field| field.column_properties.generate_by).map(|field| {
        let lifetime = quote! {'a};
        let parameter = quote! {value};
        let field_type_constraints = get_bind_type_where_constraint(&field.ty, database_type, &lifetime).unwrap();
        let (param_type, param_value) = get_bind_param_type_and_usage(&parameter, &field.ty, &lifetime).unwrap();

        let by_fn = format_ident!("by_{}",&field.field);
        let placeholder = db_placeholder(field.base_field.span(), 1).unwrap();

        let column_name = &field.column_name;

        let sql_ident = format!("SELECT {all_columns} FROM {table_name} WHERE {column_name} = {placeholder}");

        let signature = quote! {
            async fn #by_fn<#lifetime>(executor: E, #parameter: #param_type) -> lorm::errors::Result<#struct_name> where #field_type_constraints
        };

        let trait_code = quote! {
            #signature;
        };

        let impl_code = quote! {
            #signature {
                let r = sqlx::query_as::<_, #struct_name>(#sql_ident)
                    .bind(#param_value)
                    .fetch_one(executor).await?;
                Ok(r)
            }
        };
        (trait_code, impl_code)
    }).collect::<Vec<(_, _)>>();
    let primary_key_by: Box<dyn Iterator<Item = (TokenStream, TokenStream)>> = Box::new(
        match &model.primary_key {
            PrimaryKey::Generated(..) => None.into_iter(),
            PrimaryKey::Manual(fields) => {
                if fields.len() > 1 {
                    let by_fn_name = &model.primary_key_by_name;
                    let lifetime = quote! {'a};

                    let type_constraints = fields
                        .iter()
                        .map(|field| {
                            get_bind_type_where_constraint(&field.ty, database_type, &lifetime)
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    let type_constraints = quote! { #(#type_constraints),* };

                    let (param_decl, param_usage): (Vec<_>, Vec<_>) = model
                        .primary_key
                        .fields()
                        .iter()
                        .map(|field| {
                            (|| -> syn::Result<_> {
                                let ident = &field.field;
                                let param = quote! {#ident};
                                let (param_type, usage) =
                                    get_bind_param_type_and_usage(&param, &field.ty, &lifetime)?;
                                Ok((quote! {#ident: #param_type}, usage))
                            })()
                        })
                        .collect::<Result<Vec<_>, _>>()?
                        .into_iter()
                        .unzip();

                    let parameters = quote! { #(#param_decl),* };

                    let signature = quote! {async fn #by_fn_name<#lifetime>(executor: E, #parameters) -> lorm::errors::Result<#struct_name> where #type_constraints};
                    let trait_code = quote! {#signature;};

                    let where_clause = fields
                        .iter()
                        .enumerate()
                        .map(|(i, field)| {
                            db_placeholder(field.base_field.span(), i + 1).map(|placeholder| {
                                let column_name = &field.column_name;
                                format!("{column_name} = {placeholder}")
                            })
                        })
                        .collect::<Result<Vec<_>, _>>()?
                        .join(" AND ");

                    let sql_ident =
                        format!("SELECT {all_columns} FROM {table_name} WHERE {where_clause}");

                    let impl_code = quote! {
                        #signature {
                            let r = sqlx::query_as::<_, #struct_name>(#sql_ident)
                                #(
                                .bind(#param_usage)
                                )*
                                .fetch_one(executor).await?;
                            Ok(r)
                        }
                    };
                    Some((trait_code, impl_code)).into_iter()
                } else {
                    None.into_iter()
                }
            }
        },
    );
    let (trait_tokens, impl_tokens): (Vec<TokenStream>, Vec<TokenStream>) =
        stream.into_iter().chain(primary_key_by).unzip();

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
