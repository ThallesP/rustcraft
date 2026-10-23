//! net/minecraft/client/renderer/texture/TextureManager.java
//!
//! The textures the game binds by location: what `getTexture` hands the GUI. Java registers every
//! `AbstractTexture` here and loads a missing one from the resources (`SimpleTexture`); the port's
//! holds the textures registered with it, each as the view and sampler its `AbstractTexture` has,
//! and loading by location comes with the first caller that needs it (the vignette, the panorama).

use std::rc::Rc;

use crate::com::mojang::renderpearl::api::textures::gpu_sampler::GpuSampler;
use crate::com::mojang::renderpearl::api::textures::gpu_texture_view::GpuTextureView;
use crate::net::minecraft::resources::identifier::Identifier;

#[derive(Default)]
pub struct TextureManager {
    by_path: Vec<(Identifier, GpuTextureView, Rc<dyn GpuSampler>)>,
}

impl TextureManager {
    pub fn new() -> TextureManager {
        TextureManager::default()
    }

    /// `register`: the texture at `location`, replacing one already there.
    pub fn register(
        &mut self,
        location: Identifier,
        texture_view: GpuTextureView,
        sampler: Rc<dyn GpuSampler>,
    ) {
        self.release(&location);
        self.by_path.push((location, texture_view, sampler));
    }

    pub fn release(&mut self, location: &Identifier) {
        self.by_path.retain(|(path, _, _)| path != location);
    }

    /// `getTexture(location)`'s view and sampler. Java loads a texture it hasn't got from the
    /// resources, which isn't ported: that panics.
    pub fn get_texture(&self, location: &Identifier) -> (&GpuTextureView, &Rc<dyn GpuSampler>) {
        self.by_path
            .iter()
            .find(|(path, _, _)| path == location)
            .map(|(_, view, sampler)| (view, sampler))
            .unwrap_or_else(|| panic!("loading {location} by location isn't ported"))
    }
}
