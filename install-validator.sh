#!/bin/bash

# Tachyon Oracle - Validator Auto-Installer
# This script sets up everything automatically

set -e

echo "╔════════════════════════════════════════════════════════════════╗"
echo "║                                                                ║"
echo "║        🚀 Tachyon Oracle - Validator Installer                 ║"
echo "║                                                                ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Check if running as root
if [ "$EUID" -eq 0 ]; then 
   echo -e "${RED}❌ Please do not run as root${NC}"
   exit 1
fi

echo -e "${GREEN}Step 1/8: Installing system dependencies...${NC}"
sudo apt update -qq
sudo apt install -y curl jq build-essential git > /dev/null 2>&1

echo -e "${GREEN}Step 2/8: Installing Node.js 20...${NC}"
if ! command -v node &> /dev/null; then
    curl -fsSL https://deb.nodesource.com/setup_20.x | sudo bash - > /dev/null 2>&1
    sudo apt-get install -y nodejs > /dev/null 2>&1
fi

echo -e "${GREEN}Step 3/8: Installing PM2...${NC}"
if ! command -v pm2 &> /dev/null; then
    sudo npm install -g pm2 > /dev/null 2>&1
fi

echo -e "${GREEN}Step 4/8: Installing project dependencies...${NC}"
cd ~/tachyon-oracles
npm install > /dev/null 2>&1

echo -e "${GREEN}Step 5/8: Building services...${NC}"
cd signer && npm install > /dev/null 2>&1 && npm run build > /dev/null 2>&1 && cd ..
cd relayer && npm install > /dev/null 2>&1 && npm run build > /dev/null 2>&1 && cd ..

echo -e "${GREEN}Step 6/8: Creating validator keypair...${NC}"
mkdir -p keys
if [ ! -f keys/validator-signer.json ]; then
    echo -e "${YELLOW}⚠️  Please save your seed phrase!${NC}"
    solana-keygen new --outfile keys/validator-signer.json
    PUBKEY=$(solana-keygen pubkey keys/validator-signer.json)
    echo -e "${GREEN}✅ Your validator public key: ${PUBKEY}${NC}"
    echo -e "${YELLOW}⚠️  Fund this address with at least 1 XNT${NC}"
else
    echo -e "${YELLOW}⚠️  Keypair already exists, skipping...${NC}"
    PUBKEY=$(solana-keygen pubkey keys/validator-signer.json)
fi

echo -e "${GREEN}Step 7/8: Configuring environment...${NC}"
if [ ! -f .env ]; then
    cat > .env << EOF
# Validator Configuration
VALIDATOR_KEYPAIR=./keys/validator-signer.json

# Relayer Settings
RELAYER_PORT=7777
RELAYER_URLS=http://localhost:7777

# RPC Settings
RPC_URL=https://rpc.mainnet.x1.xyz

# Program ID
PROGRAM_ID=TACH9r2uZzoFM6daofesADjeDn9NqB1pKFWP5mfByb1
EOF
    echo -e "${GREEN}✅ .env file created${NC}"
else
    echo -e "${YELLOW}⚠️  .env already exists, skipping...${NC}"
fi

echo -e "${GREEN}Step 8/8: Checking balance...${NC}"
BALANCE=$(solana balance keys/validator-signer.json --url https://rpc.mainnet.x1.xyz 2>/dev/null | awk '{print $1}')
if (( $(echo "$BALANCE < 0.1" | bc -l) )); then
    echo -e "${RED}❌ Low balance: ${BALANCE} XNT${NC}"
    echo -e "${YELLOW}⚠️  Please fund your address: ${PUBKEY}${NC}"
    echo -e "${YELLOW}⚠️  You need at least 1 XNT to register and operate${NC}"
else
    echo -e "${GREEN}✅ Balance: ${BALANCE} XNT${NC}"
fi

echo ""
echo "╔════════════════════════════════════════════════════════════════╗"
echo "║                                                                ║"
echo "║        ✅ Installation Complete!                               ║"
echo "║                                                                ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo ""
echo -e "${GREEN}Next steps:${NC}"
echo ""
echo "1. Fund your validator address:"
echo -e "   ${YELLOW}${PUBKEY}${NC}"
echo ""
echo "2. Register as publisher:"
echo -e "   ${GREEN}node scripts/register-publisher-simple.js${NC}"
echo -e "   ${GREEN}node scripts/activate-publisher.js${NC}"
echo ""
echo "3. Start services:"
echo -e "   ${GREEN}cd signer && pm2 start dist/index.js --name tachyon-signer${NC}"
echo -e "   ${GREEN}cd ../relayer && RELAYER_PORT=7777 pm2 start dist/index.js --name tachyon-relayer --update-env${NC}"
echo -e "   ${GREEN}pm2 save${NC}"
echo ""
echo "4. Use Telegram bot for management:"
echo -e "   ${GREEN}Search: @tachyon_oracle_bot${NC}"
echo ""
echo "5. Or use the console:"
echo -e "   ${GREEN}./tachyon${NC}"
echo ""
echo -e "${GREEN}📚 Full guide: cat VALIDATOR_QUICKSTART.md${NC}"
echo ""

