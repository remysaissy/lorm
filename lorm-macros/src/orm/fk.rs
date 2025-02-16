use crate::helpers::get_fk_method;
use crate::models::OrmModel;
use quote::{__private::TokenStream, format_ident, quote};

pub fn generate_fk(db_pool_type: &TokenStream, model: &OrmModel) -> syn::Result<TokenStream> {
    static SUFFIX: &str = "_id";
    let trait_ident = format_ident!("{}FkTrait", model.struct_name);
    let struct_name = model.struct_name;
    let struct_visibility = model.struct_visibility;

    let stream: Vec<(TokenStream, TokenStream)> = model.fk_fields.iter().filter_map(|field| {
        let field_ident = field.ident.as_ref().unwrap();
        let field_ident_name = field_ident.to_string();
        let fk_type_ident = get_fk_method(field).ok()?;

        let get_fn = match field_ident_name.ends_with(SUFFIX) {
            true => format_ident!("get_{}", field_ident_name[..field_ident_name.len() - SUFFIX.len()]),
            false => format_ident!("get_{}", field_ident),
        };

        let trait_code = quote! {
            async fn #get_fn(&self, pool: &#db_pool_type) -> lorm::errors::Result<Option<#fk_type_ident>>;
        };

        let impl_code = quote! {
            async fn #get_fn(&self, pool: &#db_pool_type) -> lorm::errors::Result<Option<#fk_type_ident>> {
                let obj = #fk_type_ident::by_id(pool, self.#field_ident.clone()).await?;
                Ok(obj)
            }
        };
        Some((trait_code, impl_code))
    }).collect::<Vec<(_, _)>>();
    let (trait_tokens, impl_tokens): (Vec<TokenStream>, Vec<TokenStream>) =
        stream.into_iter().unzip();

    Ok(quote! {
        #struct_visibility trait #trait_ident {
            #(#trait_tokens)*
        }

        impl #trait_ident for #struct_name {
            #(#impl_tokens)*
        }
    })
}
