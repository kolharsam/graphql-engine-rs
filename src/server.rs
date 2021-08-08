use serde::{Deserialize, Serialize};

#[path = "./context.rs"]
mod context;
#[path = "./types.rs"]
mod types;

pub async fn healthz(_req: actix_web::HttpRequest) -> String {
    // actix_web::HttpResponse::Ok().json(json!({"Ok": true}))
    "OK".to_string()
    // actix_web::HttpResponse::Ok().finish()
    // TODO: add something for "ERROR"
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "args")]
#[serde(rename_all = "snake_case")]
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

#[derive(Serialize)]
pub struct MetadataError {
    error: String,
}

pub async fn metadata_route(rbody: actix_web::web::Bytes) -> actix_web::HttpResponse {
    let parse_result = json::parse(std::str::from_utf8(&rbody).unwrap());
    let body: json::JsonValue = match parse_result {
        Ok(v) => v,
        Err(e) => json::object! { "error" => e.to_string() },
    };

    match serde_json::from_str::<'_, MetadataMessage>(&body.dump()) {
        Ok(b) => actix_web::HttpResponse::Ok()
            .content_type("application/json")
            .body(serde_json::to_string(&b).unwrap()),
        Err(e) => actix_web::HttpResponse::build(actix_web::http::StatusCode::BAD_REQUEST).json(
            MetadataError {
                error: e.to_string(),
            },
        ),
    }

    // MetadataResponse::Success(MetadataSuccess {
    //     success: true,
    //     message: "OK".to_string(),
    // })
}
