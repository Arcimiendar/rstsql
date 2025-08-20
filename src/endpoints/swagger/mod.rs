use std::collections::HashMap;

use convert_case::{self, Case, Casing};
use axum::Router;
use log::warn;
use sqlx::PgPool;
use serde_yaml_ng;
use utoipa::openapi::{
    path::{Operation, OperationBuilder, PathItemBuilder}, ComponentsBuilder, HttpMethod, InfoBuilder, OpenApi, OpenApiBuilder, PathsBuilder, Schema
};
use utoipa_swagger_ui::SwaggerUi;

use crate::endpoints::parser::{Endpoint, EndpointCollections, EndpointMethod, Project};
use crate::endpoints::swagger::response::get_response;
use crate::endpoints::swagger::params::{get_query_params, get_request_body};

mod params;
mod response;
mod utils;


fn get_operation_and_schema(endpoint: &Endpoint, project: &Project) -> (Operation, HashMap<String, Schema>) {
    let mut operation = OperationBuilder::new()
        .operation_id(Some(endpoint.url_path.clone()))
        .tag(project.project_name.clone());
    let mut schema_openapi_map = HashMap::new();

    if !endpoint.contains_schema() {
       return (operation.build(), schema_openapi_map);
    }

    let schema_str = endpoint.extract_schema();
    // warn!("{}", schema_str);

    let schema_res = serde_yaml_ng::from_str::<serde_yaml_ng::Value>(&schema_str);

    if schema_res.is_err() {
        warn!("Found declaration in {}.sql, but it's maleformatted", endpoint.url_path);
        return (operation.build(), schema_openapi_map);
    }
    let schema = schema_res.unwrap();

    let declaration_opt = schema.as_mapping()
        .and_then(|s| s.get("declaration"))
        .and_then(|d| d.as_mapping());

    if declaration_opt.is_none() {
        warn!("Found declaration in {}.sql, but it's maleformatted", endpoint.url_path);
        return (operation.build(), schema_openapi_map);
    }
    let declaration = declaration_opt.unwrap();

    if let Some(description_val) = declaration.get("description") {
        if !description_val.is_string() {
            warn!("Found declaration in {}.sql, declaration.description must be a string", endpoint.url_path);
        }
        
        let description = description_val.as_str().unwrap();
        operation = operation.description(Some(description.to_string()));
    }


    if let Some((response, schema)) = get_response(&declaration) {
        operation = operation.response("200", response);
        
        schema_openapi_map.insert(format!("Response{}", endpoint.url_path.replace("/", "_")).to_case(Case::UpperCamel), schema);
    }

    if endpoint.method == EndpointMethod::GET {

    } else if endpoint.method == EndpointMethod::POST {
        if let Some((request_body, schema)) = get_request_body(&declaration){
            operation = operation.request_body(Some(request_body));

            schema_openapi_map.insert(format!("Post{}", endpoint.url_path.replace("/", "_")).to_case(Case::UpperCamel), schema);
        }
    }
    
    (operation.build(), schema_openapi_map)
}


fn build_schema_for_endpoint(
    mut path_builder: PathsBuilder, 
    mut components_builder: ComponentsBuilder,
    endpoint: &Endpoint,
    project: &Project,
) -> (PathsBuilder, ComponentsBuilder) {
    let path_item_builder = PathItemBuilder::new();

    let (operation, schema_opt) = get_operation_and_schema(&endpoint, &project);

    for (key, value) in schema_opt {
        components_builder = components_builder.schema(key, value);
    }

    path_builder = path_builder.path(
        endpoint.url_path.clone(), 
        path_item_builder.operation(match endpoint.method {
            EndpointMethod::GET => HttpMethod::Get,
            EndpointMethod::POST => HttpMethod::Post,
        }, operation).build()
    );


    (path_builder, components_builder)
}


fn build_open_api(collection: &EndpointCollections) -> OpenApi {
    
    let builder = OpenApiBuilder::new()
        .info(
            InfoBuilder::new()
                .title("RstSQL API")
                .version(env!("CARGO_PKG_VERSION"))
                .build()
        );

    let mut components_builder = ComponentsBuilder::new();
    let mut paths_builder = PathsBuilder::new();
    

    for project in &collection.projects {
        for endpoint in &project.endpoints {
            (paths_builder, components_builder) = build_schema_for_endpoint(paths_builder, components_builder, &endpoint, &project);
        }
    }


    builder.paths(paths_builder.build()).components(Some(components_builder.build())).build()
} 

pub fn load_swagger(mut app: Router<PgPool>, collection: &EndpointCollections) -> Router<PgPool>  {
    app = app.merge(
        SwaggerUi::new("/docs")
            .url("/docs/openapi.json", build_open_api(&collection))
    );

    app
}