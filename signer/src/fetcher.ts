import axios from 'axios';
import { logger } from './logger';

export class PriceFetcher {
  private timeout = 5000;
  
  async fetchPrice(source: string, symbol: any): Promise<number | null> {
    try {
      switch (source) {
        case 'binance':
          return await this.fetchBinance(symbol.binance || symbol);
        case 'coinbase':
          return await this.fetchCoinbase(symbol.coinbase || symbol);
        case 'kraken':
          return await this.fetchKraken(symbol.kraken || symbol);
        default:
          logger.warn(`Unknown source: ${source}`);
          return null;
      }
    } catch (error) {
      logger.debug(`Error fetching from ${source}:`, error);
      return null;
    }
  }
  
  private async fetchBinance(symbol: string): Promise<number | null> {
    const response = await axios.get(
      `https://api.binance.com/api/v3/ticker/price?symbol=${symbol}`,
      { timeout: this.timeout }
    );
    
    if (response.data && response.data.price) {
      return parseFloat(response.data.price);
    }
    
    return null;
  }
  
  private async fetchCoinbase(symbol: string): Promise<number | null> {
    const response = await axios.get(
      `https://api.coinbase.com/v2/exchange-rates?currency=${symbol.split('-')[0]}`,
      { timeout: this.timeout }
    );
    
    if (response.data && response.data.data && response.data.data.rates) {
      const targetCurrency = symbol.split('-')[1] || 'USD';
      const rate = response.data.data.rates[targetCurrency];
      if (rate) {
        return parseFloat(rate);
      }
    }
    
    // Fallback to pro API
    try {
      const proResponse = await axios.get(
        `https://api.exchange.coinbase.com/products/${symbol}/ticker`,
        { timeout: this.timeout }
      );
      
      if (proResponse.data && proResponse.data.price) {
        return parseFloat(proResponse.data.price);
      }
    } catch (error) {
      // Ignore fallback error
    }
    
    return null;
  }
  
  private async fetchKraken(symbol: string): Promise<number | null> {
    const response = await axios.get(
      `https://api.kraken.com/0/public/Ticker?pair=${symbol}`,
      { timeout: this.timeout }
    );
    
    if (response.data && response.data.result) {
      const pairs = Object.keys(response.data.result);
      if (pairs.length > 0) {
        const pairData = response.data.result[pairs[0]];
        if (pairData && pairData.c && pairData.c[0]) {
          return parseFloat(pairData.c[0]);
        }
      }
    }
    
    return null;
  }
}

