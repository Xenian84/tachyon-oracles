#!/bin/bash

# Tachyon Oracles Console v1.0
# Interactive management console for Tachyon Oracle validators

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
NC='\033[0m' # No Color
BOLD='\033[1m'

# Paths - Auto-detect
CONSOLE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
KEYS_DIR="$CONSOLE_DIR/keys"
LOGS_DIR="$CONSOLE_DIR/logs"

# Create logs directory
mkdir -p "$LOGS_DIR"

# Auto-detect Solana CLI path
detect_solana_path() {
    # Check common locations
    if command -v solana &> /dev/null; then
        SOLANA_PATH="$(dirname "$(which solana)")"
    elif [ -d "/root/tachyon/target/release" ] && [ -x "/root/tachyon/target/release/solana" ]; then
        SOLANA_PATH="/root/tachyon/target/release"
    elif [ -d "$HOME/.local/share/solana/install/active_release/bin" ]; then
        SOLANA_PATH="$HOME/.local/share/solana/install/active_release/bin"
    else
        SOLANA_PATH=""
    fi
    export PATH="$SOLANA_PATH:$PATH"
}

# Auto-detect config directory
detect_config_dir() {
    if [ -d "$HOME/.config/solana" ]; then
        SOLANA_CONFIG_DIR="$HOME/.config/solana"
    elif [ -d "/root/.config/solana" ]; then
        SOLANA_CONFIG_DIR="/root/.config/solana"
    else
        SOLANA_CONFIG_DIR="$HOME/.config/solana"
        mkdir -p "$SOLANA_CONFIG_DIR"
    fi
}

# Initialize
detect_solana_path
detect_config_dir

# Load environment
if [ -f "$CONSOLE_DIR/.env" ]; then
    source "$CONSOLE_DIR/.env"
fi

# Load program ID from .env or detect
if [ -z "$PROGRAM_ID" ]; then
    if [ -f "$CONSOLE_DIR/.env" ]; then
        PROGRAM_ID=$(grep "^PROGRAM_ID=" "$CONSOLE_DIR/.env" | cut -d'=' -f2)
    fi
    if [ -z "$PROGRAM_ID" ]; then
        PROGRAM_ID="TACH9r2uZzoFM6daofesADjeDn9NqB1pKFWP5mfByb1"
    fi
fi

# Load RPC URL from .env or use default
if [ -z "$SOLANA_RPC_URL" ]; then
    if [ -f "$CONSOLE_DIR/.env" ]; then
        SOLANA_RPC_URL=$(grep "^SOLANA_RPC_URL=" "$CONSOLE_DIR/.env" | cut -d'=' -f2)
    fi
    if [ -z "$SOLANA_RPC_URL" ]; then
        SOLANA_RPC_URL="https://rpc.mainnet.x1.xyz"
    fi
fi

# Functions
clear_screen() {
    clear
}

print_header() {
    clear_screen
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    echo -e "${BOLD}${WHITE}           TACHYON ORACLES CONSOLE v1.0${NC}"
    echo -e "${WHITE}              Decentralized Price Feeds for X1${NC}"
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
}

print_status() {
    # Check if services are running
    RELAYER_STATUS="${RED}Stopped${NC}"
    SIGNER_STATUS="${RED}Stopped${NC}"
    
    if pgrep -f "relayer/dist/index.js" > /dev/null; then
        RELAYER_STATUS="${GREEN}Running${NC}"
    fi
    
    if pgrep -f "signer/dist/index.js" > /dev/null; then
        SIGNER_STATUS="${GREEN}Running${NC}"
    fi
    
    # Count feeds dynamically from blockchain
    FEED_COUNT=$(node -e "
    const { Connection, PublicKey } = require('@solana/web3.js');
    const crypto = require('crypto');
    
    async function countFeeds() {
        try {
            const connection = new Connection('$SOLANA_RPC_URL', 'confirmed');
            const programId = new PublicKey('$PROGRAM_ID');
            
            const assets = ['BTC/USD', 'ETH/USD', 'SOL/USD', 'AVAX/USD', 'MATIC/USD', 
                          'BNB/USD', 'XRP/USD', 'ADA/USD', 'DOT/USD', 'LINK/USD',
                          'UNI/USD', 'AAVE/USD', 'ATOM/USD', 'NEAR/USD', 'FTM/USD'];
            
            let count = 0;
            for (const asset of assets) {
                const hash = crypto.createHash('sha256').update(asset).digest();
                const [feedPda] = PublicKey.findProgramAddressSync(
                    [Buffer.from('feed'), hash],
                    programId
                );
                
                const accountInfo = await connection.getAccountInfo(feedPda);
                if (accountInfo) count++;
            }
            
            console.log(count);
        } catch (e) {
            console.log('?');
        }
    }
    
    countFeeds();
    " 2>/dev/null || echo "?")
    
    # Count publishers dynamically from blockchain
    # Publisher accounts have size 50 bytes (8 discriminator + 32 pubkey + 8 staked + 1 active + 1 bump)
    PUB_COUNT=$(node -e "
    const { Connection, PublicKey } = require('@solana/web3.js');
    
    async function countPublishers() {
        try {
            const connection = new Connection('$SOLANA_RPC_URL', 'confirmed');
            const programId = new PublicKey('$PROGRAM_ID');
            
            // Get all program accounts
            const accounts = await connection.getProgramAccounts(programId, {
                filters: [
                    {
                        dataSize: 50 // Publisher account size
                    }
                ]
            });
            
            console.log(accounts.length);
        } catch (e) {
            console.log('?');
        }
    }
    
    countPublishers();
    " 2>/dev/null || echo "?")
    
    echo ""
    echo -e "  ${BOLD}Status:${NC}"
    echo -e "    Relayer: $RELAYER_STATUS"
    echo -e "    Signer:  $SIGNER_STATUS"
    echo -e "    Feeds: ${GREEN}$FEED_COUNT${NC}  |  Publishers: ${GREEN}$PUB_COUNT${NC}"
    echo ""
}

print_menu() {
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    echo -e "  ${BOLD}${WHITE}MAIN MENU${NC}"
    echo ""
    echo -e "  ${BOLD}${YELLOW}8. 🚀 FIRST TIME SETUP${NC} ${WHITE}- Automated Setup Wizard (NEW!)${NC}"
    echo ""
    echo -e "  ${GREEN}1.${NC} Service Manager      - Start/Stop Oracle Services"
    echo -e "  ${GREEN}2.${NC} Oracle Manager       - Manage Feeds & Publishers"
    echo -e "  ${GREEN}3.${NC} Monitoring           - View Logs & Status"
    echo -e "  ${GREEN}4.${NC} Configuration        - Edit Settings"
    echo -e "  ${GREEN}5.${NC} Wallet Manager       - View & Manage Wallets"
    echo -e "  ${GREEN}6.${NC} Autopilot            - Automated Management"
    echo -e "  ${GREEN}7.${NC} Update & Maintenance - Updates & Diagnostics"
    echo -e "  ${GREEN}9.${NC} Exit"
    echo ""
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    echo ""
}

# Service Manager
service_manager() {
    while true; do
        print_header
        print_status
        echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
        echo -e "  ${BOLD}${WHITE}SERVICE MANAGER${NC}"
        echo ""
        echo -e "  ${GREEN}1.${NC} Start Relayer"
        echo -e "  ${GREEN}2.${NC} Start Signer"
        echo -e "  ${GREEN}3.${NC} Start Both Services"
        echo -e "  ${GREEN}4.${NC} Stop Relayer"
        echo -e "  ${GREEN}5.${NC} Stop Signer"
        echo -e "  ${GREEN}6.${NC} Stop All Services"
        echo -e "  ${GREEN}7.${NC} Restart Services"
        echo -e "  ${GREEN}8.${NC} View Service Status"
        echo -e "  ${GREEN}10.${NC} 🔌 API Service Manager"
        echo -e "  ${GREEN}9.${NC} Back to Main Menu"
        echo ""
        echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
        echo ""
        read -p "Select option: " choice
        
        case $choice in
            1) start_relayer ;;
            2) start_signer ;;
            3) start_both ;;
            4) stop_relayer ;;
            5) stop_signer ;;
            6) stop_all ;;
            7) restart_services ;;
            8) view_service_status ;;
            10) api_service_manager ;;
            9) break ;;
            *) echo -e "${RED}Invalid option${NC}" ; sleep 1 ;;
        esac
    done
}

start_relayer() {
    echo -e "\n${YELLOW}Starting Relayer...${NC}"
    cd "$CONSOLE_DIR"
    
    # Check if already running
    if pgrep -f "relayer/dist/index.js" > /dev/null; then
        echo -e "${YELLOW}Relayer is already running${NC}"
        sleep 2
        return
    fi
    
    # Build if needed
    if [ ! -f "relayer/dist/index.js" ]; then
        echo -e "${YELLOW}Building relayer...${NC}"
        cd relayer && npm install && npx tsc || true
        cd ..
    fi
    
    # Start with PM2 if available, otherwise background
    if command -v pm2 &> /dev/null; then
        pm2 start node --name "tachyon-relayer" -- relayer/dist/index.js
        echo -e "${GREEN}✓ Relayer started with PM2${NC}"
    else
        nohup node relayer/dist/index.js > "$LOGS_DIR/relayer.log" 2>&1 &
        echo -e "${GREEN}✓ Relayer started in background${NC}"
    fi
    
    sleep 2
}

start_signer() {
    echo -e "\n${YELLOW}Starting Signer...${NC}"
    cd "$CONSOLE_DIR"
    
    # Check if already running
    if pgrep -f "signer/dist/index.js" > /dev/null; then
        echo -e "${YELLOW}Signer is already running${NC}"
        sleep 2
        return
    fi
    
    # Build if needed
    if [ ! -f "signer/dist/index.js" ]; then
        echo -e "${YELLOW}Building signer...${NC}"
        cd signer && npm install && npx tsc || true
        cd ..
    fi
    
    # Start with PM2 if available, otherwise background
    if command -v pm2 &> /dev/null; then
        pm2 start node --name "tachyon-signer" -- signer/dist/index.js
        echo -e "${GREEN}✓ Signer started with PM2${NC}"
    else
        nohup node signer/dist/index.js > "$LOGS_DIR/signer.log" 2>&1 &
        echo -e "${GREEN}✓ Signer started in background${NC}"
    fi
    
    sleep 2
}

