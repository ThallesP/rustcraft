//! com/mojang/renderpearl/backend/opengl/VertexArray.java
//!
//! A pipeline's vertex array: where each attribute's data lies in which buffer slot. `Separate`
//! sets the formats up once with `ARB_vertex_attrib_binding` and binds buffers to slots, which
//! llvmpipe has; `Emulated` points every attribute at its buffer on each bind. Its GL name is
//! deleted when it's dropped (Java's `close()`).

use crate::com::mojang::renderpearl::api::buffers::gpu_buffer_slice::GpuBufferSlice;
use crate::com::mojang::renderpearl::backend::api::backend_render_pipeline::CreateInfo;
use crate::com::mojang::renderpearl::backend::opengl::gl_buffer::GlBuffer;
use crate::com::mojang::renderpearl::backend::opengl::gl_const::GlConst;
use glow::HasContext;
use std::rc::Rc;

pub struct VertexArray {
    gl: Rc<glow::Context>,
    vertex_array_id: glow::NativeVertexArray,
    kind: Kind,
}

enum Kind {
    Emulated(Box<Emulated>),
    Separate(Separate),
}

#[derive(Default)]
struct Emulated {
    buffer_stride: [i32; 16],
    buffer_divisor: [i32; 16],
    buffer_index: [i32; 16],
    channel_count: [i32; 16],
    gl_type: [u32; 16],
    is_integer: [bool; 16],
    is_normalized: [bool; 16],
    element_offset: [i32; 16],
}

struct Separate {
    needs_mesa_workaround: bool,
    strides: [i32; 16],
}

fn gl_buffer(slice: &GpuBufferSlice) -> &GlBuffer {
    slice
        .buffer
        .as_any()
        .downcast_ref::<GlBuffer>()
        .expect("an OpenGL buffer")
}

impl VertexArray {
    /// `createSource`'s choice: `Separate` when the device turned on `ARB_vertex_attrib_binding`.
    pub fn new(gl: &Rc<glow::Context>, separate: bool, create_info: &CreateInfo) -> VertexArray {
        // SAFETY: the device's context is current.
        let vertex_array_id = unsafe { gl.create_vertex_array() }.expect("glGenVertexArrays");
        let kind = if separate {
            Kind::Separate(VertexArray::separate(gl, vertex_array_id, create_info))
        } else {
            Kind::Emulated(Box::new(VertexArray::emulated(
                gl,
                vertex_array_id,
                create_info,
            )))
        };
        VertexArray {
            gl: gl.clone(),
            vertex_array_id,
            kind,
        }
    }

    fn emulated(
        gl: &glow::Context,
        vertex_array_id: glow::NativeVertexArray,
        create_info: &CreateInfo,
    ) -> Emulated {
        let mut emulated = Emulated::default();
        // SAFETY: the device's context is current, and the vertex array is its.
        unsafe {
            gl.bind_vertex_array(Some(vertex_array_id));
            for vertex_buffer in &create_info.vertex_buffers {
                emulated.buffer_stride[vertex_buffer.buffer_slot as usize] = vertex_buffer.stride;
                emulated.buffer_divisor[vertex_buffer.buffer_slot as usize] =
                    vertex_buffer.step_rate;
            }
            emulated.buffer_index = [-1; 16];
            for attrib_binding in &create_info.attrib_bindings {
                let attrib_location = attrib_binding.location as usize;
                gl.enable_vertex_attrib_array(attrib_binding.location as u32);
                emulated.buffer_index[attrib_location] = attrib_binding.buffer_slot;
                let gl_external_id = GlConst::to_gl_external_id(attrib_binding.format);
                emulated.gl_type[attrib_location] = GlConst::to_gl_type(attrib_binding.format);
                emulated.is_integer[attrib_location] =
                    GlConst::is_gl_format_integer(gl_external_id);
                emulated.is_normalized[attrib_location] =
                    GlConst::is_format_normalized(attrib_binding.format);
                emulated.channel_count[attrib_location] =
                    GlConst::gl_format_channel_count(gl_external_id);
                emulated.element_offset[attrib_location] = attrib_binding.offset;
            }
            gl.bind_vertex_array(None);
        }
        emulated
    }

