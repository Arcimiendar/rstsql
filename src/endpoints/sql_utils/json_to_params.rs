use serde_json::Value;
use sqlx::{postgres::PgArguments, query::Query, Postgres, types::Json};


fn best_effort_bind<'a>(query: Query<'a,Postgres, PgArguments>, value: &'a Value) -> Query<'a,Postgres, PgArguments> {
    if value.is_string() {
        return query.bind(value.as_str().unwrap());
    }

    if value.is_boolean() {
        return query.bind(value.as_bool().unwrap());
    }

    if value.is_f64() {
        return query.bind(value.as_f64().unwrap());
    }

    if value.is_i64() {
        return query.bind(value.as_i64().unwrap());
    }

    if value.is_null() {
        let none: Option<&String> = None;
        return query.bind(none);
    }

    // not sure what to do with array, so pass it as ::json value
    return query.bind(Json(value));

}

pub fn bind_json_to_query<'a>(
    mut query: Query<'a, Postgres, PgArguments>,
    args: &'a Vec<(&String, Option<&Value>)>,
) -> Result<Query<'a,Postgres, PgArguments>, String> {
    for pair in args {
        if pair.1.is_none() {
            return Err(format!("missing parameter '{}'", pair.0));
        }
        query = best_effort_bind(query, &pair.1.unwrap());
    }


    Ok(query)
}