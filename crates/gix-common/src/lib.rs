pub mod errors;

use serde::{Deserialize, Serialize};

// --- Re-export GixError so it's accessible as gix_common::GixError
pub use errors::GixError;

/// Unique identifier for a compute job (UUID v4)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JobId(pub [u8; 16]);

/// Unique identifier for a Sovereign Liquidity Pool
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SlpId(pub String);

/// Lane identifier for AJR routing (e.g., "Flash", "Deep")
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LaneId(pub u8);