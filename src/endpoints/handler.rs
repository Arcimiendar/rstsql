use serde_json::{Value, json};
// use uuid;
use crate::endpoints::sql_utils::json_to_params::bind_json_to_query;
use crate::endpoints::sql_utils::preprocess::rewrite_sql_with_named_params;
use crate::endpoints::sql_utils::row_to_json::row_to_json;
use serde_json;
use sqlx::PgPool;
use std::collections::HashMap;

#[derive(Clone)]
pub struct EndpointHandler {
    sql: String,
    params_order: Vec<String>,
}

impl EndpointHandler {
    pub fn new(file_content: &String) -> EndpointHandler {
        let (rewritten, order) = rewrite_sql_with_named_params(&file_content);

        EndpointHandler {
            sql: rewritten,
            params_order: order,
        }
    }

    pub fn param_list_empty(&self) -> bool {
        return self.params_order.len() == 0;
    }

    async fn handle_query(
        &self,
        params: &serde_json::Map<String, Value>,
        pool: PgPool,
    ) -> anyhow::Result<Value> {
        let args: Vec<(&String, Option<&Value>)> = self
            .params_order
            .iter()
            .map(|k| (k, params.get(k)))
            .collect();
        let query = sqlx::query(&self.sql);

        let query = bind_json_to_query(query, &args)?;

        let rows = query.fetch_all(&pool).await?;

        let mut out = Vec::with_capacity(rows.len());
        for row in rows.iter() {
            out.push(row_to_json(row));
        }

        Ok(Value::Array(out))
    }

    pub async fn handle_get(
        &self,
        params: &HashMap<String, String>,
        pool: PgPool,
    ) -> anyhow::Result<Value> {
        self.handle_query(
            &params.iter().map(|x| (x.0.clone(), json!(x.1))).collect(),
            pool,
        )
        .await
    }

    pub async fn handle_post(&self, params: &Value, pool: PgPool) -> anyhow::Result<Value> {
        if let Some(v) = params.as_object() {
            return self.handle_query(v, pool).await;
        }

        return self.handle_query(&serde_json::Map::new(), pool).await;
    }
}