    fn separate(
        gl: &glow::Context,
        vertex_array_id: glow::NativeVertexArray,
        create_info: &CreateInfo,
    ) -> Separate {
        // SAFETY: the device's context is current, and the vertex array is its.
        unsafe {
            let needs_mesa_workaround = if gl.get_parameter_string(glow::VENDOR) == "Mesa" {
                let version = gl.get_parameter_string(glow::VERSION);
                version.contains("25.0.0")
                    || version.contains("25.0.1")
                    || version.contains("25.0.2")
            } else {
                false
            };
            let mut strides = [0; 16];
            gl.bind_vertex_array(Some(vertex_array_id));
            for vertex_buffer in &create_info.vertex_buffers {
                strides[vertex_buffer.buffer_slot as usize] = vertex_buffer.stride;
                gl.vertex_binding_divisor(
                    vertex_buffer.buffer_slot as u32,
                    vertex_buffer.step_rate as u32,
                );
            }
            for attrib_binding in &create_info.attrib_bindings {
                let attrib_location = attrib_binding.location as u32;
                gl.enable_vertex_attrib_array(attrib_location);
                let gl_external_id = GlConst::to_gl_external_id(attrib_binding.format);
                let gl_type = GlConst::to_gl_type(attrib_binding.format);
                let is_integer_format = GlConst::is_gl_format_integer(gl_external_id);
                let is_normalized_format = GlConst::is_format_normalized(attrib_binding.format);
                let channel_count = GlConst::gl_format_channel_count(gl_external_id);
                if is_integer_format {
                    gl.vertex_attrib_format_i32(
                        attrib_location,
                        channel_count,
                        gl_type,
                        attrib_binding.offset as u32,
                    );
                } else {
                    gl.vertex_attrib_format_f32(
                        attrib_location,
                        channel_count,
                        gl_type,
                        is_normalized_format,
                        attrib_binding.offset as u32,
                    );
                }
                gl.vertex_attrib_binding(attrib_location, attrib_binding.buffer_slot as u32);
            }
            gl.bind_vertex_array(None);
            Separate {
                needs_mesa_workaround,
                strides,
            }
        }
    }

    /// Panics where Java throws `IllegalStateException` for a slot the pipeline reads with no
    /// buffer.
    pub fn bind(&self, vertex_buffers: &[Option<GpuBufferSlice>; 16]) {
        let gl = &self.gl;
        // SAFETY: the device's context is current, and the vertex array and buffers are its.
        unsafe {
            gl.bind_vertex_array(Some(self.vertex_array_id));
            match &self.kind {
                Kind::Emulated(emulated) => {
                    for attribute_index in 0..16 {
                        let vertex_buffer_index = emulated.buffer_index[attribute_index];
                        if vertex_buffer_index == -1 {
                            continue;
                        }
                        let slice = vertex_buffers[vertex_buffer_index as usize]
                            .as_ref()
                            .unwrap_or_else(|| {
                                panic!(
                                    "Vertex buffer slot {vertex_buffer_index} not specified but required"
                                )
                            });
                        gl.bind_buffer(glow::ARRAY_BUFFER, Some(gl_buffer(slice).handle));
                        let total_offset =
                            slice.offset + i64::from(emulated.element_offset[attribute_index]);
                        let stride = emulated.buffer_stride[vertex_buffer_index as usize];
                        if emulated.is_integer[attribute_index] {
                            gl.vertex_attrib_pointer_i32(
                                attribute_index as u32,
                                emulated.channel_count[attribute_index],
                                emulated.gl_type[attribute_index],
                                stride,
                                total_offset as i32,
                            );
                        } else {
                            gl.vertex_attrib_pointer_f32(
                                attribute_index as u32,
                                emulated.channel_count[attribute_index],
                                emulated.gl_type[attribute_index],
                                emulated.is_normalized[attribute_index],
                                stride,
                                total_offset as i32,
                            );
                        }
                        gl.vertex_attrib_divisor(
                            attribute_index as u32,
                            emulated.buffer_divisor[vertex_buffer_index as usize] as u32,
                        );
                    }
                }
                Kind::Separate(separate) => {
                    for (i, vertex_buffer_slice) in vertex_buffers.iter().enumerate() {
                        if separate.strides[i] == 0 {
                            continue;
                        }
                        let slice = vertex_buffer_slice.as_ref().unwrap_or_else(|| {
                            panic!("Vertex buffer slot {i} not specified but required")
                        });
                        if separate.needs_mesa_workaround {
                            gl.bind_vertex_buffer(i as u32, None, 0, 0);
                        }
                        gl.bind_vertex_buffer(
                            i as u32,
                            Some(gl_buffer(slice).handle),
                            slice.offset as i32,
                            separate.strides[i],
                        );
                    }
                }
            }
        }
    }
}

impl Drop for VertexArray {
    fn drop(&mut self) {
        // SAFETY: the vertex array is the context's, which outlives it.
        unsafe { self.gl.delete_vertex_array(self.vertex_array_id) }
    }
}
