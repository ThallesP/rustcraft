//! com/mojang/renderpearl/api/buffers/GpuBuffer.java
//!
//! Mapping hands out a copy of the buffer's bytes for now (`map(true, false)`, reading), which is
//! all a screenshot does with one.

use std::any::Any;

pub const USAGE_MAP_READ: i32 = 1;
pub const USAGE_MAP_WRITE: i32 = 2;
pub const USAGE_HINT_CLIENT_STORAGE: i32 = 4;
pub const USAGE_COPY_DST: i32 = 8;
pub const USAGE_COPY_SRC: i32 = 16;
pub const USAGE_VERTEX: i32 = 32;
pub const USAGE_INDEX: i32 = 64;
pub const USAGE_UNIFORM: i32 = 128;
pub const USAGE_UNIFORM_TEXEL_BUFFER: i32 = 256;
pub const USAGE_INDIRECT_PARAMETERS: i32 = 512;

pub trait GpuBuffer {
    fn size(&self) -> i64;

    fn usage(&self) -> i32;

    /// The buffer's bytes, read back.
    fn map_read(&self) -> Vec<u8>;

    fn as_any(&self) -> &dyn Any;
}
