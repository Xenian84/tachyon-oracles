#!/usr/bin/env node

/**
 * Simple register publisher script
 */

const { Connection, Keypair, Transaction, TransactionInstruction, SystemProgram, PublicKey } = require('@solana/web3.js');
const fs = require('fs');
const dotenv = require('dotenv');
const crypto = require('crypto');

dotenv.config();

async function main() {
  const keypairPath = process.argv[2];
  
  if (!keypairPath) {
    console.error('Usage: node register-publisher-simple.js <KEYPAIR_PATH>');
    console.error('Example: node register-publisher-simple.js ./keys/signer.json');
    process.exit(1);
  }
  
  console.log(`Registering publisher from: ${keypairPath}\n`);
  
  // Load configuration
  const rpcUrl = process.env.X1_RPC || 'https://rpc.mainnet.x1.xyz';
  const programId = new PublicKey(process.env.PROGRAM_ID || 'TACH9r2uZzoFM6daofesADjeDn9NqB1pKFWP5mfByb1');
  
  // Load publisher keypair
  const keypairData = JSON.parse(fs.readFileSync(keypairPath, 'utf-8'));
  const publisherKeypair = Keypair.fromSecretKey(new Uint8Array(keypairData));
  
  console.log(`Publisher pubkey: ${publisherKeypair.publicKey.toBase58()}\n`);
  
  // Connect
  const connection = new Connection(rpcUrl, 'confirmed');
  
  // Check balance
  const balance = await connection.getBalance(publisherKeypair.publicKey);
  console.log(`Balance: ${balance / 1e9} XNT`);
  
  if (balance < 0.001 * 1e9) {
    console.error('\n❌ Insufficient balance! Need at least 0.001 XNT for rent');
    process.exit(1);
  }
  
  // Derive publisher PDA
  const [publisherPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from('publisher'), publisherKeypair.publicKey.toBuffer()],
    programId
  );
  
  console.log(`Publisher PDA: ${publisherPDA.toBase58()}`);
  
  // Check if already registered
  const publisherAccount = await connection.getAccountInfo(publisherPDA);
  if (publisherAccount) {
    console.log('\n⚠️  Publisher already registered!');
    console.log('Account data length:', publisherAccount.data.length);
    console.log('Owner:', publisherAccount.owner.toBase58());
    process.exit(0);
  }
  
  console.log('\n✅ Publisher not registered yet, proceeding...');
  
  // Build instruction data
  // Discriminator for register_publisher
  const discriminator = crypto.createHash('sha256').update('global:register_publisher').digest().slice(0, 8);
  
  console.log(`\nCreating register_publisher instruction...`);
  
  // Create instruction
  const instruction = new TransactionInstruction({
    keys: [
      { pubkey: publisherPDA, isSigner: false, isWritable: true },
      { pubkey: publisherKeypair.publicKey, isSigner: true, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    programId,
    data: discriminator,
  });
  
  // Create and send transaction
  console.log('Sending transaction...');
  const transaction = new Transaction().add(instruction);
  
  try {
    const signature = await connection.sendTransaction(transaction, [publisherKeypair]);
    console.log(`Transaction sent: ${signature}`);
    
    // Wait for confirmation
    console.log('Waiting for confirmation...');
    await connection.confirmTransaction(signature, 'confirmed');
    
    console.log('\n✅ Publisher registered successfully!');
    console.log(`Transaction signature: ${signature}`);
    console.log(`Publisher PDA: ${publisherPDA.toBase58()}`);
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

