use crate::models::OrmModel;
use crate::utils::{get_field_name, get_type_as_reference};
use quote::{__private::TokenStream, format_ident, quote};

pub fn generate_select(executor_type: &TokenStream, model: &OrmModel) -> syn::Result<TokenStream> {
    let trait_ident = format_ident!("{}SelectTrait", model.struct_name);
    let builder_struct_ident = format_ident!("{}SelectBuilder", model.struct_name);
    let struct_name = model.struct_name;
    let struct_visibility = model.struct_visibility;
    let table_name = &model.table_name;
    let table_columns = &model.table_columns;

    let impl_tokens: Vec<TokenStream> = model.by_fields.iter().map(|field| {
        let field_ty = get_type_as_reference(&field.ty).unwrap();
        let field_ident = field.ident.as_ref().unwrap();
        let field_name = get_field_name(field);
        let where_between_fn = format_ident!("where_between_{}", field_ident);
        let where_equal_fn = format_ident!("where_equal_{}", field_ident);
        let where_not_equal_fn = format_ident!("where_not_equal_{}", field_ident);
        let where_is_less_fn = format_ident!("where_less_{}", field_ident);
        let where_is_less_or_equal_fn = format_ident!("where_less_equal_{}", field_ident);
        let where_is_more_fn = format_ident!("where_more_{}", field_ident);
        let where_is_more_or_equal_fn = format_ident!("where_more_equal_{}", field_ident);
        let order_by_fn = format_ident!("order_by_{}", field_ident);
        let group_by_fn = format_ident!("group_by_{}", field_ident);

        let code = quote! {
            #struct_visibility fn #where_equal_fn(mut self, value: #field_ty) -> #builder_struct_ident {
                let stmt = format!("{} = {}", #field_name, value).to_string();
                self.where_stmt.push(stmt);
                self
            }

            #struct_visibility fn #where_not_equal_fn(mut self, value: #field_ty) -> #builder_struct_ident {
                let stmt = format!("{} != {}", #field_name, value).to_string();
                self.where_stmt.push(stmt);
                self
            }

            #struct_visibility fn #where_is_less_fn(mut self, value: #field_ty) -> #builder_struct_ident {
                let stmt = format!("{} < {}", #field_name, value).to_string();
                self.where_stmt.push(stmt);
                self
            }

            #struct_visibility fn #where_is_less_or_equal_fn(mut self, value: #field_ty) -> #builder_struct_ident {
                let stmt = format!("{} <= {}", #field_name, value).to_string();
                self.where_stmt.push(stmt);
                self
            }

            #struct_visibility fn #where_is_more_fn(mut self, value: #field_ty) -> #builder_struct_ident {
                let stmt = format!("{} > {}", #field_name, value).to_string();
                self.where_stmt.push(stmt);
                self
            }

            #struct_visibility fn #where_is_more_or_equal_fn(mut self, value: #field_ty) -> #builder_struct_ident {
                let stmt = format!("{} >= {}", #field_name, value).to_string();
                self.where_stmt.push(stmt);
                self
            }

            #struct_visibility fn #where_between_fn(mut self, left: #field_ty, right: #field_ty) -> #builder_struct_ident {
                let stmt = format!("{} BETWEEN {} AND {}", #field_name, left, right).to_string();
                self.where_stmt.push(stmt);
                self
            }

            #struct_visibility fn #order_by_fn(mut self, order_by: lorm::predicates::OrderBy) -> #builder_struct_ident {
                let stmt = format!("{} {}", #field_name, order_by).to_string();
                self.order_by_stmt.push(stmt);
                self
            }

            #struct_visibility fn #group_by_fn(mut self) -> #builder_struct_ident {
                let stmt = format!("{}", #field_name).to_string();
                self.group_by_stmt.push(stmt);
                self
            }
        };
        code
    }).collect::<Vec<_>>();

    Ok(quote! {
        #struct_visibility trait #trait_ident {
            fn select() -> #builder_struct_ident;
        }

        impl #trait_ident for #struct_name {
            fn select() -> #builder_struct_ident {
                #builder_struct_ident::default()
            }
        }

        #[derive(Default)]
        #struct_visibility struct #builder_struct_ident {
            where_stmt: Vec<String>,
            order_by_stmt: Vec<String>,
            group_by_stmt: Vec<String>,
            limit: Option<i64>,
            offset: Option<i64>,
        }

        impl #builder_struct_ident {
            #struct_visibility fn limit(mut self, limit: i64) -> #builder_struct_ident {
                self.limit = Some(limit);
                self
            }

            #struct_visibility fn offset(mut self, offset: i64) -> #builder_struct_ident {
                self.offset = Some(offset);
                self
            }

            #(#impl_tokens)*

            #struct_visibility async fn build<'e, E: #executor_type>(self, executor: E) -> lorm::errors::Result<Vec<#struct_name>> {
                let where_stmt = match self.where_stmt.is_empty() {
                    true => "".to_string(),
                    false => format!("WHERE {}", self.where_stmt.join(" AND ")),
                };
                let from = #table_name;
                let order_by_stmt = match self.order_by_stmt.is_empty() {
                    true => "".to_string(),
                    false => format!("ORDER BY {}", self.order_by_stmt.join(",")),
                };
                let group_by_stmt = match self.group_by_stmt.is_empty() {
                    true => "".to_string(),
                    false => format!("GROUP BY {}", self.group_by_stmt.join(",")),
                };
                let limit = match self.limit {
                    None => "".to_string(),
                    Some(v) => format!("LIMIT {}", v),
                };
                let offset = match self.offset {
                    None => "".to_string(),
                    Some(v) => format!("OFFSET {}", v),
                };
                let sql = format!(
                    "SELECT {} FROM {} {} {} {} {} {}",
                    #table_columns, from, where_stmt, group_by_stmt, order_by_stmt, limit, offset
                );
                let sql = sql.trim();
                let r = sqlx::query_as::<_, #struct_name>(sql).fetch_all(executor).await?;
                Ok(r)
            }
        }
    })
}
