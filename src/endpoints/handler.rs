use serde_json::{Value, json};
use log::{warn};
use sqlx::{any::AnyRow, Any, Column, Pool, Row, TypeInfo, ValueRef};
use std::collections::HashMap;

fn rewrite_sql_with_named_params(sql: &str) -> (String, Vec<String>) {
    let mut result = String::with_capacity(sql.len());
    let mut params = Vec::new();
    let mut chars = sql.chars().peekable();
    let mut index = 0;

    while let Some(c) = chars.next() {
        if c == ':' {
            // Peek to check if this is a typecast "::"
            if chars.peek() == Some(&':') {
                // It's "::", just push one ":" and continue
                chars.next();
                result.push(':');
                result.push(':');
                continue;
            }

            // Collect identifier name
            let mut name = String::new();
            while let Some(&nc) = chars.peek() {
                if name.is_empty() {
                    if nc.is_ascii_alphabetic() || nc == '_' {
                        name.push(nc);
                        chars.next();
                    } else {
                        break; // not a valid param, just output ':'
                    }
                } else {
                    if nc.is_ascii_alphanumeric() || nc == '_' {
                        name.push(nc);
                        chars.next();
                    } else {
                        break;
                    }
                }
            }

            if !name.is_empty() {
                index += 1;
                params.push(name);
                result.push('$');
                result.push_str(&index.to_string());
                continue;
            } else {
                // lone ":" not followed by identifier
                result.push(':');
                continue;
            }
        }

        // default: copy character
        result.push(c);
    }

    (result, params)
}

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
pub struct EndpointHandler {
    file_content: String,
}

impl EndpointHandler {
    pub fn new(file_content: String) -> EndpointHandler {
        EndpointHandler { file_content: file_content, }
    }

    async fn handle_query(&self, params: &HashMap<String, String>, pool: Pool<Any>) -> Value {

        let (rewritten, order) = rewrite_sql_with_named_params(&self.file_content);
        let args: Vec<(&String, Option<&String>)> = order.iter()
        .map(|k| (k, params.get(k)))
        .collect();

        let mut query = sqlx::query(&rewritten);
        
        for pair in args {
            if pair.1.is_none() {
                return json!({
                    "error": format!("missing parameter '{}'", pair.0)
                });
            }
            query = query.bind(pair.1.unwrap());
        }

        let rows: Vec<AnyRow> = query.fetch_all(&pool).await.unwrap();
        // let rows: Vec<AnyRow> = sqlx::query(&self.file_content)
        //     .fetch_all(&pool).await.unwrap();


        let mut out = Vec::with_capacity(rows.len());
        for row in rows.iter() {
            out.push(row_to_json(row));
        }

        Value::Array(out)
    }

    pub async fn handle_get(&self, params: &HashMap<String, String>, pool: Pool<Any>) -> Value {
        self.handle_query(&params, pool).await
    }

    pub async fn handle_post(&self, params: &Value) -> String {
        warn!("{:?}", params);
        self.file_content.clone()
    }
}


