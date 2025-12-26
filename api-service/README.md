# 🚀 Tachyon Oracle API Service

Lightweight REST API for remote oracle monitoring and management.

## Features

- ✅ **Secure** - API key authentication, rate limiting, IP whitelist
- ✅ **Flexible** - Three security modes (readonly, monitoring, full)
- ✅ **Lightweight** - Minimal resource usage
- ✅ **Easy Setup** - One command installation
- ✅ **Production Ready** - PM2 process management, logging

---

## Quick Start

### 1. Setup

```bash
cd /root/tachyon-oracles/api-service
./setup.sh
```

The setup wizard will:
- Generate a secure API key
- Configure security mode
- Optionally set IP whitelist
- Start the service

### 2. Test

```bash
# Replace YOUR_API_KEY with the key from setup
curl -H "Authorization: Bearer YOUR_API_KEY" http://localhost:7171/api/status
```

---

## API Endpoints

### Health Check (No Auth Required)
```
GET /health
```

### Get Oracle Status
```
GET /api/status

Response:
{
  "services": {
    "signer": "running",
    "relayer": "stopped"
  },
  "network": {
    "feeds": 9,
    "publishers": 1
  },
  "rpc": "https://rpc.mainnet.x1.xyz",
  "timestamp": 1703001234567
}
```

### Get Price Feeds
```
GET /api/feeds

Response:
{
  "feeds": [
    {
      "assetId": "6c8fdf98...",
      "price": "168880000000",
      "confidence": "120000",
      "timestamp": "1703001234"
    }
  ],
  "count": 9
}
```

### Get Publishers
```
GET /api/publishers

Response:
{
  "publishers": [
    {
      "pubkey": "Bqc3QJsDpXx...",
      "pda": "7xKXtg2CW9...",
      "active": true
    }
  ],
  "count": 1
}
```

### Get Logs (Monitoring/Full Mode)
```
GET /api/logs/signer?lines=50
GET /api/logs/relayer?lines=50

Response:
{
  "logs": "2025-12-26T10:00:00.000Z [info]: ...",
  "lines": 50
}
```

### Service Control (Full Mode Only)
```
POST /api/services/start/signer
POST /api/services/stop/signer
POST /api/services/restart/signer
POST /api/services/start/relayer
POST /api/services/stop/relayer
POST /api/services/restart/relayer

Response:
{
  "success": true,
  "action": "start",
  "service": "signer",
  "status": "running"
}
```

### Get API Info
```
GET /api/info

Response:
{
  "version": "1.0.0",
  "mode": "monitoring",
  "features": {
    "status": true,
    "feeds": true,
    "publishers": true,
    "logs": true,
    "serviceControl": false
  }
}
```

---

## Security Modes

### Readonly (Safest)
- ✅ View status
- ✅ View feeds
- ✅ View publishers
- ❌ No logs
- ❌ No service control

### Monitoring (Recommended)
- ✅ View status
- ✅ View feeds
- ✅ View publishers
- ✅ View logs (limited)
- ❌ No service control

### Full (Use with Caution)
- ✅ Everything from Monitoring
- ✅ Start/stop/restart services
- ⚠️ Requires extra security

---

## Configuration

Edit `.env` file:

```env
# Server
API_PORT=7171
API_HOST=0.0.0.0

# Security
API_KEY=your-generated-api-key
API_MODE=monitoring

# IP Whitelist (comma-separated)
ALLOWED_IPS=1.2.3.4,5.6.7.8

# Rate Limiting
RATE_LIMIT_WINDOW_MS=60000
RATE_LIMIT_MAX_REQUESTS=30

# Oracle
ORACLE_PROJECT_PATH=/root/tachyon-oracles
RPC_URL=https://rpc.mainnet.x1.xyz
PROGRAM_ID=TACH9r2uZzoFM6daofesADjeDn9NqB1pKFWP5mfByb1

# Logs
MAX_LOG_LINES=50
```

---

## PM2 Management

```bash
# Status
pm2 status tachyon-api

# Logs
pm2 logs tachyon-api

# Restart
pm2 restart tachyon-api

# Stop
pm2 stop tachyon-api

# Remove
pm2 delete tachyon-api
```

---

## Firewall Setup

### UFW (Ubuntu)
```bash
sudo ufw allow 7171/tcp
sudo ufw reload
```

### Firewalld (CentOS/RHEL)
```bash
sudo firewall-cmd --permanent --add-port=7171/tcp
sudo firewall-cmd --reload
```

### iptables
```bash
sudo iptables -A INPUT -p tcp --dport 7171 -j ACCEPT
sudo service iptables save
```

---

## Security Best Practices

1. **Use Strong API Keys**
   - Generate with: `openssl rand -hex 32`
   - Never share or commit to git

2. **Enable IP Whitelist**
   - Only allow bot server IPs
   - Update when bot server changes

3. **Use HTTPS in Production**
   - Get SSL cert from Let's Encrypt
   - Set `ENABLE_HTTPS=true` in .env

4. **Monitor Access**
   - Check logs regularly: `pm2 logs tachyon-api`
   - Look for unauthorized access attempts

5. **Start with Readonly Mode**
   - Test with readonly first
   - Upgrade to monitoring when comfortable
   - Only use full mode if absolutely needed

6. **Keep API Key Secret**
   - Don't log it
   - Don't send over insecure channels
   - Rotate periodically

---

## Connecting from Telegram Bot

### Setup Profile with API
1. Open @tachyon_oracle_bot
2. Tap "🗂️ Profiles" → "➕ Add Profile"
3. Name: "My Validator"
4. Type: "api"
5. Enter API URL: `https://your-server.com:7171`
6. Enter API Key: (from setup)
7. Done!

---

## Troubleshooting

### API not responding
```bash
# Check if service is running
pm2 status tachyon-api

# Check logs
pm2 logs tachyon-api --lines 50

# Restart service
pm2 restart tachyon-api
```

### Unauthorized errors
- Check API key is correct
- Verify Authorization header format: `Bearer YOUR_KEY`

### IP blocked
- Check ALLOWED_IPS in .env
- Add your IP or leave empty to allow all

### Port already in use
```bash
# Find what's using port 7171
sudo lsof -i :7171

# Kill the process
sudo kill -9 PID
```

---

## Development

### Run in dev mode
```bash
npm run dev
```

### Build
```bash
npm run build
```

### Test endpoints
```bash
# Set your API key
export API_KEY="your-api-key-here"

# Test status
curl -H "Authorization: Bearer $API_KEY" http://localhost:7171/api/status

# Test feeds
curl -H "Authorization: Bearer $API_KEY" http://localhost:7171/api/feeds

# Test publishers
curl -H "Authorization: Bearer $API_KEY" http://localhost:7171/api/publishers

# Test logs
curl -H "Authorization: Bearer $API_KEY" http://localhost:7171/api/logs/signer

# Test service control (full mode only)
curl -X POST -H "Authorization: Bearer $API_KEY" http://localhost:7171/api/services/restart/signer
```

---

## Support

For issues or questions:
- Check logs: `pm2 logs tachyon-api`
- Review configuration: `cat .env`
- Test connectivity: `curl http://localhost:7171/health`

---

**Version:** 1.0.0  
**License:** MIT


