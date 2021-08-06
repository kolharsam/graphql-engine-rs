use log::{debug, error, info, trace, warn};

mod logger;
mod options;
mod server;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    debug!("Setting up logger...");
    logger::setup_logging().expect("Failed to set up the logger");

    debug!("GraphQL-Engine-RS is being initialised...");

    let serve_options = options::parsed_options();
    if serve_options.source_name == "default" {
        warn!("No source-name was provided, setting \"default\" as source-name.");
    }
    trace!("Server options have been parsed: {:?}", serve_options);

    info!("Starting up API server on port {}", serve_options.port);

    // let gqlrs_figment = rocket::Config::figment().merge(("port", serve_options.port));

    let server_result = rocket::custom(gqlrs_figment)
        .mount(
            "/",
            rocket::routes![server::healthz, server::metadata_route],
        )
        .launch()
        .await;
    if server_result.is_err() {
        error!("Failure in API server: {:?}", server_result.unwrap_err());
    }
}
