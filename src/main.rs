use actix_web::http;
use log::{debug, info, trace, warn};

mod context;
mod db;
mod error;
mod logger;
mod options;
mod server;
mod types;
mod utils;
mod ws;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "actix_web=info");

    debug!("Setting up logger...");
    logger::setup_logging().expect("Failed to set up the logger");

    debug!("GraphQL-Engine-RS is being initialised...");

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
            .wrap(actix_web::middleware::Logger::default())
            .wrap(
                actix_cors::Cors::default()
                    .allow_any_origin()
                    .allowed_methods(vec!["GET", "POST"])
                    .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
                    .allowed_header(http::header::CONTENT_TYPE)
                    .max_age(3600),
            )
            .service(
                actix_web::web::resource("/healthz")
                    .route(actix_web::web::get().to(server::healthz_handler)),
            )
            // TODO: make the version "v1" as one `resource` and then add these routes there
            .service(
                actix_web::web::resource("/v1/metadata")
                    .route(actix_web::web::post().to(server::metadata_handler)),
            )
            .service(
                actix_web::web::resource("/v1/graphql")
                    .route(actix_web::web::post().to(server::graphql_handler)),
            )
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
