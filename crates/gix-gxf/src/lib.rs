//! GXF (GIX Exchange Format) ABI v3
//!
//! This crate defines the schema, validators, and serialization for GXF,
//! the standardized format for job execution envelopes in the GIX system.

use gix_common::JobId;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;

/// GXF schema version constant
pub const GXF_VERSION: u8 = 3;

/// GXF-specific error types
#[derive(Error, Debug, Clone, PartialEq, Eq)]
pub enum GxfError {
    #[error("Invalid schema version: expected {expected}, got {actual}")]
    InvalidVersion { expected: u8, actual: u8 },
    #[error("Invalid job ID: {0}")]
    InvalidJobId(String),
    #[error("Invalid payload: {0}")]
    InvalidPayload(String),
    #[error("Invalid metadata: {0}")]
    InvalidMetadata(String),
    #[error("Envelope expired at timestamp {expires_at}, current time {current_time}")]
    Expired { expires_at: u64, current_time: u64 },
    #[error("Invalid precision level")]
    InvalidPrecision,
    #[error("Invalid sequence length: must be > 0, got {0}")]
    InvalidSequenceLength(u32),
    #[error("Serialization error: {0}")]
    Serialization(String),
    #[error("Deserialization error: {0}")]
    Deserialization(String),
}

/// Precision levels for compute operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum PrecisionLevel {
    /// Brain Float 16
    BF16,
    /// Float 8
    FP8,
    /// E5M2 format
    E5M2,
    /// Integer 8
    INT8,
}

impl PrecisionLevel {
    /// Validate that the precision level is supported
    pub fn is_valid(&self) -> bool {
        matches!(self, PrecisionLevel::BF16 | PrecisionLevel::FP8 | PrecisionLevel::E5M2 | PrecisionLevel::INT8)
    }
}

/// Job priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum JobPriority {
    /// Low priority (0-63)
    Low = 0,
    /// Normal priority (64-127)
    Normal = 64,
    /// High priority (128-191)
    High = 128,
    /// Critical priority (192-255)
    Critical = 192,
}

impl JobPriority {
    /// Create from u8 value
    pub fn from_u8(value: u8) -> Self {
        match value {
            0..=63 => JobPriority::Low,
            64..=127 => JobPriority::Normal,
            128..=191 => JobPriority::High,
            192..=255 => JobPriority::Critical,
        }
    }

    /// Get u8 value
    pub fn as_u8(&self) -> u8 {
        *self as u8
    }
}

/// GXF Job structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GxfJob {
    /// Job identifier
    pub job_id: JobId,
    /// Precision level for computation
    pub precision: PrecisionLevel,
    /// KV cache sequence length
    pub kv_cache_seq_len: u32,
    /// Additional job parameters (key-value pairs)
    #[serde(default)]
    pub parameters: std::collections::HashMap<String, String>,
}

impl GxfJob {
    /// Create a new GXF job
    pub fn new(job_id: JobId, precision: PrecisionLevel, kv_cache_seq_len: u32) -> Self {
        GxfJob {
            job_id,
            precision,
            kv_cache_seq_len,
            parameters: std::collections::HashMap::new(),
        }
    }

    /// Validate the job structure
    pub fn validate(&self) -> Result<(), GxfError> {
        if !self.precision.is_valid() {
            return Err(GxfError::InvalidPrecision);
        }

        if self.kv_cache_seq_len == 0 {
            return Err(GxfError::InvalidSequenceLength(self.kv_cache_seq_len));
        }

        Ok(())
    }
}

/// GXF Metadata structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GxfMetadata {
    /// Schema version
    pub schema_version: u8,
    /// Job priority (0-255)
    pub priority: u8,
    /// Creation timestamp (Unix epoch in seconds)
    pub created_at: u64,
    /// Expiration timestamp (Unix epoch in seconds, None if no expiration)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<u64>,
    /// Source SLP identifier (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_slp: Option<String>,
    /// Target lane identifier (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_lane: Option<String>,
    /// Additional metadata fields
    #[serde(default)]
    pub additional_fields: std::collections::HashMap<String, String>,
}

impl GxfMetadata {
    /// Create new metadata with current timestamp
    pub fn new(priority: u8) -> Result<Self, GxfError> {
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| GxfError::InvalidMetadata(format!("Failed to get timestamp: {}", e)))?
            .as_secs();

