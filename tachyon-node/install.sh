#!/bin/bash

# Tachyon Node Installation Script
set -e

echo "🚀 Installing Tachyon Oracle Node..."

# Check if running as root
if [ "$EUID" -ne 0 ]; then 
    echo "❌ Please run as root (use sudo)"
    exit 1
fi

# Create tachyon user if it doesn't exist
if ! id -u tachyon > /dev/null 2>&1; then
    echo "👤 Creating tachyon user..."
    useradd -r -s /bin/false -d /opt/tachyon tachyon
fi

# Create directories
echo "📁 Creating directories..."
mkdir -p /opt/tachyon
mkdir -p /etc/tachyon
mkdir -p /var/lib/tachyon
mkdir -p /var/log/tachyon

# Copy binary
echo "📦 Installing binary..."
cp target/release/tachyon-node /usr/local/bin/
chmod +x /usr/local/bin/tachyon-node

# Copy systemd service
echo "⚙️  Installing systemd service..."
cp tachyon-node.service /etc/systemd/system/
systemctl daemon-reload

# Set permissions
chown -R tachyon:tachyon /opt/tachyon
chown -R tachyon:tachyon /var/lib/tachyon
chown -R tachyon:tachyon /var/log/tachyon

echo "✅ Installation complete!"
echo ""
echo "📝 Next steps:"
echo "  1. Initialize node: sudo -u tachyon tachyon-node init"
echo "  2. Edit config: sudo nano /etc/tachyon/node-config.toml"
echo "  3. Start service: sudo systemctl start tachyon-node"
echo "  4. Enable on boot: sudo systemctl enable tachyon-node"
echo "  5. Check status: sudo systemctl status tachyon-node"
echo "  6. View logs: sudo journalctl -u tachyon-node -f"

