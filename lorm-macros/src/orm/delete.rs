use crate::models::OrmModel;
use crate::utils::db_placeholder;
use quote::{__private::TokenStream, format_ident, quote};

pub fn generate_delete(executor_type: &TokenStream, model: &OrmModel) -> syn::Result<TokenStream> {
    let trait_ident = format_ident!("{}DeleteTrait", model.struct_name);
    let struct_name = model.struct_name;
    let struct_visibility = model.struct_visibility;
    let table_name = &model.table_name;

    // Primary key(s)
    let pk_fields = model.primary_key.fields();
    let mut where_parts = Vec::new();
    let mut bind_values = Vec::new();

    for (i, pk_col) in pk_fields.iter().enumerate() {
        let placeholder = db_placeholder(pk_col.base_field, i + 1)?;
        where_parts.push(format!("{} = {}", pk_col.column_name, placeholder));
        let accessor = pk_col.self_accessor();
        bind_values.push(quote! { .bind(#accessor) });
    }

    let where_clause = where_parts.join(" AND ");
    let sql_ident = format!("DELETE FROM {table_name} WHERE {where_clause}");

    Ok(quote! {
        #struct_visibility trait #trait_ident<'e, E: #executor_type>: Sized {
            async fn delete(&self, executor: E) -> lorm::errors::Result<()>;
        }

        #[automatically_derived]
        impl<'e, E: #executor_type> #trait_ident<'e, E> for #struct_name {
            async fn delete(&self, executor: E) -> lorm::errors::Result<()> {
                sqlx::query(#sql_ident)
                #(#bind_values)*
                .execute(executor).await?;
                Ok(())
            }
        }
    })
}