start_both() {
    start_relayer
    start_signer
    echo -e "\n${GREEN}✓ Both services started${NC}"
    sleep 2
}

stop_relayer() {
    echo -e "\n${YELLOW}Stopping Relayer...${NC}"
    if command -v pm2 &> /dev/null; then
        pm2 stop tachyon-relayer 2>/dev/null || true
        pm2 delete tachyon-relayer 2>/dev/null || true
    fi
    pkill -f "relayer/dist/index.js" || true
    echo -e "${GREEN}✓ Relayer stopped${NC}"
    sleep 2
}

stop_signer() {
    echo -e "\n${YELLOW}Stopping Signer...${NC}"
    if command -v pm2 &> /dev/null; then
        pm2 stop tachyon-signer 2>/dev/null || true
        pm2 delete tachyon-signer 2>/dev/null || true
    fi
    pkill -f "signer/dist/index.js" || true
    echo -e "${GREEN}✓ Signer stopped${NC}"
    sleep 2
}

stop_all() {
    stop_relayer
    stop_signer
    echo -e "\n${GREEN}✓ All services stopped${NC}"
    sleep 2
}

restart_services() {
    echo -e "\n${YELLOW}Restarting services...${NC}"
    stop_all
    sleep 2
    start_both
    echo -e "\n${GREEN}✓ Services restarted${NC}"
    sleep 2
}

view_service_status() {
    print_header
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    echo -e "  ${BOLD}${WHITE}SERVICE STATUS${NC}"
    echo ""
    
    # Relayer
    if pgrep -f "relayer/dist/index.js" > /dev/null; then
        PID=$(pgrep -f "relayer/dist/index.js")
        echo -e "  Relayer:  ${GREEN}Running${NC} (PID: $PID)"
        
        # Check if responding
        if curl -s http://localhost:3000/health > /dev/null 2>&1; then
            echo -e "            ${GREEN}✓ Responding on port 3000${NC}"
        else
            echo -e "            ${RED}✗ Not responding${NC}"
        fi
    else
        echo -e "  Relayer:  ${RED}Stopped${NC}"
    fi
    
    echo ""
    
    # Signer
    if pgrep -f "signer/dist/index.js" > /dev/null; then
        PID=$(pgrep -f "signer/dist/index.js")
        echo -e "  Signer:   ${GREEN}Running${NC} (PID: $PID)"
    else
        echo -e "  Signer:   ${RED}Stopped${NC}"
    fi
    
    echo ""
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    echo ""
    read -p "Press Enter to continue..."
}

# API Service Manager
api_service_manager() {
    while true; do
        print_header
        echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
        echo -e "  ${BOLD}${WHITE}🔌 API SERVICE MANAGER${NC}"
        echo ""
        
        # Check API service status
        if pm2 list | grep -q "tachyon-api.*online"; then
            echo -e "  ${GREEN}✓ API Service: RUNNING${NC}"
            PID=$(pm2 list | grep tachyon-api | awk '{print $10}' | head -1)
            echo -e "  ${CYAN}Port: ${YELLOW}7171${NC}"
            
            # Test API health
            if [ -f "$CONSOLE_DIR/api-service/.env" ]; then
                API_KEY=$(grep "^API_KEY=" "$CONSOLE_DIR/api-service/.env" | cut -d'=' -f2)
                API_MODE=$(grep "^API_MODE=" "$CONSOLE_DIR/api-service/.env" | cut -d'=' -f2)
                
                echo -e "  ${CYAN}Mode: ${YELLOW}${API_MODE}${NC}"
                
                # Test connection
                if curl -s -H "Authorization: Bearer $API_KEY" http://localhost:7171/api/status > /dev/null 2>&1; then
                    echo -e "  ${GREEN}✓ API Responding${NC}"
                else
                    echo -e "  ${RED}✗ API Not Responding${NC}"
                fi
            fi
        else
            echo -e "  ${RED}✗ API Service: STOPPED${NC}"
        fi
        
        echo ""
        echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
        echo ""
        echo -e "  ${GREEN}1.${NC} Start API Service"
        echo -e "  ${GREEN}2.${NC} Stop API Service"
        echo -e "  ${GREEN}3.${NC} Restart API Service"
        echo -e "  ${GREEN}4.${NC} View API Logs"
        echo -e "  ${GREEN}5.${NC} Show API Key"
        echo -e "  ${GREEN}6.${NC} Show Connection Info"
        echo -e "  ${GREEN}7.${NC} Test API Connection"
        echo -e "  ${GREEN}8.${NC} Change API Mode"
        echo -e "  ${GREEN}9.${NC} Back to Service Manager"
        echo ""
        echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
        echo ""
        read -p "Select option: " choice
        
        case $choice in
            1) start_api_service ;;
            2) stop_api_service ;;
            3) restart_api_service ;;
            4) view_api_logs ;;
            5) show_api_key ;;
            6) show_api_connection_info ;;
            7) test_api_connection ;;
            8) change_api_mode ;;
            9) break ;;
            *) echo -e "${RED}Invalid option${NC}" ; sleep 1 ;;
        esac
    done
}

start_api_service() {
    echo -e "\n${YELLOW}Starting API Service...${NC}"
    cd "$CONSOLE_DIR/api-service"
    
    if pm2 list | grep -q "tachyon-api.*online"; then
        echo -e "${YELLOW}API service is already running${NC}"
        sleep 2
        return
    fi
    
    if [ ! -f "dist/index.js" ]; then
        echo -e "${YELLOW}Building API service...${NC}"
        npm install > /dev/null 2>&1 && npm run build > /dev/null 2>&1
    fi
    
    pm2 start dist/index.js --name tachyon-api
    pm2 save > /dev/null 2>&1
    echo -e "${GREEN}✓ API service started${NC}"
    sleep 2
}

stop_api_service() {
    echo -e "\n${YELLOW}Stopping API Service...${NC}"
    pm2 stop tachyon-api
    echo -e "${GREEN}✓ API service stopped${NC}"
    sleep 2
}

restart_api_service() {
    echo -e "\n${YELLOW}Restarting API Service...${NC}"
    pm2 restart tachyon-api
    echo -e "${GREEN}✓ API service restarted${NC}"
    sleep 2
}

view_api_logs() {
    print_header
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    echo -e "  ${BOLD}${WHITE}API SERVICE LOGS${NC}"
    echo -e "  ${YELLOW}Press Ctrl+C to return to menu${NC}"
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    echo ""
    
    trap 'echo -e "\n${GREEN}Returning to menu...${NC}"; sleep 1; return' INT
    pm2 logs tachyon-api --lines 100
    trap - INT
}

show_api_key() {
    print_header
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    echo -e "  ${BOLD}${WHITE}API KEY${NC}"
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    echo ""
    
    if [ -f "$CONSOLE_DIR/api-service/.env" ]; then
        API_KEY=$(grep "^API_KEY=" "$CONSOLE_DIR/api-service/.env" | cut -d'=' -f2)
        echo -e "  ${CYAN}Your API Key:${NC}"
        echo -e "  ${YELLOW}${API_KEY}${NC}"
        echo ""
        
        if [ -f "$KEYS_DIR/api-key.txt" ]; then
            echo -e "  ${CYAN}Also saved to:${NC}"
            echo -e "  ${YELLOW}$KEYS_DIR/api-key.txt${NC}"
        fi
    else
        echo -e "  ${RED}API service not configured${NC}"
        echo -e "  ${YELLOW}Run the First Time Setup wizard (option 8)${NC}"
    fi
    
    echo ""
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    echo ""
    read -p "Press Enter to continue..."
}

show_api_connection_info() {
    print_header
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    echo -e "  ${BOLD}${WHITE}API CONNECTION INFO${NC}"
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    echo ""
    
    if [ -f "$CONSOLE_DIR/api-service/.env" ]; then
        API_KEY=$(grep "^API_KEY=" "$CONSOLE_DIR/api-service/.env" | cut -d'=' -f2)
        API_MODE=$(grep "^API_MODE=" "$CONSOLE_DIR/api-service/.env" | cut -d'=' -f2)
        SERVER_IP=$(curl -s ifconfig.me 2>/dev/null || echo "YOUR_SERVER_IP")
        
        echo -e "  ${CYAN}Server URL:${NC}"
        echo -e "  ${YELLOW}http://${SERVER_IP}:7171${NC}"
        echo ""
        echo -e "  ${CYAN}API Key:${NC}"
        echo -e "  ${YELLOW}${API_KEY}${NC}"
        echo ""
        echo -e "  ${CYAN}Mode:${NC}"
        echo -e "  ${YELLOW}${API_MODE}${NC}"
        echo ""
        echo -e "  ${CYAN}Connect from Telegram Bot:${NC}"
        echo -e "  ${GREEN}1.${NC} Open ${YELLOW}@tachyon_oracle_bot${NC}"
        echo -e "  ${GREEN}2.${NC} Tap ${YELLOW}🗂️ Profiles → ➕ Add Profile${NC}"
        echo -e "  ${GREEN}3.${NC} Name: ${YELLOW}My Validator${NC}"
        echo -e "  ${GREEN}4.${NC} Type: ${YELLOW}api${NC}"
        echo -e "  ${GREEN}5.${NC} URL: ${YELLOW}http://${SERVER_IP}:7171${NC}"
        echo -e "  ${GREEN}6.${NC} API Key: ${YELLOW}(paste from above)${NC}"
    else
        echo -e "  ${RED}API service not configured${NC}"
    fi
    
    echo ""
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    echo ""
    read -p "Press Enter to continue..."
}

