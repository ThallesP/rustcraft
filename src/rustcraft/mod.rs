//! The port's side of the oracle (TESTING.md), mirroring `oracle/client/`'s packages: what drives
//! the port through a scenario the way the Fabric mod drives vanilla, the port's own live tests
//! (`stall_test`), and the benchmark's measurements (`bench`).

pub mod bench;
pub mod oracle;
pub mod stall_test;
