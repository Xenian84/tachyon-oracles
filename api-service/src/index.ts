import express, { Request, Response, NextFunction } from 'express';
import cors from 'cors';
import rateLimit from 'express-rate-limit';
import * as dotenv from 'dotenv';
import * as fs from 'fs';
import * as path from 'path';
import { exec } from 'child_process';
import { promisify } from 'util';
import { Connection, PublicKey } from '@solana/web3.js';
import { createLogger, format, transports } from 'winston';

dotenv.config();

const execAsync = promisify(exec);

// Configuration
const PORT = parseInt(process.env.API_PORT || '7171');
const HOST = process.env.API_HOST || '0.0.0.0';
const API_KEY = process.env.API_KEY || '';
const API_MODE = process.env.API_MODE || 'readonly';
const ALLOWED_IPS = process.env.ALLOWED_IPS?.split(',').filter(ip => ip.trim()) || [];
const PROJECT_PATH = process.env.ORACLE_PROJECT_PATH || '/root/tachyon-oracles';
const RPC_URL = process.env.RPC_URL || 'https://rpc.mainnet.x1.xyz';
const PROGRAM_ID = process.env.PROGRAM_ID || 'TACH9r2uZzoFM6daofesADjeDn9NqB1pKFWP5mfByb1';
const MAX_LOG_LINES = parseInt(process.env.MAX_LOG_LINES || '50');

// Logger
const logger = createLogger({
  level: process.env.LOG_LEVEL || 'info',
  format: format.combine(
    format.timestamp(),
    format.json()
  ),
  transports: [
    new transports.Console({
      format: format.combine(
        format.colorize(),
        format.simple()
      )
    }),
    new transports.File({ filename: path.join(PROJECT_PATH, 'logs/api-service.log') })
  ]
});

// Express app
const app = express();

// Middleware
app.use(cors());
app.use(express.json());

// Rate limiting
const limiter = rateLimit({
  windowMs: parseInt(process.env.RATE_LIMIT_WINDOW_MS || '60000'),
  max: parseInt(process.env.RATE_LIMIT_MAX_REQUESTS || '30'),
  message: 'Too many requests from this IP, please try again later.'
});

app.use(limiter);

// IP whitelist middleware
const ipWhitelist = (req: Request, res: Response, next: NextFunction) => {
  if (ALLOWED_IPS.length === 0) {
    return next();
  }

  const clientIp = req.ip || req.socket.remoteAddress || '';
  
  if (ALLOWED_IPS.includes(clientIp)) {
    return next();
  }

  logger.warn(`Blocked request from unauthorized IP: ${clientIp}`);
  return res.status(403).json({ error: 'Forbidden: IP not whitelisted' });
};

app.use(ipWhitelist);

// API key authentication middleware
const authenticate = (req: Request, res: Response, next: NextFunction) => {
  const authHeader = req.headers.authorization;
  
  if (!authHeader || !authHeader.startsWith('Bearer ')) {
    logger.warn(`Unauthorized request from ${req.ip}`);
    return res.status(401).json({ error: 'Unauthorized: Missing or invalid API key' });
  }

  const token = authHeader.substring(7);
  
  if (token !== API_KEY) {
    logger.warn(`Invalid API key attempt from ${req.ip}`);
    return res.status(401).json({ error: 'Unauthorized: Invalid API key' });
  }

  next();
};

app.use('/api', authenticate);

// Helper functions
async function getServiceStatus(serviceName: string): Promise<boolean> {
  try {
    const { stdout } = await execAsync(`ps aux | grep "${serviceName}/dist/index.js" | grep -v grep`);
    return stdout.trim().length > 0;
  } catch {
    return false;
  }
}

async function getOracleStatus() {
  try {
    const signerRunning = await getServiceStatus('signer');
    const relayerRunning = await getServiceStatus('relayer');

    const connection = new Connection(RPC_URL, 'confirmed');
    const programId = new PublicKey(PROGRAM_ID);
    const [configPda] = PublicKey.findProgramAddressSync([Buffer.from('config')], programId);
    
    const accountInfo = await connection.getAccountInfo(configPda);
    const feedCount = accountInfo ? accountInfo.data.readUInt16LE(55) : 0;

    const publisherAccounts = await connection.getProgramAccounts(programId, {
      filters: [{ dataSize: 50 }]
    });

    return {
      services: {
        signer: signerRunning ? 'running' : 'stopped',
        relayer: relayerRunning ? 'running' : 'stopped'
      },
      network: {
        feeds: feedCount,
        publishers: publisherAccounts.length
      },
      rpc: RPC_URL,
      timestamp: Date.now()
    };
  } catch (error: any) {
    logger.error('Error getting oracle status:', error);
    throw error;
  }
}

