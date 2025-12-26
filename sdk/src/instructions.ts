import {
  Connection,
  PublicKey,
  SystemProgram,
  TransactionInstruction,
  SYSVAR_INSTRUCTIONS_PUBKEY,
} from '@solana/web3.js';
import * as borsh from '@coral-xyz/borsh';
import BN from 'bn.js';
import { PROGRAM_ID } from './constants';
import {
  getConfigPDA,
  getPublisherPDA,
  getPriceFeedPDA,
  hashAssetId,
} from './utils';
import { SignedMessage } from './types';

/**
 * Create initialize_config instruction
 */
export function createInitializeConfigInstruction(
  admin: PublicKey,
  updateFeeLamports: BN,
  relayerCutBps: number,
  minPublishers: number,
  maxAgeSec: number
): TransactionInstruction {
  const [configPDA] = getConfigPDA();
  
  const data = Buffer.concat([
    Buffer.from([175, 175, 109, 31, 13, 152, 155, 237]), // discriminator for initialize_config
    updateFeeLamports.toArrayLike(Buffer, 'le', 8),
    Buffer.from([relayerCutBps & 0xff, (relayerCutBps >> 8) & 0xff]),
    Buffer.from([minPublishers]),
    Buffer.from([
      maxAgeSec & 0xff,
      (maxAgeSec >> 8) & 0xff,
      (maxAgeSec >> 16) & 0xff,
      (maxAgeSec >> 24) & 0xff,
    ]),
  ]);
  
  return new TransactionInstruction({
    keys: [
      { pubkey: configPDA, isSigner: false, isWritable: true },
      { pubkey: admin, isSigner: true, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    programId: PROGRAM_ID,
    data,
  });
}

/**
 * Create add_asset instruction
 */
export function createAddAssetInstruction(
  admin: PublicKey,
  assetIdString: string
): TransactionInstruction {
  const [configPDA] = getConfigPDA();
  const [feedPDA] = getPriceFeedPDA(assetIdString);
  
  const assetIdBytes = Buffer.from(assetIdString, 'utf-8');
  const lengthPrefix = Buffer.alloc(4);
  lengthPrefix.writeUInt32LE(assetIdBytes.length, 0);
  
  const data = Buffer.concat([
    Buffer.from([51, 210, 245, 186, 182, 43, 243, 24]), // discriminator for add_asset
    lengthPrefix,
    assetIdBytes,
  ]);
  
  return new TransactionInstruction({
    keys: [
      { pubkey: configPDA, isSigner: false, isWritable: true },
      { pubkey: feedPDA, isSigner: false, isWritable: true },
      { pubkey: admin, isSigner: true, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    programId: PROGRAM_ID,
    data,
  });
}

/**
 * Create register_publisher instruction
 */
export function createRegisterPublisherInstruction(
  publisher: PublicKey
): TransactionInstruction {
  const [publisherPDA] = getPublisherPDA(publisher);
  
  const data = Buffer.from([211, 117, 225, 234, 172, 228, 4, 222]); // discriminator
  
  return new TransactionInstruction({
    keys: [
      { pubkey: publisherPDA, isSigner: false, isWritable: true },
      { pubkey: publisher, isSigner: true, isWritable: true },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    programId: PROGRAM_ID,
    data,
  });
}

/**
 * Create set_publisher_status instruction
 */
export function createSetPublisherStatusInstruction(
  admin: PublicKey,
  publisher: PublicKey,
  isActive: boolean
): TransactionInstruction {
  const [configPDA] = getConfigPDA();
  const [publisherPDA] = getPublisherPDA(publisher);
  
  const data = Buffer.concat([
    Buffer.from([254, 169, 123, 123, 60, 233, 57, 84]), // discriminator
    Buffer.from([isActive ? 1 : 0]),
  ]);
  
  return new TransactionInstruction({
    keys: [
      { pubkey: configPDA, isSigner: false, isWritable: false },
      { pubkey: publisherPDA, isSigner: false, isWritable: true },
      { pubkey: admin, isSigner: true, isWritable: false },
    ],
    programId: PROGRAM_ID,
    data,
  });
}

/**
 * Create post_update instruction
 */
export function createPostUpdateInstruction(
  payer: PublicKey,
  assetIdString: string,
  messages: SignedMessage[]
): TransactionInstruction {
  const [configPDA] = getConfigPDA();
  const [feedPDA] = getPriceFeedPDA(assetIdString);
  const assetIdHash = hashAssetId(assetIdString);
  
  // Serialize messages
  const messagesData: Buffer[] = [];
  for (const msg of messages) {
    const msgBuffer = Buffer.alloc(32 + 32 + 8 + 8 + 8 + 64);
    msg.publisher.toBuffer().copy(msgBuffer, 0);
    msg.assetIdHash.copy(msgBuffer, 32);
    msg.priceI64.toArrayLike(Buffer, 'le', 8).copy(msgBuffer, 64);
    msg.confI64.toArrayLike(Buffer, 'le', 8).copy(msgBuffer, 72);
    msg.publishTime.toArrayLike(Buffer, 'le', 8).copy(msgBuffer, 80);
    msg.signature.copy(msgBuffer, 88);
    messagesData.push(msgBuffer);
  }
  
  const messagesCount = Buffer.alloc(4);
  messagesCount.writeUInt32LE(messages.length, 0);
  
  const data = Buffer.concat([
    Buffer.from([242, 193, 106, 181, 190, 108, 4, 241]), // discriminator for post_update
    assetIdHash,
    messagesCount,
    ...messagesData,
  ]);
  
  return new TransactionInstruction({
    keys: [
      { pubkey: configPDA, isSigner: false, isWritable: false },
      { pubkey: feedPDA, isSigner: false, isWritable: true },
      { pubkey: payer, isSigner: true, isWritable: true },
      { pubkey: SYSVAR_INSTRUCTIONS_PUBKEY, isSigner: false, isWritable: false },
      { pubkey: SystemProgram.programId, isSigner: false, isWritable: false },
    ],
    programId: PROGRAM_ID,
    data,
  });
}

