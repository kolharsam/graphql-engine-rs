use actix_web::{web, HttpRequest, HttpResponse};

use crate::context::ServerCtx;

pub async fn healthz_handler(srv_ctx: web::Data<ServerCtx>, _req: HttpRequest) -> HttpResponse {
    HttpResponse::Ok().json(srv_ctx.get_status_json())
    // TODO: add something for "ERROR"
}
