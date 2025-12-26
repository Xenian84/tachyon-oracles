# Tachyon Oracle Telegram Bot

A Telegram bot for managing and monitoring your Tachyon Oracle validator remotely.

## Features

- 📊 **Real-time Status Monitoring** - Check oracle status, feeds, and publishers
- ⚙️ **Service Management** - Start/stop/restart signer and relayer services
- 📝 **Log Viewing** - View recent logs from services
- 💰 **Balance Checking** - Check wallet balances
- 🔒 **Secure** - Only authorized users can access the bot

## Setup Instructions

### 1. Create Your Telegram Bot

1. Open Telegram and search for `@BotFather`
2. Send `/newbot` command
3. Choose a name for your bot (e.g., "My Tachyon Oracle")
4. Choose a username (e.g., "my_tachyon_oracle_bot")
5. **BotFather will give you a TOKEN** - save this!

### 2. Get Your Telegram User ID

1. Search for `@userinfobot` on Telegram
2. Start a chat with it
3. It will reply with your **User ID** (a number like 123456789)
4. Save this number - you'll need it for authorization

### 3. Install the Bot

```bash
cd /root/tachyon-oracles/telegram-bot
npm install
```

### 4. Configure the Bot

```bash
# Copy the example config
cp .env.example .env

# Edit the configuration
nano .env
```

**Required Configuration:**

```bash
# Put your bot token from BotFather here
TELEGRAM_BOT_TOKEN=1234567890:ABCdefGHIjklMNOpqrsTUVwxyz

# Put your Telegram user ID here (comma-separated for multiple admins)
ADMIN_USER_IDS=123456789,987654321

# These should already be correct
RPC_URL=https://rpc.mainnet.x1.xyz
PROGRAM_ID=TACH9r2uZzoFM6daofesADjeDn9NqB1pKFWP5mfByb1
RELAYER_URL=http://localhost:3000
ORACLE_PROJECT_PATH=/root/tachyon-oracles
```

### 5. Build and Start the Bot

```bash
# Build the bot
npm run build

# Start the bot
npm start

# Or run in background with nohup
nohup npm start >> bot.log 2>&1 &

# Or use pm2 (recommended)
pm2 start dist/index.js --name tachyon-bot
pm2 save
```

### 6. Test Your Bot

1. Open Telegram
2. Search for your bot username (e.g., `@my_tachyon_oracle_bot`)
3. Send `/start` command
4. You should see the welcome message with all commands!

## Available Commands

### Status & Monitoring
- `/status` - View oracle status (services, feeds, publishers)
- `/feeds` - List all registered price feeds
- `/prices` - Show live price updates
- `/publishers` - List all registered publishers

### Service Management
- `/start_signer` - Start the signer service
- `/start_relayer` - Start the relayer service
- `/stop_signer` - Stop the signer service
- `/stop_relayer` - Stop the relayer service
- `/restart_signer` - Restart the signer service
- `/restart_relayer` - Restart the relayer service

### Logs & Debugging
- `/logs_signer` - View last 20 lines of signer logs
- `/logs_relayer` - View last 20 lines of relayer logs

### Wallet
- `/balance` - Check your wallet balance

### Help
- `/start` - Show welcome message and all commands
- `/help` - Show help message

## Security

- Only users listed in `ADMIN_USER_IDS` can use the bot
- All other users will receive "Unauthorized" message
- Keep your bot token secret!
- Never share your `.env` file

## Running as a Service

### Option 1: Using PM2 (Recommended)

```bash
# Install PM2 globally
npm install -g pm2

# Start the bot
cd /root/tachyon-oracles/telegram-bot
pm2 start dist/index.js --name tachyon-bot

# Save PM2 configuration
pm2 save

# Setup PM2 to start on reboot
pm2 startup
```

### Option 2: Using systemd

Create `/etc/systemd/system/tachyon-bot.service`:

```ini
[Unit]
Description=Tachyon Oracle Telegram Bot
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=/root/tachyon-oracles/telegram-bot
ExecStart=/usr/bin/node /root/tachyon-oracles/telegram-bot/dist/index.js
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
```

Then:

```bash
sudo systemctl daemon-reload
sudo systemctl enable tachyon-bot
sudo systemctl start tachyon-bot
sudo systemctl status tachyon-bot
```

## Troubleshooting

### Bot doesn't respond

1. Check if the bot is running:
   ```bash
   ps aux | grep "telegram-bot"
   # or if using pm2
   pm2 list
   ```

2. Check the logs:
   ```bash
   tail -f bot.log
   # or if using pm2
   pm2 logs tachyon-bot
   ```

3. Verify your bot token is correct in `.env`

4. Make sure your user ID is in `ADMIN_USER_IDS`

### "Unauthorized" message

- Your Telegram user ID is not in the `ADMIN_USER_IDS` list
- Get your user ID from `@userinfobot` and add it to `.env`
- Restart the bot after updating `.env`

### Services won't start/stop

- Make sure the bot has permission to execute commands
- Check if the `ORACLE_PROJECT_PATH` in `.env` is correct
- Verify that the signer and relayer are built (`npm run build` in their directories)

## Adding Multiple Admins

You can add multiple authorized users:

```bash
# In .env file
ADMIN_USER_IDS=123456789,987654321,555666777
```

Each admin will have full access to all bot commands.

## Updating the Bot

```bash
cd /root/tachyon-oracles/telegram-bot
git pull  # if using git
npm install
npm run build

# Restart the bot
pm2 restart tachyon-bot
# or
pkill -f "telegram-bot" && npm start
```

## Support

For issues or questions:
- Check the logs: `tail -f bot.log` or `pm2 logs tachyon-bot`
- Verify configuration in `.env`
- Make sure all services (signer, relayer) are properly configured

## License

MIT

