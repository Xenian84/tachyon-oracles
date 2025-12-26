#!/bin/bash

GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${GREEN}"
echo "╔════════════════════════════════════════════════════════════════╗"
echo "║                                                                ║"
echo "║        🚀 Tachyon Oracle API Service Setup                     ║"
echo "║                                                                ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

# Check if .env exists
if [ -f ".env" ]; then
    echo -e "${YELLOW}⚠️  .env file already exists${NC}"
    read -p "Do you want to regenerate it? (y/n): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Setup cancelled."
        exit 0
    fi
fi

# Copy example
cp .env.example .env

# Generate API key
echo -e "\n${GREEN}🔑 Generating API key...${NC}"
API_KEY=$(openssl rand -hex 32)
sed -i "s/API_KEY=.*/API_KEY=$API_KEY/" .env

# Ask for mode
echo -e "\n${YELLOW}📋 Select API mode:${NC}"
echo "1) readonly - Only status/monitoring (safest)"
echo "2) monitoring - Status + logs (recommended)"
echo "3) full - Full control including service management"
read -p "Enter choice (1-3) [2]: " mode_choice

case $mode_choice in
    1) API_MODE="readonly" ;;
    3) API_MODE="full" ;;
    *) API_MODE="monitoring" ;;
esac

sed -i "s/API_MODE=.*/API_MODE=$API_MODE/" .env
echo -e "${GREEN}✅ Mode set to: $API_MODE${NC}"

# Ask for IP whitelist
echo -e "\n${YELLOW}🔒 IP Whitelist (optional):${NC}"
echo "Enter allowed IP addresses (comma-separated), or press Enter to allow all:"
read -p "IPs: " allowed_ips

if [ ! -z "$allowed_ips" ]; then
    sed -i "s/ALLOWED_IPS=.*/ALLOWED_IPS=$allowed_ips/" .env
    echo -e "${GREEN}✅ IP whitelist configured${NC}"
else
    echo -e "${YELLOW}⚠️  No IP whitelist - all IPs allowed${NC}"
fi

# Display summary
echo -e "\n${GREEN}"
echo "╔════════════════════════════════════════════════════════════════╗"
echo "║                                                                ║"
echo "║        ✅ API Service Configured!                              ║"
echo "║                                                                ║"
echo "╚════════════════════════════════════════════════════════════════╝"
echo -e "${NC}"

echo -e "${YELLOW}📝 Configuration Summary:${NC}"
echo -e "Port: ${GREEN}7171${NC}"
echo -e "Mode: ${GREEN}$API_MODE${NC}"
echo -e "API Key: ${GREEN}$API_KEY${NC}"
echo ""
echo -e "${YELLOW}⚠️  IMPORTANT: Save this API key! You'll need it to connect from the bot.${NC}"
echo ""

# Ask to start service
read -p "Start API service now? (y/n): " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    echo -e "\n${GREEN}🚀 Starting API service...${NC}"
    pm2 start dist/index.js --name tachyon-api
    pm2 save
    echo -e "${GREEN}✅ API service started!${NC}"
    echo ""
    echo -e "${YELLOW}📋 Useful commands:${NC}"
    echo "pm2 status tachyon-api  - Check status"
    echo "pm2 logs tachyon-api    - View logs"
    echo "pm2 restart tachyon-api - Restart service"
    echo "pm2 stop tachyon-api    - Stop service"
fi

echo ""
echo -e "${YELLOW}🔥 Firewall Setup (if needed):${NC}"
echo "sudo ufw allow 7171/tcp"
echo ""
echo -e "${YELLOW}🧪 Test the API:${NC}"
echo "curl -H \"Authorization: Bearer $API_KEY\" http://localhost:7171/api/status"
echo ""
echo -e "${GREEN}✅ Setup complete!${NC}"

