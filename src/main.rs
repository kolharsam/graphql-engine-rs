use log::{debug, info, trace, warn};

mod context;
mod db;
mod error;
mod gql_types;
#[path = "handlers/graphql.rs"]
mod graphql;
#[path = "handlers/healthz.rs"]
mod healthz;
mod logger;
mod metadata;
#[path = "handlers/metadata.rs"]
mod metadata_handler;
mod options;
mod utils;

use graphql::graphql_handler;
use healthz::healthz_handler;
use metadata_handler::metadata_handler;

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

    let pg_connection_pool_res = db::get_pg_pool(&serve_options.connection_string);

    let server_ctx = match pg_connection_pool_res {
        Ok(pg_pool) => context::ServerCtx::new(pg_pool, serve_options.source_name.as_str()),
        Err(e) => panic!("failed to initiate the connection pool with given connection string {}, see error: {:?}", serve_options.connection_string, e),
    };

    let app_state = context::AppState::new_state(server_ctx);

    actix_web::HttpServer::new(move || {
        actix_web::App::new()
            .app_data(app_state.clone())
            .wrap(actix_web::middleware::Logger::default())
            .wrap(
                actix_cors::Cors::default()
                    .allow_any_origin()
                    .allowed_methods(vec!["GET", "POST", "OPTIONS"])
                    .max_age(3600),
            )
            .service(
                actix_web::web::resource("/healthz")
                    .route(actix_web::web::get().to(healthz_handler)),
            )
            .service(
                actix_web::web::scope("/v1")
                    .route("/metadata", actix_web::web::post().to(metadata_handler))
                    .route("/graphql", actix_web::web::post().to(graphql_handler)),
            )
            .default_service(actix_web::web::to(actix_web::HttpResponse::NotFound))
    })
    .bind(("127.0.0.1", serve_options.port))?
    .run()
    .await
}

#[cfg(test)]
mod tests {
    use actix_web::{test, web, App};
    use serde::{Serialize, Deserialize};

    use crate::context::{AppState, ServerCtx};
    use crate::db::get_pg_pool;
    use crate::graphql::{empty_query_variables, graphql_handler, GraphQLRequest};
    use crate::healthz::healthz_handler;
    use crate::metadata::{Metadata, QualifiedTable};
    use crate::metadata_handler::metadata_handler;

    const DEFAULT_DATABASE_URL: &str =
        "postgresql://postgres:postgrespassword@localhost:5432/postgres";
    const GRAPHQL_TEST_TYPE: &str = "GraphQL";
    const PATH_TEST_BASE: &str = "test";
    const QUERY_FILE_NAME: &str = "query.graphql";
    const RESPONSE_FILE_NAME: &str = "response.json";
    const HEALTHZ_ENDPOINT: &str = "/healthz";
    const GRAPHQL_ENDPOINT: &str = "/v1/graphql";
    const METADATA_ENDPOINT: &str = "/v1/metadata";
    const CONTENT_TYPE_KEY: &str = "Content-Type";
    const APPLICATION_JSON_VALUE: &str = "application/json";

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

    fn get_test_request(endpoint: &str, payload: String) -> test::TestRequest {
        test::TestRequest::post()
            .uri(endpoint)
            .header(CONTENT_TYPE_KEY, APPLICATION_JSON_VALUE)
            .set_payload(payload)
    }

