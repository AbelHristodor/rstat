mod healthcheck;
mod service;
mod scheduler;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();
    
    
    let handle = tokio::spawn(async {
        let mut scheduler = scheduler::Scheduler::new();
        scheduler.start().await;
    });
    
    handle.await?;

    Ok(())
}
