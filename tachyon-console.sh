#!/bin/bash

# Tachyon Oracle Node Console
# Management console for node operators

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
NC='\033[0m'
BOLD='\033[1m'

# Configuration
NODE_CONFIG="/etc/tachyon/node-config.toml"
RPC_URL="https://rpc.mainnet.x1.xyz"

# Functions
clear_screen() {
    clear
}

print_header() {
    clear_screen
    echo -e "${CYAN}╔══════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║${NC}${BOLD}${WHITE}           TACHYON ORACLE NODE CONSOLE                    ${NC}${CYAN}║${NC}"
    echo -e "${CYAN}║${NC}${WHITE}              Node Management & Monitoring                    ${NC}${CYAN}║${NC}"
    echo -e "${CYAN}╚══════════════════════════════════════════════════════════════╝${NC}"
}

print_status() {
    # Check node status
    NODE_STATUS="${RED}●${NC} Stopped"
    if systemctl is-active --quiet tachyon-node 2>/dev/null; then
        NODE_STATUS="${GREEN}●${NC} Running"
    fi
    
    echo ""
    echo -e "  ${BOLD}Node Status:${NC} $NODE_STATUS"
    echo ""
}

print_menu() {
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    echo -e "  ${BOLD}${WHITE}MAIN MENU${NC}"
    echo ""
    echo -e "  ${GREEN}1.${NC}  Node Control          - Start/Stop/Restart/Status"
    echo -e "  ${GREEN}2.${NC}  View Logs             - Real-time & recent logs"
    echo -e "  ${GREEN}3.${NC}  Stake Information     - View your stake details"
    echo -e "  ${GREEN}4.${NC}  Performance Metrics   - Uptime & accuracy stats"
    echo -e "  ${GREEN}5.${NC}  Claim Rewards         - Claim earned TACH rewards"
    echo -e "  ${GREEN}6.${NC}  Staking Operations    - Stake/Unstake TACH"
    echo -e "  ${GREEN}7.${NC}  Wallet Info           - View balances & addresses"
    echo -e "  ${GREEN}8.${NC}  Network Status        - L2 state & validators"
    echo -e "  ${GREEN}9.${NC}  Configuration         - View/Edit node config"
    echo -e "  ${GREEN}10.${NC} System Health         - Diagnostics & monitoring"
    echo -e "  ${GREEN}11.${NC} Exit"
    echo ""
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    echo ""
}

# Node Control
node_control() {
    while true; do
        print_header
        print_status
        echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
        echo -e "  ${BOLD}${WHITE}NODE CONTROL${NC}"
        echo ""
        echo -e "  ${GREEN}1.${NC} Start Node"
        echo -e "  ${GREEN}2.${NC} Stop Node"
        echo -e "  ${GREEN}3.${NC} Restart Node"
        echo -e "  ${GREEN}4.${NC} Node Status (Detailed)"
        echo -e "  ${GREEN}5.${NC} Back to Main Menu"
        echo ""
        echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
        echo -n -e "  ${BOLD}Select option:${NC} "
        read choice
        
        case $choice in
            1)
                echo ""
                echo -e "${YELLOW}Starting node...${NC}"
                sudo systemctl start tachyon-node
                if [ $? -eq 0 ]; then
                    echo -e "${GREEN}✓ Node started successfully${NC}"
                else
                    echo -e "${RED}✗ Failed to start node${NC}"
                fi
                sleep 2
                ;;
            2)
                echo ""
                echo -e "${YELLOW}Stopping node...${NC}"
                sudo systemctl stop tachyon-node
                if [ $? -eq 0 ]; then
                    echo -e "${GREEN}✓ Node stopped successfully${NC}"
                else
                    echo -e "${RED}✗ Failed to stop node${NC}"
                fi
                sleep 2
                ;;
            3)
                echo ""
                echo -e "${YELLOW}Restarting node...${NC}"
                sudo systemctl restart tachyon-node
                if [ $? -eq 0 ]; then
                    echo -e "${GREEN}✓ Node restarted successfully${NC}"
                else
                    echo -e "${RED}✗ Failed to restart node${NC}"
                fi
                sleep 2
                ;;
            4)
                echo ""
                echo -e "${BOLD}Detailed Node Status:${NC}"
                echo ""
                sudo systemctl status tachyon-node --no-pager
                echo ""
                echo -n "Press Enter to continue..."
                read
                ;;
            5)
                return
                ;;
            *)
                echo -e "${RED}Invalid option${NC}"
                sleep 1
                ;;
        esac
    done
}

