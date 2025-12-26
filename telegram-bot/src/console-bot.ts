import TelegramBot from 'node-telegram-bot-api';
import { Connection, PublicKey } from '@solana/web3.js';
import axios from 'axios';
import { exec } from 'child_process';
import { promisify } from 'util';
import * as dotenv from 'dotenv';
import * as fs from 'fs';
import * as path from 'path';

dotenv.config();

const execAsync = promisify(exec);

// Configuration
const BOT_TOKEN = process.env.TELEGRAM_BOT_TOKEN!;
const RPC_URL = process.env.RPC_URL || 'https://rpc.mainnet.x1.xyz';
const WS_URL = process.env.WS_URL || 'wss://rpc.mainnet.x1.xyz';
const PROGRAM_ID = process.env.PROGRAM_ID || 'TACH9r2uZzoFM6daofesADjeDn9NqB1pKFWP5mfByb1';
const PROJECT_PATH = process.env.ORACLE_PROJECT_PATH || '/root/tachyon-oracles';
const SOLANA_PATH = process.env.SOLANA_PATH || '/root/tachyon/target/release';

// Connection pool to reuse connections and reduce RPC load
class ConnectionPool {
  private connections: Map<number, { connection: Connection; lastUsed: number }> = new Map();
  private readonly maxAge = 5 * 60 * 1000; // 5 minutes
  private readonly cleanupInterval = 60 * 1000; // 1 minute

  constructor() {
    // Cleanup old connections periodically
    setInterval(() => this.cleanup(), this.cleanupInterval);
  }

  getConnection(userId: number): Connection {
    const cached = this.connections.get(userId);
    const now = Date.now();

    if (cached && now - cached.lastUsed < this.maxAge) {
      cached.lastUsed = now;
      return cached.connection;
    }

    // Create new connection with WebSocket for subscriptions
    const connection = new Connection(RPC_URL, {
      commitment: 'confirmed',
      wsEndpoint: WS_URL,
      // Reduce polling frequency
      confirmTransactionInitialTimeout: 60000,
    });

    this.connections.set(userId, { connection, lastUsed: now });
    return connection;
  }

  cleanup() {
    const now = Date.now();
    for (const [userId, { lastUsed }] of this.connections.entries()) {
      if (now - lastUsed > this.maxAge) {
        this.connections.delete(userId);
      }
    }
  }
}

const connectionPool = new ConnectionPool();

// User database with multi-profile support
interface ValidatorProfile {
  name: string;
  apiUrl: string;  // API service URL
  apiKey: string;  // API authentication key
}

interface UserConfig {
  userId: number;
  username?: string;
  registeredAt: number;
  activeProfile: number;  // Index of active profile
  profiles: ValidatorProfile[];
}

class UserDatabase {
  private dbPath: string;
  private users: Map<number, UserConfig>;

  constructor() {
    this.dbPath = path.join(__dirname, '../data/users.json');
    this.users = new Map();
    this.load();
  }

  private load() {
    try {
      if (fs.existsSync(this.dbPath)) {
        const data = JSON.parse(fs.readFileSync(this.dbPath, 'utf8'));
        this.users = new Map(Object.entries(data).map(([k, v]: any) => [parseInt(k), v]));
      } else {
        const dir = path.dirname(this.dbPath);
        if (!fs.existsSync(dir)) {
          fs.mkdirSync(dir, { recursive: true });
        }
      }
    } catch (error) {
      console.error('Error loading user database:', error);
    }
  }

  private save() {
    try {
      const data = Object.fromEntries(this.users);
      fs.writeFileSync(this.dbPath, JSON.stringify(data, null, 2));
    } catch (error) {
      console.error('Error saving user database:', error);
    }
  }

  getUser(userId: number): UserConfig | undefined {
    return this.users.get(userId);
  }

  setUser(userId: number, config: Partial<UserConfig>) {
    const existing = this.users.get(userId) || { 
      userId, 
      registeredAt: Date.now(),
      activeProfile: 0,
      profiles: []
    };
    // Ensure profiles array exists
    const updated = { ...existing, ...config };
    if (!updated.profiles) {
      updated.profiles = [];
    }
    this.users.set(userId, updated);
    this.save();
  }

