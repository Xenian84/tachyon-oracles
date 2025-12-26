#!/bin/bash
CONSOLE_DIR="/root/tachyon-oracles"
LOG_FILE="$CONSOLE_DIR/logs/autopilot.log"

mkdir -p "/root/tachyon-oracles/logs"

log() {
    echo "[2025-12-26 14:14:35] " >> ""
}

log "Autopilot started"

while true; do
    # Check if relayer is running
    if ! pgrep -f "relayer/dist/index.js" > /dev/null; then
        log "Relayer down, restarting..."
        cd "/root/tachyon-oracles/relayer"
        nohup node dist/index.js >> "/root/tachyon-oracles/logs/relayer.log" 2>&1 &
    fi
    
    # Check if signer is running
    if ! pgrep -f "signer/dist/index.js" > /dev/null; then
        log "Signer down, restarting..."
        cd "/root/tachyon-oracles/signer"
        nohup node dist/index.js >> "/root/tachyon-oracles/logs/signer.log" 2>&1 &
    fi
    
    # Check relayer health
    if ! curl -s http://localhost:7777/health > /dev/null 2>&1; then
        log "Relayer health check failed, restarting..."
        pkill -f "relayer/dist/index.js"
        sleep 2
        cd "/root/tachyon-oracles/relayer"
        nohup node dist/index.js >> "/root/tachyon-oracles/logs/relayer.log" 2>&1 &
    fi
    
    sleep 30
done
