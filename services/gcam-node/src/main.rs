//! GCAM (Global Compute Auction Mechanism) Node Service
//!
//! Clearing engine and bridge services for the global compute auction.
//! Handles job matching, pricing, and route selection with persistent storage.

use gcam_node::AuctionEngine;
use anyhow::{Context, Result};
use gix_gxf::GxfJob;
use gix_proto::v1::{GetAuctionStatsRequest, GetAuctionStatsResponse, JobId as ProtoJobId, LaneId as ProtoLaneId, RunAuctionRequest, RunAuctionResponse, SlpId as ProtoSlpId};
use gix_proto::{AuctionService, AuctionServiceServer};
use metrics_exporter_prometheus::PrometheusBuilder;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::signal;
use tonic::{Request, Response, Status};
use tracing::info;

const GCAM_SERVER_ADDR: &str = "0.0.0.0:50052";
const METRICS_ADDR: &str = "0.0.0.0:9002";
const DB_PATH: &str = "./data/gcam_db";

/// Auction service implementation
struct AuctionServiceImpl {
    engine: Arc<AuctionEngine>,
}

#[tonic::async_trait]
impl AuctionService for AuctionServiceImpl {
    async fn run_auction(
        &self,
        request: Request<RunAuctionRequest>,
    ) -> Result<Response<RunAuctionResponse>, Status> {
        let req = request.into_inner();
        
        // Deserialize GXF job from bytes
        let job: GxfJob = serde_json::from_slice(&req.job)
            .map_err(|e| Status::invalid_argument(format!("Invalid job: {}", e)))?;
        
        // Run auction
        let match_result = self.engine
            .run_auction(&job, req.priority as u8)
            .await
            .map_err(|e| Status::internal(format!("Auction failed: {}", e)))?;
        
        Ok(Response::new(RunAuctionResponse {
            job_id: Some(ProtoJobId { id: match_result.job_id.0.to_vec() }),
            slp_id: Some(ProtoSlpId { id: match_result.slp_id.0 }),
            lane_id: Some(ProtoLaneId { id: match_result.lane_id.0 as u32 }),
            price: match_result.price,
            route: match_result.route,
            success: true,
            error: String::new(),
        }))
    }

    async fn get_auction_stats(
        &self,
        _request: Request<GetAuctionStatsRequest>,
    ) -> Result<Response<GetAuctionStatsResponse>, Status> {
        let stats = self.engine.get_stats().await;
        
        let mut matches_by_precision = std::collections::HashMap::new();
        for (precision, count) in stats.matches_by_precision.iter() {
            matches_by_precision.insert(format!("{:?}", precision), *count);
        }
        
        let mut matches_by_lane = std::collections::HashMap::new();
        for (lane_id, count) in stats.matches_by_lane.iter() {
            matches_by_lane.insert(lane_id.0 as u32, *count);
        }
        
        Ok(Response::new(GetAuctionStatsResponse {
            total_auctions: stats.total_auctions,
            total_matches: stats.total_matches,
            total_volume: stats.total_volume,
            matches_by_precision,
            matches_by_lane,
        }))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "gcam_node=info".into()),
        )
        .init();

    info!("GCAM Node Service starting...");
    
    // Initialize Prometheus metrics exporter
    let metrics_addr: SocketAddr = METRICS_ADDR.parse()
        .context("Invalid metrics address")?;
    
    info!("Starting Prometheus metrics endpoint on {}", metrics_addr);
    
    PrometheusBuilder::new()
        .with_http_listener(metrics_addr)
        .install()
        .context("Failed to install Prometheus recorder")?;
    
    // Ensure data directory exists
    std::fs::create_dir_all("./data")
        .context("Failed to create data directory")?;

    // Initialize auction engine with persistent storage
    info!("Opening database at {}", DB_PATH);
    let engine = Arc::new(
        AuctionEngine::new(DB_PATH)
            .context("Failed to initialize auction engine with database")?
    );
    info!("Auction engine initialized with persistent storage");

    // Create service implementation
    let service = AuctionServiceImpl {
        engine: engine.clone(),
    };

    // Parse server address
    let addr = GCAM_SERVER_ADDR.parse()
        .context("Invalid server address")?;
    
    info!("Starting gRPC server on {}", addr);
    
    // Create server with graceful shutdown
    let server = tonic::transport::Server::builder()
        .add_service(AuctionServiceServer::new(service))
        .serve_with_shutdown(addr, shutdown_signal(engine.clone()));
    
    // Run server
    server.await.context("Server error")?;
    
    info!("GCAM Node Service stopped");
    Ok(())
}

/// Wait for shutdown signal and flush database
async fn shutdown_signal(engine: Arc<AuctionEngine>) {
    // Wait for CTRL+C
    signal::ctrl_c()
        .await
        .expect("Failed to install CTRL+C signal handler");
    
    info!("Shutdown signal received, flushing database...");
    
    // Flush database to ensure all data is persisted
    if let Err(e) = engine.flush().await {
        eprintln!("Error flushing database: {}", e);
    } else {
        info!("Database flushed successfully");
    }
}
