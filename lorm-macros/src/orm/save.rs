use crate::models::PrimaryKey::{Generated, Manual};
use crate::models::{OrmModel, PrimaryKey};
use crate::utils::{
    create_insert_placeholders, get_field_name, get_is_set, get_new_method, is_readonly,
};
use quote::{__private::TokenStream, format_ident, quote};
use syn::spanned::Spanned;

pub fn generate_save(executor_type: &TokenStream, model: &OrmModel) -> syn::Result<TokenStream> {
    let save_trait_ident = format_ident!("{}SaveTrait", model.struct_name);
    let struct_name = model.struct_name;
    let struct_visibility = model.struct_visibility;
    let table_name = &model.table_name;
    let table_columns = &model.table_columns;

    // Created at
    let created_at_var = quote! {created_at};
    let created_at_code = match model.created_at_field.as_ref() {
        None => quote! {},
        Some(field) => {
            if model.is_created_at_readonly {
                quote! {}
            } else {
                let new_method = get_new_method(field);
                quote! {
                    let #created_at_var = #new_method;
                }
            }
        }
    };
    // Updated at
    let updated_at_var = quote! {updated_at};
    let updated_at_code = match model.updated_at_field.as_ref() {
        None => quote! {},
        Some(field) => {
            if model.is_updated_at_readonly {
                quote! {}
            } else {
                let new_method = get_new_method(field);
                quote! {
                    let #updated_at_var = #new_method;
                }
            }
        }
    };
    // Primary key
    let primary_key_var = quote! {primary_key};
    let pk_code = match model.primary_key {
        Generated(field) => {
            if !is_readonly(field) {
                let field_ident = field.ident.as_ref().ok_or_else(|| {
                    syn::Error::new(field.span(), "Primary key field must have an identifier.")
                })?;
                let pk_is_default_method = get_is_set(field);
                let pk_new_method = get_new_method(field);
                quote! {
                    let #primary_key_var = if self.#pk_is_default_method {
                        &#pk_new_method
                    } else {
                        &self.#field_ident
                    };
                }
            } else {
                quote! {}
            }
        }
        Manual(..) => quote! {},
    };

    let is_created_at_field = |field: &syn::Field| -> bool {
        if let Some(created_at_field) = model.created_at_field {
            field == created_at_field
        } else {
            false
        }
    };

    let is_updated_at_field = |field: &syn::Field| -> bool {
        if let Some(created_at_field) = model.updated_at_field {
            field == created_at_field
        } else {
            false
        }
    };

    let is_generated_pk_field = |field: &syn::Field| -> bool {
        if let Generated(pk_field) = model.primary_key {
            field == pk_field
        } else {
            false
        }
    };

    // prepare `insertable` fields
    let insert_columns_vec: Vec<String> = model
        .insert_fields
        .iter()
        .map(|field| get_field_name(field))
        .collect();
    let insert_values: Vec<_> = model
        .insert_fields
        .iter()
        .map(|field| {
            if is_created_at_field(field) {
                created_at_var.clone()
            } else if is_updated_at_field(field) {
                updated_at_var.clone()
            } else if is_generated_pk_field(field) {
                primary_key_var.clone()
            } else {
                let ident = field.ident.as_ref();
                quote! {&self.#ident}
            }
        })
        .collect();
    let insert_columns = insert_columns_vec.join(",");
    let insert_value_placeholders = create_insert_placeholders(&model.insert_fields);

    // find `updatable` fields and generate the set clause for upsert
    let upsert_clause = model
        .update_fields
        .iter()
        .filter(|f| !is_created_at_field(f))
        .map(|f| {
            let field_name = get_field_name(f);
            format!("{field_name} = excluded.{field_name}")
        })
        .collect::<Vec<_>>()
        .join(",");

    let pk_columns = model.primary_key.columns();

    let upsert_sql_ident = format!(
        "INSERT INTO {table_name} ({insert_columns}) VALUES ({insert_value_placeholders}) ON CONFLICT ({pk_columns}) DO UPDATE SET {upsert_clause} RETURNING {table_columns}",
    );

    Ok(quote! {
        #struct_visibility trait #save_trait_ident<'e, E: #executor_type>: Sized {
            async fn save(&self, executor: E) -> lorm::errors::Result<#struct_name>;
        }

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
