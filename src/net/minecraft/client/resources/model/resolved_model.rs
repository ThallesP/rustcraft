//! net/minecraft/client/resources/model/ResolvedModel.java
//!
//! Not ported yet: `findTopTransform(s)`, which come with item models. `ModelDiscovery`'s models
//! are the only ones there are, and they cache what each getter finds, so the getters are each
//! model's own and their results are shared (`Arc`); the static `find_top_*` do the finding.

use std::sync::{Arc, LazyLock};

use crate::net::minecraft::client::renderer::block::dispatch::model_state::ModelState;
use crate::net::minecraft::client::resources::model::geometry::quad_collection::QuadCollection;
use crate::net::minecraft::client::resources::model::geometry::unbaked_geometry::{
    self, UnbakedGeometry,
};
use crate::net::minecraft::client::resources::model::model_baker::ModelBaker;
use crate::net::minecraft::client::resources::model::model_debug_name::ModelDebugName;
use crate::net::minecraft::client::resources::model::sprite::material::Baked;
use crate::net::minecraft::client::resources::model::sprite::texture_slots::{
    Resolver, TextureSlots,
};
use crate::net::minecraft::client::resources::model::unbaked_model::{
    GuiLight, PARTICLE_TEXTURE_REFERENCE, UnbakedModel,
};

pub const DEFAULT_AMBIENT_OCCLUSION: bool = true;
pub const DEFAULT_GUI_LIGHT: GuiLight = GuiLight::SIDE;

static EMPTY_GEOMETRY: LazyLock<Arc<dyn UnbakedGeometry>> =
    LazyLock::new(|| Arc::new(unbaked_geometry::Empty));

/// A model with its parents found: what it is, and what it inherits.
pub trait ResolvedModel: ModelDebugName + Send + Sync {
    fn wrapped(&self) -> &dyn UnbakedModel;

    fn parent(&self) -> Option<&dyn ResolvedModel>;

    fn get_top_texture_slots(&self) -> Arc<TextureSlots>;

    fn get_top_ambient_occlusion(&self) -> bool;

    fn get_top_gui_light(&self) -> GuiLight;

    fn get_top_geometry(&self) -> Arc<dyn UnbakedGeometry>;

    fn bake_top_geometry(
        &self,
        texture_slots: &TextureSlots,
        baker: &dyn ModelBaker,
        state: &'static dyn ModelState,
    ) -> Arc<QuadCollection>;

    fn resolve_particle_material(
        &self,
        texture_slots: &TextureSlots,
        baker: &dyn ModelBaker,
    ) -> Baked;
}

/// Each model's slots, the model's first and its parents' after, resolved.
pub fn find_top_texture_slots(top: &dyn ResolvedModel) -> TextureSlots {
    let mut current = Some(top);
    let mut resolver = Resolver::default();
    while let Some(model) = current {
        resolver.add_last(model.wrapped().texture_slots().clone());
        current = model.parent();
    }
    resolver.resolve(top)
}

pub fn find_top_ambient_occlusion(mut current: Option<&dyn ResolvedModel>) -> bool {
    while let Some(model) = current {
        if let Some(has_ambient_occlusion) = model.wrapped().ambient_occlusion() {
            return has_ambient_occlusion;
        }
        current = model.parent();
    }
    DEFAULT_AMBIENT_OCCLUSION
}

pub fn find_top_gui_light(mut current: Option<&dyn ResolvedModel>) -> GuiLight {
    while let Some(model) = current {
        if let Some(gui_light) = model.wrapped().gui_light() {
            return gui_light;
        }
        current = model.parent();
    }
    DEFAULT_GUI_LIGHT
}

pub fn find_top_geometry(mut current: Option<&dyn ResolvedModel>) -> Arc<dyn UnbakedGeometry> {
    while let Some(model) = current {
        if let Some(geometry) = model.wrapped().geometry() {
            return geometry;
        }
        current = model.parent();
    }
    EMPTY_GEOMETRY.clone()
}

/// The static `resolveParticleMaterial`.
pub fn resolve_particle_material(
    texture_slots: &TextureSlots,
    baker: &dyn ModelBaker,
    resolved_model: &dyn ModelDebugName,
) -> Baked {
    baker
        .materials()
        .resolve_slot(texture_slots, PARTICLE_TEXTURE_REFERENCE, resolved_model)
}
