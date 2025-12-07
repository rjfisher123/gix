//! GSEE (Secure Execution Envelope) Runtime Service
//!
//! Enclave execution runtime that securely executes jobs within encrypted
//! envelopes. Supports both simulation mode and production enclave mode.

use gsee_runtime::RuntimeState;
use anyhow::{Context, Result};
use gix_gxf::GxfEnvelope;
use gix_proto::v1::{ExecuteJobRequest, ExecuteJobResponse, ExecutionStatus as ProtoExecutionStatus, GetRuntimeStatsRequest, GetRuntimeStatsResponse, JobId as ProtoJobId};
use gix_proto::{ExecutionService, ExecutionServiceServer};
use std::sync::Arc;
use tonic::{Request, Response, Status};
use tracing::info;

const GSEE_SERVER_ADDR: &str = "0.0.0.0:50053";

/// Runtime service implementation
struct ExecutionServiceImpl {
    runtime: Arc<RuntimeState>,
}

#[tonic::async_trait]
impl ExecutionService for ExecutionServiceImpl {
    async fn execute_job(
        &self,
        request: Request<ExecuteJobRequest>,
    ) -> Result<Response<ExecuteJobResponse>, Status> {
        let req = request.into_inner();
        
        // Deserialize GXF envelope from bytes
        let envelope = GxfEnvelope::from_json(&req.envelope)
            .map_err(|e| Status::invalid_argument(format!("Invalid envelope: {}", e)))?;
        
        // Execute job
        let result = gsee_runtime::process_envelope(&self.runtime, envelope)
            .await
            .map_err(|e| Status::internal(format!("Execution failed: {}", e)))?;
        
        // Convert execution status
        let status = match result.status {
            gsee_runtime::ExecutionStatus::Completed => ProtoExecutionStatus::Completed,
            gsee_runtime::ExecutionStatus::Failed(_) => ProtoExecutionStatus::Failed,
            gsee_runtime::ExecutionStatus::Rejected(_) => ProtoExecutionStatus::Rejected,
        };
        
        Ok(Response::new(ExecuteJobResponse {
            job_id: Some(ProtoJobId { id: result.job_id.0.to_vec() }),
            status: status as i32,
            duration_ms: result.duration_ms,
            output_hash: result.output_hash.to_vec(),
            success: matches!(result.status, gsee_runtime::ExecutionStatus::Completed),
            error: String::new(),
        }))
    }

    async fn get_runtime_stats(
        &self,
        _request: Request<GetRuntimeStatsRequest>,
    ) -> Result<Response<GetRuntimeStatsResponse>, Status> {
        let stats = self.runtime.get_stats().await;
        
        let mut jobs_by_precision = std::collections::HashMap::new();
        for (precision, count) in stats.jobs_by_precision.iter() {
            jobs_by_precision.insert(format!("{:?}", precision), *count);
        }
        
        Ok(Response::new(GetRuntimeStatsResponse {
            total_executed: stats.total_executed,
            total_completed: stats.total_completed,
            total_failed: stats.total_failed,
            total_rejected: stats.total_rejected,
            jobs_by_precision,
        }))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "gsee_runtime=info".into()),
        )
        .init();

    info!("GSEE Runtime Service starting...");

    let runtime = Arc::new(RuntimeState::new());
    info!("Runtime initialized");

    // Create service implementation
    let service = ExecutionServiceImpl {
        runtime: runtime.clone(),
    };

    // Start gRPC server
    let addr = GSEE_SERVER_ADDR.parse()
        .context("Invalid server address")?;
    
    info!("Starting gRPC server on {}", addr);
    
    tonic::transport::Server::builder()
        .add_service(ExecutionServiceServer::new(service))
        .serve(addr)
        .await
        .context("Server error")?;

    Ok(())
}
