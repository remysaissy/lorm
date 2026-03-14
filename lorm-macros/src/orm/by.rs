use crate::models::{OrmModel, PrimaryKey};
use crate::utils::{
    db_placeholder, get_bind_type_constraint, get_field_name, get_primary_key_by_ident,
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
        let field_type_constraints = get_bind_type_constraint(field, database_type).unwrap();
        let field_name = get_field_name(field);
        let by_fn = format_ident!("by_{}",field_ident);
        let placeholder = db_placeholder(field, 1).unwrap();
        let sql_ident = format!("SELECT {} FROM {} WHERE {} = {}", table_columns, table_name, field_name, placeholder);
        let trait_code = quote! {
            async fn #by_fn<T: #field_type_constraints>(executor: E, value: T) -> lorm::errors::Result<#struct_name>;
        };

        let impl_code = quote! {
            async fn #by_fn<T: #field_type_constraints>(executor: E, value: T) -> lorm::errors::Result<#struct_name> {
                let r = sqlx::query_as::<_, #struct_name>(#sql_ident)
                    .bind(value)
                    .fetch_one(executor).await?;
                Ok(r)
            }
        };
        (trait_code, impl_code)
    }).collect::<Vec<(_, _)>>();
    let primary_key_by: Box<dyn Iterator<Item = (TokenStream, TokenStream)>> = Box::new(
        match &model.primary_key {
            PrimaryKey::Generated(field) => None.into_iter(),
            PrimaryKey::Manual(fields) => {
                if fields.len() > 1 {
                    let by_fn_name = &model.primary_key_by_name;
                    let type_var_idents = (0..fields.len())
                        .map(|i| format_ident!("T{}", i))
                        .collect::<Vec<_>>();

                    let type_constraints = fields
                        .iter()
                        .enumerate()
                        .map(|(i, field)| {
                            get_bind_type_constraint(field, database_type).map(|constraint| {
                                let type_var = &type_var_idents[i];
                                quote! {
                                    #type_var: #constraint
                                }
                            })
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    let type_constraints = quote! { #(#type_constraints),* };

                    let field_idents = model.primary_key.columns()?;

                    let parameters = field_idents
                        .iter()
                        .zip(type_var_idents.iter())
                        .map(|(field, type_var)| {
                            quote! {#field: #type_var}
                        })
                        .collect::<Vec<_>>();
                    let parameters = quote! { #(#parameters),* };

                    let trait_code = quote! {
                        async fn #by_fn_name<#type_constraints>(executor: E, #parameters) -> lorm::errors::Result<#struct_name>;
                    };

                    let where_clause = fields
                        .iter()
                        .enumerate()
                        .map(|(i, field)| {
                            db_placeholder(field, i + 1).map(|placeholder| {
                                let field_name = get_field_name(field);
                                format!("{field_name} = {placeholder}")
                            })
                        })
                        .collect::<Result<Vec<_>, _>>()?
                        .join(" AND ");

                    let sql_ident =
                        format!("SELECT {table_columns} FROM {table_name} WHERE {where_clause}");

                    let impl_code = quote! {
                        async fn #by_fn_name<#type_constraints>(executor: E, #parameters) -> lorm::errors::Result<#struct_name> {
                            let r = sqlx::query_as::<_, #struct_name>(#sql_ident)
                                #(
                                .bind(#field_idents)
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

        impl<'e, E: #executor_type> #trait_ident<'e, E> for #struct_name {
            #(#impl_tokens)*
        }
    })
}
