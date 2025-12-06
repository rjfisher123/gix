//! GSEE Runtime Library
//!
//! Provides runtime state and envelope processing functionality.

use anyhow::Result;
use gix_common::JobId;
use gix_crypto::hash_blake3;
use gix_gxf::{GxfEnvelope, GxfJob, PrecisionLevel};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Execution result
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    /// Job ID
    pub job_id: JobId,
    /// Execution status
    pub status: ExecutionStatus,
    /// Execution duration in milliseconds
    pub duration_ms: u64,
    /// Output data hash (simulated)
    pub output_hash: [u8; 32],
}

/// Execution status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionStatus {
    /// Job completed successfully
    Completed,
    /// Job failed during execution
    Failed(String),
    /// Job was rejected due to compliance violation
    Rejected(String),
}

/// Shape validation requirements
#[derive(Debug, Clone)]
pub struct ShapeRequirements {
    /// Maximum sequence length
    pub max_sequence_length: u32,
    /// Maximum batch size
    pub max_batch_size: u32,
    /// Required dimensions
    pub required_dimensions: Vec<u32>,
}

impl ShapeRequirements {
    /// Create default shape requirements
    pub fn default() -> Self {
        ShapeRequirements {
            max_sequence_length: 8192,
            max_batch_size: 32,
            required_dimensions: vec![],
        }
    }

    /// Validate shape against requirements
    pub fn validate(&self, job: &GxfJob) -> Result<(), ComplianceError> {
        if job.kv_cache_seq_len > self.max_sequence_length {
            return Err(ComplianceError::ShapeViolation(format!(
                "Sequence length {} exceeds maximum {}",
                job.kv_cache_seq_len, self.max_sequence_length
            )));
        }
        if let Some(batch_size_str) = job.parameters.get("batch_size") {
            if let Ok(batch_size) = batch_size_str.parse::<u32>() {
                if batch_size > self.max_batch_size {
                    return Err(ComplianceError::ShapeViolation(format!(
                        "Batch size {} exceeds maximum {}",
                        batch_size, self.max_batch_size
                    )));
                }
            }
        }
        Ok(())
    }
}

/// Residency requirements
#[derive(Debug, Clone)]
pub struct ResidencyRequirements {
    /// Allowed regions/countries
    pub allowed_regions: Vec<String>,
    /// Required data residency
    pub required_residency: Option<String>,
}

impl ResidencyRequirements {
    /// Create default residency requirements
    pub fn default() -> Self {
        ResidencyRequirements {
            allowed_regions: vec!["US".to_string(), "EU".to_string()],
            required_residency: None,
        }
    }

    /// Validate residency requirements
    pub fn validate(&self, job: &GxfJob) -> Result<(), ComplianceError> {
        if let Some(job_region) = job.parameters.get("region") {
            if !self.allowed_regions.contains(job_region) {
                return Err(ComplianceError::ResidencyViolation(format!(
                    "Region '{}' not in allowed regions: {:?}",
                    job_region, self.allowed_regions
                )));
            }
        }
        if let Some(required) = &self.required_residency {
            if let Some(job_residency) = job.parameters.get("residency") {
                if job_residency != required {
                    return Err(ComplianceError::ResidencyViolation(format!(
                        "Required residency '{}' but got '{}'",
                        required, job_residency
                    )));
                }
            } else {
                return Err(ComplianceError::ResidencyViolation(format!(
                    "Required residency '{}' not specified",
                    required
                )));
            }
        }
        Ok(())
    }
}

/// Compliance error types
#[derive(Debug, Clone, thiserror::Error)]
pub enum ComplianceError {
    #[error("Precision violation: {0}")]
    PrecisionViolation(String),
    #[error("Shape violation: {0}")]
    ShapeViolation(String),
    #[error("Residency violation: {0}")]
    ResidencyViolation(String),
}

/// GSEE Runtime state
#[derive(Clone)]
pub struct RuntimeState {
    /// Precision requirements
    supported_precisions: Vec<PrecisionLevel>,
    /// Shape requirements
    shape_requirements: ShapeRequirements,
    /// Residency requirements
    residency_requirements: ResidencyRequirements,
    /// Execution statistics
    stats: Arc<RwLock<ExecutionStats>>,
}

