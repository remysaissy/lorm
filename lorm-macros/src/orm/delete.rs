use crate::helpers::{db_placeholder, get_field_name};
use crate::models::OrmModel;
use quote::{__private::TokenStream, format_ident, quote};

pub fn generate_delete(db_pool_type: &TokenStream, model: &OrmModel) -> syn::Result<TokenStream> {
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

    Ok(quote! {
        #struct_visibility trait #trait_ident {
            async fn delete(&self, pool: &#db_pool_type) -> lorm::errors::Result<()>;
        }

        impl #trait_ident for #struct_name {
            async fn delete(&self, pool: &#db_pool_type) -> lorm::errors::Result<()> {
                let sql = format!("DELETE FROM {} WHERE {}", #table_name, #pk_placeholder);
                let _ = sqlx::query(&sql)
                .bind(&self.#pk_column)
                .execute(pool).await?;
                Ok(())
            }
        }
    })
}
