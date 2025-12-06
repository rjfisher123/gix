//! Wallet management for GIX CLI
//!
//! Handles secure storage and loading of Dilithium keypairs.

use anyhow::{Context, Result};
use gix_crypto::pqc::dilithium::KeyPair;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

/// Wallet structure stored in JSON
#[derive(Debug, Serialize, Deserialize)]
pub struct Wallet {
    /// Version for future compatibility
    pub version: u32,
    /// Dilithium keypair
    pub keypair: KeyPair,
}

/// Get the default wallet directory (~/.gix)
pub fn get_default_wallet_dir() -> PathBuf {
    let home = dirs::home_dir().expect("Unable to determine home directory");
    home.join(".gix")
}

/// Get the default wallet path (~/.gix/wallet.json)
pub fn get_default_wallet_path() -> PathBuf {
    get_default_wallet_dir().join("wallet.json")
}

/// Save a wallet to a file with secure permissions
pub fn save_wallet(keypair: &KeyPair, path: &str) -> Result<()> {
    let wallet = Wallet {
        version: 1,
        keypair: keypair.clone(),
    };
    
    let wallet_json = serde_json::to_string_pretty(&wallet)
        .context("Failed to serialize wallet")?;
    
    // Ensure parent directory exists
    let path_obj = Path::new(path);
    if let Some(parent) = path_obj.parent() {
        fs::create_dir_all(parent)
            .context(format!("Failed to create directory: {:?}", parent))?;
    }
    
    // Write wallet file
    fs::write(path, wallet_json)
        .context(format!("Failed to write wallet to: {}", path))?;
    
    // Set restrictive permissions (600 - owner read/write only) on Unix
    #[cfg(unix)]
    {
        let metadata = fs::metadata(path)?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o600); // rw------- (owner only)
        fs::set_permissions(path, permissions)
            .context("Failed to set wallet permissions")?;
    }
    
    Ok(())
}

/// Load a wallet from a file
pub fn load_wallet(path: &str) -> Result<KeyPair> {
    // Check if file exists
    if !Path::new(path).exists() {
        return Err(anyhow::anyhow!(
            "Wallet file not found: {}\n\nRun 'gix keygen' to create a new wallet.",
            path
        ));
    }
    
    // Warn if permissions are too open on Unix
    #[cfg(unix)]
    {
        let metadata = fs::metadata(path)?;
        let permissions = metadata.permissions();
        let mode = permissions.mode();
        
        // Check if file is readable by group or others
        if mode & 0o077 != 0 {
            eprintln!("⚠️  Warning: Wallet file has insecure permissions!");
            eprintln!("   Recommended: chmod 600 {}", path);
        }
    }
    
    // Read and parse wallet
    let wallet_json = fs::read_to_string(path)
        .context(format!("Failed to read wallet from: {}", path))?;
    
    let wallet: Wallet = serde_json::from_str(&wallet_json)
        .context("Failed to parse wallet JSON")?;
    
    // Check version
    if wallet.version != 1 {
        return Err(anyhow::anyhow!(
            "Unsupported wallet version: {}. Expected version 1.",
            wallet.version
        ));
    }
    
    Ok(wallet.keypair)
}

#[cfg(test)]
mod tests {
    use super::*;
    use gix_crypto::pqc::dilithium;
    
    #[test]
    fn test_wallet_save_load_roundtrip() {
        let temp_dir = std::env::temp_dir();
        let wallet_path = temp_dir.join("test_wallet.json");
        let wallet_path_str = wallet_path.to_str().unwrap();
        
        // Generate keypair
        let original_keypair = dilithium::KeyPair::generate();
        
        // Save wallet
        save_wallet(&original_keypair, wallet_path_str).unwrap();
        
        // Load wallet
        let loaded_keypair = load_wallet(wallet_path_str).unwrap();
        
        // Verify keypair matches
        assert_eq!(original_keypair.public.bytes, loaded_keypair.public.bytes);
        assert_eq!(original_keypair.secret.bytes, loaded_keypair.secret.bytes);
        
        // Clean up
        std::fs::remove_file(wallet_path).ok();
    }
    
    #[test]
    fn test_load_nonexistent_wallet() {
        let result = load_wallet("/nonexistent/path/wallet.json");
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }
}

