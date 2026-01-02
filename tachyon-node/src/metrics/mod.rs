use anyhow::Result;
use tracing::info;

pub async fn start_metrics_server(
    port: u16,
    mut shutdown: tokio::sync::broadcast::Receiver<()>,
) -> Result<()> {
    info!("📊 Metrics server integrated with API on port {}", port);
    
    // Metrics are now served via the API server's /metrics endpoint
    // This function just waits for shutdown
    shutdown.recv().await.ok();
    
    Ok(())
}

