import express from 'express';
import { Connection, Keypair, sendAndConfirmTransaction, Transaction, PublicKey, TransactionInstruction } from '@solana/web3.js';
import { config } from './config';
import { logger } from './logger';
import { MessageCache } from './cache';
import { readFileSync } from 'fs';
import BN from 'bn.js';

const app = express();
app.use(express.json({ limit: '10mb' }));

// Initialize connection
const connection = new Connection(config.rpcUrl, 'confirmed');

// Load relayer keypair
let relayerKeypair: Keypair;
try {
  const keypairData = JSON.parse(readFileSync(config.relayerKeypair, 'utf-8'));
  relayerKeypair = Keypair.fromSecretKey(new Uint8Array(keypairData));
  logger.info(`Relayer public key: ${relayerKeypair.publicKey.toBase58()}`);
} catch (error) {
  logger.error('Failed to load relayer keypair:', error);
  process.exit(1);
}

// Message cache
const messageCache = new MessageCache();

// Health check endpoint
app.get('/health', (req, res) => {
  res.json({
    status: 'ok',
    relayer: relayerKeypair.publicKey.toBase58(),
    rpc: config.rpcUrl,
    uptime: process.uptime(),
  });
});

// Rate limiting per IP
const submissionCounts = new Map<string, { count: number; resetTime: number }>();

function checkRateLimit(ip: string): boolean {
  const now = Date.now();
  const record = submissionCounts.get(ip);
  
  if (!record || now > record.resetTime) {
    submissionCounts.set(ip, { count: 1, resetTime: now + config.rateLimitWindowMs });
    return true;
  }
  
  if (record.count >= config.maxMessagesPerPublisher) {
    return false;
  }
  
  record.count++;
  return true;
}

// Submit signed message
app.post('/submit', async (req, res) => {
  try {
    // Rate limiting
    const ip = req.ip || req.connection.remoteAddress || 'unknown';
    if (!checkRateLimit(ip)) {
      return res.status(429).json({ error: 'Rate limit exceeded' });
    }
    
    const { publisher_pubkey, asset_id_hash, price_i64, conf_i64, publish_time_i64, signature_base64 } = req.body;
    
    // Validate input types and presence
    if (!publisher_pubkey || typeof publisher_pubkey !== 'string') {
      return res.status(400).json({ error: 'Invalid publisher_pubkey' });
    }
    if (!asset_id_hash || typeof asset_id_hash !== 'string') {
      return res.status(400).json({ error: 'Invalid asset_id_hash' });
    }
    if (price_i64 === undefined || typeof price_i64 !== 'string') {
      return res.status(400).json({ error: 'Invalid price_i64' });
    }
    if (conf_i64 === undefined || typeof conf_i64 !== 'string') {
      return res.status(400).json({ error: 'Invalid conf_i64' });
    }
    if (!publish_time_i64 || typeof publish_time_i64 !== 'string') {
      return res.status(400).json({ error: 'Invalid publish_time_i64' });
    }
    if (!signature_base64 || typeof signature_base64 !== 'string') {
      return res.status(400).json({ error: 'Invalid signature_base64' });
    }
    
    // Validate hex/base64 formats
    if (!/^[0-9a-f]+$/i.test(asset_id_hash)) {
      return res.status(400).json({ error: 'asset_id_hash must be hex' });
    }
    
    // Validate numeric strings
    try {
      new BN(price_i64);
      new BN(conf_i64);
      new BN(publish_time_i64);
    } catch (e) {
      return res.status(400).json({ error: 'Invalid numeric format' });
    }
    
    // Parse and validate
    const publishTime = new BN(publish_time_i64);
    const now = Math.floor(Date.now() / 1000);
    
    // Reject future timestamps (with small tolerance)
    if (publishTime.toNumber() > now + 10) {
      logger.warn(`Rejected future timestamp from ${publisher_pubkey}: ${publishTime.toNumber()} > ${now}`);
      return res.status(400).json({ error: 'Publish time in future' });
    }
    
    // Reject stale timestamps
    if (now - publishTime.toNumber() > config.maxAgeSec) {
      logger.warn(`Rejected stale message from ${publisher_pubkey}: age=${now - publishTime.toNumber()}s`);
      return res.status(400).json({ error: 'Message too stale' });
    }
    
    // Validate price is reasonable (basic sanity check)
    const priceValue = new BN(price_i64);
    if (priceValue.isNeg() || priceValue.isZero()) {
      logger.warn(`Rejected invalid price from ${publisher_pubkey}: ${price_i64}`);
      return res.status(400).json({ error: 'Price must be positive' });
    }
    
    // Validate confidence is reasonable
    const confValue = new BN(conf_i64);
    if (confValue.isNeg()) {
      logger.warn(`Rejected negative confidence from ${publisher_pubkey}: ${conf_i64}`);
      return res.status(400).json({ error: 'Confidence cannot be negative' });
    }
    
    // Add to cache
    messageCache.addMessage({
      publisherPubkey: publisher_pubkey,
      assetIdHash: asset_id_hash,
      priceI64: price_i64,
      confI64: conf_i64,
      publishTime: publish_time_i64,
      signatureBase64: signature_base64,
      receivedAt: Date.now(),
    });
    
    logger.info(`Received message from ${publisher_pubkey} for asset ${asset_id_hash}`);
    
    res.json({ status: 'accepted' });
  } catch (error) {
    logger.error('Error processing submission:', error);
    res.status(500).json({ error: 'Internal server error' });
  }
});

