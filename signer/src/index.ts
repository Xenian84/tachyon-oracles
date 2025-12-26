import { Keypair } from '@solana/web3.js';
import { readFileSync } from 'fs';
import { config } from './config';
import { logger } from './logger';
import { PriceFetcher } from './fetcher';
import nacl from 'tweetnacl';
import { hashAssetId, getMessageBytes, priceToI64 } from '@tachyon-oracles/sdk';
import axios from 'axios';
import BN from 'bn.js';

// Load signer keypair
let signerKeypair: Keypair;
try {
  const keypairData = JSON.parse(readFileSync(config.signerKeypair, 'utf-8'));
  signerKeypair = Keypair.fromSecretKey(new Uint8Array(keypairData));
  logger.info(`Signer public key: ${signerKeypair.publicKey.toBase58()}`);
} catch (error) {
  logger.error('Failed to load signer keypair:', error);
  process.exit(1);
}

const priceFetcher = new PriceFetcher();

async function signingLoop() {
  while (true) {
    try {
      await new Promise(resolve => setTimeout(resolve, config.intervalMs));
      
      for (const assetConfig of config.assets) {
        try {
          await processAsset(assetConfig);
        } catch (error: any) {
          logger.error(`Error processing asset ${assetConfig.id}: ${error.message || error}`);
        }
      }
    } catch (error) {
      logger.error('Error in signing loop:', error);
    }
  }
}

async function processAsset(assetConfig: any) {
  logger.debug(`Processing asset: ${assetConfig.id}`);
  
  // Fetch prices from sources
  const prices: number[] = [];
  
  for (const source of assetConfig.sources) {
    try {
      const price = await priceFetcher.fetchPrice(source, assetConfig.symbol);
      if (price && price > 0) {
        prices.push(price);
        logger.debug(`${source}: ${assetConfig.id} = ${price}`);
      }
    } catch (error) {
      logger.warn(`Failed to fetch from ${source} for ${assetConfig.id}:`, error);
    }
  }
  
  if (prices.length === 0) {
    logger.warn(`No prices available for ${assetConfig.id}`);
    return;
  }
  
  // Calculate median
  prices.sort((a, b) => a - b);
  const median = prices.length % 2 === 0
    ? (prices[prices.length / 2 - 1] + prices[prices.length / 2]) / 2
    : prices[Math.floor(prices.length / 2)];
  
  // Calculate confidence (simple: use spread)
  const min = prices[0];
  const max = prices[prices.length - 1];
  const spread = max - min;
  const confidence = spread / 2; // Simple confidence estimate
  
  logger.info(`${assetConfig.id}: price=${median.toFixed(6)}, conf=${confidence.toFixed(6)}, sources=${prices.length}`);
  
  // Convert to fixed-point
  const priceI64 = priceToI64(median);
  const confI64 = priceToI64(confidence);
  const publishTime = new BN(Math.floor(Date.now() / 1000));
  
  // Hash asset ID
  const assetIdHash = hashAssetId(assetConfig.id);
  
  // Create message bytes
  const messageBytes = getMessageBytes(assetIdHash, priceI64, confI64, publishTime);
  
  // Sign message using nacl
  const signature = nacl.sign.detached(messageBytes, signerKeypair.secretKey);
  
  // Submit to relayers
  for (const relayerUrl of config.relayerUrls) {
    try {
      await submitToRelayer(relayerUrl, {
        publisher_pubkey: signerKeypair.publicKey.toBase58(),
        asset_id_hash: assetIdHash.toString('hex'),
        price_i64: priceI64.toString(),
        conf_i64: confI64.toString(),
        publish_time_i64: publishTime.toString(),
        signature_base64: Buffer.from(signature).toString('base64'),
      });
      
      logger.info(`Submitted ${assetConfig.id} to ${relayerUrl}`);
    } catch (error) {
      logger.error(`Failed to submit to ${relayerUrl}:`, error);
    }
  }
}

async function submitToRelayer(relayerUrl: string, payload: any) {
  const response = await axios.post(`${relayerUrl}/submit`, payload, {
    timeout: 5000,
    headers: { 'Content-Type': 'application/json' },
  });
  
  if (response.status !== 200) {
    throw new Error(`Relayer returned status ${response.status}`);
  }
  
  return response.data;
}

// Start signing loop
logger.info('Starting signer daemon...');
logger.info(`Monitoring ${config.assets.length} assets`);
logger.info(`Submitting to ${config.relayerUrls.length} relayers`);

signingLoop().catch(error => {
  logger.error('Fatal error in signing loop:', error);
  process.exit(1);
});

// Graceful shutdown
process.on('SIGINT', () => {
  logger.info('Shutting down gracefully...');
  process.exit(0);
});

process.on('SIGTERM', () => {
  logger.info('Shutting down gracefully...');
  process.exit(0);
});

