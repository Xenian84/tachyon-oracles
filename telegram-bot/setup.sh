#!/bin/bash

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║                                                                ║"
echo "║           🤖 Tachyon Oracle Telegram Bot Setup                ║"
echo "║                                                                ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

# Check if .env exists
if [ ! -f .env ]; then
    echo "📝 Creating .env file..."
    cp .env.example .env
    echo ""
    echo "⚠️  IMPORTANT: You need to configure the bot!"
    echo ""
    echo "1. Create a bot with @BotFather on Telegram"
    echo "2. Get your User ID from @userinfobot on Telegram"
    echo "3. Edit .env file and add:"
    echo "   - TELEGRAM_BOT_TOKEN (from BotFather)"
    echo "   - ADMIN_USER_IDS (your Telegram user ID)"
    echo ""
    echo "Run: nano .env"
    echo ""
    exit 1
fi

# Check if token is configured
if grep -q "your_bot_token_from_botfather" .env; then
    echo "❌ Bot token not configured!"
    echo "Please edit .env and add your bot token from @BotFather"
    echo ""
    echo "Run: nano .env"
    exit 1
fi

# Install dependencies
echo "📦 Installing dependencies..."
npm install

# Build
echo "🔨 Building bot..."
npm run build

echo ""
echo "✅ Setup complete!"
echo ""
echo "To start the bot:"
echo "  npm start"
echo ""
echo "Or run in background:"
echo "  pm2 start dist/index.js --name tachyon-bot"
echo ""
