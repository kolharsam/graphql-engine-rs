use actix_web::{http::StatusCode, web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::context::AppState;
use crate::error::GQLRSError;
use crate::metadata::{Metadata, QualifiedTable};

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "type", content = "args", rename_all = "snake_case")]
pub enum MetadataRequestBody {
    TrackTable(QualifiedTable),
    UntrackTable(QualifiedTable),
    // NOTE: args will be `null` for `export_metadata`
    ExportMetadata,
    ImportMetadata(Metadata),
}

#[derive(Serialize, Debug, Clone)]
pub enum MetadataResponse<T = Metadata> {
    // NOTE: `T` is `Metadata` temporarily for `export_metadata` request
    Success(String),
    Data(T),
    Error(GQLRSError),
}

impl<T> Responder for MetadataResponse<T>
where
    T: Serialize,
{
    type Error = actix_web::Error;
    type Future = HttpResponse;

    fn respond_to(self, _: &HttpRequest) -> Self::Future {
        match self {
            MetadataResponse::Error(err_resp) => {
                HttpResponse::build(StatusCode::BAD_REQUEST).json(err_resp)
            }
            MetadataResponse::Success(msg) => {
                HttpResponse::Ok().json(json!({"success": true, "message": msg}))
            }
            MetadataResponse::Data(body) => HttpResponse::Ok().json(body),
        }
    }
}

pub async fn metadata_handler(
    app_state: web::Data<AppState>,
    payload: web::Json<MetadataRequestBody>,
) -> impl Responder {
    let mut server_ctx = app_state.0.lock().unwrap();

    match payload.into_inner() {
        MetadataRequestBody::TrackTable(table) => {
            match (*server_ctx).metadata_track_table(table.clone()) {
                Ok(_) => MetadataResponse::Success(format!(
                    "{} is now being tracked!",
                    table.to_string()
                )),
                Err(err) => MetadataResponse::Error(err),
            }
        }
        MetadataRequestBody::UntrackTable(table) => {
            match (*server_ctx).metadata_untrack_table(table.clone()) {
                Ok(_) => MetadataResponse::Success(format!(
                    "{} has now been un-tracked!",
                    table.to_string()
                )),
                Err(err) => MetadataResponse::Error(err),
            }
        }
        MetadataRequestBody::ExportMetadata => {
            MetadataResponse::Data((*server_ctx).get_metadata().clone())
        }
        MetadataRequestBody::ImportMetadata(md) => {
            (*server_ctx).replace_metadata(&md);
            MetadataResponse::Success("Imported metadata successfully!".to_string())
        }
    }
}