        Ok(GxfMetadata {
            schema_version: GXF_VERSION,
            priority,
            created_at,
            expires_at: None,
            source_slp: None,
            target_lane: None,
            additional_fields: std::collections::HashMap::new(),
        })
    }

    /// Validate metadata structure
    pub fn validate(&self) -> Result<(), GxfError> {
        // Check schema version
        if self.schema_version != GXF_VERSION {
            return Err(GxfError::InvalidVersion {
                expected: GXF_VERSION,
                actual: self.schema_version,
            });
        }

        // Check expiration
        if let Some(expires_at) = self.expires_at {
            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| GxfError::InvalidMetadata(format!("Failed to get timestamp: {}", e)))?
                .as_secs();

            if expires_at <= current_time {
                return Err(GxfError::Expired {
                    expires_at,
                    current_time,
                });
            }

            // Expiration must be after creation
            if expires_at <= self.created_at {
                return Err(GxfError::InvalidMetadata(
                    "Expiration time must be after creation time".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Check if metadata is expired
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .ok()
                .map(|d| d.as_secs())
                .unwrap_or(0);

            expires_at <= current_time
        } else {
            false
        }
    }
}

/// GXF Envelope structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GxfEnvelope {
    /// Metadata
    pub meta: GxfMetadata,
    /// Encrypted payload (contains serialized GxfJob)
    pub payload: Vec<u8>,
}

impl GxfEnvelope {
    /// Create a new GXF envelope
    pub fn new(meta: GxfMetadata, payload: Vec<u8>) -> Self {
        GxfEnvelope { meta, payload }
    }

    /// Create envelope from job
    pub fn from_job(job: GxfJob, priority: u8) -> Result<Self, GxfError> {
        // Validate job first
        job.validate()?;

        // Create metadata
        let meta = GxfMetadata::new(priority)?;

        // Serialize job to payload
        let payload = serde_json::to_vec(&job)
            .map_err(|e| GxfError::Serialization(format!("Failed to serialize job: {}", e)))?;

        Ok(GxfEnvelope::new(meta, payload))
    }

    /// Deserialize job from payload
    pub fn deserialize_job(&self) -> Result<GxfJob, GxfError> {
        serde_json::from_slice(&self.payload)
            .map_err(|e| GxfError::Deserialization(format!("Failed to deserialize job: {}", e)))
    }

    /// Validate the entire envelope
    pub fn validate(&self) -> Result<(), GxfError> {
        // Validate metadata
        self.meta.validate()?;

        // Check payload is not empty
        if self.payload.is_empty() {
            return Err(GxfError::InvalidPayload("Payload cannot be empty".to_string()));
        }

        // Try to deserialize and validate job
        let job = self.deserialize_job()?;
        job.validate()?;

        Ok(())
    }

    /// Serialize envelope to JSON bytes
    pub fn to_json(&self) -> Result<Vec<u8>, GxfError> {
        serde_json::to_vec(self)
            .map_err(|e| GxfError::Serialization(format!("Failed to serialize envelope: {}", e)))
    }

    /// Deserialize envelope from JSON bytes
    pub fn from_json(data: &[u8]) -> Result<Self, GxfError> {
        serde_json::from_slice(data)
            .map_err(|e| GxfError::Deserialization(format!("Failed to deserialize envelope: {}", e)))
    }
}

/// Validate a GXF job
pub fn validate_job(job: &GxfJob) -> Result<(), GxfError> {
    job.validate()
}

