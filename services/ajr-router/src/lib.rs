//! AJR Router Library
//!
//! Provides router state and envelope processing functionality.

use anyhow::Result;
use gix_common::{GixError, LaneId};
use gix_gxf::{GxfEnvelope, GxfJob};
use metrics::{counter, gauge, increment_counter};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// AJR Router state
#[derive(Clone)]
pub struct RouterState {
    /// Active routing lanes
    lanes: Vec<LaneInfo>,
    /// Statistics: jobs routed per lane
    stats: Arc<RwLock<HashMap<LaneId, u64>>>,
    /// Total jobs routed
    total_routed: Arc<RwLock<u64>>,
}

/// Lane information
#[derive(Debug, Clone)]
struct LaneInfo {
    /// Lane identifier
    id: LaneId,
    /// Lane name (e.g., "Flash", "Deep")
    #[allow(dead_code)]
    name: String,
    /// Lane capacity (max concurrent jobs)
    capacity: u32,
    /// Current active jobs
    active_jobs: Arc<RwLock<u32>>,
}

/// Router statistics
#[derive(Debug, Clone)]
pub struct RouterStats {
    pub total_routed: u64,
    pub lane_stats: HashMap<LaneId, u64>,
}

impl RouterState {
    /// Create a new router state with default lanes
    pub fn new() -> Self {
        let lanes = vec![
            LaneInfo {
                id: LaneId(0),
                name: "Flash".to_string(),
                capacity: 100,
                active_jobs: Arc::new(RwLock::new(0)),
            },
            LaneInfo {
                id: LaneId(1),
                name: "Deep".to_string(),
                capacity: 50,
                active_jobs: Arc::new(RwLock::new(0)),
            },
        ];

        RouterState {
            lanes,
            stats: Arc::new(RwLock::new(HashMap::new())),
            total_routed: Arc::new(RwLock::new(0)),
        }
    }

    /// Select a lane for routing based on job priority and lane capacity
    async fn select_lane(&self, _job: &GxfJob, _priority: u8) -> Result<LaneId, GixError> {
        let lane_index = if _priority >= 128 {
            0 // Flash lane for high priority
        } else {
            1 // Deep lane for normal/low priority
        };

        if lane_index >= self.lanes.len() {
            return Err(GixError::InternalError("Invalid lane index".to_string()));
        }

        let lane = &self.lanes[lane_index];
        let active = *lane.active_jobs.read().await;

        if active >= lane.capacity {
            // Fallback to other lane if available
            let fallback_index = if lane_index == 0 { 1 } else { 0 };
            if fallback_index < self.lanes.len() {
                let fallback_lane = &self.lanes[fallback_index];
                let fallback_active = *fallback_lane.active_jobs.read().await;
                if fallback_active < fallback_lane.capacity {
                    return Ok(fallback_lane.id.clone());
                }
            }
            return Err(GixError::InternalError("All lanes at capacity".to_string()));
        }

        Ok(lane.id.clone())
    }

    /// Route an envelope through the selected lane
    async fn route_envelope(
        &self,
        _envelope: GxfEnvelope,
        lane_id: LaneId,
    ) -> Result<(), GixError> {
        // Record metrics
        let lane_id_str = format!("{}", lane_id.0);
        increment_counter!("gix_packets_routed_total", "lane" => lane_id_str.clone());
        
        {
            let mut stats = self.stats.write().await;
            *stats.entry(lane_id.clone()).or_insert(0) += 1;
        }

        {
            let mut total = self.total_routed.write().await;
            *total += 1;
            
            // Update total routed gauge
            gauge!("gix_router_total_routed", *total as f64);
        }

        if let Some(lane) = self.lanes.iter().find(|l| l.id == lane_id) {
            let mut active = lane.active_jobs.write().await;
            *active += 1;
            
            // Update active jobs gauge for this lane
            gauge!("gix_router_active_jobs", *active as f64, "lane" => lane_id_str);
        }

        Ok(())
    }

    /// Get routing statistics
    pub async fn get_stats(&self) -> RouterStats {
        let stats = self.stats.read().await;
        let total = *self.total_routed.read().await;

        RouterStats {
            total_routed: total,
            lane_stats: stats.clone(),
        }
    }
}

/// Process a GXF envelope through the router
pub async fn process_envelope(
    router: &RouterState,
    envelope: GxfEnvelope,
) -> Result<LaneId> {
    envelope.validate().map_err(|e| anyhow::anyhow!("Envelope validation failed: {}", e))?;

    if envelope.meta.is_expired() {
        return Err(anyhow::anyhow!("Envelope expired"));
    }

    let job = envelope
        .deserialize_job()
        .map_err(|e| anyhow::anyhow!("Failed to deserialize job: {}", e))?;

    job.validate()
        .map_err(|e| anyhow::anyhow!("Job validation failed: {}", e))?;

    let lane_id = router
        .select_lane(&job, envelope.meta.priority)
        .await
        .map_err(|e| anyhow::anyhow!("Lane selection failed: {}", e))?;

    router
        .route_envelope(envelope, lane_id.clone())
        .await
        .map_err(|e| anyhow::anyhow!("Routing failed: {}", e))?;

    Ok(lane_id)
}

