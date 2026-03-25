use crate::models::OrmModel;
use crate::utils::get_bind_type_constraint;
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

    let impl_tokens: Vec<TokenStream> = model.query_columns().map(|column| {
        let field_name = &column.field;
        let column_name = &column.column_name;
        let field_type_constraints = get_bind_type_constraint(column.base_field, database_type).unwrap();
        let where_between_fn = format_ident!("where_between_{}", field_name);
        let where_fn = format_ident!("where_{}", field_name);
        let having_fn = format_ident!("having_{}", field_name);
        let order_by_fn = format_ident!("order_by_{}", field_name);
        let group_by_fn = format_ident!("group_by_{}", field_name);

        let code = quote! {
            #struct_visibility fn #having_fn<T: #field_type_constraints>(mut self, op: lorm::predicates::Having, fun: lorm::predicates::Function, value: T) -> #builder_struct_ident {
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
                self.builder.push_bind(value);
                self
            }

            #struct_visibility fn #where_fn<T: #field_type_constraints>(mut self, op: lorm::predicates::Where, value: T) -> #builder_struct_ident {
                if self.is_where == false {
                    self.builder.push(" WHERE");
                    self.is_where = true;
                } else {
                    self.builder.push(" AND");
                }
                let stmt = format!(" {} {} ", #column_name, op).to_string();
                    self.builder.push(stmt);
                    self.builder.push_bind(value);
                self
            }

            #struct_visibility fn #where_between_fn<T: #field_type_constraints>(mut self, left: T, right: T) -> #builder_struct_ident {
                if self.is_where == false {
                    self.builder.push(" WHERE");
                    self.is_where = true;
                } else {
                    self.builder.push(" AND");
                }
                let stmt = format!(" {} BETWEEN ", #column_name).to_string();
                self.builder.push(stmt);
                self.builder.push_bind(left);
                self.builder.push(" AND ");
                self.builder.push_bind(right);
                self
            }

            #struct_visibility fn #order_by_fn(mut self) -> #builder_struct_ident {
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

            #struct_visibility fn #group_by_fn(mut self) -> #builder_struct_ident {
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
        code
    }).collect::<Vec<_>>();

    let select_columns = model.full_column_select();
    let table_name = &model.table_name;
    let select_base = format!("SELECT {select_columns} from {table_name}");

    Ok(quote! {
        #struct_visibility trait #trait_ident {
            fn select() -> #builder_struct_ident;
        }

        impl #trait_ident for #struct_name {
            fn select() -> #builder_struct_ident {
                let builder = sqlx::QueryBuilder::new(#select_base);
                #builder_struct_ident { builder, is_where: false, is_having: false, is_group_by: false, is_order_by: false }
            }
        }

        #[derive(Default)]
        #struct_visibility struct #builder_struct_ident {
            builder: sqlx::QueryBuilder<'static, #database_type>,
            is_where: bool,
            is_having: bool,
            is_group_by: bool,
            is_order_by: bool
        }

        impl #builder_struct_ident {
            #struct_visibility fn having_all_count(mut self, op: lorm::predicates::Having, value: i64) -> #builder_struct_ident {
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

            #struct_visibility fn asc(mut self) -> #builder_struct_ident {
                self.builder.push(" ASC ");
                self
            }

            #struct_visibility fn desc(mut self) -> #builder_struct_ident {
                self.builder.push(" DESC ");
                self
            }

            #struct_visibility fn limit(mut self, limit: i64) -> #builder_struct_ident {
                self.builder.push(" LIMIT ");
                self.builder.push_bind(limit);
                self
            }

            #struct_visibility fn offset(mut self, offset: i64) -> #builder_struct_ident {
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
