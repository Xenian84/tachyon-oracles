import * as dotenv from 'dotenv';
import { resolve } from 'path';
import { readFileSync } from 'fs';
import YAML from 'yaml';

// Load environment variables
dotenv.config({ path: resolve(__dirname, '../../.env') });

// Load signer configuration
let signerConfig: any = {
  assets: [
    {
      id: 'BTC/USD',
      symbol: { binance: 'BTCUSDT', coinbase: 'BTC-USD', kraken: 'XBTUSD' },
      sources: ['binance', 'coinbase', 'kraken'],
    },
    {
      id: 'ETH/USD',
      symbol: { binance: 'ETHUSDT', coinbase: 'ETH-USD', kraken: 'ETHUSD' },
      sources: ['binance', 'coinbase', 'kraken'],
    },
    {
      id: 'SOL/USD',
      symbol: { binance: 'SOLUSDT', coinbase: 'SOL-USD', kraken: 'SOLUSD' },
      sources: ['binance', 'coinbase', 'kraken'],
    },
  ],
};

// Try to load from config file if exists
try {
  const configPath = process.env.SIGNER_CONFIG || resolve(__dirname, '../config.yaml');
  const configFile = readFileSync(configPath, 'utf-8');
  signerConfig = YAML.parse(configFile);
} catch (error) {
  // Use default config
}

export const config = {
  // Signer keypair
  signerKeypair: process.env.SIGNER_KEYPAIR || './keys/signer.json',
  
  // Relayer URLs
  relayerUrls: (process.env.RELAYER_URLS || 'http://localhost:7777')
    .split(',')
    .map(url => url.trim()),
  
  // Timing
  intervalMs: parseInt(process.env.SIGNER_INTERVAL_MS || '5000'),
  
  // Assets configuration
  assets: signerConfig.assets || [],
};

