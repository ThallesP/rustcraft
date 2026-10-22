//! com/mojang/renderpearl/backend/opengl/GlRenderPipeline.java
//!
//! A compiled pipeline: its program and vertex array, and the fixed-function state `bind` sets,
//! taken from the create info. A target without a state writes nothing, and the first target
//! that blends decides the blend function for all of them. 1280 (`GL_INVALID_ENUM`) stands for
//! none, like Java's.
//!
//! `close()` is dropping it, which deletes its program and vertex array.
//! `markAmdShaderCompilerAngry` only matters on AMD's driver (`GlDevice`).

use crate::com::mojang::renderpearl::backend::api::backend_render_pipeline::{
    BackendRenderPipeline, CreateInfo,
};
use crate::com::mojang::renderpearl::backend::opengl::gl_const::GlConst;
use crate::com::mojang::renderpearl::backend::opengl::gl_program::GlProgram;
use crate::com::mojang::renderpearl::backend::opengl::vertex_array::VertexArray;
use glow::HasContext;
use std::any::Any;
use std::rc::Rc;

pub struct GlRenderPipeline {
    gl: Rc<glow::Context>,
    program: GlProgram,
    vertex_array: VertexArray,
    primitive_topology: u32,
    cull: bool,
    polygon_mode: u32,
    depth_enabled: bool,
    depth_func: u32,
    write_depth: bool,
    depth_bias_constant: f32,
    depth_bias_scale_factor: f32,
    write_masks: Vec<i32>,
    blend_enabled: Vec<bool>,
    src_rgb: u32,
    dst_rgb: u32,
    mode_rgb: u32,
    src_alpha: u32,
    dst_alpha: u32,
    mode_alpha: u32,
}

impl GlRenderPipeline {
    pub fn new(
        gl: Rc<glow::Context>,
        create_info: &CreateInfo,
        program: GlProgram,
        vertex_array: VertexArray,
    ) -> GlRenderPipeline {
        let (depth_enabled, depth_func, write_depth, depth_bias_constant, depth_bias_scale_factor) =
            match &create_info.depth_stencil_state {
                Some(state) => (
                    true,
                    GlConst::to_gl_compare_op(state.depth_test),
                    state.write_depth,
                    state.depth_bias_constant,
                    state.depth_bias_scale_factor,
                ),
                None => (false, 1280, false, 0.0, 0.0),
            };
        let write_masks = create_info
            .color_target_states
            .iter()
            .map(|state| state.map_or(-1, |state| state.write_mask & 15))
            .collect();
        let blend_enabled = create_info
            .color_target_states
            .iter()
            .map(|state| state.is_some_and(|state| state.blend_function.is_some()))
            .collect();
        let representative_blend_state = create_info
            .color_target_states
            .iter()
            .flatten()
            .find_map(|state| state.blend_function);
        let (src_rgb, dst_rgb, mode_rgb, src_alpha, dst_alpha, mode_alpha) =
            match representative_blend_state {
                Some(blend_function) => (
                    GlConst::to_gl_blend_factor(blend_function.color.source_factor),
                    GlConst::to_gl_blend_factor(blend_function.color.dest_factor),
                    GlConst::to_gl_blend_op(blend_function.color.op),
                    GlConst::to_gl_blend_factor(blend_function.alpha.source_factor),
                    GlConst::to_gl_blend_factor(blend_function.alpha.dest_factor),
                    GlConst::to_gl_blend_op(blend_function.alpha.op),
                ),
                None => (1280, 1280, 1280, 1280, 1280, 1280),
            };
        GlRenderPipeline {
            gl,
            program,
            vertex_array,
            primitive_topology: GlConst::to_gl_primitive_topology(create_info.primitive_topology),
            cull: create_info.cull,
            polygon_mode: GlConst::to_gl_polygon_mode(create_info.polygon_mode),
            depth_enabled,
            depth_func,
            write_depth,
            depth_bias_constant,
            depth_bias_scale_factor,
            write_masks,
            blend_enabled,
            src_rgb,
            dst_rgb,
            mode_rgb,
            src_alpha,
            dst_alpha,
            mode_alpha,
        }
    }

    pub fn program(&self) -> &GlProgram {
        &self.program
    }

    pub fn vertex_array(&self) -> &VertexArray {
        &self.vertex_array
    }

    pub fn bind(&self) {
        let gl = &self.gl;
        // SAFETY: the device's context is current, and the program is its.
        unsafe {
            gl.use_program(Some(self.program.get_program_id()));
            if self.depth_enabled {
                gl.enable(glow::DEPTH_TEST);
                gl.depth_func(self.depth_func);
                gl.depth_mask(self.write_depth);
                if self.depth_bias_constant == 0.0 && self.depth_bias_scale_factor == 0.0 {
                    gl.disable(glow::POLYGON_OFFSET_FILL);
                } else {
                    gl.polygon_offset(self.depth_bias_scale_factor, self.depth_bias_constant);
                    gl.enable(glow::POLYGON_OFFSET_FILL);
                }
            } else {
                gl.disable(glow::DEPTH_TEST);
                gl.depth_mask(false);
                gl.disable(glow::POLYGON_OFFSET_FILL);
            }

            if self.cull {
                gl.enable(glow::CULL_FACE);
            } else {
                gl.disable(glow::CULL_FACE);
            }

            for (i, &write_mask) in self.write_masks.iter().enumerate() {
                let mask = if write_mask != -1 { write_mask } else { 0 };
                gl.color_mask_draw_buffer(
                    i as u32,
                    mask & 1 != 0,
                    mask & 2 != 0,
                    mask & 4 != 0,
                    mask & 8 != 0,
                );
                if self.blend_enabled[i] {
                    gl.enable_draw_buffer(glow::BLEND, i as u32);
                } else {
                    gl.disable_draw_buffer(glow::BLEND, i as u32);
                }
            }

            if self.src_rgb != 1280 {
                gl.blend_func_separate(self.src_rgb, self.dst_rgb, self.src_alpha, self.dst_alpha);
                gl.blend_equation_separate(self.mode_rgb, self.mode_alpha);
            }

            gl.polygon_mode(glow::FRONT_AND_BACK, self.polygon_mode);
        }
    }

    pub fn primitive_topology(&self) -> u32 {
        self.primitive_topology
    }
}

impl BackendRenderPipeline for GlRenderPipeline {
    fn is_closed(&self) -> bool {
        false
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
