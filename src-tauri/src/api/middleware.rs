use axum::extract::Request;
use axum::http::header::{AUTHORIZATION, CONTENT_TYPE};
use axum::http::{HeaderValue, Method};
use axum::middleware::Next;
use axum::response::Response;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tower::ServiceBuilder;
use std::time::Duration;

pub fn cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
        .allow_origin("http://127.0.0.1:3000".parse::<HeaderValue>().unwrap())
        .allow_origin("http://127.0.0.1:5173".parse::<HeaderValue>().unwrap())
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE])
        .allow_credentials(true)
        .max_age(Duration::from_secs(3600))
}

pub fn trace_layer() -> TraceLayer<tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsFailureClass>> {
    TraceLayer::new_for_http()
        .make_span_with(tower_http::trace::DefaultMakeSpan::new()
            .level(tracing::Level::INFO))
        .on_response(tower_http::trace::DefaultOnResponse::new()
            .level(tracing::Level::INFO))
}

pub fn middleware_stack() -> ServiceBuilder<
    tower::layer::util::Stack<
        tower_http::trace::TraceLayer<tower_http::classify::SharedClassifier<tower_http::classify::ServerErrorsFailureClass>>,
        tower_http::cors::CorsLayer,
    >
> {
    ServiceBuilder::new()
        .layer(cors_layer())
        .layer(trace_layer())
}

// Optional: Request ID middleware for tracing
pub async fn request_id_middleware(request: Request, next: Next) -> Response {
    let request_id = uuid::Uuid::new_v4().to_string();
    
    // Add request ID to request extensions for access in handlers
    let mut request = request;
    request.extensions_mut().insert(request_id.clone());
    
    let mut response = next.run(request).await;
    
    // Add request ID to response headers
    response.headers_mut().insert(
        "x-request-id",
        HeaderValue::from_str(&request_id).unwrap_or_default(),
    );
    
    response
}

// Optional: Logging middleware
pub async fn logging_middleware(request: Request, next: Next) -> Response {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start = std::time::Instant::now();
    
    let response = next.run(request).await;
    
    let elapsed = start.elapsed();
    let status = response.status();
    
    log::info!(
        "{} {} {} - {}ms",
        method,
        uri,
        status,
        elapsed.as_millis()
    );
    
    response
}