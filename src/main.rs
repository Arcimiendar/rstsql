use tokio;
use axum::{Router, extract::Request, response::Response, middleware::Next};
use log::{LevelFilter, info};
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs;
use std::time::Instant;

use crate::endpoints::load_dsl_endpoints;
use crate::adapters::get_adapter_type;
mod args;
mod endpoints;
mod adapters;


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


fn print_hello() {
    println!(r#" ____      _   ____   ___  _     "#);
    println!(r#"|  _ \ ___| |_/ ___| / _ \| |    "#);
    println!(r#"| |_) / __| __\___ \| | | | |    "#);
    println!(r#"|  _ <\__ \ |_ ___) | |_| | |___ "#);
    println!(r#"|_| \_\___/\__|____/ \__\_\_____|"#);
    println!(r#"                                 "#);
}

async fn init_and_run(args: &args::types::Args) {
    
    let start = Instant::now();

    init_logging(&args).unwrap();

    print_hello();

    let database = get_adapter_type(args.db_uri.clone()).await;

    let port = args.port;
    let bind = args.bind.clone();

    let app = Router::new()
        .layer(axum::middleware::from_fn(uri_middleware));

    let app = load_dsl_endpoints(&args, app);
    
    let listener = tokio::net::TcpListener::bind(
        format!("{}:{}", bind, port)
    ).await.unwrap();


    let duration = start.elapsed();
    info!("Server startup completed in {:?}", duration);
    info!("Starting server at http://{}:{}", args.bind, args.port);

    axum::serve(listener, app).await.unwrap();

}

async fn uri_middleware(request: Request, next: Next) -> Response {
    let uri = request.uri().clone();

    let response = next.run(request).await;

    info!("{} - {}", uri, response.status());

    response
}

fn main() {
    let args = args::types::get_args();
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async move {
            init_and_run(&args).await
        })
}
