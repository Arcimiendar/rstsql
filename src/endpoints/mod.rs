use std::collections::HashMap;

use serde_json::Value;
use log::{info, warn};
use itertools::Itertools;
use axum::{routing::MethodRouter, Router, extract::{Query, Json}, extract::State};
use sqlx::{PgPool};

use crate::endpoints::parser::{Endpoint, EndpointMethod};
use crate::endpoints::handler::EndpointHandler;

mod parser;
mod handler;



fn get_route(endpoints: Vec<&Endpoint>) -> MethodRouter<PgPool> {

    let mut method_router = MethodRouter::new();

    for endpoint in endpoints {
        let endpoint_handler = EndpointHandler::new(endpoint.file_content.clone());

        if endpoint.method == EndpointMethod::GET {
            method_router = method_router.get(
                |State(pool): State<PgPool>, q: Query<HashMap<String, String>>| async move {
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

pub fn load_dsl_endpoints(args: &crate::args::types::Args, mut app: Router<PgPool>) -> Router<PgPool> {
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