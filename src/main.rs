use actix_web::{App, HttpServer, middleware::Logger};
use std::time::Instant;
use log::{info, LevelFilter};
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs;

use crate::endpoints::load_dsl_endpoints;
mod args;
mod endpoints;


fn init_logging(args: &args::types::Args) -> std::io::Result<()> {
    match &args.log_config {
        Some(path) => {
            log4rs::init_file(path, Default::default())
                .map_err(|e| {
                    eprintln!("Failed to initialize logging: {}", e);
                    std::io::Error::new(std::io::ErrorKind::Other, "Logging initialization failed")
                })?;
        }
        None => {
            let stdout = ConsoleAppender::builder().build();

            let config = Config::builder()
                .appender(Appender::builder().build("stdout", Box::new(stdout)))
                .build(Root::builder().appender("stdout").build(LevelFilter::Debug))
                .unwrap();
            log4rs::init_config(config).unwrap();

        }
    }

    Ok(())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let start = Instant::now();

    let args = args::types::get_args();
    init_logging(&args)?;

    let port = args.port;
    let bind = args.bind.clone();

    let log = format!("Starting server at http://{}:{}", bind, port);

    let server = HttpServer::new(move || {
        App::new()
            .configure(load_dsl_endpoints(&args))
            .wrap(Logger::default())
    }).bind((bind, port))?;

    let duration = start.elapsed();
    info!("Server startup completed in {:?}", duration);
    info!("{}", log);
    
    server.run().await
}