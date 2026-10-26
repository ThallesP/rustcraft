//! net/minecraft/client/renderer/texture/TextureManager.java
//!
//! The textures the game binds by location, each as the view and sampler its `AbstractTexture`
//! has: the ones registered with it (the GUI atlas, the panorama's `CubeMapTexture`, loaded as it's
//! registered), and the ones loaded from the resources the first time they're asked for
//! (`getTexture`'s `SimpleTexture`, the skins). A texture whose
//! resource doesn't load is the missing texture's image, like Java's, which logs why. Not ported
//! yet: the missing texture's own slot, tickable textures, the reload and the dumps.

use std::rc::Rc;

use crate::com::mojang::blaze3d::systems::sampler_cache::SamplerCache;
use crate::com::mojang::renderpearl::api::device::gpu_device::GpuDevice;
use crate::com::mojang::renderpearl::api::textures::gpu_sampler::GpuSampler;
use crate::com::mojang::renderpearl::api::textures::gpu_texture_view::GpuTextureView;
use crate::net::minecraft::client::renderer::texture::cube_map_texture::CubeMapTexture;
use crate::net::minecraft::client::renderer::texture::simple_texture::SimpleTexture;
use crate::net::minecraft::client::renderer::texture::texture_contents::TextureContents;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::server::packs::resources::resource_manager::ResourceManager;

#[derive(Default)]
pub struct TextureManager {
    by_path: Vec<(Identifier, GpuTextureView, Rc<dyn GpuSampler>)>,
    resource_manager: Option<Rc<dyn ResourceManager>>,
}

impl TextureManager {
    /// A manager that only has the textures registered with it.
    pub fn new() -> TextureManager {
        TextureManager::default()
    }

    /// `new TextureManager(resourceManager)`: one that loads what it hasn't got from the
    /// resources.
    pub fn with_resource_manager(resource_manager: Rc<dyn ResourceManager>) -> TextureManager {
        TextureManager {
            by_path: Vec::new(),
            resource_manager: Some(resource_manager),
        }
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

    /// `register(location, new CubeMapTexture(location))` and the reload's load: the cube map's
    /// faces from the resources, or the missing texture's image where they don't load.
    pub fn register_cube_map(
        &mut self,
        device: &dyn GpuDevice,
        sampler_cache: &SamplerCache,
        location: &Identifier,
    ) {
        let mut texture = CubeMapTexture::new(location.clone(), sampler_cache);
        let resource_manager = self
            .resource_manager
            .as_ref()
            .expect("a texture manager with resources");
        let contents = match texture.load_contents(&**resource_manager) {
            Ok(contents) => contents,
            Err(error) => {
                eprintln!("Failed to load texture {location} into slot {location}: {error}");
                TextureContents::create_missing()
            }
        };
        texture.apply(device, sampler_cache, contents);
        let base = &texture.reloadable.base;
        let view = base.get_texture_view().clone();
        let sampler = Rc::clone(base.get_sampler());
        self.register(location.clone(), view, sampler);
    }

    pub fn release(&mut self, location: &Identifier) {
        self.by_path.retain(|(path, _, _)| path != location);
    }

    /// `getTexture(location)`'s view and sampler, of a texture already there. Panics for one that
    /// isn't, which `get_or_load_texture` loads.
    pub fn get_texture(&self, location: &Identifier) -> (&GpuTextureView, &Rc<dyn GpuSampler>) {
        self.by_path
            .iter()
            .find(|(path, _, _)| path == location)
            .map(|(_, view, sampler)| (view, sampler))
            .unwrap_or_else(|| panic!("no texture at {location}: get_or_load_texture loads one"))
    }

    /// `getTexture(location)`: the texture there, or a `SimpleTexture` of the resource at
    /// `location` registered there now (`registerAndLoad`). Panics for a manager without
    /// resources.
    pub fn get_or_load_texture(
        &mut self,
        device: &dyn GpuDevice,
        sampler_cache: &SamplerCache,
        location: &Identifier,
    ) -> (&GpuTextureView, &Rc<dyn GpuSampler>) {
        if !self.by_path.iter().any(|(path, _, _)| path == location) {
            let mut texture = SimpleTexture::new(location.clone(), sampler_cache);
            let contents = self.load_contents_safe(location, &texture);
            texture.reloadable.apply(device, sampler_cache, contents);
            let base = &texture.reloadable.base;
            let view = base.get_texture_view().clone();
            let sampler = Rc::clone(base.get_sampler());
            self.register(location.clone(), view, sampler);
        }
        self.get_texture(location)
    }

    fn load_contents_safe(
        &self,
        texture_id: &Identifier,
        texture: &SimpleTexture,
    ) -> TextureContents {
        let resource_manager = self
            .resource_manager
            .as_ref()
            .expect("a texture manager with resources");
        match texture.load_contents(&**resource_manager) {
            Ok(contents) => contents,
            Err(error) => {
                eprintln!(
                    "Failed to load texture {} into slot {texture_id}: {error}",
                    texture.reloadable.resource_id()
                );
                TextureContents::create_missing()
            }
        }
    }
}