/// Execution statistics
#[derive(Debug, Clone, Default)]
pub struct ExecutionStats {
    /// Total jobs executed
    pub total_executed: u64,
    /// Total jobs completed successfully
    pub total_completed: u64,
    /// Total jobs failed
    pub total_failed: u64,
    /// Total jobs rejected
    pub total_rejected: u64,
    /// Jobs by precision level
    pub jobs_by_precision: HashMap<PrecisionLevel, u64>,
}

impl RuntimeState {
    /// Create new runtime state
    pub fn new() -> Self {
        RuntimeState {
            supported_precisions: vec![
                PrecisionLevel::BF16,
                PrecisionLevel::FP8,
                PrecisionLevel::E5M2,
                PrecisionLevel::INT8,
            ],
            shape_requirements: ShapeRequirements::default(),
            residency_requirements: ResidencyRequirements::default(),
            stats: Arc::new(RwLock::new(ExecutionStats::default())),
        }
    }

    fn check_precision(&self, job: &GxfJob) -> Result<(), ComplianceError> {
        if !self.supported_precisions.contains(&job.precision) {
            return Err(ComplianceError::PrecisionViolation(format!(
                "Precision {:?} not supported. Supported: {:?}",
                job.precision, self.supported_precisions
            )));
        }
        if !job.precision.is_valid() {
            return Err(ComplianceError::PrecisionViolation(format!(
                "Invalid precision level: {:?}",
                job.precision
            )));
        }
        Ok(())
    }

    fn check_shape(&self, job: &GxfJob) -> Result<(), ComplianceError> {
        self.shape_requirements.validate(job)
    }

    fn check_residency(&self, job: &GxfJob) -> Result<(), ComplianceError> {
        self.residency_requirements.validate(job)
    }

    fn check_compliance(&self, job: &GxfJob) -> Result<(), ComplianceError> {
        self.check_precision(job)?;
        self.check_shape(job)?;
        self.check_residency(job)?;
        Ok(())
    }

    async fn simulate_execution(&self, job: &GxfJob) -> ExecutionResult {
        let start_time = std::time::Instant::now();
        let duration_ms = (job.kv_cache_seq_len as f64 / 1000.0).ceil() as u64 + 10;
        tokio::time::sleep(tokio::time::Duration::from_millis(duration_ms)).await;
        let output_hash = hash_blake3(&job.job_id.0);
        let elapsed = start_time.elapsed().as_millis() as u64;
        ExecutionResult {
            job_id: job.job_id,
            status: ExecutionStatus::Completed,
            duration_ms: elapsed,
            output_hash,
        }
    }

    async fn execute_job(&self, job: GxfJob) -> Result<ExecutionResult, ComplianceError> {
        self.check_compliance(&job)?;
        {
            let mut stats = self.stats.write().await;
            stats.total_executed += 1;
            *stats.jobs_by_precision.entry(job.precision).or_insert(0) += 1;
        }
        let result = self.simulate_execution(&job).await;
        {
            let mut stats = self.stats.write().await;
            match result.status {
                ExecutionStatus::Completed => stats.total_completed += 1,
                ExecutionStatus::Failed(_) => stats.total_failed += 1,
                ExecutionStatus::Rejected(_) => stats.total_rejected += 1,
            }
        }
        Ok(result)
    }

    /// Get execution statistics
    pub async fn get_stats(&self) -> ExecutionStats {
        self.stats.read().await.clone()
    }
}

/// Process a GXF envelope through the runtime
pub async fn process_envelope(
    runtime: &RuntimeState,
    envelope: GxfEnvelope,
) -> Result<ExecutionResult> {
    envelope.validate().map_err(|e| anyhow::anyhow!("Envelope validation failed: {}", e))?;
    if envelope.meta.is_expired() {
        return Err(anyhow::anyhow!("Envelope expired"));
    }
    let job = envelope
        .deserialize_job()
        .map_err(|e| anyhow::anyhow!("Failed to deserialize job: {}", e))?;
    job.validate()
        .map_err(|e| anyhow::anyhow!("Job validation failed: {}", e))?;

    runtime
        .execute_job(job)
        .await
        .map_err(|e| anyhow::anyhow!("Compliance check failed: {}", e))
}

