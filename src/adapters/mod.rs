use crate::adapters::{adapter_trait::Adapter, postgresql_adapter::PostgresqlAdapter};

mod adapter_trait;
mod postgresql_adapter;


pub async fn get_adapter_type(connection_string: String) -> Box<impl Adapter> {
    if connection_string.starts_with("postgresql://") {
        return Box::new(PostgresqlAdapter::connect_by_string(connection_string).await.unwrap());
    }

    panic!("Requested adapter not found");
}