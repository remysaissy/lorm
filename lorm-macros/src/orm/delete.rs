use crate::models::OrmModel;
use crate::utils::{db_placeholder, get_field_name};
use quote::{__private::TokenStream, format_ident, quote};
use syn::spanned::Spanned;

pub fn generate_delete(executor_type: &TokenStream, model: &OrmModel) -> syn::Result<TokenStream> {
    let trait_ident = format_ident!("{}DeleteTrait", model.struct_name);
    let struct_name = model.struct_name;
    let struct_visibility = model.struct_visibility;
    let table_name = &model.table_name;

    // Primary key
    let pk_placeholder = model
        .primary_key
        .fields()
        .iter()
        .enumerate()
        .map(|(i, f)| match db_placeholder(f, i + 1) {
            Ok(placeholder) => {
                let field_name = get_field_name(f);
                Ok(format!("{field_name} = {placeholder}"))
            }
            Err(e) => Err(e),
        })
        .collect::<Result<Vec<_>, syn::Error>>()?
        .join(" AND ");
    let pk_columns = model.primary_key.columns()?;

    let sql_ident = format!("DELETE FROM {} WHERE {}", table_name, pk_placeholder);

    Ok(quote! {
        #struct_visibility trait #trait_ident<'e, E: #executor_type>: Sized {
            async fn delete(&self, executor: E) -> lorm::errors::Result<()>;
        }

        impl<'e, E: #executor_type> #trait_ident<'e, E> for #struct_name {
            async fn delete(&self, executor: E) -> lorm::errors::Result<()> {
                sqlx::query(#sql_ident)
                #(
                    .bind(&self.#pk_columns)
                )*
                .execute(executor).await?;
                Ok(())
            }
        }
    })
}
