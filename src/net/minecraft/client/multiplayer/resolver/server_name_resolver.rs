//! net/minecraft/client/multiplayer/resolver/ServerNameResolver.java
//!
//! Not ported yet: the SRV redirect (`ServerRedirectHandler.createDnsSrvRedirectHandler`), which
//! looks `_minecraft._tcp.<host>` up when the port is the default 25565, and the address check
//! (`AddressCheck.createFromService`), which refuses the servers Mojang blocks. `DEFAULT` resolves
//! with `ServerAddressResolver.SYSTEM` alone until they come.

use crate::java::net::inet_address::InetAddress;
use crate::net::minecraft::client::multiplayer::resolver::resolved_server_address::ResolvedServerAddress;
use crate::net::minecraft::client::multiplayer::resolver::server_address::ServerAddress;

/// Turns the address the player typed into one to connect to.
pub struct ServerNameResolver;

impl ServerNameResolver {
    /// `DEFAULT.resolveAddress(address)`, with `ServerAddressResolver.SYSTEM`: the host's first
    /// address, or `None` when it doesn't resolve.
    pub fn resolve_address(address: &ServerAddress) -> Option<ResolvedServerAddress> {
        let resolved = InetAddress::get_by_name(address.get_host());
        if resolved.is_none() {
            eprintln!("Couldn't resolve server {} address", address.get_host());
        }
        resolved.map(|resolved| ResolvedServerAddress::from(resolved, address.get_port()))
    }
}