// Asset ID to trading pair mapping
const ASSET_PAIRS: Record<string, string> = {
  "7b4c9651c426361ed0e6bd9a9b3e70d71ec9507686a12b899c50c1faba8db94d": "BTC/USD",
  "d3ab6e05c4721ca257dc0c7bf211254a85ad26a586561fc9d0f1695ce8f8b083": "ETH/USD",
  "2efdfc62e56a3c17d9ac35a4259b14917250ee3937edc13bbc68b262c37725c1": "SOL/USD",
  "9a4c7d1476b676855648789959e9db2bb3c36e9bcda1c1f214bfff7b4123b2dd": "AVAX/USD",
  "fa5f4fda3247f29826f7abad0d0fc3de785a2f85f181112cc08ed42c3cb53864": "MATIC/USD",
  "6fbed19f44f4a783f5e04c0f0354bf981d355440c0db410fd5b7624f7a11eba5": "BNB/USD",
  "f1e97e7a804817dee13c858338fc92ab4a9f41efc886d6c04c37161512e14448": "XRP/USD",
  "50cd6650c96bf3c016e7ce6acd4659cb6fc648e091813433f17ed75842833993": "ADA/USD",
  "6c8fdf98d5c768238ff66b2a2bf807ffbe4536a98d8c9cec998832f42313eced": "DOT/USD",
};

async function getPriceFeeds() {
  try {
    const connection = new Connection(RPC_URL, 'confirmed');
    const programId = new PublicKey(PROGRAM_ID);
    
    // Get all feed accounts (size 73 bytes)
    const feedAccounts = await connection.getProgramAccounts(programId, {
      filters: [{ dataSize: 73 }]
    });

    const feeds = [];
    for (const acc of feedAccounts) {
      try {
        // Feed account structure: 8 bytes discriminator + 32 bytes asset_id + rest
        const assetIdHash = acc.account.data.slice(8, 40);
        const assetIdHex = assetIdHash.toString('hex');
        
        // Read price data
        const price = acc.account.data.readBigInt64LE(40);
        const conf = acc.account.data.readBigInt64LE(48);
        const timestamp = acc.account.data.readBigInt64LE(56);
        
        feeds.push({
          pair: ASSET_PAIRS[assetIdHex] || 'UNKNOWN',
          assetId: assetIdHex,
          pda: acc.pubkey.toBase58(),
          price: price.toString(),
          confidence: conf.toString(),
          timestamp: timestamp.toString()
        });
      } catch (err) {
        logger.warn('Error parsing feed account:', err);
      }
    }

    // Sort by pair name
    feeds.sort((a, b) => a.pair.localeCompare(b.pair));

    return feeds;
  } catch (error: any) {
    logger.error('Error getting price feeds:', error);
    throw error;
  }
}

async function getPublishers() {
  try {
    const connection = new Connection(RPC_URL, 'confirmed');
    const programId = new PublicKey(PROGRAM_ID);
    
    const accounts = await connection.getProgramAccounts(programId, {
      filters: [{ dataSize: 50 }]
    });

    const publishers = [];
    for (const acc of accounts) {
      const pubkey = new PublicKey(acc.account.data.slice(8, 40));
      const isActive = acc.account.data[48] === 1;
      
      publishers.push({
        pubkey: pubkey.toBase58(),
        pda: acc.pubkey.toBase58(),
        active: isActive
      });
    }

    return publishers;
  } catch (error: any) {
    logger.error('Error getting publishers:', error);
    throw error;
  }
}

async function getLogs(service: string, lines: number = MAX_LOG_LINES) {
  try {
    const logFile = path.join(PROJECT_PATH, 'logs', `${service}.log`);
    
    if (!fs.existsSync(logFile)) {
      return { logs: '', lines: 0 };
    }

    const { stdout } = await execAsync(`tail -n ${lines} ${logFile}`);
    const logLines = stdout.split('\n').filter(line => line.trim());
    
    return {
      logs: logLines.join('\n'),
      lines: logLines.length
    };
  } catch (error: any) {
    logger.error(`Error getting ${service} logs:`, error);
    throw error;
  }
}

// API Routes

// Health check
app.get('/health', (req, res) => {
  res.json({ status: 'ok', timestamp: Date.now() });
});

// Get oracle status
app.get('/api/status', async (req, res) => {
  try {
    const status = await getOracleStatus();
    logger.info(`Status check from ${req.ip}`);
    res.json(status);
  } catch (error: any) {
    logger.error('Status check failed:', error);
    res.status(500).json({ error: 'Failed to get oracle status', message: error.message });
  }
});

