use actix_web::{Responder, HttpResponse, get, post, web};
use log::info;

mod parser;


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

pub fn load_dsl_endpoints(args: &crate::args::types::Args) -> impl Fn(&mut web::ServiceConfig) {
    move |cfg: &mut web::ServiceConfig| {
        info!("Loading DSL endpoints from path: {}", args.dsl_path);

        let collection = parser::EndpointCollections::parse_from_dir(&args.dsl_path);
        info!("Loaded next endpoints collection: {}", collection);

        cfg
        .service(hello)
        .service(echo)
        .route("/hey", web::get().to(manual_hello));
    }
}