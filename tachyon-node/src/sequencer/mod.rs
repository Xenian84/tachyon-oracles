use std::sync::Arc;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    pubkey::Pubkey,
    signature::Signer,
    transaction::Transaction,
};
use std::str::FromStr;
use tokio::sync::mpsc;
use tracing::{info, error};
use borsh::BorshSerialize;

use crate::config::NodeConfig;
use crate::consensus::ConsensusResult;

pub async fn start_sequencer(
    config: Arc<NodeConfig>,
    mut consensus_rx: mpsc::Receiver<ConsensusResult>,
    mut shutdown: tokio::sync::broadcast::Receiver<()>,
) -> anyhow::Result<()> {
    info!("🚀 Starting sequencer...");
    
    let rpc_client = RpcClient::new_with_commitment(
        config.rpc_url.clone(),
        CommitmentConfig::confirmed(),
    );
    
    let program_id = Pubkey::from_str(&config.l2_program_id)?;
    
    loop {
        tokio::select! {
            Some(result) = consensus_rx.recv() => {
                // Only submit if this node is the leader
                if !result.is_leader {
                    continue;
                }
                
                info!("🚀 Submitting Merkle root to X1: {}", &result.batch.root[..8]);
                
                match submit_to_chain(&rpc_client, &config, &program_id, &result).await {
                    Ok(signature) => {
                        info!("✅ Submitted successfully! Tx: {}", signature);
                    }
                    Err(e) => {
                        error!("❌ Failed to submit to chain: {}", e);
                    }
                }
            }
            _ = shutdown.recv() => {
                info!("🚀 Sequencer shutting down...");
                break;
            }
        }
    }
    
    Ok(())
}

async fn submit_to_chain(
    rpc_client: &RpcClient,
    config: &NodeConfig,
    program_id: &Pubkey,
    result: &ConsensusResult,
) -> anyhow::Result<String> {
    // Convert root hash to bytes
    let root_bytes = hex::decode(&result.batch.root)?;
    if root_bytes.len() != 32 {
        return Err(anyhow::anyhow!("Invalid root hash length"));
    }
    
    let mut root_array = [0u8; 32];
    root_array.copy_from_slice(&root_bytes);
    
    // Find L2 state PDA (must match the seeds in the smart contract!)
    let (l2_state_pda, _) = Pubkey::find_program_address(
        &[b"l2-state"],  // Note: hyphen, not underscore!
        program_id,
    );
    
    // Find governance state PDA
    let governance_program = Pubkey::from_str(&config.program_id)?;
    let (governance_state_pda, _) = Pubkey::find_program_address(
        &[b"governance"],
        &governance_program,
    );
    
    // Build instruction data for submit_root_with_consensus
    // Parameters: root (32), feed_count (4), timestamp (8), total_stake (8), votes (Vec<ConsensusVote>)
    
    // Serialize using Borsh format (Anchor's default)
    use borsh::BorshSerialize;
    
    #[derive(BorshSerialize)]
    struct ConsensusVote {
        validator: [u8; 32],
        root: [u8; 32],
        stake: u64,
        signature: [u8; 64],
    }
    
    #[derive(BorshSerialize)]
    struct SubmitRootParams {
        root: [u8; 32],
        feed_count: u32,
        timestamp: i64,
        total_stake: u64,
        votes: Vec<ConsensusVote>,
    }
    
    // Create our own vote (single validator)
    let our_vote = ConsensusVote {
        validator: config.identity.pubkey().to_bytes(),
        root: root_array,
        stake: result.total_stake,
        signature: [0u8; 64], // TODO: Sign the root in production
    };
    
    let params = SubmitRootParams {
        root: root_array,
        feed_count: result.batch.feeds.len() as u32,
        timestamp: result.batch.timestamp,
        total_stake: result.total_stake,
        votes: vec![our_vote], // Include our vote
    };
    
    let mut instruction_data = vec![0xd7, 0x29, 0x2e, 0xe9, 0x2c, 0x1b, 0x83, 0x05]; // Discriminator
    params.serialize(&mut instruction_data)?;
    
    // Create instruction for submit_root_with_consensus
    let instruction = solana_sdk::instruction::Instruction {
        program_id: *program_id,
        accounts: vec![
            solana_sdk::instruction::AccountMeta::new(l2_state_pda, false),
            solana_sdk::instruction::AccountMeta::new_readonly(governance_state_pda, false),
            solana_sdk::instruction::AccountMeta::new_readonly(config.identity.pubkey(), true),
            solana_sdk::instruction::AccountMeta::new_readonly(governance_program, false),
        ],
        data: instruction_data,
    };
    
    // Get recent blockhash
    let recent_blockhash = rpc_client.get_latest_blockhash()?;
    
    // Create and sign transaction
    let transaction = Transaction::new_signed_with_payer(
        &[instruction],
        Some(&config.identity.pubkey()),
        &[&config.identity],
        recent_blockhash,
    );
    
    // Send transaction
    let signature = rpc_client.send_and_confirm_transaction(&transaction)?;
    
    Ok(signature.to_string())
}

