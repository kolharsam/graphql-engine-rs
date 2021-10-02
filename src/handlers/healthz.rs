use actix_web::{web, HttpRequest, HttpResponse};

use crate::context::AppState;

pub async fn healthz_handler(app_state: web::Data<AppState>, _req: HttpRequest) -> HttpResponse {
    let server_ctx = app_state.0.lock().unwrap();
    HttpResponse::Ok().json(server_ctx.get_status_json())
    // TODO: add something for "ERROR"
}
