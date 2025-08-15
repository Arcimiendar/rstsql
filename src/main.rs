use actix_web::{get, post, web, App, HttpResponse, HttpServer, Responder, middleware::Logger};
use std::time::Instant;
use log::{info, LevelFilter};
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs;
mod args;



#[get("/")]
async fn hello() -> impl Responder {
    HttpResponse::Ok().body("Hello world!")
}

#[post("/echo")]
async fn echo(req_body: String) -> impl Responder {
    HttpResponse::Ok().body(req_body)
}

async fn manual_hello() -> impl Responder {
    HttpResponse::Ok().body("Hey there!")
}


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

    let log = format!("Starting server at http://{}:{}", &args.bind, &args.port);

    let server = HttpServer::new(|| {
        App::new()
            .service(hello)
            .service(echo)
            .route("/hey", web::get().to(manual_hello))
            .wrap(Logger::default())
    }).bind((args.bind, args.port))?;

    let duration = start.elapsed();
    info!("Server startup completed in {:?}", duration);
    info!("{}", log);
    
    server.run().await
}