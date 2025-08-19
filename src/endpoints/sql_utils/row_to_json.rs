use serde_json::{Value, json};
use base64::prelude::*;
use sqlx::{postgres::PgRow, types::{chrono, Uuid}, Column, Row, TypeInfo, ValueRef};


/// Best-effort decode of a cell into JSON.
/// We try a set of common types. If all fail, fall back to string.
fn cell_to_json(row: &PgRow, idx: usize) -> Value {
    // Null?
    if row.try_get_raw(idx).map_or_else(|_| false, |d| d.is_null()) {
        return Value::Null;
    }

    // Try JSON (native json/jsonb columns)
    if let Ok(v) = row.try_get::<Value, _>(idx) {
        return v;
    }

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
        return json!(v);
    }
    if let Ok(v) = row.try_get::<bool, _>(idx) {
        return json!(v);
    }

    // chrono date/time (if enabled in DB + sqlx feature)
    if let Ok(v) = row.try_get::<chrono::NaiveDate, _>(idx) {
        return json!(v.to_string());
    }
    if let Ok(v) = row.try_get::<chrono::NaiveDateTime, _>(idx) {
        return json!(v.to_string());
    }
    if let Ok(v) = row.try_get::<chrono::DateTime<chrono::Utc>, _>(idx) {
        return json!(v.to_rfc3339());
    }

    // UUID
    if let Ok(v) = row.try_get::<Uuid, _>(idx) {
        return json!(v.to_string());
    }

    // Text-ish
    if let Ok(v) = row.try_get::<String, _>(idx) {
        return json!(v);
    }
    if let Ok(v) = row.try_get::<&str, _>(idx) {
        return json!(v);
    }

    // Bytes -> base64
    if let Ok(v) = row.try_get::<Vec<u8>, _>(idx) {
        return json!({ "type": "bytes", "base64": BASE64_STANDARD.encode(v) });
    }

    // Last resort: attempt debug string via `format!("{:?}", ...)` by first trying `String`.
    // If we still can't decode, provide a placeholder with type name.
    let tname = row.columns()[idx].type_info().name().to_string();
    json!({ "unsupported": tname })
}


pub fn row_to_json(row: &PgRow) -> Value {
    let cols = row.columns();
    let mut obj = serde_json::Map::with_capacity(cols.len());

    for (i, col) in cols.iter().enumerate() {
        let name = col.name();
        let val = cell_to_json(row, i);
        obj.insert(name.to_string(), val);
    }

    Value::Object(obj)
}