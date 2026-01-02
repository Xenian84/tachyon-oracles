#![allow(dead_code)]
// Oracle Socket - Adapted from Solana Streamer for Tachyon Oracle Network
// Socket utilities for oracle networking

use std::{
    io::Result,
    net::{SocketAddr, UdpSocket},
};

/// Socket address space filtering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketAddrSpace {
    /// Allow all addresses
    Unspecified,
    /// Allow only global addresses (no private/localhost)
    Global,
}

impl Default for SocketAddrSpace {
    fn default() -> Self {
        Self::Unspecified
    }
}

impl SocketAddrSpace {
    /// Check if an address is allowed in this address space
    pub fn check(&self, addr: &SocketAddr) -> bool {
        match self {
            Self::Unspecified => true,
            Self::Global => {
                let ip = addr.ip();
                // Allow only global addresses
                match ip {
                    std::net::IpAddr::V4(ipv4) => {
                        !ipv4.is_private()
                            && !ipv4.is_loopback()
                            && !ipv4.is_link_local()
                            && !ipv4.is_broadcast()
                            && !ipv4.is_documentation()
                            && !ipv4.is_unspecified()
                    }
                    std::net::IpAddr::V6(ipv6) => {
                        !ipv6.is_loopback()
                            && !ipv6.is_unspecified()
                            && !ipv6.is_multicast()
                    }
                }
            }
        }
    }
}

/// Validator port range for oracle nodes
pub const VALIDATOR_PORT_RANGE: std::ops::Range<u16> = 8000..10000;

/// Create a UDP socket bound to the specified address
pub fn bind_to(addr: SocketAddr) -> Result<UdpSocket> {
    let socket = UdpSocket::bind(addr)?;
    
    // Set socket options for performance
    socket.set_read_timeout(None)?;
    socket.set_write_timeout(None)?;
    
    // Note: Buffer size methods are platform-specific
    // They're available via socket2 crate if needed for optimization
    
    Ok(socket)
}

/// Create a UDP socket bound to localhost on any available port
pub fn bind_to_localhost() -> Result<UdpSocket> {
    let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
    bind_to(addr)
}

/// Create a UDP socket bound to the specified port on all interfaces
pub fn bind_to_port(port: u16) -> Result<UdpSocket> {
    let addr: SocketAddr = format!("0.0.0.0:{}", port).parse().unwrap();
    bind_to(addr)
}

/// Find an available port in the validator range
pub fn find_available_port() -> Result<u16> {
    for port in VALIDATOR_PORT_RANGE {
        if let Ok(_socket) = bind_to_port(port) {
            return Ok(port);
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::AddrNotAvailable,
        "No available ports in validator range",
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_socket_addr_space_unspecified() {
        let space = SocketAddrSpace::Unspecified;
        let addr: SocketAddr = "127.0.0.1:8000".parse().unwrap();
        assert!(space.check(&addr));
    }

    #[test]
    fn test_socket_addr_space_global() {
        let space = SocketAddrSpace::Global;
        
        // Localhost should be rejected
        let localhost: SocketAddr = "127.0.0.1:8000".parse().unwrap();
        assert!(!space.check(&localhost));
        
        // Private IP should be rejected
        let private: SocketAddr = "192.168.1.1:8000".parse().unwrap();
        assert!(!space.check(&private));
        
        // Public IP should be allowed
        let public: SocketAddr = "8.8.8.8:8000".parse().unwrap();
        assert!(space.check(&public));
    }

    #[test]
    fn test_bind_to_localhost() {
        let socket = bind_to_localhost().unwrap();
        assert!(socket.local_addr().unwrap().ip().is_loopback());
    }

    #[test]
    fn test_find_available_port() {
        let port = find_available_port().unwrap();
        assert!(VALIDATOR_PORT_RANGE.contains(&port));
    }
}

