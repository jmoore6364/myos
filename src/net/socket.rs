// Socket API - Application interface to the network stack
// Provides BSD-style socket interface

use super::ipv4::Ipv4Addr;

/// Socket types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketType {
    Stream,  // TCP
    Dgram,   // UDP
    Raw,     // Raw IP
}

/// Socket domain
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SocketDomain {
    Inet,    // IPv4
    Inet6,   // IPv6
}

/// Socket address
#[derive(Debug, Clone, Copy)]
pub struct SocketAddr {
    pub ip: Ipv4Addr,
    pub port: u16,
}

impl SocketAddr {
    pub fn new(ip: Ipv4Addr, port: u16) -> Self {
        SocketAddr { ip, port }
    }
}

/// Socket structure
pub struct Socket {
    pub domain: SocketDomain,
    pub sock_type: SocketType,
    pub local_addr: Option<SocketAddr>,
    pub remote_addr: Option<SocketAddr>,
}

impl Socket {
    pub fn new(domain: SocketDomain, sock_type: SocketType) -> Self {
        Socket {
            domain,
            sock_type,
            local_addr: None,
            remote_addr: None,
        }
    }

    /// Bind socket to local address
    pub fn bind(&mut self, addr: SocketAddr) -> Result<(), &'static str> {
        self.local_addr = Some(addr);
        Ok(())
    }

    /// Connect socket to remote address
    pub fn connect(&mut self, addr: SocketAddr) -> Result<(), &'static str> {
        self.remote_addr = Some(addr);
        // TODO: Initiate TCP connection or setup UDP association
        crate::println!("Socket: Connecting to {}:{}", addr.ip, addr.port);
        Ok(())
    }

    /// Send data through socket
    pub fn send(&mut self, _data: &[u8]) -> Result<usize, &'static str> {
        // TODO: Implement actual sending
        Err("Socket send not yet implemented")
    }

    /// Receive data from socket
    pub fn recv(&mut self, _buffer: &mut [u8]) -> Result<usize, &'static str> {
        // TODO: Implement actual receiving
        Err("Socket recv not yet implemented")
    }

    /// Close socket
    pub fn close(&mut self) -> Result<(), &'static str> {
        crate::println!("Socket: Closing");
        self.local_addr = None;
        self.remote_addr = None;
        Ok(())
    }
}
