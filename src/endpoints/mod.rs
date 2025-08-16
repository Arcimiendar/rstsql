use actix_web::{web, HttpResponse, Route, Handler};
use log::info;
use std::time::Instant;

use crate::endpoints::parser::{Endpoint, EndpointMethod};

mod parser;


#[derive(Clone)]
struct ActixEndpoint {
    file_content: String
}

impl ActixEndpoint {
    pub fn new(file_content: String) -> ActixEndpoint {
        ActixEndpoint { file_content: file_content }
    }
}

impl Handler<()> for ActixEndpoint {
    type Output = HttpResponse;
    type Future = std::future::Ready<Self::Output>;

    fn call(&self, _: ()) -> Self::Future {
        std::future::ready(HttpResponse::Ok().body(self.file_content.clone()))
    }
}


fn get_route(endpoint: &Endpoint) -> Route {
    let actix_endpoint = ActixEndpoint::new(endpoint.file_content.clone());
    match endpoint.method {
        EndpointMethod::GET => web::get(),
        EndpointMethod::POST => web::post(),
    }.to(actix_endpoint)
}


pub fn load_dsl_endpoints(args: &crate::args::types::Args, ) -> impl Fn(&mut web::ServiceConfig) {
    move |cfg: &mut web::ServiceConfig| {

        let start = Instant::now();

        info!("Loading DSL endpoints from path: {}", args.dsl_path);

        let collection = parser::EndpointCollections::parse_from_dir(&args.dsl_path);
        info!("Loaded next endpoints collection: {}", collection);

        collection.projects.iter().for_each(|p| {
            p.endpoints.iter().for_each(|e| {
                cfg.route(&e.url_path, get_route(&e));
            });
        });

        let duration = start.elapsed();
        info!("Server startup completed in {:?}", duration);
        info!("Starting server at http://{}:{}", args.bind, args.port);
    }
}