use crate::models::OrmModel;
use crate::orm::column::Column;
use crate::utils::db_placeholder;
use quote::{__private::TokenStream, format_ident, quote};

pub fn generate_save(executor_type: &TokenStream, model: &OrmModel) -> syn::Result<TokenStream> {
    let save_trait_ident = format_ident!("{}SaveTrait", model.struct_name);
    let struct_name = model.struct_name;
    let struct_visibility = model.struct_visibility;
    let table_name = &model.table_name;

    let full_select_columns = model.full_column_select();

    // Primary key
    let primary_key = model.primary_key();
    let primary_key_var = quote! {primary_key};
    let pk_column = &primary_key.column_name;
    let pk_value = primary_key.self_accessor();
    let pk_placeholder = format!(
        "{} = {}",
        pk_column,
        db_placeholder(primary_key.base_field, model.update_columns().count() + 1)?
    );
    let pk_is_set = &primary_key
        .column_properties
        .is_set(quote! { #pk_value }, &primary_key.ty);
    let pk_code = if primary_key.column_properties.readonly {
        quote! {}
    } else {
        let pk_new_method = &primary_key.column_properties.new_expression;
        quote! {
            let #primary_key_var = #pk_new_method;
        }
    };

    // Created at
    let created_at_var = quote! {created_at};
    let created_at_code = match model.created_at() {
        None => quote! {},
        Some(column) => {
            if column.column_properties.readonly {
                quote! {}
            } else {
                let new_method = &column.column_properties.new_expression;
                quote! {
                    let #created_at_var = #new_method;
                }
            }
        }
    };

    // Updated at
    let updated_at_var = quote! {updated_at};
    let updated_at_code = match model.updated_at() {
        None => quote! {},
        Some(column) => {
            if column.column_properties.readonly {
                quote! {}
            } else {
                let new_method = &column.column_properties.new_expression;
                quote! {
                    let #updated_at_var = #new_method;
                }
            }
        }
    };

    let column_value = |column: &Column, use_created_at_var: bool| {
        if column.column_properties.created_at && use_created_at_var {
            created_at_var.clone()
        } else if column.column_properties.updated_at {
            updated_at_var.clone()
        } else if column.column_properties.primary_key {
            primary_key_var.clone()
        } else {
            column.self_accessor()
        }
    };

    // prepare `insertable` fields
    let insert_value_placeholders =
        create_insert_placeholders(&model.insert_columns().collect::<Vec<_>>());
    let insert_values = model
        .insert_columns()
        .map(|col| column_value(col, true))
        .collect::<Vec<_>>();
    let insert_columns = model
        .insert_columns()
        .map(|col| col.column_name.as_str())
        .collect::<Vec<_>>()
        .join(",");

    // find `updatable` fields
    let update_value_placeholders =
        create_update_placeholders(&model.update_columns().collect::<Vec<_>>());
    let update_values = model
        .update_columns()
        .map(|col| column_value(col, false))
        .collect::<Vec<_>>();

    let insert_sql_ident = format!(
        "INSERT INTO {table_name} ({insert_columns}) VALUES ({insert_value_placeholders}) RETURNING {full_select_columns}"
    );
    let update_sql_ident = format!(
        "UPDATE {table_name} SET {update_value_placeholders} WHERE {pk_placeholder} RETURNING {full_select_columns}"
    );

    Ok(quote! {
        #struct_visibility trait #save_trait_ident<'e, E: #executor_type>: Sized {
            async fn save(&self, executor: E) -> lorm::errors::Result<#struct_name>;
        }

        impl<'e, E: #executor_type> #save_trait_ident<'e, E> for #struct_name
        {
            async fn save(&self, executor: E) -> lorm::errors::Result<#struct_name>
            {
                #updated_at_code
                match #pk_is_set {
                    true => {
                        #pk_code
                        #created_at_code
                        let r = sqlx::query_as::<_, #struct_name>(#insert_sql_ident)
                        #(
                            .bind(#insert_values)
                        )*
                        .fetch_one(executor).await?;
                        Ok(r)
                    },
                    false => {
                        let r = sqlx::query_as::<_, #struct_name>(#update_sql_ident)
                        #(
                            .bind(#update_values)
                        )*
                        .bind(#pk_value)
                        .fetch_one(executor).await?;
                        Ok(r)
                    }
                }
            }
        }
    })
}

/// Creates SQL placeholders for INSERT statements.
///
/// Generates database-specific placeholders: `"$1, $2, $3"` for PostgreSQL/SQLite or `"?, ?, ?"` for MySQL.
pub(crate) fn create_insert_placeholders<'a>(columns: &[&Column<'a>]) -> String {
    columns
        .iter()
        .enumerate()
        .map(|(i, c)| db_placeholder(c.base_field, i + 1).unwrap())
        .collect::<Vec<_>>()
        .join(",")
}

/// Creates SQL placeholders for UPDATE statements.
///
/// Generates database-specific SET clauses: `"name = $1, email = $2"` for PostgreSQL/SQLite
/// or `"name = ?, email = ?"` for MySQL.
pub(crate) fn create_update_placeholders<'a>(fields: &[&Column<'a>]) -> String {
    fields
        .iter()
        .enumerate()
        .map(|(i, c)| {
            format!(
                "{} = {}",
                c.column_name,
                db_placeholder(c.base_field, i + 1).unwrap()
            )
        })
        .collect::<Vec<_>>()
        .join(",")
}
