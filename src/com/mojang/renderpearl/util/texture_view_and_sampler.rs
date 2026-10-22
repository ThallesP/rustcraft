//! com/mojang/renderpearl/util/TextureViewAndSampler.java

use crate::com::mojang::renderpearl::api::textures::gpu_sampler::GpuSampler;
use crate::com::mojang::renderpearl::api::textures::gpu_texture_view::GpuTextureView;
use std::rc::Rc;

#[derive(Clone)]
pub struct TextureViewAndSampler {
    pub view: GpuTextureView,
    pub sampler: Rc<dyn GpuSampler>,
}
