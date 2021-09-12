use actix::Actor;
use actix_web::http;
use log::{debug, info, trace, warn};

mod context;
mod db;
mod error;
mod logger;
mod options;
mod routes;
mod server;
mod types;
mod utils;
mod websocket;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");

    debug!("Setting up logger...");
    logger::setup_logging().expect("Failed to set up the logger");

    debug!("GraphQL-Engine-RS is being initialised...");

    let server = websocket::WebSocketServer::new().start();
    let serve_options = options::parsed_options();
    if serve_options.source_name == "default" {
        warn!("No source-name was provided, setting \"default\" as source-name.");
    }
    trace!("Server options have been parsed: {:?}", serve_options);

    info!("Starting up API server on port {}", serve_options.port);

    let connection_string = utils::string_to_static_str(serve_options.connection_string.clone());

    actix_web::HttpServer::new(move || {
        actix_web::App::new()
            // TODO: eventually, this would be the server ctx
            .data(<&str>::clone(&connection_string))
            .data(server.clone())
            .wrap(actix_web::middleware::Logger::default())
            .wrap(
                actix_cors::Cors::default()
                    .allow_any_origin()
                    .allowed_methods(vec!["GET", "POST"])
                    .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                    .allowed_header(http::header::CONTENT_TYPE)
                    .max_age(3600),
            )
            .configure(routes::routes)
            .default_service(
                // 404 for GET request
                actix_web::web::resource("")
                    .route(actix_web::web::get().to(actix_web::HttpResponse::NotFound))
                    // all requests that are not `GET`
                    .route(
                        actix_web::web::route()
                            .guard(actix_web::guard::Not(actix_web::guard::Get()))
                            .to(actix_web::HttpResponse::NotFound),
                    ),
            )
    })
    .bind(("127.0.0.1", serve_options.port))?
    .run()
    .await
}