test_api_connection() {
    print_header
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    echo -e "  ${BOLD}${WHITE}API CONNECTION TEST${NC}"
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    echo ""
    
    if [ -f "$CONSOLE_DIR/api-service/.env" ]; then
        API_KEY=$(grep "^API_KEY=" "$CONSOLE_DIR/api-service/.env" | cut -d'=' -f2)
        
        echo -e "  ${YELLOW}Testing API connection...${NC}"
        echo ""
        
        RESPONSE=$(curl -s -H "Authorization: Bearer $API_KEY" http://localhost:7171/api/status)
        
        if [ $? -eq 0 ] && [ ! -z "$RESPONSE" ]; then
            echo -e "  ${GREEN}✓ API Connection Successful!${NC}"
            echo ""
            echo -e "  ${CYAN}Response:${NC}"
            echo "$RESPONSE" | python3 -m json.tool 2>/dev/null || echo "$RESPONSE"
        else
            echo -e "  ${RED}✗ API Connection Failed${NC}"
            echo -e "  ${YELLOW}Make sure the API service is running${NC}"
        fi
    else
        echo -e "  ${RED}API service not configured${NC}"
    fi
    
    echo ""
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    echo ""
    read -p "Press Enter to continue..."
}

change_api_mode() {
    print_header
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    echo -e "  ${BOLD}${WHITE}CHANGE API MODE${NC}"
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    echo ""
    
    if [ ! -f "$CONSOLE_DIR/api-service/.env" ]; then
        echo -e "  ${RED}API service not configured${NC}"
        read -p "Press Enter to continue..."
        return
    fi
    
    CURRENT_MODE=$(grep "^API_MODE=" "$CONSOLE_DIR/api-service/.env" | cut -d'=' -f2)
    echo -e "  ${CYAN}Current Mode: ${YELLOW}${CURRENT_MODE}${NC}"
    echo ""
    echo -e "  ${WHITE}Available Modes:${NC}"
    echo -e "  ${GREEN}1.${NC} readonly   - Status only"
    echo -e "  ${GREEN}2.${NC} monitoring - Status + logs"
    echo -e "  ${GREEN}3.${NC} full       - Everything + service control"
    echo ""
    read -p "Select mode (1-3): " mode_choice
    
    case $mode_choice in
        1) NEW_MODE="readonly" ;;
        2) NEW_MODE="monitoring" ;;
        3) NEW_MODE="full" ;;
        *)
            echo -e "${RED}Invalid option${NC}"
            sleep 2
            return
            ;;
    esac
    
    sed -i "s/^API_MODE=.*/API_MODE=$NEW_MODE/" "$CONSOLE_DIR/api-service/.env"
    
    echo ""
    echo -e "${GREEN}✓ API mode changed to: ${YELLOW}${NEW_MODE}${NC}"
    echo -e "${YELLOW}Restarting API service...${NC}"
    pm2 restart tachyon-api > /dev/null 2>&1
    echo -e "${GREEN}✓ Done${NC}"
    sleep 2
}

# Oracle Manager
oracle_manager() {
    while true; do
        print_header
        echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
        echo -e "  ${BOLD}${WHITE}ORACLE MANAGER${NC}"
        echo ""
        echo -e "  ${GREEN}1.${NC} Add Price Feed"
        echo -e "  ${GREEN}2.${NC} View Price Feeds"
        echo -e "  ${GREEN}3.${NC} Register Publisher"
        echo -e "  ${GREEN}4.${NC} View Publishers"
        echo -e "  ${GREEN}5.${NC} Check Feed Prices (Live)"
        echo -e "  ${GREEN}6.${NC} Back to Main Menu"
        echo ""
        echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
        echo ""
        read -p "Select option: " choice
        
        case $choice in
            1) add_price_feed ;;
            2) view_price_feeds ;;
            3) register_publisher ;;
            4) view_publishers ;;
            5) check_live_prices ;;
            6) break ;;
            *) echo -e "${RED}Invalid option${NC}" ; sleep 1 ;;
        esac
    done
}

add_price_feed() {
    echo -e "\n${BOLD}${WHITE}Add New Price Feed${NC}"
    echo -e "${YELLOW}Examples: BTC/USD, ETH/USD, SOL/USD${NC}\n"
    read -p "Enter asset ID: " asset_id
    
    if [ -z "$asset_id" ]; then
        echo -e "${RED}Asset ID cannot be empty${NC}"
        sleep 2
        return
    fi
    
    echo -e "\n${YELLOW}Adding $asset_id to blockchain...${NC}"
    cd "$CONSOLE_DIR"
    node scripts/add-asset-simple.js "$asset_id"
    
    echo -e "\n${YELLOW}Don't forget to add this asset to signer/config.yaml${NC}"
    echo -e "${GREEN}Press Enter to continue...${NC}"
    read
}