    // NOTE/TODO/FIXME: This test might've become too tightly coupled at this point
    // We need to make sure that we can test as many smaller aspects of this not just
    // the overall picture. This can eventually become a huge pain point
    #[actix_rt::test]
    async fn test_handlers() {
        let default_pg_conn_str = String::from(DEFAULT_DATABASE_URL);
        let connection_string = std::env::var("DATABASE_URL").unwrap_or(default_pg_conn_str);
        let server_ctx = ServerCtx::new(get_pg_pool(&connection_string).unwrap(), "default");
        let app_state = AppState::new_state(server_ctx);

        let mut app = test::init_service(
            App::new()
                .app_data(app_state)
                .service(web::resource("/healthz").route(actix_web::web::get().to(healthz_handler)))
                .service(
                    web::scope("/v1")
                        .route("/metadata", web::post().to(metadata_handler))
                        .route("/graphql", web::post().to(graphql_handler)),
                ),
        )
        .await;

        // test the healthz endpoint

        #[derive(serde::Deserialize, Serialize, PartialEq, Debug)]
        struct HealthzResponse {
            status: String,
        }

        let healthz_request = test::TestRequest::get().uri(HEALTHZ_ENDPOINT).to_request();
        let response: HealthzResponse = test::read_response_json(&mut app, healthz_request).await;
        assert_eq!(
            response,
            HealthzResponse {
                status: String::from("OK")
            }
        );

        // test `graphql` and `metadata` handlers

        // Set up Metadata

        #[derive(Serialize, Deserialize, Debug, PartialEq)]
        struct MetadataSuccess {
            success: bool,
        }

        let metadata_request_files = std::fs::read_dir("test/metadata/track_table")
            .expect("Failed to access the `test/metadata/track_table` dir.");

        for metadata_request_file in metadata_request_files {
            let metadata_request_filepath = metadata_request_file.unwrap().path();
            if metadata_request_filepath.is_dir() {
                continue;
            }
            let metadata_request_payload = std::fs::read_to_string(
                metadata_request_filepath.clone(),
            )
            .expect(format!("failed to read file at {:?}", metadata_request_filepath).as_str());

            let metadata_request =
                get_test_request(METADATA_ENDPOINT, metadata_request_payload).to_request();
            let test_response: MetadataSuccess =
                test::read_response_json(&mut app, metadata_request).await;
            assert_eq!(
                test_response,
                MetadataSuccess {
                    success: true,
                    // TODO: check for the message too
                }
            );
        }

        // Test the graphql queries

        // TODO: automate this part too. we can fetch the dir. list
        // and based on that we could ensure that we don't have to
        // manually add entries for tests
        let test_folders = [
            "basic_query",
            "basic_query_test_key_order",
            "basic_query_with_limit_offset_distinct_on",
            "query_with_aliases",
            "query_order_by_asc",
            "query_order_by_desc",
            "query_order_by_asc_desc",
        ];

        // NOTE: Try and make this parallelised
        for test_dir in test_folders {
            let test_file_path = get_graphql_test_file_path(test_dir);
            let query_str = read_test_file(&test_file_path);

            let data = GraphQLRequest {
                query: query_str,
                variables: empty_query_variables(),
            };

            let payload = serde_json::to_string(&data).unwrap();

            let req = get_test_request(GRAPHQL_ENDPOINT, payload).to_request();

            #[derive(Deserialize, Serialize, Clone, Debug)]
            struct DataResponse {
                data: indexmap::IndexMap<String, serde_json::Value>,
            }

            let result: DataResponse = test::read_response_json(&mut app, req).await;
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

        // test `export_metadata` and check response

        let export_metadata_path = "test/metadata/export_metadata/export_metadata.json";
        let export_metadata_str =
            std::fs::read_to_string(std::path::Path::new(export_metadata_path))
                .expect("failed to open export_metadata file");
        let export_metadata_request =
            get_test_request(METADATA_ENDPOINT, export_metadata_str.clone()).to_request();
        let current_metadata: Metadata =
            test::read_response_json(&mut app, export_metadata_request).await;
        assert_eq!(current_metadata.source_name, String::from("default"));
        assert_eq!(
            current_metadata.tables,
            vec![
                QualifiedTable::new("public", "authors"),
                QualifiedTable::new("new_schema", "users")
            ]
        );

        // test `untrack_table`

        let md_request_files = std::fs::read_dir("test/metadata/untrack_table")
            .expect("Failed to access the `test/metadata/untrack_table` dir.");

        for metadata_request_file in md_request_files {
            let metadata_request_filepath = metadata_request_file.unwrap().path();
            let metadata_request_payload = std::fs::read_to_string(
                metadata_request_filepath.clone(),
            )
            .expect(format!("failed to read file at {:?}", metadata_request_filepath).as_str());

            let metadata_request =
                get_test_request(METADATA_ENDPOINT, metadata_request_payload).to_request();
            let test_response: MetadataSuccess =
                test::read_response_json(&mut app, metadata_request).await;
            assert_eq!(test_response, MetadataSuccess { success: true });
        }

        // test `export_metadata` after `untrack_table`

        let export_metadata_req =
            get_test_request(METADATA_ENDPOINT, export_metadata_str.clone()).to_request();
        let current_md: Metadata = test::read_response_json(&mut app, export_metadata_req).await;
        assert_eq!(current_md.source_name, String::from("default"));
        assert_eq!(current_md.tables, vec![]);

        // test `import_metadata`

        let import_metadata_str = std::fs::read_to_string(std::path::Path::new(
            "test/metadata/import_metadata/import_metadata.json",
        ))
        .expect("failed to read file at test/metadata/import_metadata/import_metadata.json");
        let import_metadata_request =
            get_test_request(METADATA_ENDPOINT, import_metadata_str).to_request();
        let test_res: MetadataSuccess =
            test::read_response_json(&mut app, import_metadata_request).await;
        assert_eq!(test_res, MetadataSuccess { success: true });

        // test `export_metadata` after `import_metadata`
        let export_md_req =
            get_test_request(METADATA_ENDPOINT, export_metadata_str.clone()).to_request();
        let crnt_md: Metadata = test::read_response_json(&mut app, export_md_req).await;
        assert_eq!(crnt_md.source_name, String::from("default2"));
        assert_eq!(
            crnt_md.tables,
            vec![
                QualifiedTable::new("public", "authors"),
                QualifiedTable::new("new_schema", "users")
            ]
        );
    }
}
