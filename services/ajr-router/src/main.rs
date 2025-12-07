//! AJR (Anonymized Job Routing) Router Service
//!
//! Mixnet service that routes jobs through anonymized lanes to prevent
//! correlation between job submission and execution.

use ajr_router::RouterState;
use anyhow::{Context, Result};
use gix_gxf::GxfEnvelope;
use gix_proto::v1::{GetRouterStatsRequest, GetRouterStatsResponse, LaneId as ProtoLaneId, RouteEnvelopeRequest, RouteEnvelopeResponse};
use gix_proto::{RouterService, RouterServiceServer};
use metrics_exporter_prometheus::PrometheusBuilder;
use std::net::SocketAddr;
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::info;

const AJR_SERVER_ADDR: &str = "0.0.0.0:50051";
const METRICS_ADDR: &str = "0.0.0.0:9001";

/// Router service implementation
struct RouterServiceImpl {
    router: Arc<RouterState>,
}

#[tonic::async_trait]
impl RouterService for RouterServiceImpl {
    async fn route_envelope(
        &self,
        request: Request<RouteEnvelopeRequest>,
    ) -> Result<Response<RouteEnvelopeResponse>, Status> {
        let req = request.into_inner();
        
        // Deserialize GXF envelope from bytes
        let envelope = GxfEnvelope::from_json(&req.envelope)
            .map_err(|e| Status::invalid_argument(format!("Invalid envelope: {}", e)))?;
        
        // Process through router
        let lane_id = ajr_router::process_envelope(&self.router, envelope)
            .await
            .map_err(|e| Status::internal(format!("Routing failed: {}", e)))?;
        
        Ok(Response::new(RouteEnvelopeResponse {
            lane_id: Some(ProtoLaneId { id: lane_id.0 as u32 }),
            success: true,
            error: String::new(),
        }))
    }

    async fn get_router_stats(
        &self,
        _request: Request<GetRouterStatsRequest>,
    ) -> Result<Response<GetRouterStatsResponse>, Status> {
        let stats = self.router.get_stats().await;
        
        let mut lane_stats = std::collections::HashMap::new();
        for (lane_id, count) in stats.lane_stats.iter() {
            lane_stats.insert(lane_id.0 as u32, *count);
        }
        
        Ok(Response::new(GetRouterStatsResponse {
            total_routed: stats.total_routed,
            lane_stats,
        }))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ajr_router=info".into()),
        )
        .init();

    info!("AJR Router Service starting...");

    // Initialize Prometheus metrics exporter
    let metrics_addr: SocketAddr = METRICS_ADDR.parse()
        .context("Invalid metrics address")?;
    
    info!("Starting Prometheus metrics endpoint on {}", metrics_addr);
    
    PrometheusBuilder::new()
        .with_http_listener(metrics_addr)
        .install()
        .context("Failed to install Prometheus recorder")?;

    // Initialize router state
    let router = Arc::new(RouterState::new());
    info!("Router initialized");

    // Create service implementation
    let service = RouterServiceImpl {
        router: router.clone(),
    };

    // Start gRPC server
    let addr = AJR_SERVER_ADDR.parse()
        .context("Invalid server address")?;
    
    info!("Starting gRPC server on {}", addr);
    
    tonic::transport::Server::builder()
        .add_service(RouterServiceServer::new(service))
        .serve(addr)
        .await
        .context("Server error")?;

    Ok(())
}
