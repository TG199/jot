use crate::authentication::TypedSession;
use actix_web::HttpResponse;

pub async fn logout(session: TypedSession) -> HttpResponse {
    session.log_out();
    HttpResponse::Ok().json(serde_json::json!({
        "message": "Logout successful"
    }))
}
