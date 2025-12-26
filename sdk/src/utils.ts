import { PublicKey } from '@solana/web3.js';
import { createHash } from 'crypto';
import { PROGRAM_ID, SEEDS, PRICE_PRECISION } from './constants';
import BN from 'bn.js';

/**
 * Hash an asset ID string to get the canonical 32-byte identifier
 */
export function hashAssetId(assetId: string): Buffer {
  const hash = createHash('sha256');
  hash.update(assetId);
  return hash.digest();
}

/**
 * Derive the config PDA
 */
export function getConfigPDA(): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [SEEDS.CONFIG],
    PROGRAM_ID
  );
}

/**
 * Derive a publisher PDA
 */
export function getPublisherPDA(publisher: PublicKey): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [SEEDS.PUBLISHER, publisher.toBuffer()],
    PROGRAM_ID
  );
}

/**
 * Derive a price feed PDA
 */
export function getPriceFeedPDA(assetIdString: string): [PublicKey, number] {
  const assetIdHash = hashAssetId(assetIdString);
  return PublicKey.findProgramAddressSync(
    [SEEDS.FEED, assetIdHash],
    PROGRAM_ID
  );
}

/**
 * Derive the fee vault PDA
 */
export function getFeeVaultPDA(): [PublicKey, number] {
  return PublicKey.findProgramAddressSync(
    [SEEDS.FEE_VAULT],
    PROGRAM_ID
  );
}

/**
 * Convert a price from fixed-point i64 to human-readable number
 */
export function priceFromI64(priceI64: BN): number {
  return priceI64.toNumber() / PRICE_PRECISION;
}

/**
 * Convert a human-readable price to fixed-point i64
 */
export function priceToI64(price: number): BN {
  return new BN(Math.round(price * PRICE_PRECISION));
}

/**
 * Get canonical message bytes for signing
 */
export function getMessageBytes(
  assetIdHash: Buffer,
  priceI64: BN,
  confI64: BN,
  publishTime: BN
): Buffer {
  const buffer = Buffer.alloc(56);
  assetIdHash.copy(buffer, 0);
  priceI64.toArrayLike(Buffer, 'le', 8).copy(buffer, 32);
  confI64.toArrayLike(Buffer, 'le', 8).copy(buffer, 40);
  publishTime.toArrayLike(Buffer, 'le', 8).copy(buffer, 48);
  return buffer;
}

/**
 * Calculate staleness in seconds
 */
export function calculateStaleness(publishTime: BN): number {
  const now = Math.floor(Date.now() / 1000);
  return now - publishTime.toNumber();
}

