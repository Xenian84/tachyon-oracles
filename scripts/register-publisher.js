#!/usr/bin/env node

/**
 * Register a publisher
 */

const { Connection, Keypair, Transaction, sendAndConfirmTransaction } = require('@solana/web3.js');
const { createRegisterPublisherInstruction } = require('../sdk/dist/instructions');
const fs = require('fs');
const dotenv = require('dotenv');

dotenv.config();

async function main() {
  const keypairPath = process.argv[2];
  
  if (!keypairPath) {
    console.error('Usage: node register-publisher.js <KEYPAIR_PATH>');
    console.error('Example: node register-publisher.js ./keys/signer.json');
    process.exit(1);
  }
  
  console.log(`Registering publisher from: ${keypairPath}\n`);
  
  // Load configuration
  const rpcUrl = process.env.X1_RPC || 'https://rpc.mainnet.x1.xyz';
  
  // Load publisher keypair
  const keypairData = JSON.parse(fs.readFileSync(keypairPath, 'utf-8'));
  const publisherKeypair = Keypair.fromSecretKey(new Uint8Array(keypairData));
  
  console.log(`Publisher pubkey: ${publisherKeypair.publicKey.toBase58()}\n`);
  
  // Connect
  const connection = new Connection(rpcUrl, 'confirmed');
  
  // Check balance
  const balance = await connection.getBalance(publisherKeypair.publicKey);
  console.log(`Balance: ${balance / 1e9} SOL`);
  
  if (balance < 0.001 * 1e9) {
    console.error('\n❌ Insufficient balance! Need at least 0.001 SOL for rent');
    process.exit(1);
  }
  
  // Create instruction
  console.log('\nCreating register_publisher instruction...');
  const instruction = createRegisterPublisherInstruction(publisherKeypair.publicKey);
  
  // Create and send transaction
  console.log('Sending transaction...');
  const transaction = new Transaction().add(instruction);
  
  try {
    const signature = await sendAndConfirmTransaction(
      connection,
      transaction,
      [publisherKeypair],
      { commitment: 'confirmed' }
    );
    
    console.log('\n✅ Publisher registered successfully!');
    console.log(`Transaction signature: ${signature}`);
    console.log(`\nPublisher can now run signer daemon with this keypair.`);
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

