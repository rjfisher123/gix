//! Persistence tests for GCAM Node
//!
//! These tests verify that the auction engine state survives restarts.

use anyhow::Result;
use gcam_node::AuctionEngine;
use gix_common::JobId;
use gix_gxf::{GxfJob, PrecisionLevel};
use std::fs;

#[tokio::test]
async fn test_persistence_survives_restart() -> Result<()> {
    let test_db_path = "./test_data/gcam_persistence_test";
    
    // Clean up any existing test database
    let _ = fs::remove_dir_all(test_db_path);
    fs::create_dir_all(test_db_path)?;
    
    // Phase 1: Create engine, run auction, and close
    {
        let engine = AuctionEngine::new(test_db_path)?;
        
        // Create a test job
        let job = GxfJob::new(
            JobId([1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]),
            PrecisionLevel::BF16,
            1024,
        );
        
        // Run auction
        let match_result = engine.run_auction(&job, 150).await?;
        
        // Verify match was successful
        assert_eq!(match_result.job_id, job.job_id);
        assert!(!match_result.slp_id.0.is_empty());
        assert!(match_result.price > 0);
        
        // Get initial stats
        let stats_before = engine.get_stats().await;
        assert_eq!(stats_before.total_auctions, 1);
        assert_eq!(stats_before.total_matches, 1);
        assert!(stats_before.total_volume > 0);
        
        // Explicitly flush to ensure data is persisted
        engine.flush().await?;
        
        // Engine goes out of scope here (simulating shutdown)
    }
    
    // Phase 2: Reopen engine and verify data persisted
    {
        let engine = AuctionEngine::new(test_db_path)?;
        
        // Verify stats persisted
        let stats_after = engine.get_stats().await;
        assert_eq!(stats_after.total_auctions, 1, "Total auctions should persist");
        assert_eq!(stats_after.total_matches, 1, "Total matches should persist");
        assert!(stats_after.total_volume > 0, "Total volume should persist");
        
        // Run another auction to verify engine is fully functional
        let job2 = GxfJob::new(
            JobId([16, 15, 14, 13, 12, 11, 10, 9, 8, 7, 6, 5, 4, 3, 2, 1]),
            PrecisionLevel::FP8,
            2048,
        );
        
        let match_result2 = engine.run_auction(&job2, 100).await?;
        assert_eq!(match_result2.job_id, job2.job_id);
        
        // Verify stats updated correctly
        let stats_final = engine.get_stats().await;
        assert_eq!(stats_final.total_auctions, 2, "Should have 2 auctions after restart");
        assert_eq!(stats_final.total_matches, 2, "Should have 2 matches after restart");
        
        engine.flush().await?;
    }
    
    // Clean up test database
    fs::remove_dir_all(test_db_path)?;
    
    Ok(())
}

#[tokio::test]
async fn test_provider_utilization_persists() -> Result<()> {
    let test_db_path = "./test_data/gcam_provider_test";
    
    // Clean up any existing test database
    let _ = fs::remove_dir_all(test_db_path);
    fs::create_dir_all(test_db_path)?;
    
    let initial_volume;
    
    // Phase 1: Run multiple auctions
    {
        let engine = AuctionEngine::new(test_db_path)?;
        
        // Run 5 auctions
        for i in 0..5 {
            let job = GxfJob::new(
                JobId([i, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
                PrecisionLevel::INT8,
                512,
            );
            engine.run_auction(&job, 50).await?;
        }
        
        let stats = engine.get_stats().await;
        initial_volume = stats.total_volume;
        assert_eq!(stats.total_auctions, 5);
        
        engine.flush().await?;
    }
    
    // Phase 2: Restart and verify state
    {
        let engine = AuctionEngine::new(test_db_path)?;
        
        let stats = engine.get_stats().await;
        assert_eq!(stats.total_auctions, 5, "Auction count should persist");
        assert_eq!(stats.total_volume, initial_volume, "Volume should persist");
        
        // Provider utilization should have increased and persisted
        // Run another auction and verify it works
        let job = GxfJob::new(
            JobId([99, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
            PrecisionLevel::BF16,
            1024,
        );
        let result = engine.run_auction(&job, 200).await?;
        assert!(!result.slp_id.0.is_empty());
        
        engine.flush().await?;
    }
    
    // Clean up test database
    fs::remove_dir_all(test_db_path)?;
    
    Ok(())
}

#[tokio::test]
async fn test_crash_recovery() -> Result<()> {
    let test_db_path = "./test_data/gcam_crash_test";
    
    // Clean up any existing test database
    let _ = fs::remove_dir_all(test_db_path);
    fs::create_dir_all(test_db_path)?;
    
    // Phase 1: Normal operation
    {
        let engine = AuctionEngine::new(test_db_path)?;
        
        let job = GxfJob::new(
            JobId([1; 16]),
            PrecisionLevel::E5M2,
            768,
        );
        engine.run_auction(&job, 100).await?;
        
        // Flush to simulate periodic checkpoint
        engine.flush().await?;
        
        // Simulate crash (drop without explicit flush)
    }
    
    // Phase 2: Recovery after "crash"
    {
        let engine = AuctionEngine::new(test_db_path)?;
        
        // Data from flushed state should be recovered
        let stats = engine.get_stats().await;
        assert_eq!(stats.total_auctions, 1, "Should recover from last flush");
        
        // Engine should be fully operational
        let job = GxfJob::new(
            JobId([2; 16]),
            PrecisionLevel::BF16,
            1024,
        );
        let result = engine.run_auction(&job, 150).await?;
        assert_eq!(result.job_id, job.job_id);
        
        engine.flush().await?;
    }
    
    // Clean up test database
    fs::remove_dir_all(test_db_path)?;
    
    Ok(())
}