  isRegistered(userId: number): boolean {
    const user = this.users.get(userId);
    return user !== undefined && user.profiles.length > 0;
  }

  getActiveProfile(userId: number): ValidatorProfile | undefined {
    const user = this.users.get(userId);
    if (!user || user.profiles.length === 0) return undefined;
    return user.profiles[user.activeProfile];
  }

  addProfile(userId: number, profile: ValidatorProfile) {
    const user = this.users.get(userId) || { 
      userId, 
      registeredAt: Date.now(),
      activeProfile: 0,
      profiles: []
    };
    user.profiles.push(profile);
    this.users.set(userId, user);
    this.save();
  }

  setActiveProfile(userId: number, profileIndex: number): boolean {
    const user = this.users.get(userId);
    if (!user || profileIndex < 0 || profileIndex >= user.profiles.length) {
      return false;
    }
    user.activeProfile = profileIndex;
    this.users.set(userId, user);
    this.save();
    return true;
  }

  deleteProfile(userId: number, profileIndex: number): boolean {
    const user = this.users.get(userId);
    if (!user || profileIndex < 0 || profileIndex >= user.profiles.length) {
      return false;
    }
    user.profiles.splice(profileIndex, 1);
    if (user.activeProfile >= user.profiles.length) {
      user.activeProfile = Math.max(0, user.profiles.length - 1);
    }
    this.users.set(userId, user);
    this.save();
    return true;
  }

  renameProfile(userId: number, profileIndex: number, newName: string): boolean {
    const user = this.users.get(userId);
    if (!user || profileIndex < 0 || profileIndex >= user.profiles.length) {
      return false;
    }
    user.profiles[profileIndex].name = newName;
    this.users.set(userId, user);
    this.save();
    return true;
  }
}

// Initialize
const bot = new TelegramBot(BOT_TOKEN, { polling: true });
const userDb = new UserDatabase();

// Helper to get connection for a user (uses connection pool)
function getConnection(userId: number): Connection {
  return connectionPool.getConnection(userId);
}

// Helper to call API service
async function callApi(userId: number, endpoint: string, method: string = 'GET'): Promise<any> {
  const profile = userDb.getActiveProfile(userId);
  
  if (!profile || !profile.apiUrl || !profile.apiKey) {
    throw new Error('API connection not configured. Use /add_profile to set up.');
  }
  
  const url = `${profile.apiUrl}${endpoint}`;
  
  try {
    const response = await axios({
      method,
      url,
      headers: {
        'Authorization': `Bearer ${profile.apiKey}`
      },
      timeout: 10000
    });
    return response.data;
  } catch (error: any) {
    if (error.response) {
      throw new Error(`API Error: ${error.response.status} - ${error.response.data?.error || error.response.statusText}`);
    } else if (error.request) {
      throw new Error('API Error: No response from server. Check if API service is running.');
    } else {
      throw new Error(`API Error: ${error.message}`);
    }
  }
}

// Console Functions (adapted from tachyon-console.sh)

async function getOracleStatus(userId: number) {
  try {
    const status = await callApi(userId, '/api/status');
    return {
      signerRunning: status.services?.signer === 'running',
      relayerRunning: status.services?.relayer === 'running',
      feedCount: status.network?.feeds || 0,
      publisherCount: status.network?.publishers || 0,
    };
  } catch (error: any) {
    return { error: error.message };
  }
}

async function testApiConnection(chatId: number, userId: number) {
  try {
    const status = await callApi(userId, '/api/status');
    bot.sendMessage(chatId, `
✅ *API Connection Successful!*

Services:
• Signer: ${status.services?.signer === 'running' ? '✅ Running' : '❌ Stopped'}
• Relayer: ${status.services?.relayer === 'running' ? '✅ Running' : '❌ Stopped'}

Network:
• Feeds: ${status.network?.feeds || 0}
• Publishers: ${status.network?.publishers || 0}

Your validator is connected and ready! 🚀
    `, { parse_mode: 'Markdown' });
  } catch (error: any) {
    bot.sendMessage(chatId, `
❌ *API Connection Failed*

Error: ${error.message}

Please check:
1. API service is running on validator
2. URL is correct
3. API key is valid
4. Firewall allows port 7171
    `, { parse_mode: 'Markdown' });
  }
}

