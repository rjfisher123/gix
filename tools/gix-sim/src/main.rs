//! GIX Localnet Simulator
//!
//! Simulates the complete GIX workflow:
//! - Job submission → AJR routing → GCAM auction → GSEE execution

use anyhow::Result;
use gix_sim::Simulation;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "gix_sim=info".into()),
        )
        .init();

    info!("GIX Simulator Starting");
    info!("Connecting to services...");
    info!("  - AJR Router:      http://127.0.0.1:50051");
    info!("  - GCAM Node:       http://127.0.0.1:50052");
    info!("  - GSEE Runtime:    http://127.0.0.1:50053");
    info!("");

    let mut simulation = Simulation::new().await?;
    
    info!("Connected! Running 5 simulation ticks...\n");

    for i in 1..=5 {
        simulation.run_tick().await?;
        info!("[Tick {}] {}", i, simulation.status().await);
    }

    info!("\nSimulation complete!");
    Ok(())
}
