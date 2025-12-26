#!/usr/bin/env node

/**
 * Simple add asset script
 */

const { Connection, Keypair, Transaction, TransactionInstruction, SystemProgram, PublicKey } = require('@solana/web3.js');
const fs = require('fs');
const dotenv = require('dotenv');
const crypto = require('crypto');

dotenv.config();

async function main() {
  const assetId = process.argv[2];
  
  if (!assetId) {
    console.error('Usage: node add-asset-simple.js <ASSET_ID>');
    console.error('Example: node add-asset-simple.js "BTC/USD"');
    process.exit(1);
  }
  
  console.log(`Adding asset: ${assetId}\n`);
  
  // Load configuration
  const rpcUrl = process.env.X1_RPC || 'https://rpc.mainnet.x1.xyz';
  const adminKeypairPath = process.env.ADMIN_KEYPAIR || './keys/admin.json';
  const programId = new PublicKey(process.env.PROGRAM_ID || 'TACH9r2uZzoFM6daofesADjeDn9NqB1pKFWP5mfByb1');
  
  // Load admin keypair
  const adminKeypairData = JSON.parse(fs.readFileSync(adminKeypairPath, 'utf-8'));
  const adminKeypair = Keypair.fromSecretKey(new Uint8Array(adminKeypairData));
  
  // Connect
  const connection = new Connection(rpcUrl, 'confirmed');
  
  // Hash asset ID
  const assetIdHash = crypto.createHash('sha256').update(assetId).digest();
  console.log(`Asset ID hash: ${assetIdHash.toString('hex')}`);
  
  // Derive PDAs
  const [configPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from('config')],
    programId
  );
  
  const [feedPDA] = PublicKey.findProgramAddressSync(
    [Buffer.from('feed'), assetIdHash],
    programId
  );
  
  console.log(`Config PDA: ${configPDA.toBase58()}`);
  console.log(`Feed PDA: ${feedPDA.toBase58()}`);
  
  // Check if feed already exists
  const feedAccount = await connection.getAccountInfo(feedPDA);
  if (feedAccount) {
    console.log('\n⚠️  Feed already exists!');
    process.exit(0);
  }
  
  // Build instruction data
  // Discriminator for add_asset
  const discriminator = crypto.createHash('sha256').update('global:add_asset').digest().slice(0, 8);
  
  const assetIdBytes = Buffer.from(assetId, 'utf-8');
  const data = Buffer.alloc(8 + 4 + assetIdBytes.length);
  discriminator.copy(data, 0);
  data.writeUInt32LE(assetIdBytes.length, 8);
  assetIdBytes.copy(data, 12);
  
  console.log(`\nCreating add_asset instruction...`);
  
  // Create instruction
  const instruction = new TransactionInstruction({
    keys: [
      { pubkey: configPDA, isSigner: false, isWritable: true },
      { pubkey: feedPDA, isSigner: false, isWritable: true },
      { pubkey: adminKeypair.publicKey, isSigner: true, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    programId,
    data,
  });
  
  // Create and send transaction
  console.log('Sending transaction...');
  const transaction = new Transaction().add(instruction);
  
  try {
    const signature = await connection.sendTransaction(transaction, [adminKeypair]);
    console.log(`Transaction sent: ${signature}`);
    
    // Wait for confirmation
    console.log('Waiting for confirmation...');
    await connection.confirmTransaction(signature, 'confirmed');
    
    console.log(`\n✅ Asset "${assetId}" added successfully!`);
    console.log(`Transaction signature: ${signature}`);
    console.log(`Feed PDA: ${feedPDA.toBase58()}`);
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

