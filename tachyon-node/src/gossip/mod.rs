#![allow(dead_code)]
use solana_sdk::signer::Signer;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{mpsc, RwLock};
use tokio::time::{interval, Duration};
use tracing::{debug, error, info, warn};

use crate::config::NodeConfig;
use crate::fetcher::PriceUpdate;

// Solana-style gossip modules
pub mod crds;
pub mod push_pull;


#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GossipMessage {
    /// Announce this node to the network
    Announce {
        node_id: String,
        addr: SocketAddr,
    },
    /// Price update to gossip
    PriceUpdate(PriceUpdate),
    /// Heartbeat to keep connection alive
    Heartbeat,
    /// Request peer list
    GetPeers,
    /// Response with peer list
    Peers(Vec<SocketAddr>),
}

pub struct GossipNetwork {
    config: Arc<NodeConfig>,
    peers: Arc<RwLock<HashMap<SocketAddr, TcpStream>>>,
    known_peers: Arc<RwLock<Vec<SocketAddr>>>,
}

impl GossipNetwork {
    pub fn new(config: Arc<NodeConfig>) -> Self {
        Self {
            config,
            peers: Arc::new(RwLock::new(HashMap::new())),
            known_peers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    pub async fn start(
        &self,
        gossip_tx: mpsc::Sender<PriceUpdate>,
        mut shutdown: tokio::sync::broadcast::Receiver<()>,
    ) -> Result<()> {
        let bind_addr = format!("0.0.0.0:{}", self.config.gossip_port);
        let listener = TcpListener::bind(&bind_addr).await?;
        
        info!("📡 Starting TCP Gossip network on {}", bind_addr);
        info!("📡 Node ID: {}", self.config.identity.pubkey());
        
        // Start accepting connections
        let peers = self.peers.clone();
        let known_peers = self.known_peers.clone();
        let gossip_tx_clone = gossip_tx.clone();
        
        tokio::spawn(async move {
            loop {
                match listener.accept().await {
                    Ok((stream, addr)) => {
                        info!("📡 New peer connected: {}", addr);
                        peers.write().await.insert(addr, stream);
                        known_peers.write().await.push(addr);
                        
                        // Handle peer messages
                        let peers_clone = peers.clone();
                        let gossip_tx_clone2 = gossip_tx_clone.clone();
                        tokio::spawn(async move {
                            if let Err(e) = Self::handle_peer(addr, peers_clone, gossip_tx_clone2).await {
                                warn!("📡 Error handling peer {}: {}", addr, e);
                            }
                        });
                    }
                    Err(e) => {
                        error!("📡 Error accepting connection: {}", e);
                    }
                }
            }
        });
        
        // Start heartbeat
        let peers_heartbeat = self.peers.clone();
        tokio::spawn(async move {
            let mut heartbeat_interval = interval(Duration::from_secs(30));
            loop {
                heartbeat_interval.tick().await;
                Self::send_heartbeat(&peers_heartbeat).await;
            }
        });
        
        info!("✅ TCP Gossip network started successfully");
        
        // Wait for shutdown
        shutdown.recv().await.ok();
        info!("📡 Gossip network shutting down...");
        
        Ok(())
    }

    async fn handle_peer(
        addr: SocketAddr,
        peers: Arc<RwLock<HashMap<SocketAddr, TcpStream>>>,
        gossip_tx: mpsc::Sender<PriceUpdate>,
    ) -> Result<()> {
        let mut stream = peers.write().await.remove(&addr).ok_or_else(|| {
            anyhow::anyhow!("Peer not found")
        })?;
        
        let mut buf = vec![0u8; 4096];
        
        loop {
            match stream.read(&mut buf).await {
                Ok(0) => {
                    info!("📡 Peer {} disconnected", addr);
                    break;
                }
                Ok(n) => {
                    // Try to deserialize message
                    if let Ok(msg) = serde_json::from_slice::<GossipMessage>(&buf[..n]) {
                        match msg {
                            GossipMessage::PriceUpdate(update) => {
                                debug!("📡 Received price update from {}: {}", addr, update.asset);
                                gossip_tx.send(update).await.ok();
                            }
                            GossipMessage::Heartbeat => {
                                debug!("📡 Heartbeat from {}", addr);
                            }
                            GossipMessage::Announce { node_id, addr: peer_addr } => {
                                info!("📡 Peer announced: {} at {}", node_id, peer_addr);
                            }
                            _ => {}
                        }
                    }
                }
                Err(e) => {
                    warn!("📡 Error reading from peer {}: {}", addr, e);
                    break;
                }
            }
        }
        
        Ok(())
    }

    async fn send_heartbeat(peers: &Arc<RwLock<HashMap<SocketAddr, TcpStream>>>) {
        let msg = GossipMessage::Heartbeat;
        if let Ok(data) = serde_json::to_vec(&msg) {
            let mut peers_write = peers.write().await;
            let mut to_remove = Vec::new();
            
            for (addr, stream) in peers_write.iter_mut() {
                if let Err(e) = stream.write_all(&data).await {
                    warn!("📡 Failed to send heartbeat to {}: {}", addr, e);
                    to_remove.push(*addr);
                }
            }
            
            for addr in to_remove {
                peers_write.remove(&addr);
                info!("📡 Removed dead peer: {}", addr);
            }
        }
    }

    pub async fn broadcast_price_update(&self, update: &PriceUpdate) -> Result<()> {
        let msg = GossipMessage::PriceUpdate(update.clone());
        let data = serde_json::to_vec(&msg)?;
        
        let mut peers = self.peers.write().await;
        let mut to_remove = Vec::new();
        
        for (addr, stream) in peers.iter_mut() {
            if let Err(e) = stream.write_all(&data).await {
                warn!("📡 Failed to broadcast to {}: {}", addr, e);
                to_remove.push(*addr);
            }
        }
        
        for addr in to_remove {
            peers.remove(&addr);
        }
        
        Ok(())
    }

    pub async fn connect_to_peer(&self, addr: SocketAddr) -> Result<()> {
        info!("📡 Connecting to peer: {}", addr);
        let mut stream = TcpStream::connect(addr).await?;
        
        // Send announcement
        let announce = GossipMessage::Announce {
            node_id: self.config.identity.pubkey().to_string(),
            addr: format!("0.0.0.0:{}", self.config.gossip_port).parse()?,
        };
        
        let data = serde_json::to_vec(&announce)?;
        stream.write_all(&data).await?;
        
        self.peers.write().await.insert(addr, stream);
        self.known_peers.write().await.push(addr);
        
        info!("✅ Connected to peer: {}", addr);
        Ok(())
    }
}

pub async fn start_gossip_network(
    config: Arc<NodeConfig>,
    gossip_tx: mpsc::Sender<PriceUpdate>,
    shutdown: tokio::sync::broadcast::Receiver<()>,
) -> Result<()> {
    let network = GossipNetwork::new(config);
    network.start(gossip_tx, shutdown).await
}

// Helper to broadcast custom price data via gossip
pub async fn broadcast_price_update(
    network: &GossipNetwork,
    update: &PriceUpdate,
) -> Result<()> {
    network.broadcast_price_update(update).await
}
