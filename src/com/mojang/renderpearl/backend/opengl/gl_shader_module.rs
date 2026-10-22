//! com/mojang/renderpearl/backend/opengl/GlShaderModule.java
//!
//! A compiled shader stage, whose GL name is deleted when it's dropped (Java's `close()`, which
//! `compileProgram` calls once the program is linked). `INVALID_SHADER` is `None` where it's
//! returned.

use crate::com::mojang::renderpearl::api::pipeline::shader_type::ShaderType;
use glow::HasContext;
use std::rc::Rc;

pub struct GlShaderModule {
    gl: Rc<glow::Context>,
    label: String,
    shader_id: glow::NativeShader,
    shader_type: ShaderType,
}

impl GlShaderModule {
    pub fn new(
        gl: Rc<glow::Context>,
        shader_id: glow::NativeShader,
        label: &str,
        shader_type: ShaderType,
    ) -> GlShaderModule {
        GlShaderModule {
            gl,
            label: label.to_owned(),
            shader_id,
            shader_type,
        }
    }

    pub fn get_label(&self) -> &str {
        &self.label
    }

    pub fn get_shader_id(&self) -> glow::NativeShader {
        self.shader_id
    }

    pub fn get_type(&self) -> ShaderType {
        self.shader_type
    }
}

impl Drop for GlShaderModule {
    fn drop(&mut self) {
        // SAFETY: the shader is the context's, which outlives it.
        unsafe { self.gl.delete_shader(self.shader_id) }
    }
}
