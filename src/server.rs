use actix_web::{get};
use serde::{Deserialize, Serialize};
use serde_json::to_string;

#[path = "./context.rs"]
mod context;
#[path = "./types.rs"]
mod types;

#[get("/healthz")]
pub async fn healthz() -> &'static str {
    "OK"
    // TODO: add something for "ERROR"
}

#[derive(Serialize, Deserialize, Copy, Clone)]
#[serde(rename_all = "snake_case")]
pub enum MetadataTypes {
    TrackTable,
    UnTrackTable,
    ExportMetadata,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum MetadataBodyArgs {
    // #[serde(borrow)]
    TrackTableBody(types::QualifiedTable),
    UnTrackTableBody(types::QualifiedTable),
    ExportMetadataBody,
}

#[derive(Serialize, Deserialize)]
pub struct MetadataMessage {
    r#type: MetadataTypes,
    // #[serde(borrow)]
    args: MetadataBodyArgs,
}

#[derive(Serialize, Deserialize)]
pub struct MetadataSuccess {
    success: bool,
    message: String,
}

#[derive(Serialize, Deserialize)]
pub struct MetadataError {
    error: String,
    details: String,
}

#[derive(Serialize, Deserialize)]
pub enum MetadataResponse {
    Success(MetadataSuccess),
    Error(MetadataError),
}

#[rocket::post("/v1/metadata", format = "json", data = "<rbody>")]
pub fn metadata_route(rbody: Json<MetadataMessage>) -> Json<MetadataResponse> {
    let (request_type, args) = (rbody.r#type, rbody.args);

    println!("{:?} {:?}", to_string(&request_type), to_string(&args));

    Json(MetadataResponse::Success(MetadataSuccess {
        success: true,
        message: "OK".to_string(),
    }))
}
