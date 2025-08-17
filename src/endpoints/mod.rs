use std::collections::HashMap;

use serde_json::{Value, json};
use log::{info, warn};
use itertools::Itertools;
use axum::{routing::MethodRouter, Router, extract::{Query, Json}, extract::State};
use sqlx::{any::AnyRow, Any, Column, Pool, Row, TypeInfo, ValueRef};

use crate::endpoints::parser::{Endpoint, EndpointMethod};

mod parser;

/// Best-effort decode of a cell into JSON.
/// We try a set of common types. If all fail, we fall back to string.
fn cell_to_json(row: &AnyRow, idx: usize) -> Value {
    // Null?
    if row.try_get_raw(idx).map_or_else(|_| false, |d| d.is_null()) {
        return Value::Null;
    }

    // Try JSON (native json/jsonb columns)
    // if let Ok(v) = row.try_get::<Value, _>(idx) {
    //     return Ok(v);
    // }

    // Try common scalar types
    if let Ok(v) = row.try_get::<i64, _>(idx) {
        return json!(v);
    }
    if let Ok(v) = row.try_get::<i32, _>(idx) {
        return json!(v);
    }
    if let Ok(v) = row.try_get::<i16, _>(idx) {
        return json!(v);
    }
    if let Ok(v) = row.try_get::<f64, _>(idx) {
        return number_from_f64(v);
    }
    if let Ok(v) = row.try_get::<bool, _>(idx) {
        return json!(v);
    }

    // chrono date/time (if enabled in DB + sqlx feature)
    // if let Ok(v) = row.try_get::<chrono::NaiveDate, _>(idx) {
    //     return Ok(json!(v.to_string()));
    // }
    // if let Ok(v) = row.try_get::<chrono::NaiveDateTime, _>(idx) {
    //     return Ok(json!(v.to_string()));
    // }
    // if let Ok(v) = row.try_get::<chrono::DateTime<chrono::Utc>, _>(idx) {
    //     return Ok(json!(v.to_rfc3339()));
    // }

    // // UUID
    // if let Ok(v) = row.try_get::<uuid::Uuid, _>(idx) {
    //     return Ok(json!(v.to_string()));
    // }

    // Text-ish
    if let Ok(v) = row.try_get::<String, _>(idx) {
        return json!(v);
    }
    if let Ok(v) = row.try_get::<&str, _>(idx) {
        return json!(v);
    }

    // Bytes -> base64
    // if let Ok(v) = row.try_get::<Vec<u8>, _>(idx) {
    //     return json!({ "type": "bytes", "base64": base64::encode(v) });
    // }

    // Last resort: attempt debug string via `format!("{:?}", ...)` by first trying `String`.
    // If we still can't decode, provide a placeholder with type name.
    let tname = row.columns()[idx].type_info().name().to_string();
    json!({ "unsupported": tname })
}

fn number_from_f64(f: f64) -> Value {
    match serde_json::Number::from_f64(f) {
        Some(n) => Value::Number(n),
        None => Value::Null,
    }
}


fn row_to_json(row: &AnyRow) -> Value {
    let cols = row.columns();
    let mut obj = serde_json::Map::with_capacity(cols.len());

    for (i, col) in cols.iter().enumerate() {
        let name = col.name();
        let val = cell_to_json(row, i);
        obj.insert(name.to_string(), val);
    }

    Value::Object(obj)
}


#[derive(Clone)]
struct EndpointHandler {
    file_content: String,
}

impl EndpointHandler {
    pub fn new(file_content: String) -> EndpointHandler {
        EndpointHandler { file_content: file_content, }
    }

    async fn handle_get(&self, params: &HashMap<String, String>, pool: Pool<Any>) -> Value {
        warn!("{:?}", params);
        let rows: Vec<AnyRow> = sqlx::query(&self.file_content).fetch_all(&pool).await.unwrap();
        info!("{:?}", rows.len());

        let mut out = Vec::with_capacity(rows.len());
        for row in rows.iter() {
            out.push(row_to_json(row));
        }

        Value::Array(out)
    }

    async fn handle_post(&self, params: &Value) -> String {
        warn!("{:?}", params);
        self.file_content.clone()
    }
}



fn get_route(endpoints: Vec<&Endpoint>) -> MethodRouter<Pool<Any>> {

    let mut method_router = MethodRouter::new();

    for endpoint in endpoints {
        let endpoint_handler = EndpointHandler::new(endpoint.file_content.clone());

        if endpoint.method == EndpointMethod::GET {
            method_router = method_router.get(
                |State(pool): State<Pool<Any>>, q: Query<HashMap<String, String>>| async move {
                    warn!("{:?}", q);
                    endpoint_handler.handle_get(&q.0, pool).await.to_string()
                }
            );
        } else if endpoint.method == EndpointMethod::POST {
            method_router = method_router.post(
                |q: Json<Value>| async move {
                    endpoint_handler.handle_post(&q.0).await
                }
            );
        }
    }

    method_router
}

pub fn load_dsl_endpoints(args: &crate::args::types::Args, mut app: Router<Pool<Any>>) -> Router<Pool<Any>> {
    info!("Loading DSL endpoints from path: {}", args.dsl_path);

    let collection: parser::EndpointCollections = parser::EndpointCollections::parse_from_dir(&args.dsl_path);
    info!("Loaded next endpoints collection: {}", collection);

    let flatten_endpoints = collection.projects
        .iter()
        .flat_map(|p| &p.endpoints)
        .chunk_by(|e| e.url_path.clone());

    for (key, chunk_iter) in &flatten_endpoints {
        let chunk: Vec<&Endpoint> = chunk_iter.collect();
        app = app.route(&key, get_route(chunk))
    }
    
    app 
}