//! net/minecraft/client/renderer/block/FluidStateModelSet.java
//!
//! Each fluid's model, and the missing model (the missing block model's particle sprite, solid)
//! for any other: the empty fluid, which nothing draws. Java's `Map.of` is keyed by the fluid,
//! so here it's a list by `Fluids`.

use std::sync::Arc;

use crate::net::minecraft::client::color::block::block_tint_sources::BlockTintSources;
use crate::net::minecraft::client::renderer::block::fluid_model::{self, FluidModel};
use crate::net::minecraft::client::resources::model::model_debug_name::ModelDebugName;
use crate::net::minecraft::client::resources::model::sprite::material::Material;
use crate::net::minecraft::client::resources::model::sprite::material_baker::MaterialBaker;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::level::material::fluids::Fluids;

pub struct FluidStateModelSet {
    model_by_fluid: Vec<(Fluids, Arc<FluidModel>)>,
    missing_model: Arc<FluidModel>,
}

/// The model's name in the bake's warnings, Java's lambda.
struct Named(&'static str);

impl ModelDebugName for Named {
    fn debug_name(&self) -> String {
        self.0.to_owned()
    }
}

impl FluidStateModelSet {
    pub fn new(
        model_by_fluid: Vec<(Fluids, Arc<FluidModel>)>,
        missing_model: Arc<FluidModel>,
    ) -> FluidStateModelSet {
        FluidStateModelSet {
            model_by_fluid,
            missing_model,
        }
    }

    fn water_model() -> fluid_model::Unbaked {
        fluid_model::Unbaked {
            still_material: Material::new(Identifier::with_default_namespace("block/water_still")),
            flowing_material: Material::new(Identifier::with_default_namespace("block/water_flow")),
            overlay_material: Some(Material::new(Identifier::with_default_namespace(
                "block/water_overlay",
            ))),
            tint_source: Some(BlockTintSources::water()),
        }
    }

    fn lava_model() -> fluid_model::Unbaked {
        fluid_model::Unbaked {
            still_material: Material::new(Identifier::with_default_namespace("block/lava_still")),
            flowing_material: Material::new(Identifier::with_default_namespace("block/lava_flow")),
            overlay_material: None,
            tint_source: None,
        }
    }

    pub fn bake(materials: &MaterialBaker<'_>) -> Vec<(Fluids, Arc<FluidModel>)> {
        let water_model =
            Arc::new(FluidStateModelSet::water_model().bake(materials, &Named("Water")));
        let lava_model = Arc::new(FluidStateModelSet::lava_model().bake(materials, &Named("Lava")));
        vec![
            (Fluids::WATER, Arc::clone(&water_model)),
            (Fluids::FLOWING_WATER, water_model),
            (Fluids::LAVA, Arc::clone(&lava_model)),
            (Fluids::FLOWING_LAVA, lava_model),
        ]
    }

    pub fn get(&self, state: FluidState) -> &Arc<FluidModel> {
        self.model_by_fluid
            .iter()
            .find(|(fluid, _)| *fluid == state.get_type())
            .map_or(&self.missing_model, |(_, model)| model)
    }
}
