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

#[cfg(test)]
mod tests {
    use actix_web::{test, web, App};

    use crate::server;
    use crate::utils::string_to_static_str;

    const DEFAULT_DATABASE_URL: &str =
        "postgresql://postgres:postgrespassword@localhost:5432/postgres";
    const GRAPHQL_TEST_TYPE: &str = "GraphQL";
    const PATH_TEST_BASE: &str = "test";
    const QUERY_FILE_NAME: &str = "query.graphql";
    const RESPONSE_FILE_NAME: &str = "response.json";

    // Helper methods for tests

    fn get_test_file_path(test_type: &str, test_name: &str, file_name: &str) -> String {
        let path_str = format!(
            "{}/{}/{}/{}",
            PATH_TEST_BASE, test_type, test_name, file_name
        );
        let path = std::path::Path::new(&path_str);

        path.display().to_string()
    }

    fn get_graphql_test_file_path(test_name: &str) -> String {
        get_test_file_path(GRAPHQL_TEST_TYPE, test_name, QUERY_FILE_NAME)
    }

    fn get_graphql_response_file_path(test_name: &str) -> String {
        get_test_file_path(GRAPHQL_TEST_TYPE, test_name, RESPONSE_FILE_NAME)
    }

    fn read_test_file(path: &str) -> String {
        std::fs::read_to_string(path).expect(format!("failed to read file at {}", path).as_str())
    }

    #[actix_rt::test]
    async fn test_healthz_handler() {
        let req = test::TestRequest::default().to_http_request();
        let resp = server::healthz_handler(req).await;
        assert_eq!(resp, "OK");
    }

    #[actix_rt::test]
    async fn test_graphql_handler() {
        let default_pg_conn_str = String::from(DEFAULT_DATABASE_URL);
        let connection_string =
            string_to_static_str(std::env::var("DATABASE_URL").unwrap_or(default_pg_conn_str));

        let mut app =
            test::init_service(App::new().data(connection_string).service(
                web::resource("/v1/graphql").route(web::post().to(server::graphql_handler)),
            ))
            .await;

        let test_folders = [
            "basic_query",
            "basic_query_test_key_order",
            "basic_query_with_limit_offset_distinct_on",
            "query_with_aliases",
        ];

        // NOTE: Try and make this parallelised
        for test_dir in test_folders {
            let test_file_path = get_graphql_test_file_path(test_dir);
            let query_str = read_test_file(&test_file_path);

            let data: server::GraphQLRequest = server::GraphQLRequest {
                query: query_str,
                variables: server::empty_query_variables(),
            };

            let payload = serde_json::to_string(&data).unwrap();

            let req = test::TestRequest::post()
                .uri("/v1/graphql")
                .header("Content-Type", "application/json")
                .set_payload(payload)
                .to_request();

            let result: server::DataResponse = test::read_response_json(&mut app, req).await;
            let result_json_str = serde_json::to_string_pretty(&result).expect(
                format!(
                    "Failed to convert result to JSON string for {}: {:?}",
                    test_dir, result
                )
                .as_str(),
            );
            let expected_result_file_path = get_graphql_response_file_path(test_dir);
            let expected_result = read_test_file(&expected_result_file_path);

            assert_eq!(result_json_str, expected_result);
        }
    }
}
