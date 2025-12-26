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
 * Layout for ConfigAccount using buffer-layout
 */
const configAccountLayout = borsh.struct([
  borsh.publicKey('admin'),
  borsh.u64('updateFeeLamports'),
  borsh.u16('relayerCutBps'),
  borsh.u8('minPublishers'),
  borsh.u32('maxAgeSec'),
  borsh.u16('assetCount'),
  borsh.u8('bump'),
]);

/**
 * Layout for PublisherAccount
 */
const publisherAccountLayout = borsh.struct([
  borsh.publicKey('publisher'),
  borsh.u64('stakedAmount'),
  borsh.bool('isActive'),
  borsh.u8('bump'),
]);

/**
 * Layout for PriceFeedAccount
 */
const priceFeedAccountLayout = borsh.struct([
  borsh.array(borsh.u8(), 32, 'assetId'),
  borsh.i64('priceI64'),
  borsh.i64('confI64'),
  borsh.i64('publishTime'),
  borsh.u64('lastUpdateSlot'),
  borsh.u8('bump'),
]);

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
  const decoded = configAccountLayout.decode(Buffer.from(data));
  
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
  const decoded = publisherAccountLayout.decode(Buffer.from(data));
  
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
  const decoded = priceFeedAccountLayout.decode(Buffer.from(data));
  
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
  const decoded = priceFeedAccountLayout.decode(Buffer.from(data));
  
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

