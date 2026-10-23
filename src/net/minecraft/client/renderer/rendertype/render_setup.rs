//! net/minecraft/client/renderer/rendertype/RenderSetup.java
//!
//! A render type's state: its pipeline, its textures by sampler name (a Java `HashMap`, whose
//! order the textures bind in; render types so far have one each), and whether it reads the
//! lightmap and the overlay, sorts its quads when uploaded, and so on. A texture binding's sampler
//! override, the OIT pipelines and the outline's texture name are kept as Java has them but only
//! the defaults are used so far.

use std::rc::Rc;
use std::sync::Arc;

use crate::com::mojang::blaze3d::systems::sampler_cache::SamplerCache;
use crate::com::mojang::renderpearl::api::device::gpu_device::GpuDevice;
use crate::com::mojang::renderpearl::api::pipeline::render_pipeline::RenderPipeline;
use crate::com::mojang::renderpearl::api::textures::filter_mode::FilterMode;
use crate::com::mojang::renderpearl::api::textures::gpu_texture_view::GpuTextureView;
use crate::net::minecraft::client::renderer::rendertype::layering_transform::LayeringTransform;
use crate::net::minecraft::client::renderer::rendertype::prepared_render_type::Texture;
use crate::net::minecraft::client::renderer::rendertype::texture_transform::TextureTransform;
use crate::net::minecraft::client::renderer::texture::texture_manager::TextureManager;
use crate::net::minecraft::resources::identifier::Identifier;

/// `RenderSetup.OutlineProperty`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutlineProperty {
    NONE,
    IS_OUTLINE,
    AFFECTS_OUTLINE,
}

/// `RenderSetup.TextureBinding`, without a sampler override.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TextureBinding {
    pub location: Identifier,
}

#[derive(Clone, Debug)]
pub struct RenderSetup {
    pub(crate) pipeline: Arc<RenderPipeline>,
    pub(crate) textures: Vec<(String, TextureBinding)>,
    pub(crate) texture_transform: TextureTransform,
    pub(crate) outline_property: OutlineProperty,
    pub(crate) use_lightmap: bool,
    pub(crate) use_overlay: bool,
    pub(crate) affects_crumbling: bool,
    pub(crate) sort_on_upload: bool,
    pub(crate) layering_transform: LayeringTransform,
    pub(crate) force_solid_model_phase: bool,
}

/// What preparing a render type's textures reads of the game.
pub struct TextureContext<'a> {
    pub device: &'a dyn GpuDevice,
    pub texture_manager: &'a mut TextureManager,
    pub sampler_cache: &'a SamplerCache,
    pub overlay_texture: &'a GpuTextureView,
    pub lightmap_texture: &'a GpuTextureView,
}

impl RenderSetup {
    pub fn builder(pipeline: &Arc<RenderPipeline>) -> RenderSetupBuilder {
        RenderSetupBuilder {
            setup: RenderSetup {
                pipeline: Arc::clone(pipeline),
                textures: Vec::new(),
                texture_transform: TextureTransform::DEFAULT_TEXTURING,
                outline_property: OutlineProperty::NONE,
                use_lightmap: false,
                use_overlay: false,
                affects_crumbling: false,
                sort_on_upload: false,
                layering_transform: LayeringTransform::NO_LAYERING,
                force_solid_model_phase: false,
            },
        }
    }

    /// The overlay, the lightmap and the setup's textures, with their samplers, in the order
    /// they bind.
    pub fn prepare_textures(&self, context: &mut TextureContext<'_>) -> Vec<Texture> {
        let mut textures = Vec::with_capacity(self.textures.len() + 2);
        if self.use_overlay {
            textures.push(Texture {
                name: "Sampler1".to_string(),
                texture_view: context.overlay_texture.clone(),
                sampler: Rc::clone(context.sampler_cache.get_clamp_to_edge(FilterMode::LINEAR)),
            });
        }

        if self.use_lightmap {
            textures.push(Texture {
                name: "Sampler2".to_string(),
                texture_view: context.lightmap_texture.clone(),
                sampler: Rc::clone(context.sampler_cache.get_clamp_to_edge(FilterMode::LINEAR)),
            });
        }

        for (name, binding) in &self.textures {
            let (texture_view, sampler) = context.texture_manager.get_or_load_texture(
                context.device,
                context.sampler_cache,
                &binding.location,
            );
            textures.push(Texture {
                name: name.clone(),
                texture_view: texture_view.clone(),
                sampler: Rc::clone(sampler),
            });
        }

        textures
    }
}

/// `RenderSetup.RenderSetupBuilder`.
pub struct RenderSetupBuilder {
    setup: RenderSetup,
}

impl RenderSetupBuilder {
    pub fn with_texture(mut self, name: &str, texture: Identifier) -> RenderSetupBuilder {
        let binding = TextureBinding { location: texture };
        match self
            .setup
            .textures
            .iter_mut()
            .find(|(bound, _)| bound == name)
        {
            Some((_, bound)) => *bound = binding,
            None => self.setup.textures.push((name.to_string(), binding)),
        }
        self
    }

    pub fn use_lightmap(mut self) -> RenderSetupBuilder {
        self.setup.use_lightmap = true;
        self
    }

    pub fn use_overlay(mut self) -> RenderSetupBuilder {
        self.setup.use_overlay = true;
        self
    }

    pub fn affects_crumbling(mut self) -> RenderSetupBuilder {
        self.setup.affects_crumbling = true;
        self
    }

    pub fn sort_on_upload(mut self) -> RenderSetupBuilder {
        self.setup.sort_on_upload = true;
        self
    }

    pub fn set_outline(mut self, outline_property: OutlineProperty) -> RenderSetupBuilder {
        self.setup.outline_property = outline_property;
        self
    }

    pub fn with_forced_solid_model_phase(mut self) -> RenderSetupBuilder {
        self.setup.force_solid_model_phase = true;
        self
    }

    pub fn create_render_setup(self) -> RenderSetup {
        self.setup
    }
}
