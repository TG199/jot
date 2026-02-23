use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpResponse,
};
use futures_util::future::LocalBoxFuture;
use std::collections::HashMap;
use std::future::{ready, Ready};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct RateLimiter {
    max_requests: usize,
    window: Duration,
    store: Arc<Mutex<HashMap<String, ClientState>>>,
}

#[derive(Debug, Clone)]
struct ClientState {
    requests: Vec<Instant>,
}

impl RateLimiter {
    pub fn new(max_requests: usize, window: Duration) -> Self {
        Self {
            max_requests,
            window,
            store: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    fn is_allowed(&self, key: &str) -> bool {
        let mut store = self.store.lock().unwrap();
        let now = Instant::now();

        let cutoff = now - self.window;

        let state = store.entry(key.to_string()).or_insert(ClientState {
            requests: Vec::new(),
        });

        state.requests.retain(|&time| time > cutoff);

        if state.requests.len() < self.max_requests {
            state.requests.push(now);
            true
        } else {
            false
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RateLimiter
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RateLimiterMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RateLimiterMiddleware {
            service,
            limiter: self.clone(),
        }))
    }
}

pub struct RateLimiterMiddleware<S> {
    service: S,
    limiter: RateLimiter,
}

impl<S, B> Service<ServiceRequest> for RateLimiterMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let key = if let Some(user_id) = req.extensions().get::<uuid::Uuid>() {
            user_id.to_string()
        } else if let Some(peer_addr) = req.peer_addr() {
            peer_addr.ip().to_string()
        } else {
            "unknown".to_string()
        };

        if !self.limiter.is_allowed(&key) {
            let response = HttpResponse::TooManyRequests().json(serde_json::json!({
                "error": "Rate limit execeeded. Please try again later"
            }));

            return Box::pin(async move { Ok(req.into_response(response)) });
        }

        let fut = self.service.call(req);
        Box::pin(async move { fut.await })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web, App, HttpResponse};

    #[actix_web::test]
    async fn rate_limiter_allows_within_limit() {
        let limiter = RateLimiter::new(5, Duration::from_secs(60));

        let app = test::init_service(
            App::new()
                .wrap(limiter)
                .route("/", web::get().to(|| async { HttpResponse::Ok().finish() })),
        )
        .await;

        for _ in 0..5 {
            let req = test::TestRequest::get().uri("/").to_request();
            let resp = test::call_service(&app, req).await;
            assert_eq!(resp.status(), 200);
        }
    }

    #[actix_web::test]
    async fn rate_limiter_blocks_over_limit() {
        let limiter = RateLimiter::new(3, Duration::from_secs(60));

        let app = test::init_service(
            App::new()
                .wrap(limiter)
                .route("/", web::get().to(|| async { HttpResponse::Ok().finish() })),
        )
        .await;

        for _ in 0..3 {
            let req = test::TestRequest::get().uri("/").to_request();
            let resp = test::call_service(&app, req).await;
            assert_eq!(resp.status(), 200);
        }

        let req = test::TestRequest::get().uri("/").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 429);
    }
}
