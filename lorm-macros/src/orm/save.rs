use crate::models::OrmModel;
use crate::models::PrimaryKey::{Generated, Manual};
use crate::utils::create_insert_placeholders;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

pub fn generate_save(executor_type: &TokenStream, model: &OrmModel) -> syn::Result<TokenStream> {
    let save_trait_ident = format_ident!("{}SaveTrait", model.struct_name);
    let struct_name = model.struct_name;
    let struct_visibility = model.struct_visibility;
    let table_name = &model.table_name;

    let upsert_fields = model
        .fields
        .iter()
        .filter(|field| !field.column_properties.readonly)
        .collect::<Vec<_>>();

    let table_columns = model.full_select_columns();
    let insert_columns = upsert_fields
        .iter()
        .map(|field| field.column_name.as_str())
        .collect::<Vec<_>>()
        .join(", ");

    // Created at
    let created_at_var = quote! {created_at};
    let created_at_field = model.fields.iter().find(|f| f.column_properties.created_at);
    let created_at_code = match created_at_field {
        None => quote! {},
        Some(field) => {
            if field.column_properties.readonly {
                quote! {}
            } else {
                let new_method = &field.column_properties.new_expression;
                quote! {
                    let #created_at_var = #new_method;
                }
            }
        }
    };
    // Updated at
    let updated_at_var = quote! {updated_at};
    let updated_at_field = model.fields.iter().find(|f| f.column_properties.updated_at);
    let updated_at_code = match updated_at_field {
        None => quote! {},
        Some(field) => {
            if field.column_properties.readonly {
                quote! {}
            } else {
                let new_method = &field.column_properties.new_expression;
                quote! {
                    let #updated_at_var = #new_method;
                }
            }
        }
    };
    // Primary key
    let primary_key_var = quote! {primary_key};
    let pk_code = match &model.primary_key {
        Generated(field) => {
            if !field.column_properties.readonly {
                let pk_is_default_method = &field.column_properties.is_set_expression;
                let pk_new_method = &field.column_properties.new_expression;
                let accessor = field.self_accessor();
                quote! {
                    let #primary_key_var = if #pk_is_default_method(#accessor) {
                        &#pk_new_method
                    } else {
                        #accessor
                    };
                }
            } else {
                quote! {}
            }
        }
        Manual(..) => quote! {},
    };

    // prepare `insertable` fields
    let insert_values: Vec<_> = upsert_fields
        .iter()
        .map(|field| {
            if field.column_properties.created_at {
                created_at_var.clone()
            } else if field.column_properties.updated_at {
                updated_at_var.clone()
            } else if field.column_properties.primary_key && model.primary_key.is_generated() {
                primary_key_var.clone()
            } else {
                let accessor = field.self_accessor();
                let value = quote! {#accessor};
                if field.column_properties.use_json {
                    quote! {sqlx::types::Json(#value)}
                } else {
                    value
                }
            }
        })
        .collect();
    let insert_value_placeholders = create_insert_placeholders(&upsert_fields);

    // find `updatable` fields and generate the set clause for upsert
    let upsert_clause = upsert_fields
        .iter()
        .filter(|field| !field.column_properties.created_at)
        .map(|f| f.column_name.as_str())
        .map(|column_name| format!("{column_name} = excluded.{column_name}"))
        .collect::<Vec<_>>()
        .join(",");

    let pk_columns = model
        .primary_key
        .column_names()
        .collect::<Vec<_>>()
        .join(",");

    let is_full_key = {
        let mut pk = model.primary_key.column_names().collect::<Vec<_>>();
        pk.sort();
        let mut cols = model
            .fields
            .iter()
            .map(|f| f.column_name.as_str())
            .collect::<Vec<_>>();
        cols.sort();
        pk == cols
    };

    let upsert_clause = if is_full_key {
        "".to_string()
    } else {
        format!("ON CONFLICT ({pk_columns}) DO UPDATE SET {upsert_clause}")
    };

    let upsert_sql_ident = format!(
        "INSERT INTO {table_name} ({insert_columns}) VALUES ({insert_value_placeholders}) {upsert_clause} RETURNING {table_columns}",
    );

    Ok(quote! {
        #struct_visibility trait #save_trait_ident<'e, E: #executor_type>: Sized {
            async fn save(&self, executor: E) -> lorm::errors::Result<#struct_name>;
        }

        #[automatically_derived]
        impl<'e, E: #executor_type> #save_trait_ident<'e, E> for #struct_name
        {
            async fn save(&self, executor: E) -> lorm::errors::Result<#struct_name>
            {
                #created_at_code
                #updated_at_code
                #pk_code
                let result = sqlx::query_as::<_, #struct_name>(#upsert_sql_ident)
                #(
                    .bind(#insert_values)
                )*
                .fetch_one(executor).await?;
                Ok(result)
            }
        }
    })
}
