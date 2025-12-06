//! GIX CLI - Command Line Interface
//!
//! Provides wallet management, job submission, and service interaction.

mod wallet;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use colored::Colorize;
use gix_common::JobId;
use gix_crypto::pqc::dilithium;
use gix_gxf::{GxfEnvelope, GxfJob, GxfMetadata, PrecisionLevel};
use gix_proto::v1::{GetAuctionStatsRequest, RunAuctionRequest};
use gix_proto::AuctionServiceClient;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// GIX Command Line Interface
#[derive(Parser)]
#[command(name = "gix")]
#[command(author = "GIX Architecture Group")]
#[command(version = "0.1.0")]
#[command(about = "Global Intelligence Exchange CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Generate a new wallet with Dilithium keypair
    Keygen {
        /// Output path for wallet file (default: ~/.gix/wallet.json)
        #[arg(short, long)]
        output: Option<String>,
    },
    
    /// Submit a job to the GIX network
    Submit {
        /// Path to job YAML file
        job_file: String,
        
        /// Wallet file path (default: ~/.gix/wallet.json)
        #[arg(short, long)]
        wallet: Option<String>,
        
        /// GCAM node address (default: http://127.0.0.1:50052)
        #[arg(short, long)]
        node: Option<String>,
        
        /// Job priority (0-255)
        #[arg(short, long, default_value = "128")]
        priority: u8,
    },
    
    /// Query auction statistics
    Status {
        /// GCAM node address (default: http://127.0.0.1:50052)
        #[arg(short, long)]
        node: Option<String>,
    },
    
    /// Display wallet information
    Wallet {
        /// Wallet file path (default: ~/.gix/wallet.json)
        #[arg(short = 'f', long)]
        wallet: Option<String>,
    },
}

/// Job specification from YAML file
#[derive(Debug, Serialize, Deserialize)]
struct JobSpec {
    /// Model identifier
    model: String,
    /// Precision level (BF16, FP8, E5M2, INT8)
    precision: String,
    /// KV cache sequence length
    kv_cache_seq_len: u32,
    /// Token count (optional)
    #[serde(default = "default_token_count")]
    token_count: u32,
    /// Batch size (optional)
    #[serde(default = "default_batch_size")]
    batch_size: u32,
}

fn default_token_count() -> u32 { 128 }
fn default_batch_size() -> u32 { 1 }

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Keygen { output } => {
            handle_keygen(output).await?;
        }
        Commands::Submit { job_file, wallet, node, priority } => {
            handle_submit(job_file, wallet, node, priority).await?;
        }
        Commands::Status { node } => {
            handle_status(node).await?;
        }
        Commands::Wallet { wallet } => {
            handle_wallet_info(wallet).await?;
        }
    }
    
    Ok(())
}

/// Handle keygen command
async fn handle_keygen(output: Option<String>) -> Result<()> {
    println!("{}", "Generating new Dilithium3 keypair...".cyan());
    
    let keypair = dilithium::KeyPair::generate();
    
    let wallet_path = output.unwrap_or_else(|| {
        wallet::get_default_wallet_path().to_string_lossy().to_string()
    });
    
    wallet::save_wallet(&keypair, &wallet_path)?;
    
    println!("{}", "✓ Keypair generated successfully!".green());
    println!("Wallet saved to: {}", wallet_path.bright_white());
    println!();
    println!("{}", "Public key (hex):".yellow());
    println!("{}", hex::encode(&keypair.public.bytes));
    
    Ok(())
}

