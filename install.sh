#!/bin/bash

###############################################################################
#                                                                             #
#                   TACHYON ORACLE NODE - ONE-CLICK INSTALLER                #
#                                                                             #
#   This script will automatically set up a Tachyon Oracle node on your      #
#   server. It handles everything: dependencies, compilation, configuration,  #
#   and service setup.                                                        #
#                                                                             #
#   Usage: curl -sSL https://install.tachyon.xyz | bash                      #
#                                                                             #
###############################################################################

set -e  # Exit on any error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Emojis
ROCKET="🚀"
CHECK="✅"
CROSS="❌"
WARN="⚠️"
INFO="ℹ️"
GEAR="⚙️"

# Print colored message
print_msg() {
    local color=$1
    local emoji=$2
    local msg=$3
    echo -e "${color}${emoji} ${msg}${NC}"
}

# Print header
print_header() {
    echo ""
    echo -e "${CYAN}╔════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║                                                                ║${NC}"
    echo -e "${CYAN}║           ${ROCKET}  TACHYON ORACLE NODE INSTALLER  ${ROCKET}            ║${NC}"
    echo -e "${CYAN}║                                                                ║${NC}"
    echo -e "${CYAN}║              Decentralized Price Feeds for X1                  ║${NC}"
    echo -e "${CYAN}║                                                                ║${NC}"
    echo -e "${CYAN}╚════════════════════════════════════════════════════════════════╝${NC}"
    echo ""
}

# Check if running as root
check_root() {
    if [[ $EUID -eq 0 ]]; then
        print_msg "$RED" "$CROSS" "This script should NOT be run as root!"
        print_msg "$YELLOW" "$WARN" "Please run as a regular user with sudo privileges."
        exit 1
    fi
}

# Check system requirements
check_requirements() {
    print_msg "$BLUE" "$GEAR" "Checking system requirements..."
    
    # Check OS
    if [[ ! -f /etc/os-release ]]; then
        print_msg "$RED" "$CROSS" "Cannot detect OS. This script supports Ubuntu/Debian."
        exit 1
    fi
    
    source /etc/os-release
    if [[ "$ID" != "ubuntu" && "$ID" != "debian" ]]; then
        print_msg "$YELLOW" "$WARN" "This script is optimized for Ubuntu/Debian."
        read -p "Continue anyway? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
    
    # Check CPU cores
    CPU_CORES=$(nproc)
    if [[ $CPU_CORES -lt 2 ]]; then
        print_msg "$YELLOW" "$WARN" "Recommended: 2+ CPU cores (found: $CPU_CORES)"
    fi
    
    # Check RAM
    TOTAL_RAM=$(free -g | awk '/^Mem:/{print $2}')
    if [[ $TOTAL_RAM -lt 4 ]]; then
        print_msg "$YELLOW" "$WARN" "Recommended: 4+ GB RAM (found: ${TOTAL_RAM}GB)"
    fi
    
    # Check disk space
    DISK_SPACE=$(df -BG / | awk 'NR==2 {print $4}' | sed 's/G//')
    if [[ $DISK_SPACE -lt 20 ]]; then
        print_msg "$YELLOW" "$WARN" "Recommended: 20+ GB free space (found: ${DISK_SPACE}GB)"
    fi
    
    print_msg "$GREEN" "$CHECK" "System requirements checked"
}

# Install dependencies
install_dependencies() {
    print_msg "$BLUE" "$GEAR" "Installing dependencies..."
    
    sudo apt-get update -qq
    sudo apt-get install -y -qq \
        curl \
        git \
        build-essential \
        pkg-config \
        libssl-dev \
        jq \
        wget \
        ca-certificates \
        gnupg \
        lsb-release > /dev/null 2>&1
    
    print_msg "$GREEN" "$CHECK" "Dependencies installed"
}

# Install Rust
install_rust() {
    if command -v rustc &> /dev/null; then
        print_msg "$GREEN" "$CHECK" "Rust already installed: $(rustc --version)"
        return
    fi
    
    print_msg "$BLUE" "$GEAR" "Installing Rust..."
    
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    
    print_msg "$GREEN" "$CHECK" "Rust installed: $(rustc --version)"
}

