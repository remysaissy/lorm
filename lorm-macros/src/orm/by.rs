use crate::models::OrmModel;
use crate::utils::{
    db_placeholder, get_bind_param_type_and_usage, get_bind_type_where_constraint, to_column_type,
};
use quote::{__private::TokenStream, format_ident, quote};

pub fn generate_by(
    executor_type: &TokenStream,
    database_type: &TokenStream,
    model: &OrmModel,
) -> syn::Result<TokenStream> {
    let trait_ident = format_ident!("{}ByTrait", model.struct_name);
    let struct_name = model.struct_name;
    let struct_visibility = model.struct_visibility;
    let table_name = &model.table_name;

    let stream: Vec<(TokenStream, TokenStream)> = model
        .query_columns()
        .map(|column| {
            let field_name = &column.field;
            let column_name = &column.column_name;

            let lifetime = quote! {'a};
            let parameter = quote! {value};
            let (param_type, param_value) =
                get_bind_param_type_and_usage(&parameter, &column.ty, &lifetime).unwrap();
            let by_fn = format_ident!("by_{}", field_name);

            let columns = model.full_column_select();
            let placeholder = db_placeholder(column.base_field, 1).unwrap();
            let sql_ident =
                format!("SELECT {columns} FROM {table_name} WHERE {column_name} = {placeholder}");

            let field_type_constraints = if column.column_properties.use_json {
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

            let signature = quote! {
                async fn #by_fn<#lifetime>(executor: E, #parameter: #param_type) -> lorm::errors::Result<#struct_name> where #field_type_constraints
            };

            let trait_code = quote! {
                #signature;
            };

            let impl_code = quote! {
                #signature {
                    let r = sqlx::query_as::<_, #struct_name>(#sql_ident)
                        .bind(#bind_value)
                        .fetch_one(executor).await?;
                    Ok(r)
                }
            };
            (trait_code, impl_code)
        })
        .collect::<Vec<(_, _)>>();
    let (mut trait_tokens, mut impl_tokens): (Vec<TokenStream>, Vec<TokenStream>) =
        stream.into_iter().unzip();

    // Composite pk selector for Manual primary keys
    if !model.primary_key.is_generated() {
        let pk_fields = model.primary_key.fields();

        // Avoid duplicate by_<pk> when user also asked for #[lorm(by)] on the pk field.
        let would_duplicate_default_single_pk = model.pk_selector_name.starts_with("by_")
            && pk_fields.len() == 1
            && pk_fields[0].column_properties.generate_by
            && model.pk_selector_name == format!("by_{}", pk_fields[0].field);

        if !would_duplicate_default_single_pk {
            let selector_ident = format_ident!("{}", model.pk_selector_name);
            let lifetime = quote! {'a};

            let mut where_parts: Vec<String> = Vec::new();
            let mut param_decls: Vec<TokenStream> = Vec::new();
            let mut binds: Vec<TokenStream> = Vec::new();
            let mut constraints: Vec<TokenStream> = Vec::new();

            for (i, col) in pk_fields.iter().enumerate() {
                let param_ident = &col.field;
                let param_expr = quote! { #param_ident };
                let (param_type, param_use) =
                    get_bind_param_type_and_usage(&param_expr, &col.ty, &lifetime).unwrap();
                let constraint =
                    get_bind_type_where_constraint(&col.ty, database_type, &lifetime).unwrap();

                param_decls.push(quote! { #param_ident: #param_type });
                binds.push(quote! { .bind(#param_use) });
                constraints.push(constraint);

                let placeholder = db_placeholder(col.base_field, i + 1).unwrap();
                where_parts.push(format!("{} = {}", col.column_name, placeholder));
            }

            let where_clause = where_parts.join(" AND ");
            let columns = model.full_column_select();
            let sql_ident = format!("SELECT {columns} FROM {table_name} WHERE {where_clause}");

            let signature = quote! {
                async fn #selector_ident<#lifetime>(executor: E, #(#param_decls),*) -> lorm::errors::Result<#struct_name> where #(#constraints),*
            };

            trait_tokens.push(quote! {
                #signature;
            });
            impl_tokens.push(quote! {
                #signature {
                    let r = sqlx::query_as::<_, #struct_name>(#sql_ident)
                        #(#binds)*
                        .fetch_one(executor).await?;
                    Ok(r)
                }
            });
        }
    }

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