/// Handle submit command
async fn handle_submit(
    job_file: String,
    wallet_path: Option<String>,
    node_addr: Option<String>,
    priority: u8,
) -> Result<()> {
    // Load job spec from YAML
    println!("{}", format!("Loading job from {}...", job_file).cyan());
    let job_spec = load_job_spec(&job_file)?;
    
    // Load wallet
    let wallet_path = wallet_path.unwrap_or_else(|| {
        wallet::get_default_wallet_path().to_string_lossy().to_string()
    });
    
    println!("{}", "Loading wallet...".cyan());
    let keypair = wallet::load_wallet(&wallet_path)?;
    
    // Create GXF job
    let job_id = JobId::new();
    let precision = parse_precision(&job_spec.precision)?;
    
    let job = GxfJob::new(job_id, precision, job_spec.kv_cache_seq_len);
    
    // Create envelope
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    let meta = GxfMetadata {
        priority,
        timestamp: now,
        ttl: 300, // 5 minutes
    };
    
    let mut envelope = GxfEnvelope::new(meta, job.clone());
    
    // Sign envelope
    println!("{}", "Signing envelope...".cyan());
    let payload_bytes = serde_json::to_vec(&job)?;
    let signature = dilithium::sign_detached(&payload_bytes, &keypair.secret)?;
    envelope.signature = Some(signature.bytes);
    
    // Connect to GCAM node
    let node_addr = node_addr.unwrap_or_else(|| "http://127.0.0.1:50052".to_string());
    println!("{}", format!("Connecting to {}...", node_addr).cyan());
    
    let mut client = AuctionServiceClient::connect(node_addr.clone())
        .await
        .context("Failed to connect to GCAM node")?;
    
    // Submit job
    println!("{}", "Submitting job to auction...".cyan());
    let request = tonic::Request::new(RunAuctionRequest {
        job: serde_json::to_vec(&job)?,
        priority: priority as u32,
    });
    
    let response = client.run_auction(request)
        .await
        .context("Failed to run auction")?
        .into_inner();
    
    // Display results
    println!();
    if response.success {
        println!("{}", "✓ Job submitted successfully!".green().bold());
        println!();
        println!("{}", "Auction Results:".yellow().bold());
        println!("  Job ID:     {}", hex::encode(&response.job_id.unwrap().id));
        println!("  SLP ID:     {}", response.slp_id.unwrap().id);
        println!("  Lane ID:    {}", response.lane_id.unwrap().id);
        println!("  Price:      {} μGIX", response.price.to_string().bright_white());
        println!("  Route:      {}", response.route.join(" → "));
    } else {
        println!("{}", "✗ Job submission failed!".red().bold());
        println!("Error: {}", response.error);
    }
    
    Ok(())
}

/// Handle status command
async fn handle_status(node_addr: Option<String>) -> Result<()> {
    let node_addr = node_addr.unwrap_or_else(|| "http://127.0.0.1:50052".to_string());
    
    println!("{}", format!("Connecting to {}...", node_addr).cyan());
    
    let mut client = AuctionServiceClient::connect(node_addr)
        .await
        .context("Failed to connect to GCAM node")?;
    
    println!("{}", "Fetching auction statistics...".cyan());
    
    let request = tonic::Request::new(GetAuctionStatsRequest {});
    let response = client.get_auction_stats(request)
        .await
        .context("Failed to get stats")?
        .into_inner();
    
    // Display stats
    println!();
    println!("{}", "=== GCAM Auction Statistics ===".yellow().bold());
    println!();
    println!("Total Auctions:  {}", response.total_auctions.to_string().bright_white());
    println!("Total Matches:   {}", response.total_matches.to_string().bright_white());
    println!("Total Volume:    {} μGIX", response.total_volume.to_string().bright_white());
    
    if !response.matches_by_precision.is_empty() {
        println!();
        println!("{}", "Matches by Precision:".cyan());
        for (precision, count) in &response.matches_by_precision {
            println!("  {:<10} {}", precision, count);
        }
    }
    
    if !response.matches_by_lane.is_empty() {
        println!();
        println!("{}", "Matches by Lane:".cyan());
        for (lane_id, count) in &response.matches_by_lane {
            println!("  Lane {:<5} {}", lane_id, count);
        }
    }
    
    Ok(())
}

/// Handle wallet info command
async fn handle_wallet_info(wallet_path: Option<String>) -> Result<()> {
    let wallet_path = wallet_path.unwrap_or_else(|| {
        wallet::get_default_wallet_path().to_string_lossy().to_string()
    });
    
    println!("{}", format!("Loading wallet from {}...", wallet_path).cyan());
    let keypair = wallet::load_wallet(&wallet_path)?;
    
    println!();
    println!("{}", "=== Wallet Information ===".yellow().bold());
    println!();
    println!("{}", "Public Key (hex):".cyan());
    println!("{}", hex::encode(&keypair.public.bytes));
    println!();
    println!("Public Key Size:  {} bytes", keypair.public.bytes.len());
    println!("Secret Key Size:  {} bytes", keypair.secret.bytes.len());
    println!("Algorithm:        Dilithium3 (NIST Level 3 PQC)");
    
    Ok(())
}

/// Load job specification from YAML file
fn load_job_spec(path: &str) -> Result<JobSpec> {
    let content = std::fs::read_to_string(path)
        .context(format!("Failed to read job file: {}", path))?;
    
    let spec: JobSpec = serde_yaml::from_str(&content)
        .context("Failed to parse job YAML")?;
    
    Ok(spec)
}

/// Parse precision level from string
fn parse_precision(s: &str) -> Result<PrecisionLevel> {
    match s.to_uppercase().as_str() {
        "BF16" => Ok(PrecisionLevel::BF16),
        "FP8" => Ok(PrecisionLevel::FP8),
        "E5M2" => Ok(PrecisionLevel::E5M2),
        "INT8" => Ok(PrecisionLevel::INT8),
        _ => Err(anyhow::anyhow!("Invalid precision level: {}", s)),
    }
}
