use crate::helpers::{
    create_insert_placeholders, create_update_placeholders, db_placeholder, get_field_name,
    get_is_set, get_new_method,
};
use crate::models::OrmModel;
use quote::{__private::TokenStream, format_ident, quote};
use syn::Ident;

pub fn generate_save(executor_type: &TokenStream, model: &OrmModel) -> syn::Result<TokenStream> {
    let save_trait_ident = format_ident!("{}SaveTrait", model.struct_name);
    let struct_name = model.struct_name;
    let struct_visibility = model.struct_visibility;
    let table_name = &model.table_name;
    let table_columns = &model.table_columns;

    // prepare `insertable` fields
    let mut insert_columns_vec: Vec<String> = vec![];
    let mut insert_values: Vec<Option<&Ident>> = vec![];
    for field in model.insert_fields.iter() {
        insert_columns_vec.push(get_field_name(field));
        insert_values.push(field.ident.as_ref());
    }
    let insert_columns = insert_columns_vec.join(",");
    let insert_value_placeholders = create_insert_placeholders(&model.insert_fields);

    // find `updatable` fields
    let update_value_placeholders = create_update_placeholders(&model.update_fields);
    let update_values = model
        .update_fields
        .iter()
        .map(|field| &field.ident)
        .collect::<Vec<_>>();

    // Primary key
    let pk_column = model.pk_field.ident.as_ref().unwrap();
    let pk_name = get_field_name(model.pk_field);
    let pk_placeholder = format!(
        "{} = {}",
        pk_name,
        db_placeholder(model.pk_field, model.update_fields.len() + 1).unwrap()
    );
    let pk_is_default_method = get_is_set(model.pk_field);
    let pk_code = if model.is_pk_readonly {
        quote! {}
    } else {
        let pk_new_method = get_new_method(model.pk_field);
        quote! {
            to_save.#pk_column = #pk_new_method;
        }
    };

    let created_at_code = match model.created_at_field.as_ref() {
        None => quote! {},
        Some(field) => {
            if model.is_created_at_readonly {
                quote! {}
            } else {
                let new_method = get_new_method(field);
                let column = field.ident.as_ref().unwrap();
                quote! {
                    to_save.#column = #new_method;
                }
            }
        }
    };

    let updated_at_code = match model.updated_at_field.as_ref() {
        None => quote! {},
        Some(field) => {
            if model.is_updated_at_readonly {
                quote! {}
            } else {
                let new_method = get_new_method(field);
                let column = field.ident.as_ref().unwrap();
                quote! {
                    to_save.#column = #new_method;
                }
            }
        }
    };

    let insert_sql_ident = format!(
        "INSERT INTO {} ({}) VALUES ({}) RETURNING {}",
        table_name, insert_columns, insert_value_placeholders, table_columns
    );
    let update_sql_ident = format!(
        "UPDATE {} SET {} WHERE {} RETURNING {}",
        table_name, update_value_placeholders, pk_placeholder, table_columns
    );

    Ok(quote! {
        #struct_visibility trait #save_trait_ident<'e, E: #executor_type>: Sized {
            async fn save(&self, executor: E) -> lorm::errors::Result<#struct_name>;
        }

        impl<'e, E: #executor_type> #save_trait_ident<'e, E> for #struct_name
        {
            async fn save(&self, executor: E) -> lorm::errors::Result<#struct_name>
            {
                let mut to_save = self.clone();
                #updated_at_code
                match to_save.#pk_is_default_method {
                    true => {
                        #pk_code
                        #created_at_code
                        let r = sqlx::query_as::<_, #struct_name>(#insert_sql_ident)
                        #(
                            .bind(&to_save.#insert_values)
                        )*
                        .fetch_one(executor).await?;
                        Ok(r)
                    },
                    false => {
                        let r = sqlx::query_as::<_, #struct_name>(#update_sql_ident)
                        #(
                            .bind(&self.#update_values)
                        )*
                        .bind(&self.#pk_column)
                        .fetch_one(executor).await?;
                        Ok(r)
                    }
                }
            }
        }
    })
}