async function getPriceFeeds(userId: number) {
  try {
    const result = await callApi(userId, '/api/feeds');
    return result.feeds || [];
  } catch (error: any) {
    return { error: error.message };
  }
}

async function getPublishers(userId: number) {
  try {
    const result = await callApi(userId, '/api/publishers');
    return result.publishers || [];
  } catch (error: any) {
    return { error: error.message };
  }
}

// Wallet balance not supported in API-only mode
async function getWalletBalance(userId: number) {
  return { error: 'Wallet balance checking not available via API. Check balance directly on your validator server.' };
}

// Bot Commands

// Helper function to create main menu keyboard
function getMainMenuKeyboard() {
  return {
    keyboard: [
      [{ text: '📊 Status' }, { text: '🗂️ Profiles' }],
      [{ text: '📈 Feeds' }, { text: '👥 Publishers' }],
      [{ text: '📝 Logs' }, { text: '❓ Help' }]
    ],
    resize_keyboard: true,
    one_time_keyboard: false
  };
}


function getProfileKeyboard() {
  return {
    keyboard: [
      [{ text: '📋 List Profiles' }, { text: '➕ Add Profile' }],
      [{ text: '🔧 Setup First Profile' }],
      [{ text: '◀️ Back to Menu' }]
    ],
    resize_keyboard: true,
    one_time_keyboard: false
  };
}

function getLogsKeyboard() {
  return {
    keyboard: [
      [{ text: '📝 Signer Logs' }, { text: '📝 Relayer Logs' }],
      [{ text: '◀️ Back to Menu' }]
    ],
    resize_keyboard: true,
    one_time_keyboard: false
  };
}

bot.onText(/\/start/, async (msg) => {
  const chatId = msg.chat.id;
  const userId = msg.from?.id!;
  const username = msg.from?.username;
  
  userDb.setUser(userId, { username });
  
  const welcomeMessage = `
🚀 *Tachyon Oracle Console Bot*

Welcome! Manage your Tachyon Oracle validators from Telegram.

✨ *NEW: Interactive Menu!*
Use the buttons below or type commands.

*Quick Start:*
1. Tap "🗂️ Profiles" to set up your first validator
2. Tap "📊 Status" to check your oracle
3. Use "⚙️ Services" to manage services

Tap any button below to get started! 👇
  `;
  
  bot.sendMessage(chatId, welcomeMessage, { 
    parse_mode: 'Markdown',
    reply_markup: getMainMenuKeyboard()
  });
});

// Handle button presses and setup flow
bot.on('message', async (msg) => {
  const chatId = msg.chat.id;
  const userId = msg.from?.id!;
  const text = msg.text?.trim();
  
  if (!text || text.startsWith('/')) return; // Ignore commands
  
  // Check if in setup flow first
  const state = userSetupState.get(userId);
  if (state) {
    // Handle setup conversation flow
    if (text === '/cancel') {
      userSetupState.delete(userId);
      bot.sendMessage(chatId, '❌ Setup cancelled.', { reply_markup: getMainMenuKeyboard() });
      return;
    }
    
    switch (state.step) {
      case 'name':
        state.profileData.name = text;
        state.step = 'apiurl';
        bot.sendMessage(chatId, `
Great! Now, what's the API service URL for this validator?

Example: \`http://209.159.154.102:7171\`

This is the URL where your validator's API service is running.
        `, { parse_mode: 'Markdown' });
        break;
        
      case 'apiurl':
        state.profileData.apiUrl = text;
        state.step = 'apikey';
        bot.sendMessage(chatId, `
What's the API key for this validator?

You can find it at:
• \`keys/api-key.txt\` in your project folder
• Or in the console: Service Manager → API Service Manager → Show API Key
        `, { parse_mode: 'Markdown' });
        break;
        
      case 'apikey':
        state.profileData.apiKey = text;
        
        // Save profile
        userDb.addProfile(userId, state.profileData as ValidatorProfile);
        userSetupState.delete(userId);
        
        const user = userDb.getUser(userId);
        const profileIndex = user!.profiles.length - 1;
        userDb.setActiveProfile(userId, profileIndex);
        
        bot.sendMessage(chatId, `
✅ *Profile Created Successfully!*

Name: *${state.profileData.name}*
URL: \`${state.profileData.apiUrl}\`

Testing connection...
        `, { 
          parse_mode: 'Markdown',
          reply_markup: getMainMenuKeyboard()
        });
        
        // Test the API connection
        testApiConnection(chatId, userId);
        break;
    }
    return; // Don't process as menu button
  }
  
  // Handle menu buttons - directly trigger actions
  switch (text) {
    case '📊 Status':
      handleStatus(msg);
      return;
    case '🗂️ Profiles':
      bot.sendMessage(chatId, 'Profile Management', { reply_markup: getProfileKeyboard() });
      return;
    case '📋 List Profiles':
      handleProfiles(msg);
      return;
    case '➕ Add Profile':
    case '🔧 Setup First Profile':
      handleAddProfile(msg);
      return;
    case '📈 Feeds':
      handleFeeds(msg);
      return;
    case '👥 Publishers':
      handlePublishers(msg);
      return;
    case '📝 Logs':
      bot.sendMessage(chatId, '📝 Service Logs', { reply_markup: getLogsKeyboard() });
      return;
    case '📝 Signer Logs':
      handleLogs(msg, 'signer');
      return;
    case '📝 Relayer Logs':
      handleLogs(msg, 'relayer');
      return;
    case '❓ Help':
      handleHelp(msg);
      return;
    case '◀️ Back to Menu':
      bot.sendMessage(chatId, '🏠 Main Menu', { reply_markup: getMainMenuKeyboard() });
      return;
    default:
      return; // Unknown button
  }
});

