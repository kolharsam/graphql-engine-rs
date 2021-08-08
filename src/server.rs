use serde::{Deserialize, Serialize};

#[path = "./context.rs"]
mod context;
#[path = "./types.rs"]
mod types;

pub async fn healthz_handler(_req: actix_web::HttpRequest) -> String {
    // actix_web::HttpResponse::Ok().json(json!({"Ok": true}))
    "OK".to_string()
    // actix_web::HttpResponse::Ok().finish()
    // TODO: add something for "ERROR"
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "args", rename_all = "snake_case")]
pub enum MetadataMessage {
    TrackTable(types::QualifiedTable),
    UntrackTable(types::QualifiedTable),
    // NOTE: args will only be `null` in this case
    ExportMetadata,
}

#[derive(Serialize)]
pub struct MetadataSuccess {
    success: bool,
    message: String,
}

#[derive(Serialize, Debug, Clone)]
pub struct ErrorResponse {
    error: String,
}

pub async fn metadata_handler(payload: actix_web::web::Bytes) -> actix_web::HttpResponse {
    let parse_result = json::parse(std::str::from_utf8(&payload).unwrap());

    // FIXME?: is this even necessary?
    let body: json::JsonValue = match parse_result {
        Ok(v) => v,
        Err(e) => json::object! { "error" => e.to_string() },
    };

    match serde_json::from_str::<'_, MetadataMessage>(&body.dump()) {
        Ok(b) => actix_web::HttpResponse::Ok()
            .content_type("application/json")
            .body(serde_json::to_string(&b).unwrap()),
        Err(e) => actix_web::HttpResponse::build(actix_web::http::StatusCode::BAD_REQUEST).json(
            ErrorResponse {
                error: e.to_string(),
            },
        ),
    }

    // MetadataResponse::Success(MetadataSuccess {
    //     success: true,
    //     message: "OK".to_string(),
    // })
}

fn empty_query_variables() -> std::collections::HashMap<String, String> {
    std::collections::HashMap::new()
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct GraphQLRequest {
    query: String,
    #[serde(default = "empty_query_variables")]
    variables: std::collections::HashMap<String, String>,
}

// NOTE: this is only for Query and Mutation will have to
// implement something different for Subscriptions
pub async fn graphql_handler(payload: actix_web::web::Bytes) -> actix_web::HttpResponse {
    let parse_result = json::parse(std::str::from_utf8(&payload).unwrap());

    // FIXME?: is this even necessary?
    let body: json::JsonValue = match parse_result {
        Ok(v) => v,
        Err(e) => json::object! { "error" => e.to_string() },
    };

    match serde_json::from_str::<'_, GraphQLRequest>(&body.dump()) {
        Ok(b) => {
            match graphql_parser::parse_query::<&str>(&b.query) {
                Ok(q) => {
                    for i in q.definitions.iter() {
                        println!("{:?}", i)
                    }
                }
                Err(e) => return actix_web::HttpResponse::Ok().json(ErrorResponse {
                    error: e.to_string(),
                })
            }

            actix_web::HttpResponse::Ok()
                .content_type("application/json")
                .body(serde_json::to_string(&b).unwrap())
        }
        Err(e) => actix_web::HttpResponse::Ok().json(ErrorResponse {
            error: e.to_string(),
        }),
    }
}
