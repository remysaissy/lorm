use crate::models::OrmModel;
use crate::utils::{get_bind_param_type_and_usage, get_bind_type_where_constraint};
use quote::{__private::TokenStream, format_ident, quote};

pub fn generate_select(
    executor_type: &TokenStream,
    database_type: &TokenStream,
    model: &OrmModel,
) -> syn::Result<TokenStream> {
    let trait_ident = format_ident!("{}SelectTrait", model.struct_name);
    let builder_struct_ident = format_ident!("{}SelectBuilder", model.struct_name);
    let struct_name = model.struct_name;
    let struct_visibility = model.struct_visibility;
    let table_name = &model.table_name;
    let table_columns = &model.full_select_columns();

    let lifetime = quote! {'a};

    let impl_tokens: Vec<TokenStream> = model.fields.iter().filter(|f| f.should_generate_selector(&model.primary_key)).map(|field| (|| -> syn::Result<_> {
        let field_ident = &field.field;

        let constraints = get_bind_type_where_constraint(&field.ty, database_type, &lifetime)?;
        let parameter = quote! {value};
        let (param_type, param_use) = get_bind_param_type_and_usage(&parameter, &field.ty, &lifetime)?;
        let param = quote! {#parameter: #param_type};
        let column_name = &field.column_name;
        let where_between_fn = format_ident!("where_between_{}", field_ident);
        let where_fn = format_ident!("where_{}", field_ident);
        let having_fn = format_ident!("having_{}", field_ident);
        let order_by_fn = format_ident!("order_by_{}", field_ident);
        let group_by_fn = format_ident!("group_by_{}", field_ident);


        let (left_type, left_use) = get_bind_param_type_and_usage(&quote! {left}, &field.ty, &lifetime)?;
        let (right_type, right_use) = get_bind_param_type_and_usage(&quote! {right}, &field.ty, &lifetime)?;
        let code = quote! {
            #struct_visibility fn #having_fn(mut self, op: lorm::predicates::Having, fun: lorm::predicates::Function, #param) -> Self where #constraints {
                if self.is_having == false {
                    self.builder.push(" HAVING");
                    self.is_having = true;
                } else {
                    self.builder.push(" AND");
                }
                let stmt = match fun {
                    lorm::predicates::Function::Null => format!(" {} {} ", #column_name, op).to_string(),
                    lorm::predicates::Function::Count { is_distinct } if is_distinct == true => format!(" {}(DISTINCT {}) {} ", fun, #column_name, op).to_string(),
                    _ => format!(" {}({}) {} ", fun, #column_name, op).to_string()
                };
                self.builder.push(stmt);
                self.builder.push_bind(#param_use);
                self
            }

            #struct_visibility fn #where_fn(mut self, op: lorm::predicates::Where, #param) -> Self where #constraints {
                if self.is_where == false {
                    self.builder.push(" WHERE");
                    self.is_where = true;
                } else {
                    self.builder.push(" AND");
                }
                let stmt = format!(" {} {} ", #column_name, op).to_string();
                    self.builder.push(stmt);
                    self.builder.push_bind(#param_use);
                self
            }

            #struct_visibility fn #where_between_fn(mut self, left: #left_type, right: #right_type) -> Self where #constraints {
                if self.is_where == false {
                    self.builder.push(" WHERE");
                    self.is_where = true;
                } else {
                    self.builder.push(" AND");
                }
                let stmt = format!(" {} BETWEEN ", #column_name).to_string();
                self.builder.push(stmt);
                self.builder.push_bind(#left_use);
                self.builder.push(" AND ");
                self.builder.push_bind(#right_use);
                self
            }

            #struct_visibility fn #order_by_fn(mut self) -> Self {
                if self.is_order_by == false {
                    self.builder.push(" ORDER BY");
                    self.is_order_by = true;
                } else {
                    self.builder.push(",");
                }
                let stmt = format!(" {}", #column_name).to_string();
                self.builder.push(stmt);
                self
            }

            #struct_visibility fn #group_by_fn(mut self) -> Self {
                if self.is_group_by == false {
                    self.builder.push(" GROUP BY");
                    self.is_group_by = true;
                } else {
                    self.builder.push(",");
                }
                let stmt = format!(" {}", #column_name).to_string();
                self.builder.push(stmt);
                self
            }
        };
        Ok(code)
    })()).collect::<Result<Vec<_>, _>>()?;

    Ok(quote! {
        #struct_visibility trait #trait_ident<#lifetime> {
            fn select() -> #builder_struct_ident<#lifetime>;
        }

        #[automatically_derived]
        impl<#lifetime> #trait_ident<#lifetime> for #struct_name {
            fn select() -> #builder_struct_ident<#lifetime> {
                let sql = format!(
                    "SELECT {} FROM {}",
                    #table_columns, #table_name
                );
                let builder = sqlx::QueryBuilder::new(sql);
                #builder_struct_ident { builder, is_where: false, is_having: false, is_group_by: false, is_order_by: false }
            }
        }

        #[derive(Default)]
        #struct_visibility struct #builder_struct_ident<#lifetime> {
            builder: sqlx::QueryBuilder<#lifetime, #database_type>,
            is_where: bool,
            is_having: bool,
            is_group_by: bool,
            is_order_by: bool
        }

        #[automatically_derived]
        impl<'a> #builder_struct_ident<'a> {
            #struct_visibility fn having_all_count(mut self, op: lorm::predicates::Having, value: i64) -> Self {
                if self.is_having == false {
                    self.builder.push(" HAVING");
                    self.is_having = true;
                } else {
                    self.builder.push(" AND");
                }
                let stmt = format!(" COUNT(*) {} ", op).to_string();
                self.builder.push(stmt);
                self.builder.push_bind(value);
                self
            }

            #struct_visibility fn asc(mut self) -> Self {
                self.builder.push(" ASC ");
                self
            }

            #struct_visibility fn desc(mut self) -> Self {
                self.builder.push(" DESC ");
                self
            }

            #struct_visibility fn limit(mut self, limit: i64) -> Self {
                self.builder.push(" LIMIT ");
                self.builder.push_bind(limit);
                self
            }

            #struct_visibility fn offset(mut self, offset: i64) -> Self {
                self.builder.push(" OFFSET ");
                self.builder.push_bind(offset);
                self
            }

            #(#impl_tokens)*

            #struct_visibility async fn build<'e, E: #executor_type>(mut self, executor: E) -> lorm::errors::Result<Vec<#struct_name>> {
                let r = self
                    .builder
                    .build_query_as::<_>()
                    .fetch_all(executor)
                    .await?;
                Ok(r)
            }
        }
    })
}
