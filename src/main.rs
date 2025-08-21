use axum::{Router, extract::Request, middleware::Next, response::Response};
use log::{LevelFilter, info, warn};
use log4rs;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Config, Root};
use sqlx::PgPool;
use std::time::Instant;
use tokio;

use crate::endpoints::load_dsl_endpoints;
mod args;
mod endpoints;

fn init_logging(args: &args::types::Args) -> Option<()> {
    match &args.log_config {
        Some(path) => {
            log4rs::init_file(path, Default::default()).ok()?;
        }
        None => {
            let stdout = ConsoleAppender::builder().build();

            let config = Config::builder()
                .appender(Appender::builder().build("stdout", Box::new(stdout)))
                .build(Root::builder().appender("stdout").build(LevelFilter::Debug)).ok()?;
            log4rs::init_config(config).ok()?;
        }
    }

    Some(())
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

    if init_logging(&args).is_none() {
        println!("cannot initialize logging!");
        return;
    }

    print_hello();

    let pool;
    match PgPool::connect(&args.db_uri).await {
        Ok(r) => pool = r,
        Err(e) => {
            warn!("{}", e);
            return;
        }
    }

    let port = args.port;
    let bind = args.bind.clone();

    let app = Router::new().layer(axum::middleware::from_fn(uri_middleware));

    let app = load_dsl_endpoints(&args, app).with_state(pool);

    let listener;
    match tokio::net::TcpListener::bind(format!("{}:{}", bind, port)).await {
        Ok(l) => listener = l,
        Err(e) => {
            warn!("{}", e);
            return;
        }
    }

    let duration = start.elapsed();
    info!("Server startup completed in {:?}", duration);
    info!("Starting server at http://{}:{}", args.bind, args.port);

    if let Err(e) = axum::serve(listener, app).await {
        warn!("{}", e);
    }
}

async fn uri_middleware(request: Request, next: Next) -> Response {
    let uri = request.uri().clone();

    let response = next.run(request).await;

    info!("{} - {}", uri, response.status());

    response
}

fn main() {
    let args = args::types::get_args();
    match tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
    {
        Ok(r) => r.block_on(async move { init_and_run(&args).await }),
        Err(e) => println!("{}", e),
    }
}