# Install Solana CLI
install_solana() {
    if command -v solana &> /dev/null; then
        print_msg "$GREEN" "$CHECK" "Solana CLI already installed: $(solana --version)"
        return
    fi
    
    print_msg "$BLUE" "$GEAR" "Installing Solana CLI..."
    
    sh -c "$(curl -sSfL https://release.solana.com/stable/install)" > /dev/null 2>&1
    export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
    
    print_msg "$GREEN" "$CHECK" "Solana CLI installed: $(solana --version)"
}

# Clone repository
clone_repo() {
    print_msg "$BLUE" "$GEAR" "Cloning Tachyon repository..."
    
    INSTALL_DIR="$HOME/tachyon-node"
    
    if [[ -d "$INSTALL_DIR" ]]; then
        print_msg "$YELLOW" "$WARN" "Directory $INSTALL_DIR already exists"
        read -p "Remove and re-clone? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            rm -rf "$INSTALL_DIR"
        else
            print_msg "$YELLOW" "$INFO" "Using existing directory"
            cd "$INSTALL_DIR"
            git pull origin main > /dev/null 2>&1 || true
            return
        fi
    fi
    
    git clone https://github.com/xenian84/tachyon-oracles.git "$INSTALL_DIR" > /dev/null 2>&1
    cd "$INSTALL_DIR"
    
    print_msg "$GREEN" "$CHECK" "Repository cloned to $INSTALL_DIR"
}

# Build node
build_node() {
    print_msg "$BLUE" "$GEAR" "Building Tachyon node (this may take 5-10 minutes)..."
    
    cd "$HOME/tachyon-node/tachyon-node"
    
    # Show progress
    cargo build --release 2>&1 | grep -E "Compiling|Finished" || true
    
    if [[ ! -f "target/release/tachyon-node" ]]; then
        print_msg "$RED" "$CROSS" "Build failed!"
        exit 1
    fi
    
    print_msg "$GREEN" "$CHECK" "Node built successfully"
}

# Setup keypair
setup_keypair() {
    print_msg "$BLUE" "$GEAR" "Setting up node keypair..."
    
    sudo mkdir -p /var/lib/tachyon
    sudo mkdir -p /etc/tachyon
    
    if [[ -f /var/lib/tachyon/node-keypair.json ]]; then
        print_msg "$YELLOW" "$WARN" "Keypair already exists at /var/lib/tachyon/node-keypair.json"
        read -p "Generate new keypair? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            print_msg "$YELLOW" "$INFO" "Using existing keypair"
            NODE_PUBKEY=$(solana-keygen pubkey /var/lib/tachyon/node-keypair.json)
            print_msg "$GREEN" "$CHECK" "Node pubkey: $NODE_PUBKEY"
            return
        fi
    fi
    
    solana-keygen new --no-bip39-passphrase --outfile /tmp/node-keypair.json --force > /dev/null 2>&1
    sudo mv /tmp/node-keypair.json /var/lib/tachyon/node-keypair.json
    sudo chmod 600 /var/lib/tachyon/node-keypair.json
    sudo chown $USER:$USER /var/lib/tachyon/node-keypair.json
    
    NODE_PUBKEY=$(solana-keygen pubkey /var/lib/tachyon/node-keypair.json)
    
    print_msg "$GREEN" "$CHECK" "Keypair generated"
    print_msg "$CYAN" "$INFO" "Node pubkey: $NODE_PUBKEY"
    echo ""
    print_msg "$YELLOW" "$WARN" "IMPORTANT: Save this pubkey! You'll need it for staking."
}