// Get current feeds status
app.get('/feeds', (req, res) => {
  const status = messageCache.getStatus();
  res.json(status);
});

// Get publishers status
app.get('/publishers', (req, res) => {
  const publishers = messageCache.getPublishers();
  res.json(publishers);
});

// Start server
const server = app.listen(config.port, config.host, () => {
  logger.info(`Relayer listening on ${config.host}:${config.port}`);
});

// Aggregation and submission loop
async function aggregationLoop() {
  while (true) {
    try {
      await new Promise(resolve => setTimeout(resolve, config.tickIntervalMs));
      
      // Bypass SDK - use direct config account read
      const programId = new PublicKey(config.programId);
      const [configPda] = PublicKey.findProgramAddressSync(
        [Buffer.from('config')],
        programId
      );
      
      const configAccountInfo = await connection.getAccountInfo(configPda);
      if (!configAccountInfo) {
        logger.warn('Oracle config not found, skipping tick');
        continue;
      }
      
      // Parse minPublishers from config (at offset 40, u32)
      const minPublishers = configAccountInfo.data.readUInt32LE(40);
      
      const bundles = messageCache.getBundlesForSubmission(minPublishers || 1);
      
      for (const bundle of bundles) {
        try {
          await submitBundle(bundle);
        } catch (error) {
          logger.error(`Error submitting bundle for ${bundle.assetId}:`, error);
        }
      }
    } catch (error) {
      logger.error('Error in aggregation loop:', error);
    }
  }
}

