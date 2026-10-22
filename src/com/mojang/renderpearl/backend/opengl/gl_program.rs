//! com/mojang/renderpearl/backend/opengl/GlProgram.java
//!
//! A linked program, and where it reads each of its pipeline's uniforms. SPIRV-Cross names the
//! `i`th uniform `_uniform_00_{i}` (`GlPipelineRecompiler`), so `setupBindGroupLayouts` finds
//! each by that name and gives it the next uniform block binding or texture unit, in the create
//! info's order. Its GL name, and its texel buffers' textures, are deleted when it's dropped
//! (Java's `close()`).

use crate::com::mojang::renderpearl::api::pipeline::bind_group_layout::UniformDescription;
use crate::com::mojang::renderpearl::api::pipeline::uniform_type::UniformType;
use crate::com::mojang::renderpearl::backend::opengl::gl_shader_module::GlShaderModule;
use crate::com::mojang::renderpearl::backend::opengl::uniform::Uniform;
use crate::com::mojang::renderpearl::util::shader_compile_exception::ShaderCompileException;
use glow::HasContext;
use std::rc::Rc;

pub struct GlProgram {
    gl: Rc<glow::Context>,
    uniforms: Vec<Option<Uniform>>,
    push_constant: Option<Uniform>,
    program_id: glow::NativeProgram,
    debug_label: String,
}

fn shader_list(compiled_shaders: &[GlShaderModule]) -> String {
    compiled_shaders
        .iter()
        .map(|shader| format!("{:?} {}", shader.get_type(), shader.get_label()))
        .collect::<Vec<_>>()
        .join(", ")
}

impl GlProgram {
    /// Java logs the link's info log when there is one.
    pub fn link(
        gl: &Rc<glow::Context>,
        compiled_shaders: &[GlShaderModule],
        debug_label: &str,
    ) -> Result<GlProgram, ShaderCompileException> {
        // SAFETY: the device's context is current, and the shaders are its.
        unsafe {
            let program_id = gl.create_program().map_err(|id| {
                ShaderCompileException(format!(
                    "Could not create shader program (returned program ID {id})"
                ))
            })?;
            for shader_module in compiled_shaders {
                gl.attach_shader(program_id, shader_module.get_shader_id());
            }
            gl.link_program(program_id);
            let link_status = gl.get_program_link_status(program_id);
            let link_message = gl.get_program_info_log(program_id);
            if link_status && !link_message.contains("Failed for unknown reason") {
                Ok(GlProgram {
                    gl: gl.clone(),
                    uniforms: Vec::new(),
                    push_constant: None,
                    program_id,
                    debug_label: debug_label.to_owned(),
                })
            } else {
                gl.delete_program(program_id);
                Err(ShaderCompileException(format!(
                    "Error encountered when linking program containing {}. Log output: {link_message}",
                    shader_list(compiled_shaders)
                )))
            }
        }
    }

    pub fn setup_bind_group_layouts(&mut self, uniforms: &[UniformDescription]) {
        let gl = self.gl.clone();
        let mut next_ubo_binding = 0;
        let mut next_sampler_index = 0;
        // SAFETY: the device's context is current, and the program is its.
        unsafe {
            gl.use_program(Some(self.program_id));
            if let Some(push_constant_block) =
                gl.get_uniform_block_index(self.program_id, "_push_constants")
            {
                let ubo_binding = next_ubo_binding;
                next_ubo_binding += 1;
                gl.uniform_block_binding(self.program_id, push_constant_block, ubo_binding as u32);
                self.push_constant = Some(Uniform::Ubo {
                    block_binding: ubo_binding,
                });
            }

            for (i, uniform_description) in uniforms.iter().enumerate() {
                let uniform_name = format!("_uniform_{:02}_{:02}", 0, i);
                let uniform = match uniform_description.uniform_type {
                    UniformType::UNIFORM_BUFFER => gl
                        .get_uniform_block_index(self.program_id, &uniform_name)
                        .map(|index| {
                            let ubo_binding = next_ubo_binding;
                            next_ubo_binding += 1;
                            gl.uniform_block_binding(self.program_id, index, ubo_binding as u32);
                            Uniform::Ubo {
                                block_binding: ubo_binding,
                            }
                        }),
                    UniformType::TEXEL_BUFFER => gl
                        .get_uniform_location(self.program_id, &uniform_name)
                        .map(|location| {
                            let sampler_index = next_sampler_index;
                            next_sampler_index += 1;
                            gl.uniform_1_i32(Some(&location), sampler_index);
                            Uniform::Utb {
                                sampler_index,
                                format: uniform_description
                                    .gpu_format
                                    .expect("a texel buffer's format"),
                                texture: gl.create_texture().expect("glGenTextures"),
                            }
                        }),
                    UniformType::COMBINED_IMAGE_SAMPLER => gl
                        .get_uniform_location(self.program_id, &uniform_name)
                        .map(|location| {
                            let sampler_index = next_sampler_index;
                            next_sampler_index += 1;
                            gl.uniform_1_i32(Some(&location), sampler_index);
                            Uniform::Sampler { sampler_index }
                        }),
                };
                self.uniforms.push(uniform);
            }
            gl.use_program(None);
        }
    }

    pub fn get_uniform(&self, index: usize) -> Option<Uniform> {
        self.uniforms.get(index).copied().flatten()
    }

    pub fn uniform_count(&self) -> usize {
        self.uniforms.len()
    }

    pub fn get_program_id(&self) -> glow::NativeProgram {
        self.program_id
    }

    pub fn get_debug_label(&self) -> &str {
        &self.debug_label
    }

    pub fn push_constant(&self) -> Option<Uniform> {
        self.push_constant
    }
}

impl Drop for GlProgram {
    fn drop(&mut self) {
        // SAFETY: the program and textures are the context's, which outlives them.
        unsafe {
            for uniform in self.uniforms.iter().flatten() {
                if let Uniform::Utb { texture, .. } = uniform {
                    self.gl.delete_texture(*texture);
                }
            }
            self.gl.delete_program(self.program_id);
        }
    }
}
