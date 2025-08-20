use serde_yaml_ng;
use utoipa::openapi::{request_body::{RequestBody, RequestBodyBuilder}, ObjectBuilder, Schema, Content, Required};

use crate::endpoints::swagger::utils::append_field;

pub fn get_query_params() {

}

pub fn get_request_body(declaration: &serde_yaml_ng::Mapping) -> Option<(RequestBody, Schema)> {
    let field_arr_opt = declaration.get("allowlist")
        .and_then(|a| a.get("body"))
        .and_then(|b| b.as_sequence());
    if field_arr_opt.is_none() {
        return None;
    }
    let field_arr = field_arr_opt.unwrap();

    let mut object = ObjectBuilder::new();

    for field in field_arr {
        object = append_field(&field, object);
    }

    let schema = Schema::Object(object.build());

    let request_body = RequestBodyBuilder::new()
        .content("application/json", Content::new(Some(schema.clone())))
        .required(Some(Required::True))
        .build();
    
    Some((request_body, schema))
}
