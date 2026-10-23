//! Minecraft Java Edition client, ported to Rust. Modules mirror the Java packages, so
//! `net::minecraft::util::mth` is `net/minecraft/util/Mth.java`. See PORTING.md.

// Java's names are kept: class names with their acronyms (ARGB, AABB, UUID), enum constants
// (`Rotation::CLOCKWISE_90`), and method names whatever their receiver (`fromSeed(&self)` reads
// as a constructor to clippy).
#![allow(
    non_camel_case_types,
    clippy::upper_case_acronyms,
    clippy::wrong_self_convention
)]

pub mod com;
pub mod io;
pub mod it;
pub mod java;
pub mod net;
pub mod org;
pub mod rustcraft;

#[cfg(test)]
mod golden;
