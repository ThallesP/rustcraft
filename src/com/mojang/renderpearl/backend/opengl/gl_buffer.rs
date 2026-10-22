//! com/mojang/renderpearl/backend/opengl/GlBuffer.java
//!
//! `BufferStorage` picks immutable storage (`glBufferStorage`, with `bufferUsageToGlFlag`'s
//! flags) when the device turned on `ARB_buffer_storage`, and mutable storage (`glBufferData`,
//! with `bufferUsageToGlEnum`'s hint) otherwise, like Java's. Reading back is
//! `glGetBufferSubData` for now: Java maps it (`glMapBufferRange`, persistently with immutable
//! storage), which reads the same bytes. `DirectStateAccess`'s calls use the classic entry
//! points, which do the same.

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
    /// `GlConst.bufferUsageToGlFlag`: immutable storage's flags.
    pub fn buffer_usage_to_gl_flag(usage: i32) -> u32 {
        let mut result = 0;
        if usage & 1 != 0 {
            result |= 193;
        }
        if usage & 2 != 0 {
            result |= 194;
        }
        if usage & 8 != 0 {
            result |= 256;
        }
        if usage & 4 != 0 {
            result |= 512;
        }
        result
    }

    /// `GlConst.bufferUsageToGlEnum`: mutable storage's hint.
    pub fn buffer_usage_to_gl_enum(usage: i32) -> u32 {
        let client_storage = usage & 4 != 0;
        if usage & 2 != 0 {
            if client_storage { 35040 } else { 35044 }
        } else if usage & 1 != 0 {
            if client_storage { 35041 } else { 35045 }
        } else {
            35044
        }
    }

    /// `BufferStorage.createBuffer`: `size` bytes, or `data`'s when there is some.
    pub(super) fn new(
        gl: Rc<glow::Context>,
        immutable: bool,
        usage: i32,
        size: i64,
        data: Option<&[u8]>,
    ) -> GlBuffer {
        let size_i32 = i32::try_from(size).expect("buffers under 2 GiB");
        // SAFETY: the device's context is current.
        let handle = unsafe {
            let handle = gl.create_buffer().expect("glGenBuffers");
            gl.bind_buffer(glow::COPY_WRITE_BUFFER, Some(handle));
            if immutable {
                gl.buffer_storage(
                    glow::COPY_WRITE_BUFFER,
                    size_i32,
                    data,
                    GlBuffer::buffer_usage_to_gl_flag(usage),
                );
            } else {
                match data {
                    Some(data) => gl.buffer_data_u8_slice(
                        glow::COPY_WRITE_BUFFER,
                        data,
                        GlBuffer::buffer_usage_to_gl_enum(usage),
                    ),
                    None => gl.buffer_data_size(
                        glow::COPY_WRITE_BUFFER,
                        size_i32,
                        GlBuffer::buffer_usage_to_gl_enum(usage),
                    ),
                }
            }
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

    /// `DirectStateAccess.bufferSubData`.
    pub(super) fn buffer_sub_data(&self, offset: i64, data: &[u8]) {
        // SAFETY: the buffer is the context's.
        unsafe {
            self.gl
                .bind_buffer(glow::COPY_WRITE_BUFFER, Some(self.handle));
            self.gl
                .buffer_sub_data_u8_slice(glow::COPY_WRITE_BUFFER, offset as i32, data);
            self.gl.bind_buffer(glow::COPY_WRITE_BUFFER, None);
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