# View Logs
view_logs() {
    while true; do
        print_header
        echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
        echo -e "  ${BOLD}${WHITE}VIEW LOGS${NC}"
        echo ""
        echo -e "  ${GREEN}1.${NC} Live Logs (Follow)"
        echo -e "  ${GREEN}2.${NC} Recent Logs (Last 50 lines)"
        echo -e "  ${GREEN}3.${NC} Recent Logs (Last 100 lines)"
        echo -e "  ${GREEN}4.${NC} Error Logs Only"
        echo -e "  ${GREEN}5.${NC} Search Logs"
        echo -e "  ${GREEN}6.${NC} Back to Main Menu"
        echo ""
        echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
        echo -n -e "  ${BOLD}Select option:${NC} "
        read choice
        
        case $choice in
            1)
                echo ""
                echo -e "${YELLOW}Showing live logs (Ctrl+C to exit)...${NC}"
                echo ""
                sudo journalctl -u tachyon-node -f
                ;;
            2)
                echo ""
                sudo journalctl -u tachyon-node -n 50 --no-pager
                echo ""
                echo -n "Press Enter to continue..."
                read
                ;;
            3)
                echo ""
                sudo journalctl -u tachyon-node -n 100 --no-pager
                echo ""
                echo -n "Press Enter to continue..."
                read
                ;;
            4)
                echo ""
                echo -e "${BOLD}Error Logs:${NC}"
                echo ""
                sudo journalctl -u tachyon-node -p err --no-pager
                echo ""
                echo -n "Press Enter to continue..."
                read
                ;;
            5)
                echo ""
                echo -n "Enter search term: "
                read search_term
                echo ""
                sudo journalctl -u tachyon-node --no-pager | grep -i "$search_term"
                echo ""
                echo -n "Press Enter to continue..."
                read
                ;;
            6)
                return
                ;;
            *)
                echo -e "${RED}Invalid option${NC}"
                sleep 1
                ;;
        esac
    done
}

# Stake Information
stake_info() {
    print_header
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    echo -e "  ${BOLD}${WHITE}STAKE INFORMATION${NC}"
    echo ""
    echo -e "${YELLOW}Fetching stake information...${NC}"
    echo ""
    
    tachyon-node view-stake-info --config "$NODE_CONFIG"
    
    echo ""
    echo -n "Press Enter to continue..."
    read
}

# Performance Metrics
performance_metrics() {
    print_header
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    echo -e "  ${BOLD}${WHITE}PERFORMANCE METRICS${NC}"
    echo ""
    echo -e "${YELLOW}Fetching performance data...${NC}"
    echo ""
    
    tachyon-node view-performance --config "$NODE_CONFIG"
    
    echo ""
    echo -n "Press Enter to continue..."
    read
}

# Claim Rewards
claim_rewards() {
    print_header
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    echo -e "  ${BOLD}${WHITE}CLAIM REWARDS${NC}"
    echo ""
    echo -e "${YELLOW}⚠️  Warning: This will submit a transaction to claim your rewards${NC}"
    echo ""
    echo -n "Continue? (y/n): "
    read confirm
    
    if [ "$confirm" = "y" ] || [ "$confirm" = "Y" ]; then
        echo ""
        echo -e "${YELLOW}Claiming rewards...${NC}"
        echo ""
        
        tachyon-node claim-rewards --config "$NODE_CONFIG"
        
        echo ""
        echo -n "Press Enter to continue..."
        read
    fi
}

