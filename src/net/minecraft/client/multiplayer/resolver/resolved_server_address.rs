//! net/minecraft/client/multiplayer/resolver/ResolvedServerAddress.java
//!
//! Java's is an interface whose one implementation `from` makes over an `InetSocketAddress`, so
//! here it's that implementation.

use std::net::SocketAddr;

use crate::java::net::inet_address::InetAddress;

/// A server's address after looking its host up.
#[derive(Clone, Debug)]
pub struct ResolvedServerAddress {
    address: InetAddress,
    port: i32,
}

impl ResolvedServerAddress {
    pub fn from(address: InetAddress, port: i32) -> ResolvedServerAddress {
        ResolvedServerAddress { address, port }
    }

    /// The name the host was looked up by, or the one a reverse lookup of the address finds.
    pub fn get_host_name(&self) -> String {
        self.address.get_host_name()
    }

    pub fn get_host_ip(&self) -> String {
        self.address.get_host_address()
    }

    pub fn get_port(&self) -> i32 {
        self.port
    }

    pub fn as_inet_socket_address(&self) -> SocketAddr {
        SocketAddr::new(self.address.address(), self.port as u16)
    }
}