// Command handler functions
async function handleStatus(msg: TelegramBot.Message) {
  const chatId = msg.chat.id;
  const userId = msg.from?.id!;
  
  if (!userDb.isRegistered(userId)) {
    bot.sendMessage(chatId, '❌ Please setup first using /setup', {
      reply_markup: getProfileKeyboard()
    });
    return;
  }
  
  const profile = userDb.getActiveProfile(userId);
  bot.sendMessage(chatId, '⏳ Fetching oracle status...');
  
  const status = await getOracleStatus(userId);
  
  if (status.error) {
    bot.sendMessage(chatId, `❌ Error: ${status.error}`, {
      reply_markup: getMainMenuKeyboard()
    });
    return;
  }
  
  const message = `
📊 *Tachyon Oracle Status*
🏷️ Profile: *${profile?.name || 'Default'}*

*Services:*
Signer: ${status.signerRunning ? '✅ Running' : '❌ Stopped'}
Relayer: ${status.relayerRunning ? '✅ Running' : '❌ Stopped'}

*Network:*
Feeds: ${status.feedCount}
Publishers: ${status.publisherCount}

*RPC:* \`${RPC_URL}\`
  `;
  
  bot.sendMessage(chatId, message, { 
    parse_mode: 'Markdown',
    reply_markup: getMainMenuKeyboard()
  });
}

bot.onText(/\/status/, handleStatus);

async function handleFeeds(msg: TelegramBot.Message) {
  const chatId = msg.chat.id;
  const userId = msg.from?.id!;
  
  if (!userDb.isRegistered(userId)) {
    bot.sendMessage(chatId, '❌ Please setup first using /setup', {
      reply_markup: getProfileKeyboard()
    });
    return;
  }
  
  bot.sendMessage(chatId, '⏳ Fetching price feeds...');
  
  const feeds = await getPriceFeeds(userId);
  
  if (feeds.error) {
    bot.sendMessage(chatId, `❌ Error: ${feeds.error}`, {
      reply_markup: getMainMenuKeyboard()
    });
    return;
  }
  
  if (feeds.length === 0) {
    bot.sendMessage(chatId, '📭 No price feeds found.', {
      reply_markup: getMainMenuKeyboard()
    });
    return;
  }
  
  let message = '📈 *Price Feeds:*\n\n';
  
  feeds.forEach((feed: any, idx: number) => {
    const price = parseFloat(feed.price) / 1e8;
    const conf = parseFloat(feed.confidence || feed.conf || '0') / 1e8;
    const timestamp = parseInt(feed.timestamp);
    const date = timestamp > 0 ? new Date(timestamp * 1000) : null;
    
    const pair = feed.pair || 'UNKNOWN';
    const priceStr = price > 0 ? `$${price.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 6 })}` : '$0.00';
    const confStr = conf > 0 ? `±$${conf.toFixed(4)}` : '±$0.0000';
    const timeStr = date ? date.toLocaleString() : 'Never';
    const staleStr = date && Date.now() - date.getTime() > 60000 ? ' ⚠️ STALE' : '';
    
    message += `*${pair}*\n`;
    message += `  💰 ${priceStr} ${confStr}\n`;
    message += `  🕐 ${timeStr}${staleStr}\n\n`;
  });
  
  bot.sendMessage(chatId, message, { 
    parse_mode: 'Markdown',
    reply_markup: getMainMenuKeyboard()
  });
}