/// Validate a GXF envelope
pub fn validate_envelope(envelope: &GxfEnvelope) -> Result<(), GxfError> {
    envelope.validate()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_precision_level_validation() {
        assert!(PrecisionLevel::BF16.is_valid());
        assert!(PrecisionLevel::FP8.is_valid());
        assert!(PrecisionLevel::E5M2.is_valid());
        assert!(PrecisionLevel::INT8.is_valid());
    }

    #[test]
    fn test_job_priority() {
        assert_eq!(JobPriority::from_u8(0), JobPriority::Low);
        assert_eq!(JobPriority::from_u8(64), JobPriority::Normal);
        assert_eq!(JobPriority::from_u8(128), JobPriority::High);
        assert_eq!(JobPriority::from_u8(192), JobPriority::Critical);
    }

    #[test]
    fn test_gxf_job_creation() {
        let job_id = JobId([0u8; 16]);
        let job = GxfJob::new(job_id, PrecisionLevel::BF16, 1024);
        assert_eq!(job.job_id, job_id);
        assert_eq!(job.precision, PrecisionLevel::BF16);
        assert_eq!(job.kv_cache_seq_len, 1024);
    }

    #[test]
    fn test_gxf_job_validation() {
        let job_id = JobId([0u8; 16]);
        let job = GxfJob::new(job_id, PrecisionLevel::BF16, 1024);
        assert!(job.validate().is_ok());

        // Invalid: zero sequence length
        let invalid_job = GxfJob::new(job_id, PrecisionLevel::BF16, 0);
        assert!(invalid_job.validate().is_err());
    }

    #[test]
    fn test_gxf_metadata_creation() {
        let meta = GxfMetadata::new(64).unwrap();
        assert_eq!(meta.schema_version, GXF_VERSION);
        assert_eq!(meta.priority, 64);
        assert!(meta.created_at > 0);
        assert!(meta.expires_at.is_none());
    }

    #[test]
    fn test_gxf_metadata_validation() {
        let meta = GxfMetadata::new(64).unwrap();
        assert!(meta.validate().is_ok());

        // Invalid: wrong schema version
        let mut invalid_meta = meta.clone();
        invalid_meta.schema_version = 99;
        assert!(invalid_meta.validate().is_err());
    }

    #[test]
    fn test_gxf_metadata_expiration() {
        let mut meta = GxfMetadata::new(64).unwrap();
        
        // Set expiration in the future
        let future_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() + 3600;
        meta.expires_at = Some(future_time);
        assert!(!meta.is_expired());
        assert!(meta.validate().is_ok());

        // Set expiration in the past
        let past_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() - 3600;
        meta.expires_at = Some(past_time);
        assert!(meta.is_expired());
        assert!(meta.validate().is_err());
    }

    #[test]
    fn test_gxf_envelope_creation() {
        let job_id = JobId([0u8; 16]);
        let job = GxfJob::new(job_id, PrecisionLevel::BF16, 1024);
        let envelope = GxfEnvelope::from_job(job, 64).unwrap();
        
        assert_eq!(envelope.meta.schema_version, GXF_VERSION);
        assert!(!envelope.payload.is_empty());
    }

    #[test]
    fn test_gxf_envelope_validation() {
        let job_id = JobId([0u8; 16]);
        let job = GxfJob::new(job_id, PrecisionLevel::BF16, 1024);
        let envelope = GxfEnvelope::from_job(job, 64).unwrap();
        assert!(envelope.validate().is_ok());

        // Invalid: empty payload
        let mut invalid_envelope = envelope.clone();
        invalid_envelope.payload = Vec::new();
        assert!(invalid_envelope.validate().is_err());
    }

    #[test]
    fn test_gxf_envelope_serialization() {
        let job_id = JobId([0u8; 16]);
        let job = GxfJob::new(job_id, PrecisionLevel::BF16, 1024);
        let envelope = GxfEnvelope::from_job(job, 64).unwrap();

        // Serialize
        let json_bytes = envelope.to_json().unwrap();
        assert!(!json_bytes.is_empty());

        // Deserialize
        let deserialized = GxfEnvelope::from_json(&json_bytes).unwrap();
        assert_eq!(deserialized.meta.schema_version, envelope.meta.schema_version);
        assert_eq!(deserialized.payload, envelope.payload);
    }

    #[test]
    fn test_gxf_envelope_job_roundtrip() {
        let job_id = JobId([1u8; 16]);
        let mut job = GxfJob::new(job_id, PrecisionLevel::FP8, 2048);
        job.parameters.insert("key".to_string(), "value".to_string());

        let envelope = GxfEnvelope::from_job(job.clone(), 128).unwrap();
        let deserialized_job = envelope.deserialize_job().unwrap();

        assert_eq!(deserialized_job.job_id, job.job_id);
        assert_eq!(deserialized_job.precision, job.precision);
        assert_eq!(deserialized_job.kv_cache_seq_len, job.kv_cache_seq_len);
        assert_eq!(deserialized_job.parameters, job.parameters);
    }
}