# Create configuration
create_config() {
    print_msg "$BLUE" "$GEAR" "Creating configuration..."
    
    cat > /tmp/node-config.toml << 'EOF'
# Tachyon Oracle Node Configuration

# Path to your node's keypair
keypair_path = "/var/lib/tachyon/node-keypair.json"

# X1 RPC endpoint (mainnet)
rpc_url = "https://rpc.mainnet.x1.xyz"

# Tachyon Governance Program ID
program_id = "TACHdFYQ4uDuAdo6Hz4V1RaCezEpHkVRZGQ7yh24Ad9"

# L2 State Compression Program ID
l2_program_id = "L2STdFYQ4uDuAdo6Hz4V1RaCezEpHkVRZGQ7yh24Ad9"

# Price Feeds Program ID
price_feeds_program_id = "PFEDu3nNzRQQYmX1Xvso2BxtPbUQaZEVoiLbXDy6U3W"

# TACH token mint address
tach_mint = "TACHmintXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX"

# Data sources (exchanges)
[[data_sources]]
name = "coinbase"
url = "https://api.coinbase.com/v2/prices"
weight = 1.0

[[data_sources]]
name = "kraken"
url = "https://api.kraken.com/0/public/Ticker"
weight = 1.0

# Asset pairs to track
[[assets]]
id = "BTC/USD"
sources = ["coinbase", "kraken"]
update_interval = 10

[[assets]]
id = "ETH/USD"
sources = ["coinbase", "kraken"]
update_interval = 10

[[assets]]
id = "SOL/USD"
sources = ["coinbase", "kraken"]
update_interval = 10

# Aggregation settings
[aggregation]
batch_interval = 60  # seconds
min_sources = 1

# API settings
[api]
enabled = true
port = 8080
host = "127.0.0.1"
EOF
    
    sudo mv /tmp/node-config.toml /etc/tachyon/node-config.toml
    sudo chmod 644 /etc/tachyon/node-config.toml
    
    print_msg "$GREEN" "$CHECK" "Configuration created at /etc/tachyon/node-config.toml"
}

# Install binary
install_binary() {
    print_msg "$BLUE" "$GEAR" "Installing node binary..."
    
    sudo cp "$HOME/tachyon-node/tachyon-node/target/release/tachyon-node" /usr/local/bin/
    sudo chmod +x /usr/local/bin/tachyon-node
    
    print_msg "$GREEN" "$CHECK" "Binary installed to /usr/local/bin/tachyon-node"
}

# Create systemd service
create_service() {
    print_msg "$BLUE" "$GEAR" "Creating systemd service..."
    
    cat > /tmp/tachyon-node.service << EOF
[Unit]
Description=Tachyon Oracle Node
After=network.target
Wants=network-online.target

[Service]
Type=simple
User=$USER
WorkingDirectory=$HOME
ExecStart=/usr/local/bin/tachyon-node start --config /etc/tachyon/node-config.toml
Restart=always
RestartSec=10
StandardOutput=journal
StandardError=journal
SyslogIdentifier=tachyon-node

# Security settings
NoNewPrivileges=true
PrivateTmp=true
ProtectSystem=strict
ProtectHome=read-only
ReadWritePaths=/var/lib/tachyon

[Install]
WantedBy=multi-user.target
EOF
    
    sudo mv /tmp/tachyon-node.service /etc/systemd/system/
    sudo systemctl daemon-reload
    sudo systemctl enable tachyon-node > /dev/null 2>&1
    
    print_msg "$GREEN" "$CHECK" "Systemd service created"
}

# Install console script
install_console() {
    print_msg "$BLUE" "$GEAR" "Installing console script..."
    
    sudo cp "$HOME/tachyon-node/tachyon-console.sh" /usr/local/bin/tachyon-console
    sudo chmod +x /usr/local/bin/tachyon-console
    
    print_msg "$GREEN" "$CHECK" "Console installed: Run 'tachyon-console' to manage your node"
}

