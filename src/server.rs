use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};

use std::time::SystemTime;

use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;

use crate::core::Service;

pub async fn start_server(port: u16, service: Arc<Service>) {
    tracing_subscriber::fmt::init();

    // Some details about the server starting up
    let state = ServerState {
        start_time: SystemTime::now(),
        service,
    };

    let api_router = Router::new()
        .route("/healthz", get(health_check))
        .route("/series", get(list_series));

    let app = Router::new().nest("/api/", api_router).with_state(state);

    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Clone)]
struct ServerState {
    start_time: SystemTime,
    service: Arc<Service>,
}

#[derive(Serialize, Deserialize, Debug)]
struct HealthCheckResponse {
    uptime: u64,
    git_tag: &'static str,
}

async fn health_check(State(state): State<ServerState>) -> impl IntoResponse {
    (
        StatusCode::OK,
        Json(HealthCheckResponse {
            uptime: SystemTime::now()
                .duration_since(state.start_time)
                .unwrap()
                .as_secs(),
            git_tag: crate::GIT_VERSION,
        }),
    )
}

// Lists all series configured and their latest status
async fn list_series<'a>(State(state): State<ServerState>) -> impl IntoResponse {}
