#![allow(dead_code)]
use anyhow::{Context, Result};
use solana_sdk::signature::{Keypair, Signer};
use std::fs;
use std::path::Path;

pub fn load_keypair(path: &str) -> Result<Keypair> {
    let expanded_path = shellexpand::tilde(path).to_string();
    let data = fs::read(&expanded_path)
        .with_context(|| format!("Failed to read keypair file: {}", expanded_path))?;
    
    let bytes: Vec<u8> = serde_json::from_slice(&data)
        .with_context(|| "Failed to parse keypair JSON")?;
    
    Keypair::try_from(&bytes[..])
        .map_err(|e| anyhow::anyhow!("Failed to create keypair from bytes: {}", e))
}

pub fn save_keypair(keypair: &Keypair, path: &str) -> Result<()> {
    let expanded_path = shellexpand::tilde(path).to_string();
    
    // Create parent directory if it doesn't exist
    if let Some(parent) = Path::new(&expanded_path).parent() {
        fs::create_dir_all(parent)?;
    }
    
    let bytes = keypair.to_bytes();
    let json = serde_json::to_vec(&bytes.to_vec())?;
    
    fs::write(&expanded_path, json)
        .with_context(|| format!("Failed to write keypair file: {}", expanded_path))?;
    
    Ok(())
}

pub fn sign_message(keypair: &Keypair, message: &[u8]) -> Vec<u8> {
    
    keypair.sign_message(message).as_ref().to_vec()
}

pub fn verify_signature(pubkey: &[u8; 32], message: &[u8], signature: &[u8; 64]) -> bool {
    use ed25519_dalek::{Verifier, VerifyingKey, Signature};
    
    let Ok(verifying_key) = VerifyingKey::from_bytes(pubkey) else {
        return false;
    };
    
    let sig = match Signature::try_from(signature) {
        Ok(s) => s,
        Err(_) => {
            return false;
        }
    };
    
    verifying_key.verify(message, &sig).is_ok()
}

