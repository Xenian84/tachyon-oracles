#![allow(dead_code)]
// Oracle Packet - Adapted from Solana Streamer for Tachyon Oracle Network
// Simplified packet structure for oracle gossip messages

use std::{
    io::Result,
    net::{SocketAddr, UdpSocket},
    time::{Duration, Instant},
};

/// Maximum size of packet data
pub const PACKET_DATA_SIZE: usize = 1280; // Standard MTU size

/// Number of packets per batch for efficient processing
pub const PACKETS_PER_BATCH: usize = 128;

/// Packet metadata
#[derive(Debug, Clone)]
pub struct PacketMeta {
    pub size: usize,
    pub addr: SocketAddr,
    pub port: u16,
    pub v6: bool,
    pub flags: u32,
}

impl Default for PacketMeta {
    fn default() -> Self {
        Self {
            size: 0,
            addr: "0.0.0.0:0".parse().unwrap(),
            port: 0,
            v6: false,
            flags: 0,
        }
    }
}

impl PacketMeta {
    pub fn socket_addr(&self) -> SocketAddr {
        self.addr
    }

    pub fn set_socket_addr(&mut self, addr: &SocketAddr) {
        self.addr = *addr;
    }
}

/// Oracle packet for gossip messages
#[derive(Clone)]
pub struct OraclePacket {
    data: [u8; PACKET_DATA_SIZE],
    meta: PacketMeta,
}

impl Default for OraclePacket {
    fn default() -> Self {
        Self {
            data: [0u8; PACKET_DATA_SIZE],
            meta: PacketMeta::default(),
        }
    }
}

impl OraclePacket {
    pub fn new(data: [u8; PACKET_DATA_SIZE], meta: PacketMeta) -> Self {
        Self { data, meta }
    }

    pub fn data(&self, range: std::ops::Range<usize>) -> Option<&[u8]> {
        if range.end <= self.meta.size && range.end <= PACKET_DATA_SIZE {
            Some(&self.data[range])
        } else {
            None
        }
    }

    pub fn data_mut(&mut self, range: std::ops::Range<usize>) -> Option<&mut [u8]> {
        if range.end <= PACKET_DATA_SIZE {
            Some(&mut self.data[range])
        } else {
            None
        }
    }

    pub fn buffer_mut(&mut self) -> &mut [u8; PACKET_DATA_SIZE] {
        &mut self.data
    }

    pub fn meta(&self) -> &PacketMeta {
        &self.meta
    }

    pub fn meta_mut(&mut self) -> &mut PacketMeta {
        &mut self.meta
    }
}

/// Batch of oracle packets for efficient processing
pub struct OraclePacketBatch {
    packets: Vec<OraclePacket>,
}

impl OraclePacketBatch {
    pub fn new(packets: Vec<OraclePacket>) -> Self {
        Self { packets }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            packets: Vec::with_capacity(capacity),
        }
    }

    pub fn resize(&mut self, new_len: usize, value: OraclePacket) {
        self.packets.resize(new_len, value);
    }

    pub fn truncate(&mut self, len: usize) {
        self.packets.truncate(len);
    }

    pub fn len(&self) -> usize {
        self.packets.len()
    }

    pub fn is_empty(&self) -> bool {
        self.packets.is_empty()
    }

    pub fn iter(&self) -> impl Iterator<Item = &OraclePacket> {
        self.packets.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut OraclePacket> {
        self.packets.iter_mut()
    }

    pub fn set_addr(&mut self, addr: &SocketAddr) {
        for packet in &mut self.packets {
            packet.meta.set_socket_addr(addr);
        }
    }

    pub fn push(&mut self, packet: OraclePacket) {
        self.packets.push(packet);
    }
}

impl std::ops::Index<usize> for OraclePacketBatch {
    type Output = OraclePacket;

    fn index(&self, index: usize) -> &Self::Output {
        &self.packets[index]
    }
}

impl std::ops::IndexMut<usize> for OraclePacketBatch {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.packets[index]
    }
}

/// Receive packets from UDP socket in batch
pub fn recv_from(
    batch: &mut OraclePacketBatch,
    socket: &UdpSocket,
    max_wait: Duration,
) -> Result<usize> {
    let mut i = 0;
    socket.set_nonblocking(false)?;
    
    let start = Instant::now();
    loop {
        // Resize batch to accommodate more packets
        let target_size = std::cmp::min(i + 32, PACKETS_PER_BATCH);
        batch.resize(target_size, OraclePacket::default());

        // Try to receive a packet
        match socket.recv_from(batch[i].buffer_mut()) {
            Ok((size, addr)) => {
                batch[i].meta_mut().size = size;
                batch[i].meta_mut().set_socket_addr(&addr);
                i += 1;

                if i == 1 {
                    socket.set_nonblocking(true)?;
                }

                if start.elapsed() > max_wait || i >= PACKETS_PER_BATCH {
                    break;
                }
            }
            Err(e) if i > 0 => {
                if start.elapsed() > max_wait {
                    break;
                }
                // Non-blocking socket would block, continue
                if e.kind() == std::io::ErrorKind::WouldBlock {
                    continue;
                }
                break;
            }
            Err(e) => {
                return Err(e);
            }
        }
    }

    batch.truncate(i);
    Ok(i)
}

/// Send packets to UDP socket in batch
pub fn send_to(batch: &OraclePacketBatch, socket: &UdpSocket) -> Result<()> {
    for packet in batch.iter() {
        let addr = packet.meta().socket_addr();
        if let Some(data) = packet.data(0..packet.meta().size) {
            socket.send_to(data, addr)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_default() {
        let packet = OraclePacket::default();
        assert_eq!(packet.meta().size, 0);
    }

    #[test]
    fn test_packet_batch() {
        let mut batch = OraclePacketBatch::with_capacity(10);
        assert_eq!(batch.len(), 0);
        
        batch.push(OraclePacket::default());
        assert_eq!(batch.len(), 1);
    }

    #[test]
    fn test_set_addr() {
        let addr: SocketAddr = "127.0.0.1:8000".parse().unwrap();
        let mut batch = OraclePacketBatch::new(vec![OraclePacket::default()]);
        batch.set_addr(&addr);
        assert_eq!(batch[0].meta().socket_addr(), addr);
    }
}

