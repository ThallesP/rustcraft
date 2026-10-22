//! Minecraft protocol framing shared by the proxy and its probe client: VarInts, length-prefixed
//! frames with optional zlib compression, packet names from the server's `packets.json` report,
//! and the protocol state of a connection.

pub mod frame;
pub mod packets;
pub mod session;
