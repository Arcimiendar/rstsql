use serde_yaml_ng;
use utoipa::openapi::{
    path::{Parameter, ParameterBuilder, ParameterIn}, request_body::{RequestBody, RequestBodyBuilder}, 
    Content, ObjectBuilder, Required, schema::Schema, Type
};

use crate::endpoints::swagger::utils::append_field;

pub fn get_query_params(declaration: &serde_yaml_ng::Mapping) -> Option<Vec<Parameter>> {
    let query_arr_opt = declaration.get("allowlist")
        .and_then(|a| a.get("query"))
        .and_then(|q| q.as_sequence());

    if query_arr_opt.is_none() {
        return None;
    }
    
    Some(query_arr_opt
        .unwrap()
        .iter()
        .map(|qp| {
            let name_opt = qp.get("field").and_then(|f| f.as_str());
            if name_opt.is_none() {
                return None;
            }
            let name = name_opt.unwrap();
            
            let description = qp.get("description")
                .and_then(|d| d.as_str())
                .or(Some(""))
                .unwrap();

            // change it if query params will be parsed from server side, not DB side
            let qp_type = Type::String;

            let param = ParameterBuilder::new()
                .name(name.to_string())
                .parameter_in(ParameterIn::Query)
                .description(Some(description))
                .required(Required::True)
                .schema(
                    Some(Schema::Object(ObjectBuilder::new().schema_type(qp_type).build()))
                )
                .build();
            Some(param)
        })
        .filter(|qp| qp.is_some())
        .map(|qp| qp.unwrap())
        .collect())
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