// Get price feeds
app.get('/api/feeds', async (req, res) => {
  try {
    const feeds = await getPriceFeeds();
    logger.info(`Feeds check from ${req.ip}`);
    res.json({ feeds, count: feeds.length });
  } catch (error: any) {
    logger.error('Feeds check failed:', error);
    res.status(500).json({ error: 'Failed to get price feeds', message: error.message });
  }
});

// Get publishers
app.get('/api/publishers', async (req, res) => {
  try {
    const publishers = await getPublishers();
    logger.info(`Publishers check from ${req.ip}`);
    res.json({ publishers, count: publishers.length });
  } catch (error: any) {
    logger.error('Publishers check failed:', error);
    res.status(500).json({ error: 'Failed to get publishers', message: error.message });
  }
});

// Get logs (monitoring mode or higher)
app.get('/api/logs/:service', async (req, res) => {
  if (API_MODE === 'readonly') {
    return res.status(403).json({ error: 'Logs access disabled in readonly mode' });
  }

  const { service } = req.params;
  
  if (!['signer', 'relayer'].includes(service)) {
    return res.status(400).json({ error: 'Invalid service name. Use: signer or relayer' });
  }

  try {
    const lines = parseInt(req.query.lines as string) || MAX_LOG_LINES;
    const logs = await getLogs(service, Math.min(lines, MAX_LOG_LINES));
    logger.info(`Logs request for ${service} from ${req.ip}`);
    res.json(logs);
  } catch (error: any) {
    logger.error(`Logs request failed for ${service}:`, error);
    res.status(500).json({ error: 'Failed to get logs', message: error.message });
  }
});

// Service control (full mode only)
app.post('/api/services/:action/:service', async (req, res) => {
  if (API_MODE !== 'full') {
    return res.status(403).json({ error: 'Service control disabled. API mode is not set to "full"' });
  }

  const { action, service } = req.params;
  
  if (!['signer', 'relayer'].includes(service)) {
    return res.status(400).json({ error: 'Invalid service name. Use: signer or relayer' });
  }

  if (!['start', 'stop', 'restart'].includes(action)) {
    return res.status(400).json({ error: 'Invalid action. Use: start, stop, or restart' });
  }

  try {
    logger.warn(`Service control: ${action} ${service} from ${req.ip}`);

    let command = '';
    
    switch (action) {
      case 'start':
        command = `cd ${PROJECT_PATH}/${service} && nohup node dist/index.js >> ../logs/${service}.log 2>&1 &`;
        break;
      case 'stop':
        command = `pkill -f "${service}/dist/index.js"`;
        break;
      case 'restart':
        command = `pkill -f "${service}/dist/index.js"; sleep 2; cd ${PROJECT_PATH}/${service} && nohup node dist/index.js >> ../logs/${service}.log 2>&1 &`;
        break;
    }

    await execAsync(command);
    
    // Wait a bit for service to start/stop
    await new Promise(resolve => setTimeout(resolve, 2000));
    
    const isRunning = await getServiceStatus(service);
    
    res.json({
      success: true,
      action,
      service,
      status: isRunning ? 'running' : 'stopped'
    });
  } catch (error: any) {
    logger.error(`Service control failed (${action} ${service}):`, error);
    res.status(500).json({ error: 'Service control failed', message: error.message });
  }
});

// Get API info
app.get('/api/info', (req, res) => {
  res.json({
    version: '1.0.0',
    mode: API_MODE,
    features: {
      status: true,
      feeds: true,
      publishers: true,
      logs: API_MODE !== 'readonly',
      serviceControl: API_MODE === 'full'
    },
    rateLimit: {
      windowMs: parseInt(process.env.RATE_LIMIT_WINDOW_MS || '60000'),
      maxRequests: parseInt(process.env.RATE_LIMIT_MAX_REQUESTS || '30')
    }
  });
});

// Error handler
app.use((err: Error, req: Request, res: Response, next: NextFunction) => {
  logger.error('Unhandled error:', err);
  res.status(500).json({ error: 'Internal server error' });
});

// Start server
app.listen(PORT, HOST, () => {
  logger.info(`🚀 Tachyon Oracle API Service running on ${HOST}:${PORT}`);
  logger.info(`📡 Mode: ${API_MODE}`);
  logger.info(`🔒 IP Whitelist: ${ALLOWED_IPS.length > 0 ? ALLOWED_IPS.join(', ') : 'Disabled (all IPs allowed)'}`);
  logger.info(`📊 RPC: ${RPC_URL}`);
  logger.info(`📁 Project: ${PROJECT_PATH}`);
  
  if (!API_KEY || API_KEY === 'your-generated-api-key-here') {
    logger.warn('⚠️  WARNING: API_KEY not set! Generate one with: openssl rand -hex 32');
  }
});

