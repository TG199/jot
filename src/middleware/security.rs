use actix_cors::Cors;
use actix_web::http::header;

pub fn configure_cors(allowed_origin: &str) -> Cors {
    Cors::default()
        .allowed_origin(allowed_origin)
        .allowed_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"])
        .allowed_headers(vec![
            header::AUTHORIZATION,
            header::ACCEPT,
            header::CONTENT_TYPE,
        ])
        .expose_headers(vec![headers::HeaderName::from_static("x-request-id")])
        .max_age(3600)
        .supports_credentials()
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App, HttpResponse};

    #[actix_web::test]
    async fn cors_allows_configured_origin() {
        let app = test::init_service(
            App::new()
                .wrap(configure_cors("http://localhost:3000"))
                .route("/", web::get().to(|| async { HttpResponse::Ok().finish() })),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/")
            .insert_header(("Origin", "http://localhost:3000"))
            .to_request();

        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), 200);
        assert!(resp.headers().get("access-control-allow-origin").is_some());
    }

    #[actix_web::test]
    async fn cors_includes_credentials_support() {
        let app = test::init_service(
            App::new()
                .wrap(configure_cors("http://localhost:3000"))
                .route("/", web::get().to(|| async { HttpResponse::Ok().finish() })),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/")
            .insert_header(("Origin", "http://localhost:3000"))
            .to_request();

        let resp = test::call_service(&app, req).await;

        let allow_credentials = resp.headers().get("access-control-allow-credentials");
        assert!(allow_credentials.is_some());
        assert_eq!(allow_credentials.unwrap().to_str().unwrap(), "true");
    }

    #[actix_web::test]
    async fn cors_exposes_request_id_header() {
        let app = test::init_service(
            App::new()
                .wrap(configure_cors("http://localhost:3000"))
                .route("/", web::get().to(|| async { HttpResponse::Ok().finish() })),
        )
        .await;

        let req = test::TestRequest::options()
            .uri("/")
            .insert_header(("Origin", "http://localhost:3000"))
            .insert_header(("Access-Control-Request-Method", "GET"))
            .to_request();

        let resp = test::call_service(&app, req).await;

        let expose_headers = resp.headers().get("access-control-expose-headers");
        assert!(expose_headers.is_some());
        let headers_str = expose_headers.unwrap().to_str().unwrap();
        assert!(headers_str.contains("x-request-id"));
    }
}
