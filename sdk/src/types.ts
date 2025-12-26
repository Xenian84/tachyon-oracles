import { PublicKey } from '@solana/web3.js';
import BN from 'bn.js';

/**
 * Configuration account data
 */
export interface ConfigAccount {
  admin: PublicKey;
  updateFeeLamports: BN;
  relayerCutBps: number;
  minPublishers: number;
  maxAgeSec: number;
  assetCount: number;
  bump: number;
}

/**
 * Publisher account data
 */
export interface PublisherAccount {
  publisher: PublicKey;
  stakedAmount: BN;
  isActive: boolean;
  bump: number;
}

/**
 * Price feed account data
 */
export interface PriceFeedAccount {
  assetId: Buffer;
  priceI64: BN;
  confI64: BN;
  publishTime: BN;
  lastUpdateSlot: BN;
  bump: number;
}

/**
 * Signed message from a publisher
 */
export interface SignedMessage {
  publisher: PublicKey;
  assetIdHash: Buffer;
  priceI64: BN;
  confI64: BN;
  publishTime: BN;
  signature: Buffer;
}

/**
 * Human-readable price feed data
 */
export interface PriceFeed {
  assetId: string;
  price: number;
  confidence: number;
  publishTime: Date;
  lastUpdateSlot: number;
  staleness: number; // seconds since publish
}

/**
 * Oracle configuration
 */
export interface OracleConfig {
  admin: string;
  updateFeeLamports: number;
  relayerCutBps: number;
  minPublishers: number;
  maxAgeSec: number;
  assetCount: number;
}

