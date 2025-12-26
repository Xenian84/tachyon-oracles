#!/usr/bin/env node

/**
 * Add a new asset to the oracle
 */

const { Connection, Keypair, Transaction, sendAndConfirmTransaction } = require('@solana/web3.js');
const { createAddAssetInstruction } = require('../sdk/dist/instructions');
const fs = require('fs');
const dotenv = require('dotenv');

dotenv.config();

async function main() {
  const assetId = process.argv[2];
  
  if (!assetId) {
    console.error('Usage: node add-asset.js <ASSET_ID>');
    console.error('Example: node add-asset.js "BTC/USD"');
    process.exit(1);
  }
  
  console.log(`Adding asset: ${assetId}\n`);
  
  // Load configuration
  const rpcUrl = process.env.X1_RPC || 'https://rpc.mainnet.x1.xyz';
  const adminKeypairPath = process.env.ADMIN_KEYPAIR || './keys/admin.json';
  
  // Load admin keypair
  const adminKeypairData = JSON.parse(fs.readFileSync(adminKeypairPath, 'utf-8'));
  const adminKeypair = Keypair.fromSecretKey(new Uint8Array(adminKeypairData));
  
  // Connect
  const connection = new Connection(rpcUrl, 'confirmed');
  
  // Create instruction
  console.log('Creating add_asset instruction...');
  const instruction = createAddAssetInstruction(adminKeypair.publicKey, assetId);
  
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
    
    console.log(`\n✅ Asset "${assetId}" added successfully!`);
    console.log(`Transaction signature: ${signature}`);
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

