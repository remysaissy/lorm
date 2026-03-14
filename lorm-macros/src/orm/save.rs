use crate::models::PrimaryKey::{Generated, Manual};
use crate::models::{OrmModel, UpsertField};
use crate::utils::{create_insert_placeholders, get_is_set, get_new_method, is_readonly};
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
            if is_readonly(field) {
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
            if is_readonly(field) {
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
        .upsert_fields
        .iter()
        .flat_map(|field| field.column_names())
        .collect();
    let insert_values: Vec<_> = model
        .upsert_fields
        .iter()
        .flat_map(|field| match field {
            UpsertField::Field(f) if is_created_at_field(f) => vec![created_at_var.clone()],
            UpsertField::Field(f) if is_updated_at_field(f) => vec![updated_at_var.clone()],
            UpsertField::Field(f) if is_generated_pk_field(f) => vec![primary_key_var.clone()],
            UpsertField::Field(f) => {
                let ident = f.ident.as_ref();
                vec![quote! {&self.#ident}]
            }
            UpsertField::Flattened(f, flattened_fields) => {
                let field_ident = f.ident.as_ref();
                let base = quote! {&self.#field_ident};
                flattened_fields
                    .iter()
                    .map(|flattened| {
                        let field = &flattened.field;
                        quote! {#base.#field}
                    })
                    .collect()
            }
        })
        .collect();
    let insert_columns = insert_columns_vec.join(",");
    let insert_value_placeholders = create_insert_placeholders(&model.upsert_fields);

    // find `updatable` fields and generate the set clause for upsert
    let upsert_clause = model
        .upsert_fields
        .iter()
        .filter(|field| !matches!(field, UpsertField::Field(f) if is_created_at_field(f)))
        .flat_map(|f| f.column_names())
        .map(|column_name| format!("{column_name} = excluded.{column_name}"))
        .collect::<Vec<_>>()
        .join(",");

    let pk_columns = model.primary_key.column_names();

    let upsert_sql_ident = format!(
        "INSERT INTO {table_name} ({insert_columns}) VALUES ({insert_value_placeholders}) ON CONFLICT ({pk_columns}) DO UPDATE SET {upsert_clause} RETURNING {table_columns}",
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
