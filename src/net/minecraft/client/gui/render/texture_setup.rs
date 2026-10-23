//! net/minecraft/client/gui/render/TextureSetup.java
//!
//! The textures and samplers a GUI draw binds. Java's record compares its views and samplers by
//! reference, which is comparing the `Rc`s they share here. Its sort key is its `hashCode`, which
//! hashes those references by identity, so it changes from run to run: the GUI only sorts by it
//! between elements of the same pipeline in a layer, which don't overlap, so it only decides how
//! draws batch, not a pixel. The port's key is the first texture's address, as arbitrary.

use std::rc::Rc;

use crate::com::mojang::renderpearl::api::textures::gpu_sampler::GpuSampler;
use crate::com::mojang::renderpearl::api::textures::gpu_texture_view::GpuTextureView;

#[derive(Clone, Default)]
pub struct TextureSetup {
    pub texure0: Option<GpuTextureView>,
    pub texure1: Option<GpuTextureView>,
    pub texure2: Option<GpuTextureView>,
    pub sampler0: Option<Rc<dyn GpuSampler>>,
    pub sampler1: Option<Rc<dyn GpuSampler>>,
    pub sampler2: Option<Rc<dyn GpuSampler>>,
}

impl TextureSetup {
    pub fn single_texture(texture: GpuTextureView, sampler: Rc<dyn GpuSampler>) -> TextureSetup {
        TextureSetup {
            texure0: Some(texture),
            sampler0: Some(sampler),
            ..TextureSetup::default()
        }
    }

    /// `singleTextureWithLightmap`: the lightmap and its sampler are the game renderer's, which the
    /// caller passes.
    pub fn single_texture_with_lightmap(
        texture: GpuTextureView,
        sampler: Rc<dyn GpuSampler>,
        lightmap: GpuTextureView,
        lightmap_sampler: Rc<dyn GpuSampler>,
    ) -> TextureSetup {
        TextureSetup {
            texure0: Some(texture),
            texure2: Some(lightmap),
            sampler0: Some(sampler),
            sampler2: Some(lightmap_sampler),
            ..TextureSetup::default()
        }
    }

    pub fn double_texture(
        texture0: GpuTextureView,
        sampler0: Rc<dyn GpuSampler>,
        texture1: GpuTextureView,
        sampler1: Rc<dyn GpuSampler>,
    ) -> TextureSetup {
        TextureSetup {
            texure0: Some(texture0),
            texure1: Some(texture1),
            sampler0: Some(sampler0),
            sampler1: Some(sampler1),
            ..TextureSetup::default()
        }
    }

    pub fn no_texture() -> TextureSetup {
        TextureSetup::default()
    }

    pub fn get_sort_key(&self) -> usize {
        self.texure0
            .as_ref()
            .map_or(0, |view| Rc::as_ptr(view.texture()) as *const () as usize)
    }
}

fn same_view(a: &Option<GpuTextureView>, b: &Option<GpuTextureView>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => {
            Rc::ptr_eq(a.texture(), b.texture())
                && a.base_mip_level() == b.base_mip_level()
                && a.mip_levels() == b.mip_levels()
        }
        _ => false,
    }
}

fn same_sampler(a: &Option<Rc<dyn GpuSampler>>, b: &Option<Rc<dyn GpuSampler>>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => Rc::ptr_eq(a, b),
        _ => false,
    }
}

impl PartialEq for TextureSetup {
    fn eq(&self, other: &TextureSetup) -> bool {
        same_view(&self.texure0, &other.texure0)
            && same_view(&self.texure1, &other.texure1)
            && same_view(&self.texure2, &other.texure2)
            && same_sampler(&self.sampler0, &other.sampler0)
            && same_sampler(&self.sampler1, &other.sampler1)
            && same_sampler(&self.sampler2, &other.sampler2)
    }
}
