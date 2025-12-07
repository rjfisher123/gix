//! GCAM Auction Engine Library with Persistent Storage
//!
//! Provides auction engine state with persistence using the sled embedded database.

use anyhow::Result;
use gix_common::{GixError, JobId, LaneId, SlpId};
use gix_gxf::{GxfEnvelope, GxfJob, PrecisionLevel};
use metrics::{counter, gauge};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Price in micro-tokens (smallest unit)
pub type Price = u64;

/// Auction match result
#[derive(Debug, Clone)]
pub struct AuctionMatch {
    /// Job ID
    pub job_id: JobId,
    /// Matched SLP ID
    pub slp_id: SlpId,
    /// Selected lane ID
    pub lane_id: LaneId,
    /// Calculated price
    pub price: Price,
    /// Route path (sequence of nodes)
    pub route: Vec<String>,
}

/// Compute resource provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeProvider {
    /// Provider identifier (SLP ID)
    pub slp_id: SlpId,
    /// Available precision levels
    pub supported_precisions: Vec<PrecisionLevel>,
    /// Base price per unit (micro-tokens)
    pub base_price: Price,
    /// Available capacity
    pub capacity: u32,
    /// Current utilization
    pub utilization: u32,
    /// Region/location
    pub region: String,
}

impl ComputeProvider {
    /// Check if provider can handle a job
    pub fn can_handle(&self, job: &GxfJob) -> bool {
        if !self.supported_precisions.contains(&job.precision) {
            return false;
        }
        if self.utilization >= self.capacity {
            return false;
        }
        true
    }

    /// Calculate price for a job
    pub fn calculate_price(&self, job: &GxfJob) -> Price {
        let mut price = self.base_price;
        price += (job.kv_cache_seq_len as u64) * 10;
        let precision_multiplier = match job.precision {
            PrecisionLevel::INT8 => 1.0,
            PrecisionLevel::E5M2 => 1.2,
            PrecisionLevel::FP8 => 1.5,
            PrecisionLevel::BF16 => 2.0,
        };
        price = (price as f64 * precision_multiplier) as u64;
        let utilization_factor = 1.0 + (self.utilization as f64 / self.capacity as f64) * 0.5;
        price = (price as f64 * utilization_factor) as u64;
        price
    }
}

/// Route information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Route {
    /// Route identifier
    pub id: String,
    /// Lane ID
    pub lane_id: LaneId,
    /// Route path (sequence of node IDs)
    pub path: Vec<String>,
    /// Route latency estimate (ms)
    pub latency_ms: u64,
    /// Route cost
    pub cost: Price,
}

impl Route {
    /// Calculate route score (lower is better)
    pub fn score(&self) -> f64 {
        let latency_score = self.latency_ms as f64 / 1000.0;
        let cost_score = self.cost as f64 / 1000000.0;
        latency_score + cost_score
    }
}

/// Auction statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AuctionStats {
    /// Total auctions processed
    pub total_auctions: u64,
    /// Total matches found
    pub total_matches: u64,
    /// Total unmatched jobs
    pub total_unmatched: u64,
    /// Total volume (sum of all prices)
    pub total_volume: u64,
    /// Matches by precision
    pub matches_by_precision: HashMap<PrecisionLevel, u64>,
    /// Matches by lane
    pub matches_by_lane: HashMap<LaneId, u64>,
}

/// GCAM Auction Engine state with persistent storage
#[derive(Clone)]
pub struct AuctionEngine {
    /// Persistent database
    db: sled::Db,
    /// In-memory cache for providers (synced with DB)
    providers: Arc<RwLock<Vec<ComputeProvider>>>,
    /// In-memory cache for routes (synced with DB)
    routes: Arc<RwLock<Vec<Route>>>,
    /// In-memory stats (synced with DB)
    stats: Arc<RwLock<AuctionStats>>,
}

/// Helper function to open the database
pub fn open_db<P: AsRef<Path>>(path: P) -> Result<sled::Db> {
    let db = sled::open(path)?;
    Ok(db)
}

impl AuctionEngine {
    /// Create new auction engine with persistent storage
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let db = open_db(db_path)?;
        
        // Open/create specific trees
        let providers_tree = db.open_tree("providers")?;
        let routes_tree = db.open_tree("routes")?;
        let stats_tree = db.open_tree("stats")?;
        
        // Load providers from DB or initialize default
        let providers = Self::load_providers(&providers_tree)?;
        
