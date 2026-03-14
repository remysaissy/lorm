use crate::models::{OrmModel, PrimaryKey};
use crate::utils::{
    db_placeholder, get_bind_param_type_and_usage, get_bind_type_where_constraint, get_column_name,
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
    let table_columns = &model.table_columns;

    let stream: Vec<(TokenStream, TokenStream)> = model.by_fields.iter().map(|field| {
        let field_ident = field.ident.as_ref().unwrap();

        let lifetime = quote! {'a};
        let parameter = quote! {value};
        let field_type_constraints = get_bind_type_where_constraint(field, database_type, &lifetime).unwrap();
        let (param_type, param_value) = get_bind_param_type_and_usage(&parameter, field, &lifetime).unwrap();
        let field_name = get_column_name(field);
        let by_fn = format_ident!("by_{}",field_ident);
        let placeholder = db_placeholder(field, 1).unwrap();
        let sql_ident = format!("SELECT {} FROM {} WHERE {} = {}", table_columns, table_name, field_name, placeholder);

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
                        .enumerate()
                        .map(|(i, field)| {
                            get_bind_type_where_constraint(field, database_type, &lifetime)
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    let type_constraints = quote! { #(#type_constraints),* };

                    let (param_decl, param_usage): (Vec<_>, Vec<_>) = model
                        .primary_key
                        .fields()
                        .iter()
                        .map(|field| {
                            (|| -> syn::Result<_> {
                                let ident = field.ident.as_ref().ok_or_else(|| {
                                    syn::Error::new(field.span(), "No ident for field")
                                })?;
                                let param = quote! {#ident};
                                let (param_type, usage) =
                                    get_bind_param_type_and_usage(&param, field, &lifetime)?;
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
                            db_placeholder(field, i + 1).map(|placeholder| {
                                let field_name = get_column_name(field);
                                format!("{field_name} = {placeholder}")
                            })
                        })
                        .collect::<Result<Vec<_>, _>>()?
                        .join(" AND ");

                    let sql_ident =
                        format!("SELECT {table_columns} FROM {table_name} WHERE {where_clause}");

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
