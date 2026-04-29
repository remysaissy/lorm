use crate::models::OrmModel;
use crate::utils::db_placeholder;
use quote::{__private::TokenStream, format_ident, quote};

pub fn generate_delete(executor_type: &TokenStream, model: &OrmModel) -> syn::Result<TokenStream> {
    let trait_ident = format_ident!("{}DeleteTrait", model.struct_name);
    let struct_name = model.struct_name;
    let struct_visibility = model.struct_visibility;
    let table_name = &model.table_name;

    // Primary key
    let primary_key = model.primary_key();
    let pk_value = primary_key.self_accessor();
    let pk_column = &primary_key.column_name;
    let pk_placeholder = format!(
        "{pk_column} = {}",
        db_placeholder(primary_key.base_field, 1)?
    );
    let sql_ident = format!("DELETE FROM {table_name} WHERE {pk_placeholder}");

    Ok(quote! {
        #struct_visibility trait #trait_ident<'e, E: #executor_type>: Sized {
            async fn delete(&self, executor: E) -> lorm::errors::Result<()>;
        }

        #[automatically_derived]
        impl<'e, E: #executor_type> #trait_ident<'e, E> for #struct_name {
            async fn delete(&self, executor: E) -> lorm::errors::Result<()> {
                sqlx::query(#sql_ident)
                .bind(#pk_value)
                .execute(executor).await?;
                Ok(())
            }
        }
    })
}
