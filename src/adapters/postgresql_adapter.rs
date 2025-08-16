use log::info;
use tokio_postgres::{NoTls, Error, Client};
use tokio;
use crate::adapters::adapter_trait::Adapter;

pub struct PostgresqlAdapter {
    client: Client,
}

impl PostgresqlAdapter {
    pub async fn connect_by_string(connection_string: String) -> Result<Self, Error> {

        let (client, connection) =
            tokio_postgres::connect(&connection_string, NoTls).await?;

        // The connection object performs the actual communication with the database,
        // so spawn it off to run on its own.
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        info!("Connected to the DB");
        
        Ok(PostgresqlAdapter {
            client: client
        })
    }
}

impl Adapter for PostgresqlAdapter {

    async fn query<T, O>(&self, query: &String, params: T) -> O {
        todo!()
    }
}