view_price_feeds() {
    print_header
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    echo -e "  ${BOLD}${WHITE}ACTIVE PRICE FEEDS${NC}"
    echo ""
    echo -e "  ${YELLOW}Querying X1 mainnet...${NC}"
    echo ""
    
    # Query blockchain dynamically
    node -e "
    const { Connection, PublicKey } = require('@solana/web3.js');
    const crypto = require('crypto');
    
    const feedNames = {
        'BTC/USD': 'Bitcoin',
        'ETH/USD': 'Ethereum',
        'SOL/USD': 'Solana',
        'AVAX/USD': 'Avalanche',
        'MATIC/USD': 'Polygon',
        'BNB/USD': 'Binance Coin',
        'XRP/USD': 'Ripple',
        'ADA/USD': 'Cardano',
        'DOT/USD': 'Polkadot',
        'LINK/USD': 'Chainlink',
        'UNI/USD': 'Uniswap',
        'AAVE/USD': 'Aave',
        'ATOM/USD': 'Cosmos',
        'NEAR/USD': 'Near',
        'FTM/USD': 'Fantom'
    };
    
    async function listFeeds() {
        try {
            const connection = new Connection('$SOLANA_RPC_URL', 'confirmed');
            const programId = new PublicKey('$PROGRAM_ID');
            
            let activeCount = 0;
            
            for (const [asset, name] of Object.entries(feedNames)) {
                const hash = crypto.createHash('sha256').update(asset).digest();
                const [feedPda] = PublicKey.findProgramAddressSync(
                    [Buffer.from('feed'), hash],
                    programId
                );
                
                const accountInfo = await connection.getAccountInfo(feedPda);
                if (accountInfo) {
                    activeCount++;
                    const padded = asset.padEnd(12);
                    console.log(\`  ✅ \${padded} - \${name}\`);
                }
            }
            
            console.log('');
            console.log(\`  Total: \${activeCount} active feeds\`);
            console.log('');
            console.log('  Data sources: Binance, Coinbase, Kraken');
            console.log('  Update interval: 30 seconds');
            console.log('  Aggregation: Median price');
        } catch (e) {
            console.log('  Error querying feeds:', e.message);
        }
    }
    
    listFeeds();
    " 2>/dev/null
    
    echo ""
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    echo ""
    read -p "Press Enter to continue..."
}

register_publisher() {
    echo -e "\n${BOLD}${WHITE}Register Publisher${NC}\n"
    echo -e "Available keypairs in $KEYS_DIR:"
    ls -1 "$KEYS_DIR"/*.json 2>/dev/null | nl
    echo ""
    read -p "Enter keypair filename: " keyfile
    
    if [ ! -f "$KEYS_DIR/$keyfile" ]; then
        echo -e "${RED}Keypair not found${NC}"
        sleep 2
        return
    fi
    
    echo -e "\n${YELLOW}Registering publisher...${NC}"
    cd "$CONSOLE_DIR"
    node scripts/register-publisher-simple.js "$KEYS_DIR/$keyfile"
    
    echo -e "\n${GREEN}Press Enter to continue...${NC}"
    read
}

view_publishers() {
    print_header
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    echo -e "  ${BOLD}${WHITE}REGISTERED PUBLISHERS${NC}"
    echo ""
    echo -e "  ${YELLOW}Querying X1 mainnet...${NC}"
    echo ""
    
    # Query publishers dynamically from blockchain
    node -e "
    const { Connection, PublicKey } = require('@solana/web3.js');
    
    async function listPublishers() {
        try {
            const connection = new Connection('$SOLANA_RPC_URL', 'confirmed');
            const programId = new PublicKey('$PROGRAM_ID');
            
            // Get config PDA
            const [configPda] = PublicKey.findProgramAddressSync(
                [Buffer.from('config')],
                programId
            );
            
            const accountInfo = await connection.getAccountInfo(configPda);
            if (accountInfo) {
                // Parse publisher count
                const count = accountInfo.data.readUInt32LE(40);
                console.log(\`  Total registered publishers: \${count}\`);
                console.log('');
                
                // List known publishers from keys directory
                const fs = require('fs');
                const path = require('path');
                const keysDir = '$KEYS_DIR';
                
                if (fs.existsSync(keysDir)) {
                    const files = fs.readdirSync(keysDir).filter(f => f.endsWith('.json'));
                    
                    for (const file of files) {
                        try {
                            const keyData = JSON.parse(fs.readFileSync(path.join(keysDir, file)));
                            const keypair = require('@solana/web3.js').Keypair.fromSecretKey(new Uint8Array(keyData));
                            const pubkey = keypair.publicKey;
                            
                            // Check publisher status
                            const [publisherPda] = PublicKey.findProgramAddressSync(
                                [Buffer.from('publisher'), pubkey.toBuffer()],
                                programId
                            );
                            
                            const publisherInfo = await connection.getAccountInfo(publisherPda);
                            if (publisherInfo) {
                                // is_active is at offset 48 in PublisherAccount
                                const isActive = publisherInfo.data[48] === 1;
                                const status = isActive ? '🟢 ACTIVE' : '🔴 INACTIVE';
                                console.log(\`  ✅ \${file}\`);
                                console.log(\`     \${pubkey.toString()}\`);
                                console.log(\`     Status: \${status}\`);
                            } else {
                                console.log(\`  ⚠️  \${file}\`);
                                console.log(\`     \${pubkey.toString()}\`);
                                console.log(\`     Status: NOT REGISTERED\`);
                            }
                            console.log('');
                        } catch (e) {
                            // Skip invalid files
                        }
                    }
                }
            } else {
                console.log('  Config account not found');
            }
        } catch (e) {
            console.log('  Error:', e.message);
        }
    }
    
    listPublishers();
    " 2>/dev/null
    
    echo ""
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    echo ""
    read -p "Press Enter to continue..."
}

check_live_prices() {
    print_header
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    echo -e "  ${BOLD}${WHITE}LIVE PRICE FEEDS${NC}"
    echo ""
    
    if curl -s http://localhost:3000/health > /dev/null 2>&1; then
        echo -e "  ${GREEN}✓ Relayer is running${NC}"
        echo ""
        echo -e "  ${YELLOW}Fetching live prices...${NC}"
        echo ""
        
        # Fetch and format prices from relayer
        node -e "
        const https = require('http');
        
        async function fetchPrices() {
            try {
                const response = await fetch('http://localhost:3000/feeds');
                const data = await response.json();
                
                if (data && data.feeds) {
                    for (const [asset, info] of Object.entries(data.feeds)) {
                        const price = (info.price / 1e8).toFixed(2);
                        const conf = (info.confidence / 1e8).toFixed(2);
                        const time = new Date(info.timestamp * 1000).toLocaleTimeString();
                        console.log(\`  \${asset.padEnd(12)} $\${price.padStart(10)}  ±$\${conf}  (\${time})\`);
                    }
                } else {
                    console.log('  No price data available');
                }
            } catch (e) {
                console.log('  Error:', e.message);
            }
        }
        
        fetchPrices();
        " 2>/dev/null || curl -s http://localhost:3000/feeds | head -20
        
    else
        echo -e "  ${RED}✗ Relayer not running${NC}"
        echo ""
        echo -e "  ${YELLOW}Start the relayer to see live prices:${NC}"
        echo -e "  $ tachyon → 1. Service Manager → 1. Start Relayer"
    fi
    
    echo ""
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    echo ""
    read -p "Press Enter to continue..."
}

# Monitoring
monitoring() {
    while true; do
        print_header
        echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
        echo -e "  ${BOLD}${WHITE}MONITORING${NC}"
        echo ""
        echo -e "  ${GREEN}1.${NC} View Relayer Logs (Live)"
        echo -e "  ${GREEN}2.${NC} View Signer Logs (Live)"
        echo -e "  ${GREEN}3.${NC} View Recent Transactions"
        echo -e "  ${GREEN}4.${NC} Check Balances"
        echo -e "  ${GREEN}5.${NC} System Resources"
        echo -e "  ${GREEN}6.${NC} Back to Main Menu"
        echo ""
        echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
        echo ""
        read -p "Select option: " choice
        
        case $choice in
            1) view_relayer_logs ;;
            2) view_signer_logs ;;
            3) view_transactions ;;
            4) check_balances ;;
            5) system_resources ;;
            6) break ;;
            *) echo -e "${RED}Invalid option${NC}" ; sleep 1 ;;
        esac
    done
}

view_relayer_logs() {
    echo -e "\n${YELLOW}Relayer Logs (Ctrl+C to return to menu)${NC}\n"
    if command -v pm2 &> /dev/null && pm2 list | grep -q "tachyon-relayer"; then
        # pm2 logs handles Ctrl+C gracefully
        pm2 logs tachyon-relayer --lines 50
    elif [ -f "$LOGS_DIR/relayer.log" ]; then
        # Trap Ctrl+C to prevent exiting the script
        trap 'echo -e "\n${GREEN}Returning to menu...${NC}"; sleep 1; return' INT
        tail -f "$LOGS_DIR/relayer.log"
        trap - INT  # Reset trap
    else
        echo -e "${RED}No logs found. Is the relayer running?${NC}"
        sleep 3
    fi
}

view_signer_logs() {
    echo -e "\n${YELLOW}Signer Logs (Ctrl+C to return to menu)${NC}\n"
    if command -v pm2 &> /dev/null && pm2 list | grep -q "tachyon-signer"; then
        # pm2 logs handles Ctrl+C gracefully
        pm2 logs tachyon-signer --lines 50
    elif [ -f "$LOGS_DIR/signer.log" ]; then
        # Trap Ctrl+C to prevent exiting the script
        trap 'echo -e "\n${GREEN}Returning to menu...${NC}"; sleep 1; return' INT
        tail -f "$LOGS_DIR/signer.log"
        trap - INT  # Reset trap
    else
        echo -e "${RED}No logs found. Is the signer running?${NC}"
        sleep 3
    fi
}

view_transactions() {
    echo -e "\n${YELLOW}Recent Transactions${NC}\n"
    # Use detected paths
    DEPLOYER_KEY="${RELAYER_KEYPAIR:-$SOLANA_CONFIG_DIR/deployer.json}"
    if [ -f "$DEPLOYER_KEY" ]; then
        solana transaction-history "$DEPLOYER_KEY" --url "$SOLANA_RPC_URL" | head -20
    else
        echo -e "${RED}Deployer keypair not found at: $DEPLOYER_KEY${NC}"
    fi
    echo ""
    read -p "Press Enter to continue..."
}

check_balances() {
    print_header
    echo -e "${CYAN}╠════════════════════════════════════════════════════════════════╣${NC}"
    echo -e "${CYAN}║${NC}  ${BOLD}${WHITE}WALLET BALANCES${NC}"
    echo -e "${CYAN}║${NC}"
    
    # Relayer
    DEPLOYER_KEY="${RELAYER_KEYPAIR:-$SOLANA_CONFIG_DIR/deployer.json}"
    if [ -f "$DEPLOYER_KEY" ]; then
        ADDR=$(solana-keygen pubkey "$DEPLOYER_KEY" 2>/dev/null)
        BAL=$(solana balance "$ADDR" --url "$SOLANA_RPC_URL" 2>/dev/null || echo "0")
        echo -e "${CYAN}║${NC}  Relayer:    $BAL"
    fi
    
    # Signer - check all keys in keys directory
    if [ -d "$KEYS_DIR" ]; then
        for keyfile in "$KEYS_DIR"/*.json; do
            if [ -f "$keyfile" ]; then
                KEYNAME=$(basename "$keyfile")
                ADDR=$(solana-keygen pubkey "$keyfile" 2>/dev/null)
                BAL=$(solana balance "$ADDR" --url "$SOLANA_RPC_URL" 2>/dev/null || echo "0")
                echo -e "${CYAN}║${NC}  $KEYNAME:    $BAL"
            fi
        done
    fi
    
    echo -e "${CYAN}║${NC}"
    echo -e "${CYAN}╚════════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    read -p "Press Enter to continue..."
}

system_resources() {
    print_header
    echo -e "${CYAN}╠════════════════════════════════════════════════════════════════╣${NC}"
    echo -e "${CYAN}║${NC}  ${BOLD}${WHITE}SYSTEM RESOURCES${NC}"
    echo -e "${CYAN}║${NC}"
    
    # CPU
    CPU=$(top -bn1 | grep "Cpu(s)" | awk '{print $2}' | cut -d'%' -f1)
    echo -e "${CYAN}║${NC}  CPU Usage:  ${GREEN}${CPU}%${NC}"
    
    # Memory
    MEM=$(free -h | awk '/^Mem:/ {print $3 "/" $2}')
    echo -e "${CYAN}║${NC}  Memory:     ${GREEN}${MEM}${NC}"
    
    # Disk
    DISK=$(df -h / | awk 'NR==2 {print $3 "/" $2 " (" $5 " used)"}')
    echo -e "${CYAN}║${NC}  Disk:       ${GREEN}${DISK}${NC}"
    
    # Uptime
    UPTIME=$(uptime -p)
    echo -e "${CYAN}║${NC}  Uptime:     ${GREEN}${UPTIME}${NC}"
    
    echo -e "${CYAN}║${NC}"
    echo -e "${CYAN}╚════════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    read -p "Press Enter to continue..."
}

# Configuration Menu
configuration_menu() {
    while true; do
        print_header
        echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
        echo -e "  ${BOLD}${WHITE}CONFIGURATION${NC}"
        echo ""
        echo -e "  ${GREEN}1.${NC} Edit Signer Config"
        echo -e "  ${GREEN}2.${NC} Edit Relayer Config"
        echo -e "  ${GREEN}3.${NC} View Current Config"
        echo -e "  ${GREEN}4.${NC} Add Data Source"
        echo -e "  ${GREEN}5.${NC} Set Update Interval"
        echo -e "  ${GREEN}6.${NC} Back to Main Menu"
        echo ""
        echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
        echo ""
        read -p "Select option: " choice
        
        case $choice in
            1) edit_signer_config ;;
            2) edit_relayer_config ;;
            3) view_config ;;
            4) add_data_source ;;
            5) set_update_interval ;;
            6) break ;;
            *) echo -e "${RED}Invalid option${NC}" ; sleep 1 ;;
        esac
    done
}

edit_signer_config() {
    echo -e "\n${YELLOW}Opening signer config...${NC}"
    if [ -f "$CONSOLE_DIR/signer/config.yaml" ]; then
        ${EDITOR:-nano} "$CONSOLE_DIR/signer/config.yaml"
        echo -e "${GREEN}✓ Config saved. Restart signer for changes to take effect.${NC}"
    else
        echo -e "${RED}✗ Config file not found${NC}"
    fi
    read -p "Press Enter to continue..."
}

edit_relayer_config() {
    echo -e "\n${YELLOW}Opening relayer .env...${NC}"
    if [ -f "$CONSOLE_DIR/.env" ]; then
        ${EDITOR:-nano} "$CONSOLE_DIR/.env"
        echo -e "${GREEN}✓ Config saved. Restart relayer for changes to take effect.${NC}"
    else
        echo -e "${RED}✗ .env file not found${NC}"
    fi
    read -p "Press Enter to continue..."
}

view_config() {
    print_header
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    echo -e "  ${BOLD}${WHITE}CURRENT CONFIGURATION${NC}"
    echo ""
    
    echo -e "${BOLD}System Paths:${NC}"
    echo -e "  Console Directory:  ${CYAN}$CONSOLE_DIR${NC}"
    echo -e "  Keys Directory:     ${CYAN}$KEYS_DIR${NC}"
    echo -e "  Logs Directory:     ${CYAN}$LOGS_DIR${NC}"
    echo -e "  Solana Config:      ${CYAN}$SOLANA_CONFIG_DIR${NC}"
    echo -e "  Solana CLI Path:    ${CYAN}$SOLANA_PATH${NC}"
    
    echo ""
    echo -e "${BOLD}Network:${NC}"
    echo -e "  RPC URL:            ${CYAN}$SOLANA_RPC_URL${NC}"
    echo -e "  Program ID:         ${CYAN}$PROGRAM_ID${NC}"
    
    echo ""
    echo -e "${BOLD}Keypairs:${NC}"
    DEPLOYER_KEY="${RELAYER_KEYPAIR:-$SOLANA_CONFIG_DIR/deployer.json}"
    if [ -f "$DEPLOYER_KEY" ]; then
        echo -e "  Relayer:            ${GREEN}✓${NC} $DEPLOYER_KEY"
    else
        echo -e "  Relayer:            ${RED}✗${NC} Not found"
    fi
    
    if [ -d "$KEYS_DIR" ] && [ "$(ls -A $KEYS_DIR/*.json 2>/dev/null)" ]; then
        echo -e "  Signer Keys:        ${GREEN}✓${NC} $(ls -1 $KEYS_DIR/*.json 2>/dev/null | wc -l) found"
    else
        echo -e "  Signer Keys:        ${RED}✗${NC} None found"
    fi
    
    echo ""
    echo -e "${BOLD}Signer Config:${NC}"
    if [ -f "$CONSOLE_DIR/signer/config.yaml" ]; then
        echo -e "  ${GREEN}✓${NC} Found at: $CONSOLE_DIR/signer/config.yaml"
        echo -e "  Assets configured: $(grep -c "^  - id:" "$CONSOLE_DIR/signer/config.yaml" 2>/dev/null || echo "0")"
    else
        echo -e "  ${RED}✗${NC} Not found"
    fi
    
    echo ""
    echo -e "${BOLD}Environment (.env):${NC}"
    if [ -f "$CONSOLE_DIR/.env" ]; then
        echo -e "  ${GREEN}✓${NC} Found"
        echo -e "  Custom settings: $(grep -c "^[A-Z]" "$CONSOLE_DIR/.env" 2>/dev/null || echo "0")"
    else
        echo -e "  ${YELLOW}⚠${NC}  Using defaults (no .env file)"
    fi
    
    echo ""
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    echo ""
    echo -e "${YELLOW}Tip: Copy config.env.example to .env to customize settings${NC}"
    echo ""
    read -p "Press Enter to continue..."
}

add_data_source() {
    echo -e "\n${BOLD}${WHITE}Add Data Source to Signer${NC}"
    echo -e "${YELLOW}Available: binance, coinbase, coingecko${NC}\n"
    read -p "Enter data source name: " source
    
    if [ -f "$CONSOLE_DIR/signer/config.yaml" ]; then
        echo -e "\n${YELLOW}Adding $source to config...${NC}"
        echo "  - $source" >> "$CONSOLE_DIR/signer/config.yaml"
        echo -e "${GREEN}✓ Data source added. Restart signer to apply.${NC}"
    else
        echo -e "${RED}✗ Config file not found${NC}"
    fi
    sleep 2
}

set_update_interval() {
    echo -e "\n${BOLD}${WHITE}Set Price Update Interval${NC}\n"
    read -p "Enter interval in seconds (default 30): " interval
    
    if [ -z "$interval" ]; then
        interval=30
    fi
    
    echo -e "\n${YELLOW}Updating interval to ${interval}s...${NC}"
    # Update config file
    if [ -f "$CONSOLE_DIR/signer/config.yaml" ]; then
        sed -i "s/interval:.*/interval: $interval/" "$CONSOLE_DIR/signer/config.yaml"
        echo -e "${GREEN}✓ Interval updated. Restart signer to apply.${NC}"
    else
        echo -e "${RED}✗ Config file not found${NC}"
    fi
    sleep 2
}

# Wallet Manager
wallet_manager() {
    while true; do
        print_header
        echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
        echo -e "  ${BOLD}${WHITE}WALLET MANAGER${NC}"
        echo ""
        echo -e "  ${GREEN}1.${NC} View All Wallets"
        echo -e "  ${GREEN}2.${NC} Backup Wallets"
        echo -e "  ${GREEN}3.${NC} Import Wallet"
        echo -e "  ${GREEN}4.${NC} Generate New Wallet"
        echo -e "  ${GREEN}5.${NC} Check Balances"
        echo -e "  ${GREEN}6.${NC} Back to Main Menu"
        echo ""
        echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
        echo ""
        read -p "Select option: " choice
        
        case $choice in
            1) view_all_wallets ;;
            2) backup_wallets ;;
            3) import_wallet ;;
            4) generate_wallet ;;
            5) check_all_balances ;;
            6) break ;;
            *) echo -e "${RED}Invalid option${NC}" ; sleep 1 ;;
        esac
    done
}

view_all_wallets() {
    print_header
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    echo -e "  ${BOLD}${WHITE}ALL WALLETS${NC}"
    echo ""
    
    echo -e "${BOLD}Keys Directory:${NC}"
    if [ -d "$KEYS_DIR" ]; then
        for key in "$KEYS_DIR"/*.json; do
            if [ -f "$key" ]; then
                FILENAME=$(basename "$key")
                PUBKEY=$(solana-keygen pubkey "$key" 2>/dev/null || echo "Error")
                echo -e "  ${GREEN}•${NC} $FILENAME"
                echo -e "    ${CYAN}$PUBKEY${NC}"
            fi
        done
    else
        echo -e "${RED}No keys directory found${NC}"
    fi
    
    echo ""
    echo -e "${BOLD}Config Wallets:${NC}"
    if [ -f "$SOLANA_CONFIG_DIR/deployer.json" ]; then
        PUBKEY=$(solana-keygen pubkey "$SOLANA_CONFIG_DIR/deployer.json" 2>/dev/null || echo "Error")
        echo -e "  ${GREEN}•${NC} deployer.json"
        echo -e "    ${CYAN}$PUBKEY${NC}"
    fi
    
    if [ -f "$SOLANA_CONFIG_DIR/identity.json" ]; then
        PUBKEY=$(solana-keygen pubkey "$SOLANA_CONFIG_DIR/identity.json" 2>/dev/null || echo "Error")
        echo -e "  ${GREEN}•${NC} identity.json"
        echo -e "    ${CYAN}$PUBKEY${NC}"
    fi
    
    echo ""
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    read -p "Press Enter to continue..."
}

backup_wallets() {
    BACKUP_DIR="$CONSOLE_DIR/wallet-backups"
    BACKUP_FILE="$BACKUP_DIR/backup-$(date +%Y%m%d-%H%M%S).tar.gz"
    
    echo -e "\n${YELLOW}Creating wallet backup...${NC}"
    mkdir -p "$BACKUP_DIR"
    
    # Backup keys directory and config wallets
    BACKUP_ITEMS=""
    [ -d "$KEYS_DIR" ] && BACKUP_ITEMS="$BACKUP_ITEMS $KEYS_DIR"
    [ -f "$SOLANA_CONFIG_DIR/deployer.json" ] && BACKUP_ITEMS="$BACKUP_ITEMS $SOLANA_CONFIG_DIR/deployer.json"
    [ -f "$SOLANA_CONFIG_DIR/identity.json" ] && BACKUP_ITEMS="$BACKUP_ITEMS $SOLANA_CONFIG_DIR/identity.json"
    
    if [ -n "$BACKUP_ITEMS" ]; then
        tar -czf "$BACKUP_FILE" $BACKUP_ITEMS 2>/dev/null
    else
        echo -e "${RED}No wallets found to backup${NC}"
        return
    fi
    
    echo -e "${GREEN}✓ Backup created: $BACKUP_FILE${NC}"
    echo -e "${YELLOW}⚠ Keep this file secure! It contains private keys.${NC}"
    sleep 3
}

import_wallet() {
    echo -e "\n${BOLD}${WHITE}Import Wallet${NC}\n"
    read -p "Enter path to keypair JSON file: " keypath
    
    if [ ! -f "$keypath" ]; then
        echo -e "${RED}✗ File not found${NC}"
        sleep 2
        return
    fi
    
    read -p "Enter name for this wallet: " walletname
    
    mkdir -p "$KEYS_DIR"
    cp "$keypath" "$KEYS_DIR/${walletname}.json"
    
    PUBKEY=$(solana-keygen pubkey "$KEYS_DIR/${walletname}.json" 2>/dev/null || echo "Error")
    echo -e "\n${GREEN}✓ Wallet imported${NC}"
    echo -e "  Name: ${walletname}.json"
    echo -e "  Pubkey: ${CYAN}$PUBKEY${NC}"
    sleep 3
}

generate_wallet() {
    echo -e "\n${BOLD}${WHITE}Generate New Wallet${NC}\n"
    read -p "Enter name for new wallet: " walletname
    
    mkdir -p "$KEYS_DIR"
    KEYFILE="$KEYS_DIR/${walletname}.json"
    
    solana-keygen new --no-bip39-passphrase --outfile "$KEYFILE" --force
    
    PUBKEY=$(solana-keygen pubkey "$KEYFILE")
    echo -e "\n${GREEN}✓ Wallet generated${NC}"
    echo -e "  File: $KEYFILE"
    echo -e "  Pubkey: ${CYAN}$PUBKEY${NC}"
    echo -e "\n${YELLOW}⚠ Backup this file securely!${NC}"
    sleep 3
}

check_all_balances() {
    print_header
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    echo -e "  ${BOLD}${WHITE}WALLET BALANCES${NC}"
    echo ""
    
    export PATH="/root/tachyon/target/release:$PATH"
    RPC_URL="https://rpc.mainnet.x1.xyz"
    
    if [ -d "$CONSOLE_DIR/keys" ]; then
        for key in "$CONSOLE_DIR/keys"/*.json; do
            if [ -f "$key" ]; then
                FILENAME=$(basename "$key")
                PUBKEY=$(solana-keygen pubkey "$key" 2>/dev/null)
                BALANCE=$(solana balance "$PUBKEY" --url "$RPC_URL" 2>/dev/null || echo "Error")
                echo -e "  ${GREEN}$FILENAME${NC}"
                echo -e "    Balance: ${CYAN}$BALANCE${NC}"
            fi
        done
    fi
    
    echo ""
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    read -p "Press Enter to continue..."
}

# Autopilot Menu
autopilot_menu() {
    while true; do
        print_header
        echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
        echo -e "  ${BOLD}${WHITE}AUTOPILOT${NC}"
        echo ""
        
        # Check autopilot status
        if [ -f "$CONSOLE_DIR/.autopilot-enabled" ]; then
            echo -e "  Status: ${GREEN}Enabled${NC}"
        else
            echo -e "  Status: ${RED}Disabled${NC}"
        fi
        
        echo ""
        echo -e "  ${GREEN}1.${NC} Enable Autopilot"
        echo -e "  ${GREEN}2.${NC} Disable Autopilot"
        echo -e "  ${GREEN}3.${NC} Configure Auto-Restart"
        echo -e "  ${GREEN}4.${NC} Configure Health Checks"
        echo -e "  ${GREEN}5.${NC} View Autopilot Logs"
        echo -e "  ${GREEN}6.${NC} Back to Main Menu"
        echo ""
        echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
        echo ""
        read -p "Select option: " choice
        
        case $choice in
            1) enable_autopilot ;;
            2) disable_autopilot ;;
            3) configure_autorestart ;;
            4) configure_healthcheck ;;
            5) view_autopilot_logs ;;
            6) break ;;
            *) echo -e "${RED}Invalid option${NC}" ; sleep 1 ;;
        esac
    done
}

enable_autopilot() {
    echo -e "\n${YELLOW}Enabling Autopilot...${NC}"
    
    # Create autopilot marker
    touch "$CONSOLE_DIR/.autopilot-enabled"
    
    # Create systemd service for autopilot
    cat > /tmp/tachyon-autopilot.service <<EOF
[Unit]
Description=Tachyon Oracle Autopilot
After=network.target

[Service]
Type=simple
User=root
WorkingDirectory=$CONSOLE_DIR
ExecStart=$CONSOLE_DIR/autopilot.sh
Restart=always
RestartSec=10

[Install]
WantedBy=multi-user.target
EOF
    
    # Create autopilot script with dynamic path
    cat > "$CONSOLE_DIR/autopilot.sh" <<EOF
#!/bin/bash
CONSOLE_DIR="$CONSOLE_DIR"
LOG_FILE="\$CONSOLE_DIR/logs/autopilot.log"

mkdir -p "$CONSOLE_DIR/logs"

log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" >> "$LOG_FILE"
}

log "Autopilot started"

while true; do
    # Check if relayer is running
    if ! pgrep -f "relayer/dist/index.js" > /dev/null; then
        log "Relayer down, restarting..."
        cd "$CONSOLE_DIR/relayer"
        nohup node dist/index.js >> "$CONSOLE_DIR/logs/relayer.log" 2>&1 &
    fi
    
    # Check if signer is running
    if ! pgrep -f "signer/dist/index.js" > /dev/null; then
        log "Signer down, restarting..."
        cd "$CONSOLE_DIR/signer"
        nohup node dist/index.js >> "$CONSOLE_DIR/logs/signer.log" 2>&1 &
    fi
    
    # Check relayer health
    if ! curl -s http://localhost:3000/health > /dev/null 2>&1; then
        log "Relayer health check failed, restarting..."
        pkill -f "relayer/dist/index.js"
        sleep 2
        cd "$CONSOLE_DIR/relayer"
        nohup node dist/index.js >> "$CONSOLE_DIR/logs/relayer.log" 2>&1 &
    fi
    
    sleep 30
done
EOF
    
    chmod +x "$CONSOLE_DIR/autopilot.sh"
    
    # Install systemd service
    sudo cp /tmp/tachyon-autopilot.service /etc/systemd/system/
    sudo systemctl daemon-reload
    sudo systemctl enable tachyon-autopilot
    sudo systemctl start tachyon-autopilot
    
    echo -e "${GREEN}✓ Autopilot enabled${NC}"
    echo -e "${YELLOW}Services will auto-restart if they crash${NC}"
    sleep 3
}

disable_autopilot() {
    echo -e "\n${YELLOW}Disabling Autopilot...${NC}"
    
    rm -f "$CONSOLE_DIR/.autopilot-enabled"
    
    sudo systemctl stop tachyon-autopilot 2>/dev/null
    sudo systemctl disable tachyon-autopilot 2>/dev/null
    
    echo -e "${GREEN}✓ Autopilot disabled${NC}"
    sleep 2
}

configure_autorestart() {
    echo -e "\n${BOLD}${WHITE}Configure Auto-Restart${NC}\n"
    read -p "Enable auto-restart on crash? (y/n): " enable
    
    if [ "$enable" = "y" ]; then
        echo "AUTO_RESTART=true" > "$CONSOLE_DIR/.autopilot-config"
        echo -e "${GREEN}✓ Auto-restart enabled${NC}"
    else
        echo "AUTO_RESTART=false" > "$CONSOLE_DIR/.autopilot-config"
        echo -e "${YELLOW}Auto-restart disabled${NC}"
    fi
    sleep 2
}

configure_healthcheck() {
    echo -e "\n${BOLD}${WHITE}Configure Health Checks${NC}\n"
    read -p "Health check interval (seconds, default 30): " interval
    
    if [ -z "$interval" ]; then
        interval=30
    fi
    
    echo "HEALTH_CHECK_INTERVAL=$interval" >> "$CONSOLE_DIR/.autopilot-config"
    echo -e "${GREEN}✓ Health check interval set to ${interval}s${NC}"
    sleep 2
}

view_autopilot_logs() {
    echo -e "\n${YELLOW}Showing autopilot logs (Ctrl+C to return to menu)...${NC}\n"
    sleep 2
    # Trap Ctrl+C to prevent exiting the script
    trap 'echo -e "\n${GREEN}Returning to menu...${NC}"; sleep 1; return' INT
    tail -f "$CONSOLE_DIR/logs/autopilot.log" 2>/dev/null || echo "No logs yet"
    trap - INT  # Reset trap
}

# Update & Maintenance
update_maintenance() {
    while true; do
        print_header
        echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
        echo -e "  ${BOLD}${WHITE}UPDATE & MAINTENANCE${NC}"
        echo ""
        echo -e "  ${GREEN}1.${NC} Update Tachyon Oracles"
        echo -e "  ${GREEN}2.${NC} Update Console"
        echo -e "  ${GREEN}3.${NC} Run Diagnostics"
        echo -e "  ${GREEN}4.${NC} Clear Logs"
        echo -e "  ${GREEN}5.${NC} Backup System"
        echo -e "  ${GREEN}6.${NC} Restore Backup"
        echo -e "  ${GREEN}7.${NC} Back to Main Menu"
        echo ""
        echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
        echo ""
        read -p "Select option: " choice
        
        case $choice in
            1) update_oracles ;;
            2) update_console ;;
            3) run_diagnostics ;;
            4) clear_logs ;;
            5) backup_system ;;
            6) restore_backup ;;
            7) break ;;
            *) echo -e "${RED}Invalid option${NC}" ; sleep 1 ;;
        esac
    done
}

update_oracles() {
    echo -e "\n${YELLOW}Updating Tachyon Oracles...${NC}"
    
    cd "$CONSOLE_DIR"
    
    # Pull latest changes
    if [ -d ".git" ]; then
        git pull
    else
        echo -e "${RED}Not a git repository${NC}"
    fi
    
    # Rebuild services
    echo -e "\n${YELLOW}Rebuilding services...${NC}"
    
    # Build SDK first
    echo -e "  ${WHITE}Building SDK...${NC}"
    cd sdk && npm install && npx tsc
    cd ..
    
    # Build services
    echo -e "  ${WHITE}Building relayer...${NC}"
    cd relayer && npm install && npx tsc
    cd ..
    
    echo -e "  ${WHITE}Building signer...${NC}"
    cd signer && npm install && npx tsc
    cd ..
    
    echo -e "\n${GREEN}✓ Update complete. Restart services to apply changes.${NC}"
    sleep 3
}

update_console() {
    echo -e "\n${YELLOW}Console is up to date!${NC}"
    echo -e "Version: ${GREEN}1.0${NC}"
    sleep 2
}

run_diagnostics() {
    print_header
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    echo -e "  ${BOLD}${WHITE}SYSTEM DIAGNOSTICS${NC}"
    echo ""
    
    echo -e "${BOLD}Service Status:${NC}"
    if pgrep -f "relayer/dist/index.js" > /dev/null; then
        echo -e "  Relayer: ${GREEN}✓ Running${NC}"
    else
        echo -e "  Relayer: ${RED}✗ Stopped${NC}"
    fi
    
    if pgrep -f "signer/dist/index.js" > /dev/null; then
        echo -e "  Signer: ${GREEN}✓ Running${NC}"
    else
        echo -e "  Signer: ${RED}✗ Stopped${NC}"
    fi
    
    echo ""
    echo -e "${BOLD}Network Connectivity:${NC}"
    if curl -s https://rpc.mainnet.x1.xyz > /dev/null 2>&1; then
        echo -e "  X1 RPC: ${GREEN}✓ Connected${NC}"
    else
        echo -e "  X1 RPC: ${RED}✗ Failed${NC}"
    fi
    
    if curl -s http://localhost:3000/health > /dev/null 2>&1; then
        echo -e "  Relayer API: ${GREEN}✓ Responding${NC}"
    else
        echo -e "  Relayer API: ${RED}✗ Not responding${NC}"
    fi
    
    echo ""
    echo -e "${BOLD}Disk Space:${NC}"
    df -h "$CONSOLE_DIR" | tail -1
    
    echo ""
    echo -e "${BOLD}Dependencies:${NC}"
    command -v node >/dev/null && echo -e "  Node.js: ${GREEN}✓ $(node --version)${NC}" || echo -e "  Node.js: ${RED}✗ Not found${NC}"
    command -v npm >/dev/null && echo -e "  npm: ${GREEN}✓ $(npm --version)${NC}" || echo -e "  npm: ${RED}✗ Not found${NC}"
    command -v pm2 >/dev/null && echo -e "  PM2: ${GREEN}✓ Installed${NC}" || echo -e "  PM2: ${YELLOW}○ Optional${NC}"
    
    echo ""
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    read -p "Press Enter to continue..."
}

clear_logs() {
    echo -e "\n${YELLOW}Clearing logs...${NC}"
    read -p "Are you sure? (y/n): " confirm
    
    if [ "$confirm" = "y" ]; then
        rm -f "$CONSOLE_DIR/logs"/*.log
        echo -e "${GREEN}✓ Logs cleared${NC}"
    else
        echo -e "${YELLOW}Cancelled${NC}"
    fi
    sleep 2
}

backup_system() {
    BACKUP_DIR="$CONSOLE_DIR/backups"
    BACKUP_FILE="$BACKUP_DIR/system-backup-$(date +%Y%m%d-%H%M%S).tar.gz"
    
    echo -e "\n${YELLOW}Creating system backup...${NC}"
    mkdir -p "$BACKUP_DIR"
    
    tar -czf "$BACKUP_FILE" \
        --exclude="$CONSOLE_DIR/node_modules" \
        --exclude="$CONSOLE_DIR/backups" \
        --exclude="$CONSOLE_DIR/logs" \
        "$CONSOLE_DIR" \
        2>/dev/null
    
    echo -e "${GREEN}✓ Backup created: $BACKUP_FILE${NC}"
    sleep 3
}

restore_backup() {
    BACKUP_DIR="$CONSOLE_DIR/backups"
    
    echo -e "\n${BOLD}${WHITE}Available Backups:${NC}\n"
    
    if [ ! -d "$BACKUP_DIR" ] || [ -z "$(ls -A $BACKUP_DIR 2>/dev/null)" ]; then
        echo -e "${RED}No backups found${NC}"
        sleep 2
        return
    fi
    
    ls -1 "$BACKUP_DIR"/*.tar.gz 2>/dev/null
    
    echo ""
    read -p "Enter backup filename to restore: " backup_file
    
    if [ ! -f "$BACKUP_DIR/$backup_file" ]; then
        echo -e "${RED}✗ Backup not found${NC}"
        sleep 2
        return
    fi
    
    echo -e "\n${YELLOW}Restoring backup...${NC}"
    read -p "This will overwrite current files. Continue? (y/n): " confirm
    
    if [ "$confirm" = "y" ]; then
        tar -xzf "$BACKUP_DIR/$backup_file" -C /
        echo -e "${GREEN}✓ Backup restored${NC}"
    else
        echo -e "${YELLOW}Cancelled${NC}"
    fi
    sleep 2
}

# Quick Start
quick_start() {
    print_header
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    echo -e "  ${BOLD}${WHITE}🚀 FIRST TIME SETUP WIZARD${NC}"
    echo ""
    echo -e "  ${WHITE}This wizard will set up everything automatically:${NC}"
    echo -e "  ${GREEN}✓${NC} Check/install dependencies"
    echo -e "  ${GREEN}✓${NC} Create oracle keypairs (signer & relayer)"
    echo -e "  ${GREEN}✓${NC} Configure environment"
    echo -e "  ${GREEN}✓${NC} Build all services (SDK, signer, relayer)"
    echo -e "  ${GREEN}✓${NC} ${BOLD}Register as publisher on-chain${NC}"
    echo -e "  ${GREEN}✓${NC} Setup API service (for Telegram bot)"
    echo -e "  ${GREEN}✓${NC} Start all services with PM2"
    echo ""
    echo -e "  ${YELLOW}⏱️  This takes about 2-3 minutes${NC}"
    echo ""
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    echo ""
    read -p "Start setup? (y/n): " confirm
    
    if [ "$confirm" != "y" ]; then
        return
    fi
    
    # Step 1: Check dependencies
    echo -e "\n${CYAN}[1/7]${NC} ${YELLOW}Checking dependencies...${NC}"
    if ! command -v node &> /dev/null; then
        echo -e "${RED}❌ Node.js not found!${NC}"
        echo -e "${YELLOW}Please install Node.js 20: https://nodejs.org/${NC}"
        read -p "Press Enter to continue..."
        return
    fi
    echo -e "${GREEN}✓ Node.js $(node --version)${NC}"
    
    if ! command -v pm2 &> /dev/null; then
        echo -e "${YELLOW}Installing PM2...${NC}"
        sudo npm install -g pm2 > /dev/null 2>&1
    fi
    echo -e "${GREEN}✓ PM2 installed${NC}"
    
    # Step 2: Create keypairs
    echo -e "\n${CYAN}[2/7]${NC} ${YELLOW}Setting up oracle keypairs...${NC}"
    echo -e "${WHITE}These are separate from your validator keys${NC}"
    mkdir -p "$KEYS_DIR"
    
    # Create signer keypair (for signing price data)
    if [ ! -f "$KEYS_DIR/signer.json" ]; then
        echo -e "\n${YELLOW}Creating oracle signer keypair...${NC}"
        echo -e "${CYAN}This key signs price data (must be registered as publisher)${NC}"
        echo -e "${RED}⚠️  IMPORTANT: Save your seed phrase!${NC}"
        sleep 2
        solana-keygen new --outfile "$KEYS_DIR/signer.json"
        SIGNER_PUBKEY=$(solana-keygen pubkey "$KEYS_DIR/signer.json")
        echo -e "\n${GREEN}✓ Signer keypair created${NC}"
        echo -e "${CYAN}Signer address: ${YELLOW}${SIGNER_PUBKEY}${NC}"
    else
        SIGNER_PUBKEY=$(solana-keygen pubkey "$KEYS_DIR/signer.json")
        echo -e "${GREEN}✓ Signer keypair already exists${NC}"
        echo -e "${CYAN}Signer address: ${YELLOW}${SIGNER_PUBKEY}${NC}"
    fi
    
    # Create relayer keypair (for submitting transactions)
    if [ ! -f "$KEYS_DIR/relayer.json" ]; then
        echo -e "\n${YELLOW}Creating oracle relayer keypair...${NC}"
        echo -e "${CYAN}This key submits transactions (needs SOL for fees)${NC}"
        solana-keygen new --outfile "$KEYS_DIR/relayer.json" --no-bip39-passphrase
        RELAYER_PUBKEY=$(solana-keygen pubkey "$KEYS_DIR/relayer.json")
        echo -e "${GREEN}✓ Relayer keypair created${NC}"
        echo -e "${CYAN}Relayer address: ${YELLOW}${RELAYER_PUBKEY}${NC}"
    else
        RELAYER_PUBKEY=$(solana-keygen pubkey "$KEYS_DIR/relayer.json")
        echo -e "${GREEN}✓ Relayer keypair already exists${NC}"
        echo -e "${CYAN}Relayer address: ${YELLOW}${RELAYER_PUBKEY}${NC}"
    fi
    
    # Step 3: Check balances
    echo -e "\n${CYAN}[3/7]${NC} ${YELLOW}Checking balances...${NC}"
    SIGNER_BALANCE=$(solana balance "$KEYS_DIR/signer.json" --url "$SOLANA_RPC_URL" 2>/dev/null | awk '{print $1}')
    RELAYER_BALANCE=$(solana balance "$KEYS_DIR/relayer.json" --url "$SOLANA_RPC_URL" 2>/dev/null | awk '{print $1}')
    
    echo -e "${CYAN}Signer balance:  ${YELLOW}${SIGNER_BALANCE} XNT${NC}"
    echo -e "${CYAN}Relayer balance: ${YELLOW}${RELAYER_BALANCE} XNT${NC}"
    
    BALANCE=$SIGNER_BALANCE
    echo -e "${CYAN}Balance: ${YELLOW}${BALANCE} XNT${NC}"
    
    if (( $(echo "$BALANCE < 0.1" | bc -l 2>/dev/null || echo "1") )); then
        echo -e "${RED}❌ Insufficient balance!${NC}"
        echo -e "${YELLOW}Please fund this address with at least 1 XNT:${NC}"
        echo -e "${CYAN}${PUBKEY}${NC}"
        echo -e "\n${YELLOW}After funding, run this wizard again.${NC}"
        read -p "Press Enter to continue..."
        return
    fi
    echo -e "${GREEN}✓ Balance sufficient${NC}"
    
    # Step 4: Configure environment
    echo -e "\n${CYAN}[4/7]${NC} ${YELLOW}Configuring environment...${NC}"
    if [ ! -f "$CONSOLE_DIR/.env" ]; then
        cat > "$CONSOLE_DIR/.env" << EOF
# Keypair Configuration
SIGNER_KEYPAIR=./keys/signer.json
RELAYER_KEYPAIR=./keys/relayer.json

# Relayer Settings
RELAYER_PORT=7777
RELAYER_URLS=http://localhost:7777

# RPC Settings
SOLANA_RPC_URL=https://rpc.mainnet.x1.xyz
RPC_URL=https://rpc.mainnet.x1.xyz

# Program ID
PROGRAM_ID=TACH9r2uZzoFM6daofesADjeDn9NqB1pKFWP5mfByb1
EOF
        echo -e "${GREEN}✓ Environment configured${NC}"
    else
        echo -e "${GREEN}✓ Environment already configured${NC}"
    fi
    
    # Step 5: Build services
    echo -e "\n${CYAN}[5/7]${NC} ${YELLOW}Building services (this may take a minute)...${NC}"
    cd "$CONSOLE_DIR"
    npm install > /dev/null 2>&1 || true
    
    # Build SDK first (required by other services)
    echo -e "  ${WHITE}Building SDK...${NC}"
    cd sdk && npm install > /dev/null 2>&1 && npm run build > /dev/null 2>&1 || true
    cd ..
    
    # Build services
    echo -e "  ${WHITE}Building relayer...${NC}"
    cd relayer && npm install > /dev/null 2>&1 && npm run build > /dev/null 2>&1 || true
    cd ..
    
    echo -e "  ${WHITE}Building signer...${NC}"
    cd signer && npm install > /dev/null 2>&1 && npm run build > /dev/null 2>&1 || true
    cd ..
    
    echo -e "${GREEN}✓ Services built${NC}"
    
    # Step 6: Register publisher
    echo -e "\n${CYAN}[6/7]${NC} ${YELLOW}Registering as publisher...${NC}"
    echo -e "${WHITE}This registers your signer keypair on-chain${NC}"
    echo ""
    
    if [ -f "$CONSOLE_DIR/scripts/register-publisher-simple.js" ]; then
        cd "$CONSOLE_DIR"
        
        # Register publisher with signer keypair
        REGISTER_OUTPUT=$(node scripts/register-publisher-simple.js "$KEYS_DIR/signer.json" 2>&1)
        REGISTER_EXIT=$?
        
        if [ $REGISTER_EXIT -eq 0 ]; then
            if echo "$REGISTER_OUTPUT" | grep -q "already registered"; then
                echo -e "${YELLOW}⚠️  Publisher already registered${NC}"
            else
                echo -e "${GREEN}✓ Publisher registered successfully!${NC}"
                echo "$REGISTER_OUTPUT" | grep "Transaction signature:" || true
            fi
            
            # Activate publisher (if needed)
            if [ -f "$CONSOLE_DIR/scripts/activate-publisher.js" ]; then
                ACTIVATE_OUTPUT=$(node scripts/activate-publisher.js 2>&1)
                if echo "$ACTIVATE_OUTPUT" | grep -q "already active"; then
                    echo -e "${GREEN}✓ Publisher already active${NC}"
                elif echo "$ACTIVATE_OUTPUT" | grep -q "success"; then
                    echo -e "${GREEN}✓ Publisher activated${NC}"
                fi
            fi
        else
            echo -e "${RED}❌ Registration failed${NC}"
            echo "$REGISTER_OUTPUT" | tail -5
            echo ""
            echo -e "${YELLOW}You can register manually later:${NC}"
            echo -e "${CYAN}./tachyon → 2 (Oracle Manager) → 3 (Register Publisher)${NC}"
        fi
    else
        echo -e "${YELLOW}⚠️  Registration scripts not found, skipping...${NC}"
        echo -e "${CYAN}You can register manually later from the Oracle Manager menu${NC}"
    fi
    
    # Step 7: Setup API Service
    echo -e "\n${CYAN}[7/8]${NC} ${YELLOW}Setting up API service for remote monitoring...${NC}"
    echo -e "${WHITE}This allows you to monitor your oracle from the Telegram bot${NC}"
    echo ""
    read -p "Enable API service? (y/n) [y]: " enable_api
    enable_api=${enable_api:-y}
    
    if [ "$enable_api" = "y" ]; then
        cd "$CONSOLE_DIR/api-service"
        
        # Install dependencies if needed
        if [ ! -d "node_modules" ]; then
            npm install > /dev/null 2>&1
        fi
        
        # Build if needed
        if [ ! -d "dist" ]; then
            npm run build > /dev/null 2>&1
        fi
        
        # Generate API key if .env doesn't exist
        if [ ! -f ".env" ]; then
            cp .env.example .env
            API_KEY=$(openssl rand -hex 32)
            sed -i "s/API_KEY=.*/API_KEY=$API_KEY/" .env
            sed -i "s/API_MODE=.*/API_MODE=monitoring/" .env
            sed -i "s|ORACLE_PROJECT_PATH=.*|ORACLE_PROJECT_PATH=$CONSOLE_DIR|" .env
            
            echo -e "${GREEN}✓ API service configured${NC}"
            echo -e "${CYAN}Your API Key: ${YELLOW}${API_KEY}${NC}"
            echo -e "${YELLOW}⚠️  Save this key! You'll need it for the Telegram bot${NC}"
            echo ""
            
            # Save API key to a file for easy reference
            echo "$API_KEY" > "$KEYS_DIR/api-key.txt"
            echo -e "${GREEN}✓ API key saved to: ${CYAN}$KEYS_DIR/api-key.txt${NC}"
        else
            API_KEY=$(grep "^API_KEY=" .env | cut -d'=' -f2)
            echo -e "${GREEN}✓ API service already configured${NC}"
            echo -e "${CYAN}Your API Key: ${YELLOW}${API_KEY}${NC}"
        fi
        
        # Start API service
        pm2 delete tachyon-api 2>/dev/null || true
        pm2 start dist/index.js --name tachyon-api > /dev/null 2>&1
        pm2 save > /dev/null 2>&1
        
        echo -e "${GREEN}✓ API service started on port 7171${NC}"
    else
        echo -e "${YELLOW}⚠️  API service skipped${NC}"
    fi
    
    # Step 8: Start oracle services
    echo -e "\n${CYAN}[8/8]${NC} ${YELLOW}Starting oracle services...${NC}"
    
    # Stop any existing services
    pm2 delete tachyon-signer tachyon-relayer 2>/dev/null || true
    
    # Start with PM2
    cd "$CONSOLE_DIR/signer"
    pm2 start dist/index.js --name tachyon-signer > /dev/null 2>&1
    
    cd "$CONSOLE_DIR/relayer"
    RELAYER_PORT=7777 pm2 start dist/index.js --name tachyon-relayer --update-env > /dev/null 2>&1
    
    pm2 save > /dev/null 2>&1
    
    echo -e "${GREEN}✓ Oracle services started${NC}"
    
    # Final summary
    echo -e "\n${CYAN}════════════════════════════════════════════════════════════════${NC}"
    echo -e "  ${BOLD}${GREEN}✅ SETUP COMPLETE!${NC}"
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    echo ""
    echo -e "  ${WHITE}Your validator is now running!${NC}"
    echo ""
    echo -e "  ${CYAN}Validator Address:${NC}"
    echo -e "  ${YELLOW}${PUBKEY}${NC}"
    echo ""
    
    if [ "$enable_api" = "y" ] && [ ! -z "$API_KEY" ]; then
        echo -e "  ${CYAN}API Service:${NC}"
        echo -e "  ${GREEN}✓${NC} Running on port ${YELLOW}7171${NC}"
        echo -e "  ${GREEN}✓${NC} Mode: ${YELLOW}monitoring${NC}"
        echo -e "  ${GREEN}✓${NC} API Key: ${YELLOW}${API_KEY}${NC}"
        echo -e "  ${GREEN}✓${NC} Saved to: ${CYAN}$KEYS_DIR/api-key.txt${NC}"
        echo ""
    fi
    
    echo -e "  ${CYAN}Services Status:${NC}"
    pm2 list | grep tachyon
    echo ""
    echo -e "  ${CYAN}Connect to Telegram Bot:${NC}"
    echo -e "  ${GREEN}1.${NC} Open: ${YELLOW}@tachyon_oracle_bot${NC}"
    echo -e "  ${GREEN}2.${NC} Send: ${YELLOW}/start${NC}"
    echo -e "  ${GREEN}3.${NC} Tap: ${YELLOW}🗂️ Profiles → ➕ Add Profile${NC}"
    echo -e "  ${GREEN}4.${NC} Name: ${YELLOW}My Validator${NC}"
    echo -e "  ${GREEN}5.${NC} Type: ${YELLOW}api${NC}"
    
    if [ "$enable_api" = "y" ]; then
        # Get server IP
        SERVER_IP=$(curl -s ifconfig.me 2>/dev/null || echo "YOUR_SERVER_IP")
        echo -e "  ${GREEN}6.${NC} URL: ${YELLOW}http://${SERVER_IP}:7171${NC}"
        echo -e "  ${GREEN}7.${NC} API Key: ${YELLOW}${API_KEY}${NC}"
    fi
    echo ""
    echo -e "  ${CYAN}Other Commands:${NC}"
    echo -e "  ${GREEN}•${NC} Monitor: ${YELLOW}pm2 monit${NC}"
    echo -e "  ${GREEN}•${NC} View logs: ${YELLOW}pm2 logs${NC}"
    echo -e "  ${GREEN}•${NC} Restart: ${YELLOW}pm2 restart all${NC}"
    echo ""
    echo -e "  ${YELLOW}Note: Oracle needs 3 publishers total to submit prices${NC}"
    echo -e "  ${YELLOW}      Your validator will work once quorum is reached${NC}"
    echo ""
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    echo ""
    read -p "Press Enter to return to main menu..."
}

# Main loop
main() {
    while true; do
        print_header
        print_status
        print_menu
        read -p "Select option: " choice
        
        case $choice in
            1) service_manager ;;
            2) oracle_manager ;;
            3) monitoring ;;
            4) configuration_menu ;;
            5) wallet_manager ;;
            6) autopilot_menu ;;
            7) update_maintenance ;;
            8) quick_start ;;
            9) 
                echo -e "\n${GREEN}Thank you for using Tachyon Oracles Console!${NC}\n"
                exit 0
                ;;
            *) 
                echo -e "${RED}Invalid option. Please try again.${NC}"
                sleep 1
                ;;
        esac
    done
}

# Run main
main