async function submitBundle(bundle: any) {
  logger.info(`Submitting bundle for ${bundle.assetId} with ${bundle.messages.length} publishers`);
  
  try {
    const programId = new PublicKey(config.programId);
    
    // Derive PDAs
    const [configPda] = PublicKey.findProgramAddressSync(
      [Buffer.from('config')],
      programId
    );
    
    const assetIdHash = Buffer.from(bundle.assetId, 'hex');
    const [feedPda] = PublicKey.findProgramAddressSync(
      [Buffer.from('feed'), assetIdHash],
      programId
    );
    
    // Calculate correct Anchor discriminator for post_update
    const crypto = require('crypto');
    const discriminator = crypto
      .createHash('sha256')
      .update('global:post_update')
      .digest()
      .slice(0, 8);
    
    // Serialize the instruction data according to Anchor format:
    // 1. Discriminator (8 bytes)
    // 2. asset_id_hash parameter ([u8; 32])
    // 3. messages parameter (Vec<SignedMessage>)
    const numMessages = bundle.messages.length;
    const messageSize = 32 + 32 + 8 + 8 + 8 + 64; // publisher + asset_id + price + conf + ts + sig
    const dataSize = 8 + 32 + 4 + (numMessages * messageSize); // discriminator + asset_id_hash + vec_len + messages
    
    const data = Buffer.alloc(dataSize);
    let offset = 0;
    
    // Write discriminator (8 bytes)
    discriminator.copy(data, offset);
    offset += 8;
    
    // Write asset_id_hash parameter (32 bytes)
    assetIdHash.copy(data, offset);
    offset += 32;
    
    // Write number of messages (u32 little-endian for Vec length)
    data.writeUInt32LE(numMessages, offset);
    offset += 4;
    
    // Write each message
    for (const msg of bundle.messages) {
      // Publisher pubkey (32 bytes) - msg.publisher is a PublicKey object
      msg.publisher.toBuffer().copy(data, offset);
      offset += 32;
      
      // Asset ID (32 bytes) - msg.assetIdHash is already a Buffer
      msg.assetIdHash.copy(data, offset);
      offset += 32;
      
      // Price (i64 little-endian) - msg.priceI64 is a BN
      const priceBuffer = msg.priceI64.toArrayLike(Buffer, 'le', 8);
      priceBuffer.copy(data, offset);
      offset += 8;
      
      // Confidence (i64 little-endian) - msg.confI64 is a BN
      const confBuffer = msg.confI64.toArrayLike(Buffer, 'le', 8);
      confBuffer.copy(data, offset);
      offset += 8;
      
      // Timestamp (i64 little-endian) - msg.publishTime is a BN
      const timeBuffer = msg.publishTime.toArrayLike(Buffer, 'le', 8);
      timeBuffer.copy(data, offset);
      offset += 8;
      
      // Signature (64 bytes) - msg.signature is already a Buffer
      msg.signature.copy(data, offset);
      offset += 64;
    }
    
    // Build instruction with all required accounts
    const { SystemProgram, SYSVAR_INSTRUCTIONS_PUBKEY } = require('@solana/web3.js');
    
    const instruction = new TransactionInstruction({
      keys: [
        { pubkey: configPda, isSigner: false, isWritable: false },
        { pubkey: feedPda, isSigner: false, isWritable: true },
        { pubkey: relayerKeypair.publicKey, isSigner: true, isWritable: true },
        { pubkey: SYSVAR_INSTRUCTIONS_PUBKEY, isSigner: false, isWritable: false },
        { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
      ],
      programId: programId,
      data: data,
    });
    
    const transaction = new Transaction().add(instruction);
    
    // Send and confirm
    const signature = await sendAndConfirmTransaction(
      connection,
      transaction,
      [relayerKeypair],
      {
        commitment: 'confirmed',
        skipPreflight: false,
      }
    );
    
    logger.info(`✅ Bundle submitted successfully: ${signature}`);
    
    // Mark messages as submitted
    messageCache.markSubmitted(bundle.assetId);
  } catch (error: any) {
    // Extract clean error message from Solana logs
    let errorMsg = error.message || 'Unknown error';
    
    if (error.logs && Array.isArray(error.logs)) {
      // Look for the actual error in logs
      const errorLog = error.logs.find((log: string) => log.includes('Error Message:'));
      if (errorLog) {
        const match = errorLog.match(/Error Message: (.+?)\.?$/);
        if (match) {
          errorMsg = match[1];
        }
      }
    }
    
    logger.error(`❌ Failed to submit bundle for ${bundle.assetId}: ${errorMsg}`);
    // Don't throw, just continue to next bundle
  }
}

// Start aggregation loop
aggregationLoop().catch(error => {
  logger.error('Fatal error in aggregation loop:', error);
  process.exit(1);
});

// Graceful shutdown
process.on('SIGINT', () => {
  logger.info('Shutting down gracefully...');
  server.close(() => {
    logger.info('Server closed');
    process.exit(0);
  });
});

process.on('SIGTERM', () => {
  logger.info('Shutting down gracefully...');
  server.close(() => {
    logger.info('Server closed');
    process.exit(0);
  });
});

