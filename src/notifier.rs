use tokio::sync::mpsc;
use tracing::info;

pub async fn start_notifier(mut result_rx: mpsc::Receiver<String>) {
    while let Some(result) = result_rx.recv().await {
        info!("Received result: {}", result);
    }
} 