# Fund node
fund_node() {
    print_msg "$BLUE" "$INFO" "Node needs XNT for transaction fees..."
    print_msg "$CYAN" "$INFO" "Node pubkey: $NODE_PUBKEY"
    echo ""
    print_msg "$YELLOW" "$WARN" "Please send at least 0.1 XNT to this address for transaction fees"
    echo ""
    read -p "Press Enter when you've sent XNT to continue..."
    
    # Check balance
    BALANCE=$(solana balance /var/lib/tachyon/node-keypair.json --url https://rpc.mainnet.x1.xyz 2>/dev/null | awk '{print $1}')
    
    if (( $(echo "$BALANCE > 0.01" | bc -l) )); then
        print_msg "$GREEN" "$CHECK" "Balance confirmed: $BALANCE XNT"
    else
        print_msg "$YELLOW" "$WARN" "Low balance: $BALANCE XNT (recommended: 0.1+ XNT)"
    fi
}

# Stake tokens
stake_tokens() {
    print_msg "$BLUE" "$INFO" "Node requires staking to participate in consensus..."
    print_msg "$CYAN" "$INFO" "Minimum stake: 100,000 TACH"
    print_msg "$CYAN" "$INFO" "Node pubkey: $NODE_PUBKEY"
    echo ""
    print_msg "$YELLOW" "$WARN" "You need to stake TACH tokens to activate your node"
    echo ""
    
    read -p "Do you want to stake now? (y/N): " -n 1 -r
    echo
    
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        read -p "Enter amount to stake (minimum 100000): " STAKE_AMOUNT
        
        if [[ $STAKE_AMOUNT -lt 100000 ]]; then
            print_msg "$RED" "$CROSS" "Amount too low! Minimum is 100,000 TACH"
            print_msg "$YELLOW" "$INFO" "You can stake later using: tachyon-console"
            return
        fi
        
        read -p "Enter path to wallet with TACH tokens: " WALLET_PATH
        
        if [[ ! -f "$WALLET_PATH" ]]; then
            print_msg "$RED" "$CROSS" "Wallet file not found: $WALLET_PATH"
            print_msg "$YELLOW" "$INFO" "You can stake later using: tachyon-console"
            return
        fi
        
        print_msg "$BLUE" "$GEAR" "Staking $STAKE_AMOUNT TACH..."
        
        # Run stake command
        cd "$HOME/tachyon-node"
        NODE_PUBKEY=$NODE_PUBKEY STAKE_AMOUNT=$STAKE_AMOUNT WALLET_PATH=$WALLET_PATH node stake-simple.js
        
        if [[ $? -eq 0 ]]; then
            print_msg "$GREEN" "$CHECK" "Staked successfully!"
        else
            print_msg "$RED" "$CROSS" "Staking failed. You can try again using: tachyon-console"
        fi
    else
        print_msg "$YELLOW" "$INFO" "You can stake later using: tachyon-console"
    fi
}

# Start node
start_node() {
    print_msg "$BLUE" "$GEAR" "Starting Tachyon node..."
    
    sudo systemctl start tachyon-node
    sleep 3
    
    if sudo systemctl is-active --quiet tachyon-node; then
        print_msg "$GREEN" "$CHECK" "Node started successfully!"
    else
        print_msg "$RED" "$CROSS" "Node failed to start. Check logs: journalctl -u tachyon-node -f"
        exit 1
    fi
}

# Print final instructions
print_final() {
    echo ""
    echo -e "${GREEN}╔════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║                                                                ║${NC}"
    echo -e "${GREEN}║              ${CHECK}  INSTALLATION COMPLETE!  ${CHECK}                    ║${NC}"
    echo -e "${GREEN}║                                                                ║${NC}"
    echo -e "${GREEN}╚════════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    print_msg "$CYAN" "$INFO" "Your Tachyon Oracle node is now running!"
    echo ""
    print_msg "$BLUE" "$ROCKET" "Node Pubkey: $NODE_PUBKEY"
    echo ""
    print_msg "$YELLOW" "$INFO" "Useful commands:"
    echo "  • Manage node:        tachyon-console"
    echo "  • Check status:       systemctl status tachyon-node"
    echo "  • View logs:          journalctl -u tachyon-node -f"
    echo "  • Stop node:          sudo systemctl stop tachyon-node"
    echo "  • Start node:         sudo systemctl start tachyon-node"
    echo "  • Restart node:       sudo systemctl restart tachyon-node"
    echo ""
    print_msg "$YELLOW" "$WARN" "Important:"
    echo "  1. Make sure you have staked at least 100,000 TACH"
    echo "  2. Keep at least 0.1 XNT in node wallet for fees"
    echo "  3. Monitor logs for any errors"
    echo ""
    print_msg "$CYAN" "$INFO" "Documentation: https://docs.tachyon.xyz"
    print_msg "$CYAN" "$INFO" "Discord: https://discord.gg/tachyon"
    echo ""
    print_msg "$GREEN" "$ROCKET" "Happy validating!"
    echo ""
}

# Main installation flow
main() {
    print_header
    check_root
    check_requirements
    install_dependencies
    install_rust
    install_solana
    clone_repo
    build_node
    setup_keypair
    create_config
    install_binary
    create_service
    install_console
    fund_node
    stake_tokens
    start_node
    print_final
}

# Run main function
main "$@"

