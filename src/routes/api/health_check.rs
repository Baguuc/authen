use actix_web::HttpResponse;
use tracing::instrument;

/// Health check endpoint available @ GET /api/health
#[instrument(name = "Running a healh check.")]
pub async fn check_health() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "msg": "OK"
    }))
}