# Staking Operations
staking_operations() {
    while true; do
        print_header
        echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
        echo -e "  ${BOLD}${WHITE}STAKING OPERATIONS${NC}"
        echo ""
        echo -e "  ${GREEN}1.${NC} Stake TACH"
        echo -e "  ${GREEN}2.${NC} Unstake TACH"
        echo -e "  ${GREEN}3.${NC} View Current Stake"
        echo -e "  ${GREEN}4.${NC} Back to Main Menu"
        echo ""
        echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
        echo -n -e "  ${BOLD}Select option:${NC} "
        read choice
        
        case $choice in
            1)
                echo ""
                echo -n "Enter amount to stake (TACH): "
                read amount
                echo ""
                echo -e "${YELLOW}Staking $amount TACH...${NC}"
                echo ""
                
                tachyon-node stake --config "$NODE_CONFIG" --amount "$amount"
                
                echo ""
                echo -n "Press Enter to continue..."
                read
                ;;
            2)
                echo ""
                echo -n "Enter amount to unstake (TACH): "
                read amount
                echo ""
                echo -e "${YELLOW}Unstaking $amount TACH...${NC}"
                echo ""
                
                tachyon-node unstake --config "$NODE_CONFIG" --amount "$amount"
                
                echo ""
                echo -n "Press Enter to continue..."
                read
                ;;
            3)
                echo ""
                tachyon-node view-stake-info --config "$NODE_CONFIG"
                echo ""
                echo -n "Press Enter to continue..."
                read
                ;;
            4)
                return
                ;;
            *)
                echo -e "${RED}Invalid option${NC}"
                sleep 1
                ;;
        esac
    done
}

# Wallet Info
wallet_info() {
    print_header
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    echo -e "  ${BOLD}${WHITE}WALLET INFORMATION${NC}"
    echo ""
    
    # Extract keypair path from config
    KEYPAIR_PATH=$(grep "keypair_path" "$NODE_CONFIG" | cut -d'"' -f2)
    
    if [ -z "$KEYPAIR_PATH" ]; then
        echo -e "${RED}✗ Could not find keypair path in config${NC}"
        echo ""
        echo -n "Press Enter to continue..."
        read
        return
    fi
    
    # Get public key
    PUBKEY=$(solana-keygen pubkey "$KEYPAIR_PATH" 2>/dev/null)
    
    echo -e "${BOLD}Node Address:${NC}"
    echo -e "  $PUBKEY"
    echo ""
    
    echo -e "${BOLD}XNT Balance:${NC}"
    solana balance "$KEYPAIR_PATH" --url "$RPC_URL" 2>/dev/null || echo -e "${RED}  Failed to fetch balance${NC}"
    echo ""
    
    echo -e "${BOLD}Explorer Link:${NC}"
    echo -e "  https://explorer.x1.xyz/address/$PUBKEY"
    echo ""
    
    echo -n "Press Enter to continue..."
    read
}

# Network Status
network_status() {
    print_header
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    echo -e "  ${BOLD}${WHITE}NETWORK STATUS${NC}"
    echo ""
    echo -e "${YELLOW}Fetching network information...${NC}"
    echo ""
    
    # Get L2 state
    echo -e "${BOLD}L2 State Compression:${NC}"
    tachyon-node query-l2-state --config "$NODE_CONFIG" 2>/dev/null || echo -e "${RED}  Failed to query L2 state${NC}"
    echo ""
    
    # Get validator count
    echo -e "${BOLD}Active Validators:${NC}"
    tachyon-node query-validators --config "$NODE_CONFIG" 2>/dev/null || echo -e "${RED}  Failed to query validators${NC}"
    echo ""
    
    echo -n "Press Enter to continue..."
    read
}

