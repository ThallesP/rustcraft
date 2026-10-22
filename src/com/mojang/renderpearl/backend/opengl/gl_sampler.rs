//! com/mojang/renderpearl/backend/opengl/GlSampler.java
//!
//! Its GL name is deleted when it's dropped (Java's `close()`).

use crate::com::mojang::renderpearl::api::textures::address_mode::AddressMode;
use crate::com::mojang::renderpearl::api::textures::filter_mode::FilterMode;
use crate::com::mojang::renderpearl::api::textures::gpu_sampler::GpuSampler;
use crate::com::mojang::renderpearl::backend::opengl::gl_const::GlConst;
use glow::HasContext;
use std::any::Any;
use std::rc::Rc;

pub struct GlSampler {
    gl: Rc<glow::Context>,
    id: glow::NativeSampler,
    address_mode_u: AddressMode,
    address_mode_v: AddressMode,
    min_filter: FilterMode,
    mag_filter: FilterMode,
    max_anisotropy: i32,
    max_lod: Option<f64>,
}

impl GlSampler {
    pub fn new(
        gl: Rc<glow::Context>,
        address_mode_u: AddressMode,
        address_mode_v: AddressMode,
        min_filter: FilterMode,
        mag_filter: FilterMode,
        max_anisotropy: i32,
        max_lod: Option<f64>,
    ) -> GlSampler {
        // SAFETY: the device's context is current.
        let id = unsafe {
            let id = gl.create_sampler().expect("glGenSamplers");
            gl.sampler_parameter_i32(
                id,
                glow::TEXTURE_WRAP_S,
                GlConst::to_gl_address_mode(address_mode_u) as i32,
            );
            gl.sampler_parameter_i32(
                id,
                glow::TEXTURE_WRAP_T,
                GlConst::to_gl_address_mode(address_mode_v) as i32,
            );
            if max_anisotropy > 1 {
                gl.sampler_parameter_f32(
                    id,
                    glow::TEXTURE_MAX_ANISOTROPY_EXT,
                    max_anisotropy as f32,
                );
            }
            let min = match min_filter {
                FilterMode::NEAREST => glow::NEAREST_MIPMAP_LINEAR,
                FilterMode::LINEAR => glow::LINEAR_MIPMAP_LINEAR,
            };
            gl.sampler_parameter_i32(id, glow::TEXTURE_MIN_FILTER, min as i32);
            let mag = match mag_filter {
                FilterMode::NEAREST => glow::NEAREST,
                FilterMode::LINEAR => glow::LINEAR,
            };
            gl.sampler_parameter_i32(id, glow::TEXTURE_MAG_FILTER, mag as i32);
            if let Some(max_lod) = max_lod {
                gl.sampler_parameter_f32(id, glow::TEXTURE_MAX_LOD, max_lod as f32);
            }
            id
        };
        GlSampler {
            gl,
            id,
            address_mode_u,
            address_mode_v,
            min_filter,
            mag_filter,
            max_anisotropy,
            max_lod,
        }
    }

    pub fn get_id(&self) -> glow::NativeSampler {
        self.id
    }
}

impl GpuSampler for GlSampler {
    fn get_address_mode_u(&self) -> AddressMode {
        self.address_mode_u
    }

    fn get_address_mode_v(&self) -> AddressMode {
        self.address_mode_v
    }

    fn get_min_filter(&self) -> FilterMode {
        self.min_filter
    }

    fn get_mag_filter(&self) -> FilterMode {
        self.mag_filter
    }

    fn get_max_anisotropy(&self) -> i32 {
        self.max_anisotropy
    }

    fn get_max_lod(&self) -> Option<f64> {
        self.max_lod
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}

impl Drop for GlSampler {
    fn drop(&mut self) {
        // SAFETY: the sampler is the context's, which outlives it.
        unsafe { self.gl.delete_sampler(self.id) }
    }
}
