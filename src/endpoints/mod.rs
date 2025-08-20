use std::collections::HashMap;

use axum::{
    Router,
    extract::State,
    extract::{Json, Query},
    routing::MethodRouter,
};
use itertools::Itertools;
use log::info;
use serde_json::Value;
use sqlx::PgPool;

use crate::endpoints::handler::EndpointHandler;
use crate::endpoints::parser::{Endpoint, EndpointMethod};
use crate::endpoints::swagger::load_swagger;

mod handler;
mod parser;
mod sql_utils;
mod swagger;

fn get_route(endpoints: Vec<&Endpoint>) -> MethodRouter<PgPool> {
    let mut method_router = MethodRouter::new();

    for endpoint in endpoints {
        let endpoint_handler = EndpointHandler::new(&endpoint.file_content);

        if endpoint.method == EndpointMethod::GET {
            method_router = method_router.get(
                |State(pool): State<PgPool>, q: Query<HashMap<String, String>>| async move {
                    endpoint_handler.handle_get(&q.0, pool).await.to_string()
                },
            );
        } else if endpoint.method == EndpointMethod::POST {
            if endpoint_handler.param_list_empty() {
                method_router = method_router.post(|State(pool): State<PgPool>| async move {
                    endpoint_handler
                        .handle_post(&Value::Null, pool)
                        .await
                        .to_string()
                });
            } else {
                method_router =
                    method_router.post(|State(pool): State<PgPool>, q: Json<Value>| async move {
                        endpoint_handler.handle_post(&q.0, pool).await.to_string()
                    });
            }
        } else {
        }
    }

    method_router
}

pub fn load_dsl_endpoints(
    args: &crate::args::types::Args,
    mut app: Router<PgPool>,
) -> Router<PgPool> {
    info!("Loading DSL endpoints from path: {}", args.dsl_path);

    let collection: parser::EndpointCollections =
        parser::EndpointCollections::parse_from_dir(&args.dsl_path);
    info!("Loaded next endpoints collection: {}", collection);

    let flatten_endpoints = collection
        .projects
        .iter()
        .flat_map(|p| &p.endpoints)
        .chunk_by(|e| e.url_path.clone());

    for (key, chunk_iter) in &flatten_endpoints {
        let chunk: Vec<&Endpoint> = chunk_iter.collect();
        app = app.route(&key, get_route(chunk))
    }

    app = load_swagger(app, &collection);

    app
}
