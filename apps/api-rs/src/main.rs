mod antenna;
mod coverage;
mod elevation;
mod error;
mod fresnel;
mod itm_bridge;
mod models;
pub mod rounding;
mod routes;
mod signal_levels;
mod terrain;

use std::env;
use std::sync::Arc;

use axum::extract::DefaultBodyLimit;
use axum::{routing::get, routing::post, Router};
use tokio::sync::Semaphore;
use tower_http::cors::CorsLayer;

mod health {
    use axum::Json;
    use serde_json::{json, Value};

    pub async fn health_handler() -> Json<Value> {
        Json(json!({ "status": "ok" }))
    }
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let origins_env = env::var("CORS_ORIGINS")
        .ok()
        .or_else(|| env::var("DEV_ORIGINS").ok())
        .unwrap_or_else(|| "http://localhost:3000,http://localhost:3001".into());
    let origins: Vec<_> = origins_env
        .split(',')
        .filter_map(|s| {
            let parsed: Result<axum::http::HeaderValue, _> = s.trim().parse();
            match parsed {
                Ok(v) => Some(v),
                Err(_) => {
                    tracing::warn!("Invalid CORS origin ignored: {:?}", s.trim());
                    None
                }
            }
        })
        .collect();

    let cors = CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([axum::http::header::CONTENT_TYPE]);

    let semaphore = Arc::new(Semaphore::new(4));

    let app = Router::new()
        .route("/api/health", get(health::health_handler))
        .route("/api/p2p", post(routes::p2p::p2p_handler))
        .route("/api/coverage", post(routes::coverage::coverage_handler))
        .route(
            "/api/coverage-radius",
            post(routes::coverage_radius::coverage_radius_handler),
        )
        .layer(cors)
        .layer(DefaultBodyLimit::max(2 * 1024 * 1024))
        .with_state(semaphore);

    let addr = "0.0.0.0:8000";
    let listener = match tokio::net::TcpListener::bind(addr).await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("Failed to bind to {addr}: {e}");
            std::process::exit(1);
        }
    };
    let local_addr = listener.local_addr().expect("bound socket has local addr");
    tracing::info!("Server listening on {local_addr}");
    axum::serve(listener, app).await.unwrap();
}
