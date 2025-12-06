//! # GIX Protocol Buffer Definitions
//!
//! This crate provides the gRPC service definitions and message types for the Global Intelligence Exchange (GIX) network.
//! It generates Rust code from Protocol Buffer definitions and exports clients and servers for all GIX services.
//!
//! ## Services
//!
//! - **RouterService** - Anonymized job routing (AJR) on port 50051
//! - **AuctionService** - Global compute auction (GCAM) on port 50052
//! - **ExecutionService** - Secure execution envelope (GSEE) on port 50053
//!
//! ## Usage
//!
//! ### Server Implementation
//!
//! ```rust,no_run
//! use gix_proto::{RouterService, RouterServiceServer};
//! use gix_proto::v1::{RouteEnvelopeRequest, RouteEnvelopeResponse};
//! use tonic::{Request, Response, Status};
//!
//! struct MyRouterService;
//!
//! #[tonic::async_trait]
//! impl RouterService for MyRouterService {
//!     async fn route_envelope(
//!         &self,
//!         request: Request<RouteEnvelopeRequest>,
//!     ) -> Result<Response<RouteEnvelopeResponse>, Status> {
//!         // Implementation here
//!         todo!()
//!     }
//!     
//!     async fn get_router_stats(
//!         &self,
//!         request: Request<gix_proto::v1::GetRouterStatsRequest>,
//!     ) -> Result<Response<gix_proto::v1::GetRouterStatsResponse>, Status> {
//!         todo!()
//!     }
//! }
//! ```
//!
//! ### Client Usage
//!
//! ```rust,no_run
//! use gix_proto::AuctionServiceClient;
//! use gix_proto::v1::RunAuctionRequest;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut client = AuctionServiceClient::connect("http://127.0.0.1:50052").await?;
//!     
//!     let request = tonic::Request::new(RunAuctionRequest {
//!         job: vec![],
//!         priority: 128,
//!     });
//!     
//!     let response = client.run_auction(request).await?;
//!     Ok(())
//! }
//! ```
//!
//! ## Protocol Version
//!
//! This crate implements GIX Network Protocol v0.2.0 as defined in `specs/integrated/network_protocol_v0.2.0.md`.

pub mod v1 {
    tonic::include_proto!("gix.v1");
}

// Re-export clients and servers for easier access
pub use v1::router_service_client::RouterServiceClient;
pub use v1::router_service_server::{RouterService, RouterServiceServer};
pub use v1::auction_service_client::AuctionServiceClient;
pub use v1::auction_service_server::{AuctionService, AuctionServiceServer};
pub use v1::execution_service_client::ExecutionServiceClient;
pub use v1::execution_service_server::{ExecutionService, ExecutionServiceServer};
