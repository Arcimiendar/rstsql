use serde_json::Value;
use sqlx::{Postgres, postgres::PgArguments, query::Query, types::Json};
use std::error::Error;
use std::fmt::Display;

fn best_effort_bind<'a>(
    query: Query<'a, Postgres, PgArguments>,
    value: &'a Value,
) -> Query<'a, Postgres, PgArguments> {
    if let Some(v) = value.as_str() {
        return query.bind(v);
    }

    if let Some(v) = value.as_bool() {
        return query.bind(v);
    }

    if let Some(v) = value.as_f64() {
        return query.bind(v);
    }

    if let Some(v) = value.as_i64() {
        return query.bind(v);
    }

    if value.is_null() {
        let none: Option<&String> = None;
        return query.bind(none);
    }

    // not sure what to do with array, so pass it as ::json value
    return query.bind(Json(value));
}

#[derive(Debug)]
pub struct MissingParameterError {
    missed_param: String,
}

impl MissingParameterError {
    fn new(missed_param: String) -> Self {
        Self { missed_param }
    }
}

impl Display for MissingParameterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.missed_param)
    }
}

impl Error for MissingParameterError {}

pub fn bind_json_to_query<'a>(
    mut query: Query<'a, Postgres, PgArguments>,
    args: &'a Vec<(&String, Option<&Value>)>,
) -> Result<Query<'a, Postgres, PgArguments>, MissingParameterError> {
    for pair in args {
        query = best_effort_bind(
            query,
            pair.1
                .ok_or_else(|| MissingParameterError::new(pair.0.clone()))?,
        );
    }

    Ok(query)
}
