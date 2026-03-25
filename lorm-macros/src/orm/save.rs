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

    let primary_key = model.primary_key();

    // prepare `insertable` fields
    let insert_value_placeholders =
        create_insert_placeholders(&model.insert_columns().collect::<Vec<_>>());
    let insert_values = model
        .insert_columns()
        .map(|col| col.field.clone())
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
        .map(|col| col.field.clone())
        .collect::<Vec<_>>();

    // Primary key
    let pk_column = &primary_key.column_name;
    let pk_field = &primary_key.field;
    let pk_placeholder = format!(
        "{} = {}",
        pk_column,
        db_placeholder(primary_key.base_field, model.update_columns().count() + 1)?
    );
    let pk_is_set = &primary_key
        .column_properties
        .is_set(quote! { to_save.#pk_field });
    let pk_code = if primary_key.column_properties.readonly {
        quote! {}
    } else {
        let pk_new_method = &primary_key.column_properties.new_expression;
        quote! {
            to_save.#pk_field = #pk_new_method;
        }
    };

    let created_at_code = match model.created_at() {
        None => quote! {},
        Some(column) => {
            if column.column_properties.readonly {
                quote! {}
            } else {
                let new_method = &column.column_properties.new_expression;
                let field = &column.field;
                quote! {
                    to_save.#field = #new_method;
                }
            }
        }
    };

    let updated_at_code = match model.updated_at() {
        None => quote! {},
        Some(column) => {
            if column.column_properties.readonly {
                quote! {}
            } else {
                let new_method = &column.column_properties.new_expression;
                let field = &column.field;
                quote! {
                    to_save.#field = #new_method;
                }
            }
        }
    };

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
                let mut to_save = self.clone();
                #updated_at_code
                match #pk_is_set {
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
                        .bind(&self.#pk_field)
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
