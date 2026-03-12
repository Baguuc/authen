use actix_web::HttpResponse;
use tracing::instrument;

/// Health check endpoint available @ GET /api/health
#[instrument]
pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "msg": "OK"
    }))
}