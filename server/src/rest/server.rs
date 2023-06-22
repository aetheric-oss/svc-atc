//! Rest server implementation

use super::api;
use crate::config::Config;
use crate::grpc::client::GrpcClients;
use crate::shutdown_signal;
use axum::{
    error_handling::HandleErrorLayer,
    extract::Extension,
    http::{HeaderValue, StatusCode},
    routing, BoxError, Router,
};
use tower::{
    buffer::BufferLayer,
    limit::{ConcurrencyLimitLayer, RateLimitLayer},
    ServiceBuilder,
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

/// Starts the REST API server for this microservice
#[cfg(not(tarpaulin_include))]
pub async fn rest_server(config: Config) -> Result<(), ()> {
    use std::net::SocketAddr;

    rest_debug!("(rest_server) entry.");
    let grpc_clients = GrpcClients::new(config.clone());
    let rest_port = config.docker_port_rest;
    let full_rest_addr: SocketAddr = match format!("[::]:{}", rest_port).parse() {
        Ok(addr) => addr,
        Err(e) => {
            rest_error!("(rest_server) invalid address: {:?}, exiting.", e);
            return Err(());
        }
    };

    let cors_allowed_origin = match config.rest_cors_allowed_origin.parse::<HeaderValue>() {
        Ok(url) => url,
        Err(e) => {
            rest_error!(
                "(rest_server) invalid cors_allowed_origin address: {:?}, exiting.",
                e
            );
            return Err(());
        }
    };

    let rate_limit = config.rest_request_limit_per_second as u64;
    let concurrency_limit = config.rest_concurrency_limit_per_service as usize;
    let limit_middleware = ServiceBuilder::new()
        .layer(TraceLayer::new_for_http())
        .layer(HandleErrorLayer::new(|e: BoxError| async move {
            rest_warn!("(server) too many requests: {}", e);
            (
                StatusCode::TOO_MANY_REQUESTS,
                "(server) too many requests.".to_string(),
            )
        }))
        .layer(BufferLayer::new(100))
        .layer(ConcurrencyLimitLayer::new(concurrency_limit))
        .layer(RateLimitLayer::new(
            rate_limit,
            std::time::Duration::from_secs(1),
        ));

    let app = Router::new()
        .route("/health", routing::get(api::health_check)) // MUST HAVE
        .route("/ack/flight", routing::post(api::acknowledge_flight_plan))
        .layer(
            CorsLayer::new()
                .allow_origin(cors_allowed_origin)
                .allow_headers(Any)
                .allow_methods([axum::http::Method::POST]),
        )
        .layer(limit_middleware)
        .layer(Extension(grpc_clients)); // Extension layer must be last

    rest_info!("(rest) hosted at {:?}", full_rest_addr);
    match axum::Server::bind(&full_rest_addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal("rest"))
        .await
    {
        Ok(_) => rest_info!("REST server running at: {}.", full_rest_addr),
        Err(e) => {
            rest_error!("could not start REST server: {}", e);
        }
    };

    Ok(())
}
