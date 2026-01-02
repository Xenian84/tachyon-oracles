use solana_sdk::signer::Signer;
use anyhow::Result;
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::get,
    Router,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tracing::info;

use crate::config::NodeConfig;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStatus {
    pub node_pubkey: String,
    pub version: String,
    pub uptime_seconds: u64,
    pub price_updates_sent: u64,
    pub batches_created: u64,
    pub batches_submitted: u64,
    pub peers_connected: u32,
    pub is_leader: bool,
}

pub struct AppState {
    pub config: Arc<NodeConfig>,
    pub status: Arc<RwLock<NodeStatus>>,
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            config: Arc::clone(&self.config),
            status: Arc::clone(&self.status),
        }
    }
}

pub async fn start_api_server(
    config: Arc<NodeConfig>,
    mut shutdown: tokio::sync::broadcast::Receiver<()>,
) -> Result<()> {
    info!("🔌 Starting API server on port {}...", config.api_port);
    
    let api_port = config.api_port;
    let status = Arc::new(RwLock::new(NodeStatus {
        node_pubkey: config.identity.pubkey().to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime_seconds: 0,
        price_updates_sent: 0,
        batches_created: 0,
        batches_submitted: 0,
        peers_connected: 0,
        is_leader: false,
    }));
    
    let state = AppState {
        config,
        status,
    };
    
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/status", get(status_handler))
        .route("/health", get(health_handler))
        .route("/metrics", get(metrics_handler))
        .layer(CorsLayer::permissive())
        .with_state(state);
    
    let addr = format!("0.0.0.0:{}", api_port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    
    info!("✅ API server listening on http://{}", addr);
    
    axum::serve(listener, app)
        .with_graceful_shutdown(async move {
            shutdown.recv().await.ok();
        })
        .await?;
    
    info!("🔌 API server shut down");
    Ok(())
}

async fn root_handler() -> &'static str {
    "Tachyon Oracle Node API v1.0"
}

async fn status_handler(
    State(state): State<AppState>,
) -> Result<Json<NodeStatus>, StatusCode> {
    let status = state.status.read().await;
    Ok(Json(status.clone()))
}

async fn health_handler() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().timestamp()
    }))
}

async fn metrics_handler(
    State(state): State<AppState>,
) -> Result<String, StatusCode> {
    let status = state.status.read().await;
    
    // Prometheus format
    let metrics = format!(
        "# HELP tachyon_price_updates_total Total number of price updates sent\n\
         # TYPE tachyon_price_updates_total counter\n\
         tachyon_price_updates_total {}\n\
         \n\
         # HELP tachyon_batches_created_total Total number of Merkle batches created\n\
         # TYPE tachyon_batches_created_total counter\n\
         tachyon_batches_created_total {}\n\
         \n\
         # HELP tachyon_batches_submitted_total Total number of batches submitted to chain\n\
         # TYPE tachyon_batches_submitted_total counter\n\
         tachyon_batches_submitted_total {}\n\
         \n\
         # HELP tachyon_peers_connected Current number of connected peers\n\
         # TYPE tachyon_peers_connected gauge\n\
         tachyon_peers_connected {}\n\
         \n\
         # HELP tachyon_uptime_seconds Node uptime in seconds\n\
         # TYPE tachyon_uptime_seconds counter\n\
         tachyon_uptime_seconds {}\n",
        status.price_updates_sent,
        status.batches_created,
        status.batches_submitted,
        status.peers_connected,
        status.uptime_seconds,
    );
    
    Ok(metrics)
}

