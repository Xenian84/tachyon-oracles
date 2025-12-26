import { PublicKey } from '@solana/web3.js';

/**
 * Tachyon Oracles Program ID
 */
export const PROGRAM_ID = new PublicKey('TACH9r2uZzoFM6daofesADjeDn9NqB1pKFWP5mfByb1');

/**
 * Seed constants for PDA derivation
 */
export const SEEDS = {
  CONFIG: Buffer.from('config'),
  PUBLISHER: Buffer.from('publisher'),
  FEED: Buffer.from('feed'),
  FEE_VAULT: Buffer.from('fee_vault'),
} as const;

/**
 * Fixed-point precision (1e6)
 */
export const PRICE_PRECISION = 1_000_000;

/**
 * Maximum basis points (100%)
 */
export const MAX_BPS = 10_000;

