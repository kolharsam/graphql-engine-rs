use actix_web::web;

use crate::server;
use crate::websocket;

pub fn routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::resource("/healthz").route(web::get().to(server::healthz_handler)))
        // TODO: make the version "v1" as one `resource` and then add these routes there
        // .service(web::resource("/v1/metadata").route(web::post().to(server::metadata_handler)))
        // .service(
        //     web::resource("/v1/graphql")
        //         .route(web::post().to(server::graphql_handler))
        //         .route(web::get().to(websocket::ws_index)),
        // )
        .service(
            web::scope("/v1")
                .route("/metadata", web::post().to(server::metadata_handler))
                .service(
                    web::resource("/graphql")
                        .route(web::post().to(server::graphql_handler))
                        .route(web::get().to(websocket::ws_index)),
                ),
        );
}
