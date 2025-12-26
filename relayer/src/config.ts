import * as dotenv from 'dotenv';
import { resolve } from 'path';

// Load environment variables
dotenv.config({ path: resolve(__dirname, '../../.env') });

export const config = {
  // Server configuration
  host: process.env.RELAYER_HOST || '0.0.0.0',
  port: parseInt(process.env.RELAYER_PORT || '7777'),
  
  // Blockchain configuration
  rpcUrl: process.env.X1_RPC || 'https://rpc.mainnet.x1.xyz',
  programId: process.env.PROGRAM_ID || 'TACH9r2uZzoFM6daofesADjeDn9NqB1pKFWP5mfByb1',
  relayerKeypair: process.env.RELAYER_KEYPAIR || './keys/relayer.json',
  
  // Oracle configuration
  minPublishers: parseInt(process.env.MIN_PUBLISHERS || '1'),
  maxAgeSec: parseInt(process.env.MAX_AGE_SEC || '60'),
  
  // Timing configuration
  tickIntervalMs: parseInt(process.env.TICK_INTERVAL_MS || '30000'),
  
  // Rate limiting
  maxMessagesPerPublisher: 100,
  rateLimitWindowMs: 60000, // 1 minute
};

