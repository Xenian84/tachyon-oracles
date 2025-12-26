#!/usr/bin/env node

/**
 * Initialize Tachyon Oracle configuration
 */

const { Connection, Keypair, Transaction, sendAndConfirmTransaction } = require('@solana/web3.js');
const { createInitializeConfigInstruction } = require('../sdk/dist/instructions');
const BN = require('bn.js');
const fs = require('fs');
const dotenv = require('dotenv');

dotenv.config();

async function main() {
  console.log('Initializing Tachyon Oracle Configuration...\n');
  
  // Load configuration
  const rpcUrl = process.env.X1_RPC || 'https://rpc.mainnet.x1.xyz';
  const adminKeypairPath = process.env.ADMIN_KEYPAIR || './keys/admin.json';
  const updateFeeLamports = new BN(process.env.UPDATE_FEE_LAMPORTS || '0');
  const relayerCutBps = parseInt(process.env.RELAYER_CUT_BPS || '1000');
  const minPublishers = parseInt(process.env.MIN_PUBLISHERS || '3');
  const maxAgeSec = parseInt(process.env.MAX_AGE_SEC || '60');
  
  console.log(`RPC: ${rpcUrl}`);
  console.log(`Admin keypair: ${adminKeypairPath}`);
  console.log(`Update fee: ${updateFeeLamports.toString()} lamports`);
  console.log(`Relayer cut: ${relayerCutBps} bps (${relayerCutBps/100}%)`);
  console.log(`Min publishers: ${minPublishers}`);
  console.log(`Max age: ${maxAgeSec}s\n`);
  
  // Load admin keypair
  const adminKeypairData = JSON.parse(fs.readFileSync(adminKeypairPath, 'utf-8'));
  const adminKeypair = Keypair.fromSecretKey(new Uint8Array(adminKeypairData));
  console.log(`Admin pubkey: ${adminKeypair.publicKey.toBase58()}\n`);
  
  // Connect
  const connection = new Connection(rpcUrl, 'confirmed');
  
  // Check balance
  const balance = await connection.getBalance(adminKeypair.publicKey);
  console.log(`Admin balance: ${balance / 1e9} SOL`);
  
  if (balance < 0.01 * 1e9) {
    console.error('\n❌ Insufficient balance! Need at least 0.01 SOL');
    process.exit(1);
  }
  
  // Create instruction
  console.log('\nCreating initialize instruction...');
  const instruction = createInitializeConfigInstruction(
    adminKeypair.publicKey,
    updateFeeLamports,
    relayerCutBps,
    minPublishers,
    maxAgeSec
  );
  
  // Create and send transaction
  console.log('Sending transaction...');
  const transaction = new Transaction().add(instruction);
  
  try {
    const signature = await sendAndConfirmTransaction(
      connection,
      transaction,
      [adminKeypair],
      { commitment: 'confirmed' }
    );
    
    console.log('\n✅ Configuration initialized successfully!');
    console.log(`Transaction signature: ${signature}`);
    console.log(`\nView on explorer: https://explorer.x1.xyz/tx/${signature}`);
  } catch (error) {
    console.error('\n❌ Error:', error.message);
    if (error.logs) {
      console.error('Program logs:', error.logs);
    }
    process.exit(1);
  }
}

main().catch(error => {
  console.error('Fatal error:', error);
  process.exit(1);
});

