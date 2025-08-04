use crate::models::OrmModel;
use crate::utils::{db_placeholder, get_field_name};
use quote::{__private::TokenStream, format_ident, quote};

pub fn generate_delete(executor_type: &TokenStream, model: &OrmModel) -> syn::Result<TokenStream> {
    let trait_ident = format_ident!("{}DeleteTrait", model.struct_name);
    let struct_name = model.struct_name;
    let struct_visibility = model.struct_visibility;
    let table_name = &model.table_name;

    // Primary key
    let pk_column = model.pk_field.ident.as_ref().unwrap();
    let pk_name = get_field_name(model.pk_field);
    let pk_placeholder = format!(
        "{} = {}",
        pk_name,
        db_placeholder(model.pk_field, 1).unwrap()
    );
    let sql_ident = format!("DELETE FROM {} WHERE {}", table_name, pk_placeholder);

    Ok(quote! {
        #struct_visibility trait #trait_ident<'e, E: #executor_type>: Sized {
            async fn delete(&self, executor: E) -> lorm::errors::Result<()>;
        }

        impl<'e, E: #executor_type> #trait_ident<'e, E> for #struct_name {
            async fn delete(&self, executor: E) -> lorm::errors::Result<()> {
                sqlx::query(#sql_ident)
                .bind(&self.#pk_column)
                .execute(executor).await?;
                Ok(())
            }
        }
    })
}
