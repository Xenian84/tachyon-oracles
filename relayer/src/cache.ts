import { PublicKey } from '@solana/web3.js';
import BN from 'bn.js';
import { SignedMessage } from '@tachyon-oracles/sdk';

interface CachedMessage {
  publisherPubkey: string;
  assetIdHash: string;
  priceI64: string;
  confI64: string;
  publishTime: string;
  signatureBase64: string;
  receivedAt: number;
}

export class MessageCache {
  // Map: assetIdHash -> publisherPubkey -> latest message
  private cache: Map<string, Map<string, CachedMessage>> = new Map();
  
  // Track last submission time per asset
  private lastSubmission: Map<string, number> = new Map();
  
  addMessage(message: CachedMessage) {
    const { assetIdHash, publisherPubkey } = message;
    
    if (!this.cache.has(assetIdHash)) {
      this.cache.set(assetIdHash, new Map());
    }
    
    const assetCache = this.cache.get(assetIdHash)!;
    
    // Only update if newer
    const existing = assetCache.get(publisherPubkey);
    if (!existing || new BN(message.publishTime).gt(new BN(existing.publishTime))) {
      assetCache.set(publisherPubkey, message);
    }
  }
  
  getBundlesForSubmission(minPublishers: number): Array<{
    assetId: string;
    messages: SignedMessage[];
  }> {
    const bundles: Array<{ assetId: string; messages: SignedMessage[] }> = [];
    const now = Date.now();
    
    for (const [assetIdHash, publisherMessages] of this.cache.entries()) {
      // Filter fresh messages
      const freshMessages: SignedMessage[] = [];
      
      for (const [publisherPubkey, cachedMsg] of publisherMessages.entries()) {
        // Skip if too old (received more than 30 seconds ago)
        if (now - cachedMsg.receivedAt > 30000) {
          continue;
        }
        
        try {
          freshMessages.push({
            publisher: new PublicKey(publisherPubkey),
            assetIdHash: Buffer.from(cachedMsg.assetIdHash, 'hex'),
            priceI64: new BN(cachedMsg.priceI64),
            confI64: new BN(cachedMsg.confI64),
            publishTime: new BN(cachedMsg.publishTime),
            signature: Buffer.from(cachedMsg.signatureBase64, 'base64'),
          });
        } catch (error) {
          console.error('Error parsing message:', error);
        }
      }
      
      // Check if we have quorum
      if (freshMessages.length >= minPublishers) {
        // Check if we should submit (avoid double posting)
        const lastSubmit = this.lastSubmission.get(assetIdHash) || 0;
        if (now - lastSubmit > 5000) { // At least 5 seconds between submissions
          bundles.push({
            assetId: assetIdHash,
            messages: freshMessages,
          });
        }
      }
    }
    
    return bundles;
  }
  
  markSubmitted(assetIdHash: string) {
    this.lastSubmission.set(assetIdHash, Date.now());
  }
  
  getStatus(): any {
    const status: any = {};
    
    for (const [assetIdHash, publisherMessages] of this.cache.entries()) {
      status[assetIdHash] = {
        publisherCount: publisherMessages.size,
        publishers: Array.from(publisherMessages.keys()),
      };
    }
    
    return status;
  }
  
  getPublishers(): any {
    const publishers = new Set<string>();
    
    for (const publisherMessages of this.cache.values()) {
      for (const publisherPubkey of publisherMessages.keys()) {
        publishers.add(publisherPubkey);
      }
    }
    
    return {
      count: publishers.size,
      publishers: Array.from(publishers),
    };
  }
  
  // Cleanup old messages periodically
  cleanup(maxAgeMs: number = 60000) {
    const now = Date.now();
    
    for (const [assetIdHash, publisherMessages] of this.cache.entries()) {
      for (const [publisherPubkey, message] of publisherMessages.entries()) {
        if (now - message.receivedAt > maxAgeMs) {
          publisherMessages.delete(publisherPubkey);
        }
      }
      
      // Remove empty asset caches
      if (publisherMessages.size === 0) {
        this.cache.delete(assetIdHash);
      }
    }
  }
}

