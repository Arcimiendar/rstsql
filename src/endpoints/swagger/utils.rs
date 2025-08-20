use serde_yaml_ng;
use utoipa::openapi::{ArrayBuilder, Type, schema::ObjectBuilder};

enum ObjectOrArrayBuilder {
    Object(ObjectBuilder),
    Array(ArrayBuilder),
}

fn parse_type(field: &serde_yaml_ng::Value) -> ObjectOrArrayBuilder {
    let mut object = ObjectBuilder::new();

    let description = field
        .get("description")
        .and_then(|d_v| d_v.as_str())
        .or(Some(""))
        .unwrap()
        .to_string();

    let field_type = field
        .get("type")
        .and_then(|t| t.as_str())
        .or(Some("string"))
        .unwrap();

    object = match field_type {
        "string" | "timestamp" => {
            object = object.schema_type(Type::String);

            let enum_v_opt = field
                .get("enum")
                .and_then(|e| e.as_sequence())
                .and_then(|e| {
                    Some(
                        e.iter()
                            .map(|v| v.as_str())
                            .filter(|v| v.is_some())
                            .map(|v| v.unwrap().to_string())
                            .collect::<Vec<String>>(),
                    )
                });

            if let Some(enum_v) = enum_v_opt {
                object = object.enum_values(Some(enum_v))
            }

            object
        }
        "number" | "integer" => object.schema_type(Type::Number),
        "boolean" | "bool" => object.schema_type(Type::Boolean),
        "object" => {
            let inner_fields_opt = field.get("fields").and_then(|f| f.as_sequence());

            if let Some(inner_fields) = inner_fields_opt {
                for inner_field in inner_fields {
                    let name_opt = inner_field.get("field").and_then(|f| f.as_str());
                    if name_opt.is_none() {
                        continue;
                    }
                    let name = name_opt.unwrap();
                    let optional_flag = inner_field
                        .get("optional")
                        .and_then(|o| o.as_bool())
                        .or(Some(false))
                        .unwrap();

                    let inner_type = parse_type(inner_field);
                    match inner_type {
                        ObjectOrArrayBuilder::Object(o) => {
                            object = object.property(name.to_string(), o);
                        }
                        ObjectOrArrayBuilder::Array(a) => {
                            object = object.property(name.to_string(), a);
                        }
                    }

                    if !optional_flag {
                        object = object.required(name);
                    }
                }
            }

            object
        }
        "array" => {
            let arr = object.to_array_builder();
            let items_opt = field.get("items");

            if let Some(items) = items_opt {
                let arr_type = parse_type(items);

                let parsed_arr = match arr_type {
                    ObjectOrArrayBuilder::Array(a) => a.to_array_builder(),
                    ObjectOrArrayBuilder::Object(o) => o.to_array_builder(),
                };
                return ObjectOrArrayBuilder::Array(parsed_arr);
            }

            return ObjectOrArrayBuilder::Array(arr);
        }
        _ => object,
    };

    ObjectOrArrayBuilder::Object(object.description(Some(description)))
}

pub fn append_field(field: &serde_yaml_ng::Value, object: ObjectBuilder) -> ObjectBuilder {
    let field_map_opt = field.as_mapping();
    if field_map_opt.is_none() {
        return object;
    }
    let field_map = field_map_opt.unwrap();

    let name_opt = field_map.get("field").and_then(|n| n.as_str());
    if name_opt.is_none() {
        return object;
    }
    let name = name_opt.unwrap().to_string();

    let field_type = parse_type(&field);

    match field_type {
        ObjectOrArrayBuilder::Array(a) => object.property(name.to_string(), a),
        ObjectOrArrayBuilder::Object(o) => object.property(name.to_string(), o),
    }
    .required(name)
}