bot.onText(/\/feeds/, handleFeeds);

async function handlePublishers(msg: TelegramBot.Message) {
  const chatId = msg.chat.id;
  const userId = msg.from?.id!;
  
  if (!userDb.isRegistered(userId)) {
    bot.sendMessage(chatId, '❌ Please setup first using /setup', {
      reply_markup: getProfileKeyboard()
    });
    return;
  }
  
  bot.sendMessage(chatId, '⏳ Fetching publishers...');
  
  const publishers = await getPublishers(userId);
  
  if (publishers.error) {
    bot.sendMessage(chatId, `❌ Error: ${publishers.error}`, {
      reply_markup: getMainMenuKeyboard()
    });
    return;
  }
  
  if (publishers.length === 0) {
    bot.sendMessage(chatId, '📭 No publishers found.', {
      reply_markup: getMainMenuKeyboard()
    });
    return;
  }
  
  let message = `👥 *Publishers (${publishers.length}):*\n\n`;
  
  publishers.forEach((pub: any, idx: number) => {
    message += `${idx + 1}. ${pub.active ? '✅' : '❌'} \`${pub.pubkey}\`\n`;
  });
  
  bot.sendMessage(chatId, message, { 
    parse_mode: 'Markdown',
    reply_markup: getMainMenuKeyboard()
  });
}

bot.onText(/\/publishers/, handlePublishers);

bot.onText(/\/balance/, async (msg) => {
  const chatId = msg.chat.id;
  const userId = msg.from?.id!;
  
  if (!userDb.isRegistered(userId)) {
    bot.sendMessage(chatId, '❌ Please setup first using /setup');
    return;
  }
  
  bot.sendMessage(chatId, '⏳ Checking balance...');
  
  const balance = await getWalletBalance(userId);
  
  if (typeof balance === 'object' && balance.error) {
    bot.sendMessage(chatId, `❌ Error: ${balance.error}`);
    return;
  }
  
  bot.sendMessage(chatId, `💰 *Wallet Balance:*\n\n\`${balance}\``, { parse_mode: 'Markdown' });
});

// Profile Management
const userSetupState = new Map<number, { step: string; profileData: Partial<ValidatorProfile> }>();

async function handleAddProfile(msg: TelegramBot.Message) {
  const chatId = msg.chat.id;
  const userId = msg.from?.id!;
  
  userSetupState.set(userId, { step: 'name', profileData: {} });
  
  const user = userDb.getUser(userId);
  const isFirstProfile = !user || user.profiles.length === 0;
  
  bot.sendMessage(chatId, `
${isFirstProfile ? '🚀 *Create Your First Profile*' : '➕ *Add New Validator Profile*'}

What would you like to name this validator?
(e.g., "Main Validator", "Backup Server", "Test Node")

Type the name or /cancel to abort.
  `, { parse_mode: 'Markdown' });
}

bot.onText(/\/setup/, handleAddProfile);
bot.onText(/\/add_profile/, handleAddProfile);

