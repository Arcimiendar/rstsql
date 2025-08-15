use actix_web::{Responder, HttpResponse, get, post, web};
use log::{info, warn, debug};

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


// fn load_project_namespace(entry: std::fs::DirEntry, cfg: &mut web::ServiceConfig) {
//     let path = entry.path();
//     if path.is_dir() {
//         debug!("Found DSL project namespace: {:?}", path);

//         let paths = std::fs::read_dir(&path).unwrap();
//         paths.into_iter()
//             .map(|e| e.unwrap())
//             .filter(|e| {
//                 let valid = e.path().ends_with("GET") || e.path().ends_with("POST");
//                 if !valid {
//                     warn!("Path should be either GET or POST, found: {}", e.path().display());
//                 }
//                 valid
//             }).for_each(|e| {
//                 if e.path().ends_with("GET")
//             });
            
//     } else {
//         info!("Skipping non-directory entry: {:?}", path);
//     }
// }


pub fn load_dsl_endpoints(args: &crate::args::types::Args) -> impl Fn(&mut web::ServiceConfig) {
    move |cfg: &mut web::ServiceConfig| {
        info!("Loading DSL endpoints from path: {}", args.dsl_path);

        let collection = parser::EndpointCollections::parse_from_dir(&args.dsl_path);
        info!("Loaded next endpoints collection: {}", collection);

        // let paths = std::fs::read_dir(&args.dsl_path).unwrap();
        // paths.into_iter().for_each(|entry| {
        //     load_project_namespace(entry.unwrap(), cfg);
        // });


        cfg
        .service(hello)
        .service(echo)
        .route("/hey", web::get().to(manual_hello));
    }
}