# Configuration
view_config() {
    while true; do
        print_header
        echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
        echo -e "  ${BOLD}${WHITE}CONFIGURATION${NC}"
        echo ""
        echo -e "  ${GREEN}1.${NC} View Current Config"
        echo -e "  ${GREEN}2.${NC} Edit Config (nano)"
        echo -e "  ${GREEN}3.${NC} Edit Config (vim)"
        echo -e "  ${GREEN}4.${NC} Back to Main Menu"
        echo ""
        echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
        echo -n -e "  ${BOLD}Select option:${NC} "
        read choice
        
        case $choice in
            1)
                echo ""
                echo -e "${BOLD}Current Configuration:${NC}"
                echo ""
                cat "$NODE_CONFIG"
                echo ""
                echo -n "Press Enter to continue..."
                read
                ;;
            2)
                sudo nano "$NODE_CONFIG"
                echo ""
                echo -e "${YELLOW}⚠️  Restart the node for changes to take effect${NC}"
                sleep 2
                ;;
            3)
                sudo vim "$NODE_CONFIG"
                echo ""
                echo -e "${YELLOW}⚠️  Restart the node for changes to take effect${NC}"
                sleep 2
                ;;
            4)
                return
                ;;
            *)
                echo -e "${RED}Invalid option${NC}"
                sleep 1
                ;;
        esac
    done
}

# System Health
system_health() {
    print_header
    echo -e "${CYAN}════════════════════════════════════════════════════════════════${NC}"
    echo -e "  ${BOLD}${WHITE}SYSTEM HEALTH${NC}"
    echo ""
    
    echo -e "${BOLD}CPU Usage:${NC}"
    top -bn1 | grep "Cpu(s)" | sed "s/.*, *\([0-9.]*\)%* id.*/\1/" | awk '{print "  " 100 - $1"%"}'
    echo ""
    
    echo -e "${BOLD}Memory Usage:${NC}"
    free -h | awk 'NR==2{printf "  %s / %s (%.2f%%)\n", $3,$2,$3*100/$2 }'
    echo ""
    
    echo -e "${BOLD}Disk Usage:${NC}"
    df -h / | awk 'NR==2{printf "  %s / %s (%s)\n", $3,$2,$5}'
    echo ""
    
    echo -e "${BOLD}Node Service Status:${NC}"
    if systemctl is-active --quiet tachyon-node; then
        echo -e "  ${GREEN}✓ Running${NC}"
        UPTIME=$(systemctl show tachyon-node --property=ActiveEnterTimestamp --value)
        echo -e "  Started: $UPTIME"
    else
        echo -e "  ${RED}✗ Not Running${NC}"
    fi
    echo ""
    
    echo -e "${BOLD}Recent Errors (Last 10):${NC}"
    ERROR_COUNT=$(sudo journalctl -u tachyon-node -p err --since "24 hours ago" --no-pager | wc -l)
    echo -e "  $ERROR_COUNT errors in last 24 hours"
    echo ""
    
    echo -n "Press Enter to continue..."
    read
}

# Main loop
main() {
    # Check if running as root or with sudo access
    if [ "$EUID" -ne 0 ]; then 
        if ! sudo -n true 2>/dev/null; then
            echo -e "${RED}This console requires sudo access${NC}"
            exit 1
        fi
    fi
    
    # Check if config exists
    if [ ! -f "$NODE_CONFIG" ]; then
        echo -e "${RED}Configuration file not found: $NODE_CONFIG${NC}"
        echo -e "${YELLOW}Please run the setup script first${NC}"
        exit 1
    fi
    
    while true; do
        print_header
        print_status
        print_menu
        echo -n -e "  ${BOLD}Select option:${NC} "
        read choice
        
        case $choice in
            1) node_control ;;
            2) view_logs ;;
            3) stake_info ;;
            4) performance_metrics ;;
            5) claim_rewards ;;
            6) staking_operations ;;
            7) wallet_info ;;
            8) network_status ;;
            9) view_config ;;
            10) system_health ;;
            11)
                echo ""
                echo -e "${GREEN}Goodbye!${NC}"
                exit 0
                ;;
            *)
                echo -e "${RED}Invalid option${NC}"
                sleep 1
                ;;
        esac
    done
}

# Run main
main

