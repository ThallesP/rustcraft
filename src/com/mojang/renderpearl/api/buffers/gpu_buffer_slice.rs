//! com/mojang/renderpearl/api/buffers/GpuBufferSlice.java
//!
//! Mapping a slice (`map`, `MappedView`) comes with mapped buffers.

use crate::com::mojang::renderpearl::api::buffers::gpu_buffer::GpuBuffer;
use std::rc::Rc;

/// A range of a buffer's bytes, which shares the buffer.
#[derive(Clone)]
pub struct GpuBufferSlice {
    pub buffer: Rc<dyn GpuBuffer>,
    pub offset: i64,
    pub length: i64,
}

impl GpuBufferSlice {
    pub fn new(buffer: Rc<dyn GpuBuffer>, offset: i64, length: i64) -> GpuBufferSlice {
        GpuBufferSlice {
            buffer,
            offset,
            length,
        }
    }

    /// Panics where Java throws `IllegalArgumentException` for a range outside this one.
    pub fn slice(&self, offset: i64, length: i64) -> GpuBufferSlice {
        assert!(
            offset >= 0 && length >= 0 && offset + length <= self.length,
            "Offset of {offset} and length {length} would put new slice outside existing slice's range (of {},{})",
            self.offset,
            self.length
        );
        GpuBufferSlice::new(self.buffer.clone(), self.offset + offset, length)
    }
}
