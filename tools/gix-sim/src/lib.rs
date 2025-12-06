//! GIX Localnet Simulator
//!
//! Simulates the complete GIX workflow:
//! - Job submission → AJR routing → GCAM auction → GSEE execution
//!
//! Uses gRPC clients to communicate with the service daemons.

use anyhow::Result;
use gix_common::JobId;
use gix_crypto::hash_blake3;
use gix_gxf::{GxfEnvelope, GxfJob, PrecisionLevel};
use gix_proto::v1::{ExecuteJobRequest, GetAuctionStatsRequest, GetRouterStatsRequest, GetRuntimeStatsRequest, RouteEnvelopeRequest, RunAuctionRequest};
use gix_proto::{AuctionServiceClient, ExecutionServiceClient, RouterServiceClient};
use rand::Rng;
use tonic::Request;

const AJR_SERVER_ADDR: &str = "http://127.0.0.1:50051";
const GCAM_SERVER_ADDR: &str = "http://127.0.0.1:50052";
const GSEE_SERVER_ADDR: &str = "http://127.0.0.1:50053";

/// Main simulation state
pub struct Simulation {
    pub router_client: RouterServiceClient<tonic::transport::Channel>,
    pub auction_client: AuctionServiceClient<tonic::transport::Channel>,
    pub runtime_client: ExecutionServiceClient<tonic::transport::Channel>,
    pub tick: u64,
    pub jobs_processed: u64,
}

impl Simulation {
    /// Create a new simulation with gRPC clients
    pub async fn new() -> Result<Self> {
        // Connect to service daemons
        let router_client = RouterServiceClient::connect(AJR_SERVER_ADDR)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to AJR router: {}", e))?;
        
        let auction_client = AuctionServiceClient::connect(GCAM_SERVER_ADDR)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to GCAM node: {}", e))?;
        
        let runtime_client = ExecutionServiceClient::connect(GSEE_SERVER_ADDR)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to connect to GSEE runtime: {}", e))?;

        Ok(Simulation {
            router_client,
            auction_client,
            runtime_client,
            tick: 0,
            jobs_processed: 0,
        })
    }

    /// Generate a random JobId using crypto hashing
    fn generate_job_id() -> JobId {
        let mut rng = rand::thread_rng();
        let random_bytes: [u8; 16] = rng.gen();
        let hash = hash_blake3(&random_bytes);
        let mut job_id_bytes = [0u8; 16];
        job_id_bytes.copy_from_slice(&hash[..16]);
        JobId(job_id_bytes)
    }

    /// Create a random test job
    fn create_test_job() -> GxfJob {
        let job_id = Self::generate_job_id();
        let precisions = vec![
            PrecisionLevel::BF16,
            PrecisionLevel::FP8,
            PrecisionLevel::E5M2,
            PrecisionLevel::INT8,
        ];
        let precision = precisions[rand::thread_rng().gen_range(0..precisions.len())];
        let seq_len = rand::thread_rng().gen_range(512..4096);
        
        let mut job = GxfJob::new(job_id, precision, seq_len);
        
        if rand::thread_rng().gen_bool(0.5) {
            job.parameters.insert("batch_size".to_string(), format!("{}", rand::thread_rng().gen_range(1..32)));
        }
        if rand::thread_rng().gen_bool(0.5) {
            let regions = vec!["US", "EU"];
            job.parameters.insert("region".to_string(), regions[rand::thread_rng().gen_range(0..regions.len())].to_string());
        }
        
        job
    }

    /// Run one simulation tick
    pub async fn run_tick(&mut self) -> Result<()> {
        self.tick += 1;

        let job = Self::create_test_job();
        let priority = rand::thread_rng().gen_range(32..192);
        let envelope = GxfEnvelope::from_job(job.clone(), priority)?;

        // Serialize envelope and job for gRPC calls
        let envelope_bytes = envelope.to_json()
            .map_err(|e| anyhow::anyhow!("Failed to serialize envelope: {}", e))?;
        
        let job_bytes = serde_json::to_vec(&job)
            .map_err(|e| anyhow::anyhow!("Failed to serialize job: {}", e))?;

        // Step 2: Route through AJR via gRPC
        let route_request = Request::new(RouteEnvelopeRequest {
            envelope: envelope_bytes.clone(),
        });
        
        let route_response = self.router_client
            .route_envelope(route_request)
            .await
            .map_err(|e| anyhow::anyhow!("AJR routing failed: {}", e))?;
        
        let route_resp = route_response.into_inner();
        if !route_resp.success {
            return Err(anyhow::anyhow!("AJR routing failed: {}", route_resp.error));
        }

        // Step 3: Run GCAM auction via gRPC
        let auction_request = Request::new(RunAuctionRequest {
            job: job_bytes,
            priority: priority as u32,
        });
        
        let auction_response = self.auction_client
            .run_auction(auction_request)
            .await
            .map_err(|e| anyhow::anyhow!("GCAM auction failed: {}", e))?;
        
        let auction_resp = auction_response.into_inner();
        if !auction_resp.success {
            return Err(anyhow::anyhow!("GCAM auction failed: {}", auction_resp.error));
        }

        // Step 4: Execute in GSEE runtime via gRPC
        let execute_request = Request::new(ExecuteJobRequest {
            envelope: envelope_bytes,
        });
        
        let execute_response = self.runtime_client
            .execute_job(execute_request)
            .await
            .map_err(|e| anyhow::anyhow!("GSEE execution failed: {}", e))?;
        
        let execute_resp = execute_response.into_inner();
        if !execute_resp.success {
            return Err(anyhow::anyhow!("GSEE execution failed: {}", execute_resp.error));
        }

        self.jobs_processed += 1;
        Ok(())
    }

    /// Get current simulation status
    pub async fn status(&mut self) -> String {
        // Get stats from services via gRPC
        let router_stats = self.router_client
            .get_router_stats(Request::new(GetRouterStatsRequest {}))
            .await
            .ok()
            .map(|r| r.into_inner())
            .unwrap_or_default();
        
        let auction_stats = self.auction_client
            .get_auction_stats(Request::new(GetAuctionStatsRequest {}))
            .await
            .ok()
            .map(|r| r.into_inner())
            .unwrap_or_default();
        
        let runtime_stats = self.runtime_client
            .get_runtime_stats(Request::new(GetRuntimeStatsRequest {}))
            .await
            .ok()
            .map(|r| r.into_inner())
            .unwrap_or_default();

        format!(
            "Tick {}: Processed {} jobs | Router: {} routed | Auction: {} matches (volume: {}) | Runtime: {} executed ({} completed, {} rejected)",
            self.tick,
            self.jobs_processed,
            router_stats.total_routed,
            auction_stats.total_matches,
            auction_stats.total_volume,
            runtime_stats.total_executed,
            runtime_stats.total_completed,
            runtime_stats.total_rejected
        )
    }
}

impl Default for Simulation {
    fn default() -> Self {
        panic!("Simulation::default() cannot be used. Use Simulation::new().await instead.")
    }
}
