//! com/mojang/renderpearl/api/buffers/GpuBuffer.java
//!
//! Mapping hands out a copy of the buffer's bytes for now (`map(true, false)`, reading), which is
//! all a screenshot does with one. `slice()` and `slice(long, long)` take the buffer's `Rc`, so
//! they're this module's.

use crate::com::mojang::renderpearl::api::buffers::gpu_buffer_slice::GpuBufferSlice;
use std::any::Any;
use std::rc::Rc;

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

/// `slice(long, long)`. Panics where Java throws `IllegalArgumentException` for a range outside
/// the buffer.
pub fn slice_range(buffer: &Rc<dyn GpuBuffer>, offset: i64, length: i64) -> GpuBufferSlice {
    assert!(
        offset >= 0 && length >= 0 && offset + length <= buffer.size(),
        "Offset of {offset} and length {length} would put new slice outside buffer's range (of 0,{})",
        buffer.size()
    );
    GpuBufferSlice::new(buffer.clone(), offset, length)
}

/// `slice()`: the whole buffer.
pub fn slice(buffer: &Rc<dyn GpuBuffer>) -> GpuBufferSlice {
    slice_range(buffer, 0, buffer.size())
}
