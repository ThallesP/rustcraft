//! java.net.InetAddress, where the game depends on it: `getByName`, and `getHostName`, whose
//! reverse lookup names the host in the client's handshake.
//!
//! Java looks names up with the system's resolver: `getaddrinfo` for a name, and for an address
//! with no name `getnameinfo` with `NI_NAMEREQD`, then a lookup of the name it found, which has to
//! give the address back (`getHostFromNameService`), or it's the address's text after all. The
//! port calls the same functions. Java puts IPv4 addresses first (`preferIPv6Addresses` is false
//! by default), so `getByName` takes the first of those.

use std::ffi::CStr;
use std::fmt;
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};

/// An IP address, with the name it was looked up by, if any.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InetAddress {
    address: IpAddr,
    host_name: Option<String>,
}

impl InetAddress {
    /// The host's first address, or `None` where Java throws `UnknownHostException`. An address's
    /// text is the address, with no name yet.
    pub fn get_by_name(host: &str) -> Option<InetAddress> {
        if let Ok(address) = host.parse::<IpAddr>() {
            return Some(InetAddress {
                address,
                host_name: None,
            });
        }

        let addresses: Vec<IpAddr> = (host, 0)
            .to_socket_addrs()
            .ok()?
            .map(|address| address.ip())
            .collect();
        let address = addresses
            .iter()
            .find(|address| address.is_ipv4())
            .or_else(|| addresses.first())?;
        Some(InetAddress {
            address: *address,
            host_name: Some(host.to_string()),
        })
    }

    pub fn address(&self) -> IpAddr {
        self.address
    }

    /// The name it was looked up by, or the one the system's reverse lookup finds, if a lookup of
    /// that name gives the address back; otherwise the address's text.
    pub fn get_host_name(&self) -> String {
        match &self.host_name {
            Some(host_name) => host_name.clone(),
            None => self.get_host_from_name_service(),
        }
    }

    /// `getHostAddress()`: the address's text.
    pub fn get_host_address(&self) -> String {
        self.address.to_string()
    }

    fn get_host_from_name_service(&self) -> String {
        let Some(host) = InetAddress::get_name_info(self.address) else {
            return self.get_host_address();
        };
        let resolves_back = (host.as_str(), 0)
            .to_socket_addrs()
            .is_ok_and(|mut addresses| addresses.any(|address| address.ip() == self.address));
        if resolves_back {
            host
        } else {
            self.get_host_address()
        }
    }

    /// `getnameinfo` with `NI_NAMEREQD`, Java's `Inet4AddressImpl.getHostByAddr` and its IPv6
    /// counterpart. BSD's sockaddrs (macOS's) start with their own length, which Linux's don't
    /// have.
    fn get_name_info(address: IpAddr) -> Option<String> {
        let socket_address = SocketAddr::new(address, 0);
        let mut host = [0 as libc::c_char; libc::NI_MAXHOST as usize];
        let result = match socket_address {
            SocketAddr::V4(v4) => {
                let raw = libc::sockaddr_in {
                    #[cfg(any(target_os = "macos", target_os = "ios", target_os = "freebsd"))]
                    sin_len: size_of::<libc::sockaddr_in>() as u8,
                    sin_family: libc::AF_INET as libc::sa_family_t,
                    sin_port: 0,
                    sin_addr: libc::in_addr {
                        s_addr: u32::from_ne_bytes(v4.ip().octets()),
                    },
                    sin_zero: [0; 8],
                };
                // SAFETY: the address and the buffer live here, with the lengths given.
                unsafe {
                    libc::getnameinfo(
                        (&raw const raw).cast(),
                        size_of::<libc::sockaddr_in>() as libc::socklen_t,
                        host.as_mut_ptr(),
                        host.len() as libc::socklen_t,
                        std::ptr::null_mut(),
                        0,
                        libc::NI_NAMEREQD,
                    )
                }
            }
            SocketAddr::V6(v6) => {
                let raw = libc::sockaddr_in6 {
                    #[cfg(any(target_os = "macos", target_os = "ios", target_os = "freebsd"))]
                    sin6_len: size_of::<libc::sockaddr_in6>() as u8,
                    sin6_family: libc::AF_INET6 as libc::sa_family_t,
                    sin6_port: 0,
                    sin6_flowinfo: 0,
                    sin6_addr: libc::in6_addr {
                        s6_addr: v6.ip().octets(),
                    },
                    sin6_scope_id: v6.scope_id(),
                };
                // SAFETY: as above.
                unsafe {
                    libc::getnameinfo(
                        (&raw const raw).cast(),
                        size_of::<libc::sockaddr_in6>() as libc::socklen_t,
                        host.as_mut_ptr(),
                        host.len() as libc::socklen_t,
                        std::ptr::null_mut(),
                        0,
                        libc::NI_NAMEREQD,
                    )
                }
            }
        };
        if result != 0 {
            return None;
        }

        // SAFETY: `getnameinfo` wrote a NUL-terminated name into the buffer.
        let name = unsafe { CStr::from_ptr(host.as_ptr()) };
        name.to_str().ok().map(str::to_string)
    }
}

/// `localhost/127.0.0.1`, or `/127.0.0.1` with no name yet.
impl fmt::Display for InetAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}/{}",
            self.host_name.as_deref().unwrap_or(""),
            self.get_host_address()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::InetAddress;

    #[test]
    fn names_the_loopback_address_like_java() {
        // What vanilla's handshake names 127.0.0.1 on a machine whose hosts file has localhost,
        // as vanilla's walk capture shows.
        let address = InetAddress::get_by_name("127.0.0.1").unwrap();
        assert_eq!(address.get_host_address(), "127.0.0.1");
        assert_eq!(address.get_host_name(), "localhost");
        assert_eq!(
            InetAddress::get_by_name("localhost")
                .unwrap()
                .get_host_name(),
            "localhost"
        );
    }
}
