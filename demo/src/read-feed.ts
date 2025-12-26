#!/usr/bin/env node

import { Connection } from '@solana/web3.js';
import { fetchPriceFeed, fetchConfig } from '@tachyon-oracles/sdk';
import * as dotenv from 'dotenv';
import { resolve } from 'path';

// Load environment variables
dotenv.config({ path: resolve(__dirname, '../../.env') });

const RPC_URL = process.env.X1_RPC || 'https://rpc.mainnet.x1.xyz';

async function main() {
  const assetId = process.argv[2];
  
  if (!assetId) {
    console.error('Usage: node read-feed.js <ASSET_ID>');
    console.error('Example: node read-feed.js "BTC/USD"');
    process.exit(1);
  }
  
  console.log(`Connecting to ${RPC_URL}...`);
  const connection = new Connection(RPC_URL, 'confirmed');
  
  console.log(`\nFetching oracle configuration...`);
  const config = await fetchConfig(connection);
  
  if (!config) {
    console.error('Oracle config not found. Has the oracle been initialized?');
    process.exit(1);
  }
  
  console.log(`✓ Oracle configured with ${config.assetCount} assets`);
  console.log(`  Min publishers: ${config.minPublishers}`);
  console.log(`  Max age: ${config.maxAgeSec}s`);
  console.log(`  Admin: ${config.admin}`);
  
  console.log(`\nFetching price feed for "${assetId}"...`);
  const feed = await fetchPriceFeed(connection, assetId);
  
  if (!feed) {
    console.error(`Price feed for "${assetId}" not found.`);
    console.error('Available assets can be checked with the admin tools.');
    process.exit(1);
  }
  
  console.log(`\n${'='.repeat(60)}`);
  console.log(`  PRICE FEED: ${assetId}`);
  console.log(`${'='.repeat(60)}`);
  console.log(`  Price:          $${feed.price.toFixed(6)}`);
  console.log(`  Confidence:     ±$${feed.confidence.toFixed(6)}`);
  console.log(`  Publish Time:   ${feed.publishTime.toISOString()}`);
  console.log(`  Staleness:      ${feed.staleness}s ago`);
  console.log(`  Last Slot:      ${feed.lastUpdateSlot}`);
  console.log(`${'='.repeat(60)}`);
  
  // Check staleness
  if (feed.staleness > config.maxAgeSec) {
    console.log(`\n⚠️  WARNING: Data is stale (${feed.staleness}s > ${config.maxAgeSec}s max)`);
  } else {
    console.log(`\n✓ Data is fresh`);
  }
}

main().catch(error => {
  console.error('Error:', error.message);
  process.exit(1);
});

