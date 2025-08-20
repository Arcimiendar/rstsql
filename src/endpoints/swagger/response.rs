use serde_yaml_ng;
use utoipa::openapi::{
    ArrayBuilder, Content, RefOr, Response, ResponseBuilder, Schema, schema::ObjectBuilder,
};

use crate::endpoints::swagger::utils::append_field;

pub fn get_response(schema_map: &serde_yaml_ng::Mapping) -> Option<(Response, Schema)> {
    let field_arr_opt = schema_map
        .get("response")
        .and_then(|r| r.get("fields"))
        .and_then(|fs| fs.as_sequence());
    if field_arr_opt.is_none() {
        return None;
    }
    let field_arr = field_arr_opt.unwrap();
    if field_arr.is_empty() {
        return None;
    }

    let mut object = ObjectBuilder::new();

    for field in field_arr {
        object = append_field(&field, object);
    }

    let built_object = object.build();
    let schema = Schema::Object(built_object.clone());

    let response = ResponseBuilder::new()
        .content(
            "application/json",
            Content::new(Some(RefOr::T(Schema::Array(
                ArrayBuilder::new().items(built_object).build(),
            )))),
        )
        .build();

    Some((response, schema))
}
