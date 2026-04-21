mod antenna;
mod coverage;
mod elevation;
mod fresnel;
mod itm_bridge;
mod models;
mod routes;
mod signal_levels;
mod terrain;

use std::env;

use axum::{routing::post, Router};
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let origins: Vec<_> = env::var("DEV_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:3000,http://localhost:3001".into())
        .split(',')
        .map(|s| s.trim().parse().unwrap())
        .collect();

    let cors = CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([axum::http::Method::POST, axum::http::Method::OPTIONS])
        .allow_headers([axum::http::header::CONTENT_TYPE]);

    let app = Router::new()
        .route("/api/p2p", post(routes::p2p::p2p_handler))
        .route("/api/coverage", post(routes::coverage::coverage_handler))
        .route(
            "/api/coverage-radius",
            post(routes::coverage_radius::coverage_radius_handler),
        )
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    tracing::info!("Server listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}
