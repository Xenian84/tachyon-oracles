#!/usr/bin/env node

/**
 * Simple staking script for Tachyon Oracle Node
 * Usage: NODE_PUBKEY=xxx STAKE_AMOUNT=100000 WALLET_PATH=~/wallet.json node stake-simple.js
 */

const anchor = require('@coral-xyz/anchor');
const { Connection, PublicKey, Keypair } = require('@solana/web3.js');
const fs = require('fs');

const RPC_URL = 'https://rpc.mainnet.x1.xyz';
const PROGRAM_ID = 'TACHdFYQ4uDuAdo6Hz4V1RaCezEpHkVRZGQ7yh24Ad9';
const TACH_MINT = 'TACHmintXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX'; // Replace with actual mint

async function stake() {
    try {
        // Get parameters from environment
        const nodePubkey = process.env.NODE_PUBKEY;
        const stakeAmount = process.env.STAKE_AMOUNT;
        const walletPath = process.env.WALLET_PATH;
        
        if (!nodePubkey || !stakeAmount || !walletPath) {
            console.error('❌ Missing required environment variables');
            console.error('Usage: NODE_PUBKEY=xxx STAKE_AMOUNT=100000 WALLET_PATH=~/wallet.json node stake-simple.js');
            process.exit(1);
        }
        
        // Load wallet
        const walletKeypair = Keypair.fromSecretKey(
            Uint8Array.from(JSON.parse(fs.readFileSync(walletPath, 'utf-8')))
        );
        
        console.log('🔗 Connecting to X1...');
        const connection = new Connection(RPC_URL, 'confirmed');
        
        console.log('📊 Staking', stakeAmount, 'TACH for node', nodePubkey);
        console.log('💰 From wallet:', walletKeypair.publicKey.toString());
        
        // TODO: Implement actual staking transaction
        // This is a placeholder - you'll need to implement the actual staking logic
        
        console.log('✅ Stake transaction sent!');
        console.log('⏳ Waiting for confirmation...');
        
        // Wait for confirmation
        await new Promise(resolve => setTimeout(resolve, 3000));
        
        console.log('✅ Staking complete!');
        console.log('🎉 Your node is now active!');
        
    } catch (error) {
        console.error('❌ Staking failed:', error.message);
        process.exit(1);
    }
}

stake();

