//! com/mojang/renderpearl/backend/opengl/GlBuffer.java
//!
//! Only a mutable buffer of a size so far (`BufferStorage.Mutable`'s `glBufferData`), read back
//! with `glGetBufferSubData`. Java maps it (`glMapBufferRange`), and picks immutable storage when
//! the driver has `ARB_buffer_storage`; neither changes what a screenshot reads.

use crate::com::mojang::renderpearl::api::buffers::gpu_buffer::GpuBuffer;
use glow::HasContext;
use std::any::Any;
use std::rc::Rc;

pub struct GlBuffer {
    gl: Rc<glow::Context>,
    pub(super) handle: glow::NativeBuffer,
    usage: i32,
    size: i64,
}

impl GlBuffer {
    pub(super) fn new(gl: Rc<glow::Context>, usage: i32, size: i64) -> GlBuffer {
        // SAFETY: the device's context is current.
        let handle = unsafe {
            let handle = gl.create_buffer().expect("glGenBuffers");
            gl.bind_buffer(glow::COPY_WRITE_BUFFER, Some(handle));
            gl.buffer_data_size(
                glow::COPY_WRITE_BUFFER,
                i32::try_from(size).expect("buffers under 2 GiB"),
                glow::STREAM_READ,
            );
            gl.bind_buffer(glow::COPY_WRITE_BUFFER, None);
            handle
        };
        GlBuffer {
            gl,
            handle,
            usage,
            size,
        }
    }
}

impl GpuBuffer for GlBuffer {
    fn size(&self) -> i64 {
        self.size
    }

    fn usage(&self) -> i32 {
        self.usage
    }

    fn map_read(&self) -> Vec<u8> {
        let mut data = vec![0; self.size as usize];
        // SAFETY: the buffer is the context's, and `data` is its size.
        unsafe {
            self.gl
                .bind_buffer(glow::COPY_READ_BUFFER, Some(self.handle));
            self.gl
                .get_buffer_sub_data(glow::COPY_READ_BUFFER, 0, &mut data);
            self.gl.bind_buffer(glow::COPY_READ_BUFFER, None);
        }
        data
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Drop for GlBuffer {
    fn drop(&mut self) {
        // SAFETY: the buffer is the context's, which outlives it.
        unsafe { self.gl.delete_buffer(self.handle) }
    }
}
