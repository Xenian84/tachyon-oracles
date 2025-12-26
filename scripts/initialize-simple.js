#!/usr/bin/env node

/**
 * Simple initialize script without SDK dependencies
 */

const { Connection, Keypair, Transaction, TransactionInstruction, SystemProgram, PublicKey } = require('@solana/web3.js');
const fs = require('fs');
const dotenv = require('dotenv');
const crypto = require('crypto');

dotenv.config();

async function main() {
  console.log('Initializing Tachyon Oracle Configuration...\n');
  
  // Load configuration
  const rpcUrl = process.env.X1_RPC || 'https://rpc.mainnet.x1.xyz';
  const adminKeypairPath = process.env.ADMIN_KEYPAIR || './keys/admin.json';
  const programId = new PublicKey(process.env.PROGRAM_ID || 'TACH9r2uZzoFM6daofesADjeDn9NqB1pKFWP5mfByb1');
  
  console.log(`RPC: ${rpcUrl}`);
  console.log(`Program ID: ${programId.toBase58()}`);
  console.log(`Admin keypair: ${adminKeypairPath}\n`);
  
  // Load admin keypair
  const adminKeypairData = JSON.parse(fs.readFileSync(adminKeypairPath, 'utf-8'));
  const adminKeypair = Keypair.fromSecretKey(new Uint8Array(adminKeypairData));
  console.log(`Admin pubkey: ${adminKeypair.publicKey.toBase58()}\n`);
  
  // Connect
  const connection = new Connection(rpcUrl, 'confirmed');
  
  // Check balance
  const balance = await connection.getBalance(adminKeypair.publicKey);
  console.log(`Admin balance: ${balance / 1e9} XNT`);
  
  if (balance < 0.01 * 1e9) {
    console.error('\n❌ Insufficient balance! Need at least 0.01 XNT');
    process.exit(1);
  }
  
  // Derive config PDA
  const [configPDA, configBump] = PublicKey.findProgramAddressSync(
    [Buffer.from('config')],
    programId
  );
  
  console.log(`\nConfig PDA: ${configPDA.toBase58()}`);
  console.log(`Config Bump: ${configBump}`);
  
  // Check if already initialized
  const configAccount = await connection.getAccountInfo(configPDA);
  if (configAccount) {
    console.log('\n⚠️  Config already initialized!');
    console.log(`Account owner: ${configAccount.owner.toBase58()}`);
    process.exit(0);
  }
  
  // Build instruction data
  // Discriminator for initialize_config (first 8 bytes of sha256("global:initialize_config"))
  const discriminator = Buffer.from([208, 127, 21, 1, 194, 190, 196, 70]);
  
  const updateFeeLamports = BigInt(process.env.UPDATE_FEE_LAMPORTS || '0');
  const relayerCutBps = parseInt(process.env.RELAYER_CUT_BPS || '1000');
  const minPublishers = parseInt(process.env.MIN_PUBLISHERS || '3');
  const maxAgeSec = parseInt(process.env.MAX_AGE_SEC || '60');
  
  const data = Buffer.alloc(8 + 8 + 2 + 1 + 4);
  discriminator.copy(data, 0);
  data.writeBigUInt64LE(updateFeeLamports, 8);
  data.writeUInt16LE(relayerCutBps, 16);
  data.writeUInt8(minPublishers, 18);
  data.writeUInt32LE(maxAgeSec, 19);
  
  console.log(`\nParameters:`);
  console.log(`  Update fee: ${updateFeeLamports} lamports`);
  console.log(`  Relayer cut: ${relayerCutBps} bps`);
  console.log(`  Min publishers: ${minPublishers}`);
  console.log(`  Max age: ${maxAgeSec}s`);
  
  // Create instruction
  const instruction = new TransactionInstruction({
    keys: [
      { pubkey: configPDA, isSigner: false, isWritable: true },
      { pubkey: adminKeypair.publicKey, isSigner: true, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    programId,
    data,
  });
  
  // Create and send transaction
  console.log('\nSending transaction...');
  const transaction = new Transaction().add(instruction);
  
  try {
    const signature = await connection.sendTransaction(transaction, [adminKeypair]);
    console.log(`Transaction sent: ${signature}`);
    
    // Wait for confirmation
    console.log('Waiting for confirmation...');
    await connection.confirmTransaction(signature, 'confirmed');
    
    console.log('\n✅ Configuration initialized successfully!');
    console.log(`Transaction signature: ${signature}`);
    console.log(`Config PDA: ${configPDA.toBase58()}`);
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

