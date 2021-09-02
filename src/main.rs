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
    use indexmap::IndexMap;

    use crate::server;
    use crate::utils::string_to_static_str;

    const DEFAULT_DATABASE_URL: &str =
        "postgresql://postgres:postgrespassword@localhost:5432/postgres";

    #[actix_rt::test]
    async fn test_healthz_ok() {
        let req = test::TestRequest::default().to_http_request();
        let resp = server::healthz_handler(req).await;
        assert_eq!(resp, "OK");
    }

    #[actix_rt::test]
    async fn test_graphql_basic() {
        let default_pg_conn_str = String::from(DEFAULT_DATABASE_URL);
        let connection_string =
            string_to_static_str(std::env::var("DATABASE_URL").unwrap_or(default_pg_conn_str));

        let mut app =
            test::init_service(App::new().data(connection_string).service(
                web::resource("/v1/graphql").route(web::post().to(server::graphql_handler)),
            ))
            .await;

        let data: server::GraphQLRequest = server::GraphQLRequest {
            query: "query GetAuthors { authors { id \n author_name } }".to_string(),
            variables: server::empty_query_variables(),
        };

        let payload = serde_json::to_string(&data).unwrap();

        let req = test::TestRequest::post()
            .uri("/v1/graphql")
            .header("Content-Type", "application/json")
            .set_payload(payload)
            .to_request();

        let result: server::DataResponse = test::read_response_json(&mut app, req).await;

        let mut map: IndexMap<String, serde_json::Value> = IndexMap::new();
        let mut item_vec: Vec<serde_json::Value> = Vec::new();
        for (idx, item) in vec!["sam", "bam", "can", "of", "ham"].iter().enumerate() {
            item_vec.push(serde_json::json!({
                "id": idx+1,
                "author_name": item
            }));
        }

        map.insert(
            "authors".to_string(),
            serde_json::to_value(item_vec).unwrap(),
        );

        let check_result: server::DataResponse = server::DataResponse::new(map);

        assert_eq!(result, check_result);
    }

    #[actix_rt::test]
    async fn test_graphql_basic_with_aliases() {
        let default_pg_conn_str = String::from(DEFAULT_DATABASE_URL);
        let connection_string =
            string_to_static_str(std::env::var("DATABASE_URL").unwrap_or(default_pg_conn_str));

        let mut app =
            test::init_service(App::new().data(connection_string).service(
                web::resource("/v1/graphql").route(web::post().to(server::graphql_handler)),
            ))
            .await;

        let data: server::GraphQLRequest = server::GraphQLRequest {
            query: "query GetAuthors { AUthors_TABLE : authors { author_id: id \n name: author_name } }"
                .to_string(),
            variables: server::empty_query_variables(),
        };

        let payload = serde_json::to_string(&data).unwrap();

        let req = test::TestRequest::post()
            .uri("/v1/graphql")
            .header("Content-Type", "application/json")
            .set_payload(payload)
            .to_request();

        let result: server::DataResponse = test::read_response_json(&mut app, req).await;

        let mut map: IndexMap<String, serde_json::Value> = IndexMap::new();
        let mut item_vec: Vec<serde_json::Value> = Vec::new();
        for (idx, item) in vec!["sam", "bam", "can", "of", "ham"].iter().enumerate() {
            item_vec.push(serde_json::json!({
                "author_id": idx+1,
                "name": item
            }));
        }

        map.insert(
            "AUthors_TABLE".to_string(),
            serde_json::to_value(item_vec).unwrap(),
        );

        let check_result: server::DataResponse = server::DataResponse::new(map);

        assert_eq!(result, check_result);
    }

    #[actix_rt::test]
    async fn test_graphql_basic_for_json_key_ordering_in_response() {
        let default_pg_conn_str = String::from(DEFAULT_DATABASE_URL);
        let connection_string =
            string_to_static_str(std::env::var("DATABASE_URL").unwrap_or(default_pg_conn_str));

        let mut app =
            test::init_service(App::new().data(connection_string).service(
                web::resource("/v1/graphql").route(web::post().to(server::graphql_handler)),
            ))
            .await;

        let data: server::GraphQLRequest = server::GraphQLRequest {
            query: "query GetAuthors { authors { name: author_name \n author_id: id } }"
                .to_string(),
            variables: server::empty_query_variables(),
        };

        let payload = serde_json::to_string(&data).unwrap();

        let req = test::TestRequest::post()
            .uri("/v1/graphql")
            .header("Content-Type", "application/json")
            .set_payload(payload)
            .to_request();

        let result: server::DataResponse = test::read_response_json(&mut app, req).await;

        let mut map: IndexMap<String, serde_json::Value> = IndexMap::new();
        let mut item_vec: Vec<serde_json::Value> = Vec::new();
        for (idx, item) in vec!["sam", "bam", "can", "of", "ham"].iter().enumerate() {
            item_vec.push(serde_json::json!({
                "author_id": idx+1,
                "name": item
            }));
        }

        map.insert(
            "authors".to_string(),
            serde_json::to_value(item_vec).unwrap(),
        );

        let check_result: server::DataResponse = server::DataResponse::new(map);

        assert_eq!(result, check_result);
    }

    #[actix_rt::test]
    async fn test_graphql_basic_for_limit_offset_distinct_on_args_with_aliases() {
        let default_pg_conn_str = String::from(DEFAULT_DATABASE_URL);
        let connection_string =
            string_to_static_str(std::env::var("DATABASE_URL").unwrap_or(default_pg_conn_str));

        let mut app =
            test::init_service(App::new().data(connection_string).service(
                web::resource("/v1/graphql").route(web::post().to(server::graphql_handler)),
            ))
            .await;

        let data: server::GraphQLRequest = server::GraphQLRequest {
            query: "query GetAuthors { 
                authors(distinct_on: author_name, limit: 3, offset: 1) { 
                    name: author_name
                    author_id: id 
                } 
            }"
            .to_string(),
            variables: server::empty_query_variables(),
        };

        let payload = serde_json::to_string(&data).unwrap();

        let req = test::TestRequest::post()
            .uri("/v1/graphql")
            .header("Content-Type", "application/json")
            .set_payload(payload)
            .to_request();

        let result: server::DataResponse = test::read_response_json(&mut app, req).await;

        let mut map: IndexMap<String, serde_json::Value> = IndexMap::new();
        let mut item_vec: Vec<serde_json::Value> = Vec::new();
        for (num, item) in vec![(3, "can"), (5, "ham"), (4, "of")].iter() {
            item_vec.push(serde_json::json!({
                "name": item,
                "author_id": num
            }));
        }

        map.insert(
            "authors".to_string(),
            serde_json::to_value(item_vec).unwrap(),
        );

        let check_result: server::DataResponse = server::DataResponse::new(map);

        assert_eq!(result, check_result);
    }
}