async function handleProfiles(msg: TelegramBot.Message) {
  const chatId = msg.chat.id;
  const userId = msg.from?.id!;
  
  const user = userDb.getUser(userId);
  
  if (!user || user.profiles.length === 0) {
    bot.sendMessage(chatId, '❌ No profiles configured. Use /setup to create one.', {
      reply_markup: getProfileKeyboard()
    });
    return;
  }
  
  let message = `🗂️ *Your Validator Profiles:*\n\n`;
  
  user.profiles.forEach((profile, idx) => {
    const isActive = idx === user.activeProfile;
    const icon = isActive ? '🟢' : '⚪';
    const activeLabel = isActive ? ' (active)' : '';
    
    message += `${idx + 1}. ${icon} *${profile.name}*${activeLabel}\n`;
    message += `   🔗 ${profile.apiUrl}\n\n`;
  });
  
  message += `\n*Commands:*\n`;
  message += `/switch <number> - Switch profile\n`;
  message += `/add\\_profile - Add new validator\n`;
  message += `/rename\\_profile <name> - Rename current\n`;
  message += `/delete\\_profile <number> - Delete profile`;
  
  bot.sendMessage(chatId, message, { 
    parse_mode: 'Markdown',
    reply_markup: getProfileKeyboard()
  });
}

bot.onText(/\/profiles/, handleProfiles);

bot.onText(/\/switch (\d+)/, async (msg, match) => {
  const chatId = msg.chat.id;
  const userId = msg.from?.id!;
  
  const profileNum = parseInt(match![1]);
  const profileIndex = profileNum - 1;
  
  const user = userDb.getUser(userId);
  
  if (!user || user.profiles.length === 0) {
    bot.sendMessage(chatId, '❌ No profiles configured. Use /setup to create one.', {
      reply_markup: getProfileKeyboard()
    });
    return;
  }
  
  if (profileIndex < 0 || profileIndex >= user.profiles.length) {
    bot.sendMessage(chatId, `❌ Invalid profile number. You have ${user.profiles.length} profiles.`, {
      reply_markup: getMainMenuKeyboard()
    });
    return;
  }
  
  if (userDb.setActiveProfile(userId, profileIndex)) {
    const profile = user.profiles[profileIndex];
    bot.sendMessage(chatId, `✅ Switched to profile: *${profile.name}*`, { 
      parse_mode: 'Markdown',
      reply_markup: getMainMenuKeyboard()
    });
  } else {
    bot.sendMessage(chatId, '❌ Failed to switch profile.', {
      reply_markup: getMainMenuKeyboard()
    });
  }
});

bot.onText(/\/rename_profile (.+)/, async (msg, match) => {
  const chatId = msg.chat.id;
  const userId = msg.from?.id!;
  
  const newName = match![1].trim();
  
  const user = userDb.getUser(userId);
  
  if (!user || user.profiles.length === 0) {
    bot.sendMessage(chatId, '❌ No profiles configured.', {
      reply_markup: getProfileKeyboard()
    });
    return;
  }
  
  if (userDb.renameProfile(userId, user.activeProfile, newName)) {
    bot.sendMessage(chatId, `✅ Profile renamed to: *${newName}*`, { 
      parse_mode: 'Markdown',
      reply_markup: getMainMenuKeyboard()
    });
  } else {
    bot.sendMessage(chatId, '❌ Failed to rename profile.', {
      reply_markup: getMainMenuKeyboard()
    });
  }
});

bot.onText(/\/delete_profile (\d+)/, async (msg, match) => {
  const chatId = msg.chat.id;
  const userId = msg.from?.id!;
  
  const profileNum = parseInt(match![1]);
  const profileIndex = profileNum - 1;
  
  const user = userDb.getUser(userId);
  
  if (!user || user.profiles.length === 0) {
    bot.sendMessage(chatId, '❌ No profiles configured.', {
      reply_markup: getProfileKeyboard()
    });
    return;
  }
  
  if (profileIndex < 0 || profileIndex >= user.profiles.length) {
    bot.sendMessage(chatId, `❌ Invalid profile number. You have ${user.profiles.length} profiles.`, {
      reply_markup: getMainMenuKeyboard()
    });
    return;
  }
  
  const profileName = user.profiles[profileIndex].name;
  
  if (userDb.deleteProfile(userId, profileIndex)) {
    bot.sendMessage(chatId, `✅ Deleted profile: *${profileName}*`, { 
      parse_mode: 'Markdown',
      reply_markup: getMainMenuKeyboard()
    });
  } else {
    bot.sendMessage(chatId, '❌ Failed to delete profile.', {
      reply_markup: getMainMenuKeyboard()
    });
  }
});

// This handler is now merged with the button handler above

