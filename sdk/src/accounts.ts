import { Connection, PublicKey, AccountInfo } from '@solana/web3.js';
import * as borsh from '@coral-xyz/borsh';
import BN from 'bn.js';
import {
  ConfigAccount,
  PublisherAccount,
  PriceFeedAccount,
  PriceFeed,
  OracleConfig,
} from './types';
import {
  getConfigPDA,
  getPublisherPDA,
  getPriceFeedPDA,
  priceFromI64,
  calculateStaleness,
} from './utils';

/**
 * Borsh schema for ConfigAccount
 */
const configAccountSchema: any = {
  struct: {
    admin: { array: { type: 'u8', len: 32 } },
    updateFeeLamports: 'u64',
    relayerCutBps: 'u16',
    minPublishers: 'u8',
    maxAgeSec: 'u32',
    assetCount: 'u16',
    bump: 'u8',
  },
};

/**
 * Borsh schema for PublisherAccount
 */
const publisherAccountSchema: any = {
  struct: {
    publisher: { array: { type: 'u8', len: 32 } },
    stakedAmount: 'u64',
    isActive: 'u8',
    bump: 'u8',
  },
};

/**
 * Borsh schema for PriceFeedAccount
 */
const priceFeedAccountSchema: any = {
  struct: {
    assetId: { array: { type: 'u8', len: 32 } },
    priceI64: 'i64',
    confI64: 'i64',
    publishTime: 'i64',
    lastUpdateSlot: 'u64',
    bump: 'u8',
  },
};

/**
 * Fetch and decode the config account
 */
export async function fetchConfig(
  connection: Connection
): Promise<OracleConfig | null> {
  const [configPDA] = getConfigPDA();
  const accountInfo = await connection.getAccountInfo(configPDA);
  
  if (!accountInfo) {
    return null;
  }
  
  // Skip 8-byte discriminator
  const data = accountInfo.data.slice(8);
  const decoded: any = borsh.deserialize(configAccountSchema, data);
  
  return {
    admin: new PublicKey(decoded.admin).toBase58(),
    updateFeeLamports: decoded.updateFeeLamports.toNumber(),
    relayerCutBps: decoded.relayerCutBps,
    minPublishers: decoded.minPublishers,
    maxAgeSec: decoded.maxAgeSec,
    assetCount: decoded.assetCount,
  };
}

/**
 * Fetch and decode a publisher account
 */
export async function fetchPublisher(
  connection: Connection,
  publisher: PublicKey
): Promise<PublisherAccount | null> {
  const [publisherPDA] = getPublisherPDA(publisher);
  const accountInfo = await connection.getAccountInfo(publisherPDA);
  
  if (!accountInfo) {
    return null;
  }
  
  // Skip 8-byte discriminator
  const data = accountInfo.data.slice(8);
  const decoded: any = borsh.deserialize(publisherAccountSchema, data);
  
  return {
    publisher: new PublicKey(decoded.publisher),
    stakedAmount: new BN(decoded.stakedAmount),
    isActive: decoded.isActive === 1,
    bump: decoded.bump,
  };
}

/**
 * Fetch and decode a price feed account
 */
export async function fetchPriceFeed(
  connection: Connection,
  assetId: string
): Promise<PriceFeed | null> {
  const [feedPDA] = getPriceFeedPDA(assetId);
  const accountInfo = await connection.getAccountInfo(feedPDA);
  
  if (!accountInfo) {
    return null;
  }
  
  // Skip 8-byte discriminator
  const data = accountInfo.data.slice(8);
  const decoded: any = borsh.deserialize(priceFeedAccountSchema, data);
  
  const publishTime = new BN(decoded.publishTime);
  
  return {
    assetId,
    price: priceFromI64(new BN(decoded.priceI64)),
    confidence: priceFromI64(new BN(decoded.confI64)),
    publishTime: new Date(publishTime.toNumber() * 1000),
    lastUpdateSlot: decoded.lastUpdateSlot.toNumber(),
    staleness: calculateStaleness(publishTime),
  };
}

/**
 * Fetch raw price feed account data
 */
export async function fetchPriceFeedRaw(
  connection: Connection,
  assetId: string
): Promise<PriceFeedAccount | null> {
  const [feedPDA] = getPriceFeedPDA(assetId);
  const accountInfo = await connection.getAccountInfo(feedPDA);
  
  if (!accountInfo) {
    return null;
  }
  
  // Skip 8-byte discriminator
  const data = accountInfo.data.slice(8);
  const decoded: any = borsh.deserialize(priceFeedAccountSchema, data);
  
  return {
    assetId: Buffer.from(decoded.assetId),
    priceI64: new BN(decoded.priceI64),
    confI64: new BN(decoded.confI64),
    publishTime: new BN(decoded.publishTime),
    lastUpdateSlot: new BN(decoded.lastUpdateSlot),
    bump: decoded.bump,
  };
}

/**
 * Check if a price feed exists
 */
export async function priceFeedExists(
  connection: Connection,
  assetId: string
): Promise<boolean> {
  const [feedPDA] = getPriceFeedPDA(assetId);
  const accountInfo = await connection.getAccountInfo(feedPDA);
  return accountInfo !== null;
}

