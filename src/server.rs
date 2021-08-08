// use actix_web::{get, post};
// use log::{debug, info, trace, warn};
use serde::{Deserialize, Serialize};
// use serde_json::to_string;

#[path = "./context.rs"]
mod context;
#[path = "./types.rs"]
mod types;

// #[get("/healthz")]
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
    details: String,
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum MetadataResponse {
    Success(MetadataSuccess),
    Error(MetadataError),
}

// #[post("/v1/metadata")]
pub async fn metadata_route(
    rbody: actix_web::web::Bytes,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    // println!(
    //     "{:?} {:?}",
    //     to_string(&rbody.r#type),
    //     to_string(&rbody.args)
    // );
    let parse_result = json::parse(std::str::from_utf8(&rbody).unwrap());
    let body: json::JsonValue = match parse_result {
        Ok(v) => v,
        Err(e) => json::object! {"error" => e.to_string() },
    };
    if !body.is_object() {
        return Err(actix_web::error::ErrorBadRequest("Invalid Request Body!"));
    }
    // TODO: report "parse" errors in HTTP response
    let md_body: MetadataMessage =
        serde_json::from_str(&body.dump()).expect("Invalid \"args\" passed");
    // match md_body.r#type {
    //     MetadataTypes::TrackTable =>
    // }
    Ok(actix_web::HttpResponse::Ok()
        .content_type("application/json")
        .body(serde_json::to_string(&md_body).unwrap()))
    // req.

    // format!("{:?} {:?}", to_string(&rbody.r#type), to_string(&rbody.args));

    // actix_web::HttpResponse::Ok().json(rbody.0)
    // format!(MetadataResponse::Success(MetadataSuccess {
    //     success: true,
    //     message: "OK".to_string(),
    // }))
}