bot.onText(/\/config/, async (msg) => {
  const chatId = msg.chat.id;
  const userId = msg.from?.id!;
  
  const user = userDb.getUser(userId);
  const profile = userDb.getActiveProfile(userId);
  
  if (!user || !profile) {
    bot.sendMessage(chatId, '❌ Please setup first using /setup', {
      reply_markup: getProfileKeyboard()
    });
    return;
  }
  
  const message = `
⚙️ *Current Configuration*

*Active Profile:*
Name: *${profile.name}*
API URL: \`${profile.apiUrl}\`

*Network:*
RPC: \`${RPC_URL}\`
Program: \`${PROGRAM_ID}\`

*Manage:*
/profiles - View all profiles
/switch <number> - Switch profile
/add\\_profile - Add new validator
  `;
  
  bot.sendMessage(chatId, message, { 
    parse_mode: 'Markdown',
    reply_markup: getMainMenuKeyboard()
  });
});

async function handleLogs(msg: TelegramBot.Message, service: string) {
  const chatId = msg.chat.id;
  const userId = msg.from?.id!;
  
  if (!userDb.isRegistered(userId)) {
    bot.sendMessage(chatId, '❌ Please setup first using /setup', {
      reply_markup: getProfileKeyboard()
    });
    return;
  }
  
  bot.sendMessage(chatId, `⏳ Fetching ${service} logs...`);
  
  try {
    const result = await callApi(userId, `/api/logs/${service}`);
    
    if (!result.logs || result.logs.trim().length === 0) {
      bot.sendMessage(chatId, `📝 No logs found for ${service}`, {
        reply_markup: getLogsKeyboard()
      });
      return;
    }
    
    // Split logs into chunks if too long (Telegram has 4096 char limit)
    const logs = result.logs;
    const maxLength = 4000;
    
    if (logs.length <= maxLength) {
      bot.sendMessage(chatId, `📝 *${service.toUpperCase()} Logs (last ${result.lines} lines):*\n\n\`\`\`\n${logs}\n\`\`\``, {
        parse_mode: 'Markdown',
        reply_markup: getLogsKeyboard()
      });
    } else {
      // Send in chunks
      const chunks = [];
      for (let i = 0; i < logs.length; i += maxLength) {
        chunks.push(logs.substring(i, i + maxLength));
      }
      
      for (let i = 0; i < chunks.length; i++) {
        const isLast = i === chunks.length - 1;
        bot.sendMessage(chatId, `📝 *${service.toUpperCase()} Logs (part ${i + 1}/${chunks.length}):*\n\n\`\`\`\n${chunks[i]}\n\`\`\``, {
          parse_mode: 'Markdown',
          reply_markup: isLast ? getLogsKeyboard() : undefined
        });
      }
    }
  } catch (error: any) {
    bot.sendMessage(chatId, `❌ Error fetching logs: ${error.message}`, {
      reply_markup: getLogsKeyboard()
    });
  }
}

async function handleHelp(msg: TelegramBot.Message) {
  const chatId = msg.chat.id;
  
  const helpMessage = `
📚 *Tachyon Oracle Console Bot*

*🗂️ Profile Management:*
/setup - Create first profile
/profiles - List all profiles
/add\\_profile - Add new validator
/switch <number> - Switch profile
/rename\\_profile <name> - Rename current
/delete\\_profile <number> - Delete profile
/config - View current config

*📊 Monitoring:*
/status - Oracle status
/feeds - Price feeds
/publishers - Publishers

*📝 Logs:*
Tap "📝 Logs" button to view service logs

*ℹ️ Help:*
/help - This message

*Examples:*
\`/switch 2\` - Switch to profile #2
\`/rename_profile Backup Node\` - Rename current profile

*💡 Tip:* Use the menu buttons below for quick access!
  `;
  
  bot.sendMessage(chatId, helpMessage, { 
    parse_mode: 'Markdown',
    reply_markup: getMainMenuKeyboard()
  });
}

bot.onText(/\/help/, handleHelp);

// Error handling
bot.on('polling_error', (error) => {
  console.error('Polling error:', error);
});

console.log('🤖 Tachyon Oracle Console Bot is running...');
console.log(`📡 Connected to: ${RPC_URL}`);
console.log(`📁 Project path: ${PROJECT_PATH}`);

