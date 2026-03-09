use actix_web::HttpResponse;

/// Health check endpoint available @ GET /api/health
pub async fn health_check() -> HttpResponse {
    HttpResponse::Ok().json(serde_json::json!({
        "msg": "OK"
    }))
}