        // Load routes from DB or initialize default
        let routes = Self::load_routes(&routes_tree)?;
        
        // Load stats from DB or initialize default
        let stats = Self::load_stats(&stats_tree)?;
        
        Ok(AuctionEngine {
            db,
            providers: Arc::new(RwLock::new(providers)),
            routes: Arc::new(RwLock::new(routes)),
            stats: Arc::new(RwLock::new(stats)),
        })
    }
    
    /// Load providers from database
    fn load_providers(tree: &sled::Tree) -> Result<Vec<ComputeProvider>> {
        let mut providers = Vec::new();
        
        for item in tree.iter() {
            let (_key, value) = item?;
            let provider: ComputeProvider = bincode::deserialize(&value)?;
            providers.push(provider);
        }
        
        // If no providers in DB, initialize with default providers
        if providers.is_empty() {
            providers = vec![
                ComputeProvider {
                    slp_id: SlpId("slp-us-east-1".to_string()),
                    supported_precisions: vec![
                        PrecisionLevel::BF16,
                        PrecisionLevel::FP8,
                        PrecisionLevel::E5M2,
                        PrecisionLevel::INT8,
                    ],
                    base_price: 1000,
                    capacity: 100,
                    utilization: 30,
                    region: "US".to_string(),
                },
                ComputeProvider {
                    slp_id: SlpId("slp-eu-west-1".to_string()),
                    supported_precisions: vec![
                        PrecisionLevel::BF16,
                        PrecisionLevel::FP8,
                        PrecisionLevel::INT8,
                    ],
                    base_price: 1200,
                    capacity: 80,
                    utilization: 20,
                    region: "EU".to_string(),
                },
            ];
            
            // Save default providers to DB
            for provider in &providers {
                let key = provider.slp_id.0.as_bytes();
                let value = bincode::serialize(provider)?;
                tree.insert(key, value)?;
            }
            tree.flush()?;
        }
        
        Ok(providers)
    }
    
    /// Load routes from database
    fn load_routes(tree: &sled::Tree) -> Result<Vec<Route>> {
        let mut routes = Vec::new();
        
        for item in tree.iter() {
            let (_key, value) = item?;
            let route: Route = bincode::deserialize(&value)?;
            routes.push(route);
        }
        
        // If no routes in DB, initialize with default routes
        if routes.is_empty() {
            routes = vec![
                Route {
                    id: "route-flash-1".to_string(),
                    lane_id: LaneId(0),
                    path: vec!["node-1".to_string(), "node-2".to_string()],
                    latency_ms: 50,
                    cost: 100,
                },
                Route {
                    id: "route-deep-1".to_string(),
                    lane_id: LaneId(1),
                    path: vec!["node-3".to_string(), "node-4".to_string(), "node-5".to_string()],
                    latency_ms: 150,
                    cost: 80,
                },
            ];
            
            // Save default routes to DB
            for route in &routes {
                let key = route.id.as_bytes();
                let value = bincode::serialize(route)?;
                tree.insert(key, value)?;
            }
            tree.flush()?;
        }
        
        Ok(routes)
    }
    
    /// Load statistics from database
    fn load_stats(tree: &sled::Tree) -> Result<AuctionStats> {
        if let Some(value) = tree.get("stats")? {
            let stats: AuctionStats = bincode::deserialize(&value)?;
            Ok(stats)
        } else {
            Ok(AuctionStats::default())
        }
    }
    
    /// Save providers to database
    async fn save_providers(&self) -> Result<()> {
        let tree = self.db.open_tree("providers")?;
        let providers = self.providers.read().await;
        
        for provider in providers.iter() {
            let key = provider.slp_id.0.as_bytes();
            let value = bincode::serialize(provider)?;
            tree.insert(key, value)?;
        }
        
        tree.flush()?;
        Ok(())
    }
    
    /// Save statistics to database
    async fn save_stats(&self) -> Result<()> {
        let tree = self.db.open_tree("stats")?;
        let stats = self.stats.read().await;
        
        let value = bincode::serialize(&*stats)?;
        tree.insert("stats", value)?;
        tree.flush()?;
        
        Ok(())
    }
    
    /// Flush all data to disk
    pub async fn flush(&self) -> Result<()> {
        self.save_providers().await?;
        self.save_stats().await?;
        self.db.flush_async().await?;
        Ok(())
    }

    async fn match_job(&self, job: &GxfJob) -> Option<Vec<ComputeProvider>> {
        let providers = self.providers.read().await;
        let mut matches = Vec::new();
        for provider in providers.iter() {
            if provider.can_handle(job) {
                matches.push(provider.clone());
            }
        }
        matches.sort_by_key(|p| p.calculate_price(job));
        if matches.is_empty() {
            None
        } else {
            Some(matches)
        }
    }

    async fn select_route(&self, _job: &GxfJob, _priority: u8) -> Option<Route> {
        let routes = self.routes.read().await;
        let filtered_routes: Vec<&Route> = if _priority >= 128 {
            routes.iter().filter(|r| r.lane_id == LaneId(0)).collect()
        } else {
            routes.iter().filter(|r| r.lane_id == LaneId(1)).collect()
        };
        if filtered_routes.is_empty() {
            routes.iter().min_by(|a, b| a.score().partial_cmp(&b.score()).unwrap())
        } else {
            filtered_routes
                .iter()
                .min_by(|a, b| a.score().partial_cmp(&b.score()).unwrap())
                .copied()
        }
        .cloned()
    }

    pub async fn run_auction(
        &self,
        job: &GxfJob,
        priority: u8,
    ) -> Result<AuctionMatch, GixError> {
        let matches = self
            .match_job(job)
            .await
            .ok_or_else(|| GixError::InternalError("No matching providers found".to_string()))?;

        if matches.is_empty() {
            return Err(GixError::InternalError("No providers can handle this job".to_string()));
        }

        let provider = &matches[0];
        let price = provider.calculate_price(job);
        let route = self
            .select_route(job, priority)
            .await
            .ok_or_else(|| GixError::InternalError("No route available".to_string()))?;

        // Record metrics
        let slp_id_str = provider.slp_id.0.clone();
        let precision_str = format!("{:?}", job.precision);
        
        increment_counter!("gix_auctions_total");
        increment_counter!("gix_auction_matches_total", "slp" => slp_id_str.clone());
        gauge!("gix_clearing_price", slp_id_str.clone() => price as f64);
        increment_gauge!("gix_auction_volume_total", price as f64);
        increment_counter!("gix_matches_by_precision", "precision" => precision_str);

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.total_auctions += 1;
            stats.total_matches += 1;
            stats.total_volume += price;
            *stats.matches_by_precision.entry(job.precision).or_insert(0) += 1;
            *stats.matches_by_lane.entry(route.lane_id.clone()).or_insert(0) += 1;
            
            // Update gauge metrics for stats
            gauge!("gix_total_auctions", stats.total_auctions as f64);
            gauge!("gix_total_matches", stats.total_matches as f64);
            gauge!("gix_total_volume", stats.total_volume as f64);
        }

        // Update provider utilization
        {
            let mut providers = self.providers.write().await;
            if let Some(p) = providers.iter_mut().find(|p| p.slp_id == provider.slp_id) {
                p.utilization += 1;
                
                // Update utilization gauge
                gauge!("gix_provider_utilization", p.utilization as f64, "slp" => slp_id_str);
            }
        }

        // Persist changes to database
        self.save_providers().await.map_err(|e| GixError::InternalError(format!("Failed to save providers: {}", e)))?;
        self.save_stats().await.map_err(|e| GixError::InternalError(format!("Failed to save stats: {}", e)))?;

        Ok(AuctionMatch {
            job_id: job.job_id,
            slp_id: provider.slp_id.clone(),
            lane_id: route.lane_id.clone(),
            price,
            route: route.path,
        })
    }

    /// Get auction statistics
    pub async fn get_stats(&self) -> AuctionStats {
        self.stats.read().await.clone()
    }
}

/// Process a GXF envelope through the auction
pub async fn process_envelope(
    engine: &AuctionEngine,
    envelope: GxfEnvelope,
) -> Result<AuctionMatch> {
    envelope.validate().map_err(|e| anyhow::anyhow!("Envelope validation failed: {}", e))?;
    if envelope.meta.is_expired() {
        return Err(anyhow::anyhow!("Envelope expired"));
    }
    let job = envelope
        .deserialize_job()
        .map_err(|e| anyhow::anyhow!("Failed to deserialize job: {}", e))?;
    job.validate()
        .map_err(|e| anyhow::anyhow!("Job validation failed: {}", e))?;

    engine
        .run_auction(&job, envelope.meta.priority)
        .await
        .map_err(|e| anyhow::anyhow!("Auction failed: {}", e))
}
