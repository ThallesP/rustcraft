//! net/minecraft/client/resources/model/ModelBakery.java
//!
//! Not ported yet: the item models' bake (`clientInfos`, `ItemModel`, `MissingItemModel`), the
//! missing fluid model, the destroy stages and fire sprites, which come with their renderers.
//!
//! Java bakes the block states on an executor (`ParallelMapTransform`) and logs a state whose bake
//! throws; here they're baked in turn. The interner shares equal material infos by their record
//! fields, like Guava's strong interner, and copies vectors, which is the same value.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use crate::net::minecraft::client::renderer::block::dispatch::block_model_rotation::BlockModelRotation;
use crate::net::minecraft::client::renderer::block::dispatch::block_state_model::{
    BlockStateModel, UnbakedRoot,
};
use crate::net::minecraft::client::renderer::block::dispatch::block_state_model_part::BlockStateModelPart;
use crate::net::minecraft::client::renderer::block::dispatch::single_variant::SingleVariant;
use crate::net::minecraft::client::renderer::chunk::chunk_section_layer::ChunkSectionLayer;
use crate::net::minecraft::client::resources::model::geometry::baked_quad::MaterialInfo;
use crate::net::minecraft::client::resources::model::model_baker::{Interner, ModelBaker, Shared};
use crate::net::minecraft::client::resources::model::model_discovery::ModelWrapper;
use crate::net::minecraft::client::resources::model::resolved_model::ResolvedModel;
use crate::net::minecraft::client::resources::model::simple_model_wrapper::SimpleModelWrapper;
use crate::net::minecraft::client::resources::model::sprite::material_baker::MaterialBaker;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;

pub struct ModelBakery {
    unbaked_block_state_models: HashMap<BlockState, Arc<dyn UnbakedRoot>>,
    resolved_models: HashMap<Identifier, Arc<ModelWrapper>>,
    missing_model: Arc<ModelWrapper>,
}

/// `ModelBakery.BakingResult`, the block states' half.
pub struct BakingResult {
    pub missing_models: MissingModels,
    pub block_state_models: HashMap<BlockState, Arc<dyn BlockStateModel>>,
}

/// `ModelBakery.MissingModels`, the block's half.
pub struct MissingModels {
    pub block_part: Arc<dyn BlockStateModelPart>,
    pub block: Arc<dyn BlockStateModel>,
}

/// A material info's record fields, its sprite by address.
type MaterialInfoKey = (usize, ChunkSectionLayer, i32, Option<Direction>, i32);

#[derive(Default)]
struct InternerImpl {
    // Keyed by the record's fields, the sprite by its address.
    material_infos: RefCell<HashMap<MaterialInfoKey, Arc<MaterialInfo>>>,
}

impl Interner for InternerImpl {
    fn material_info(&self, material: MaterialInfo) -> Arc<MaterialInfo> {
        let key = (
            Arc::as_ptr(&material.sprite) as usize,
            material.layer,
            material.tint_index,
            material.shade_direction_override,
            material.light_emission,
        );
        self.material_infos
            .borrow_mut()
            .entry(key)
            .or_insert_with(|| Arc::new(material))
            .clone()
    }
}

impl ModelBakery {
    // A state hashes its definition's address and its index, which never change: the definition's
    // cell is its registration, set once.
    #[allow(clippy::mutable_key_type)]
    pub fn new(
        unbaked_block_state_models: HashMap<BlockState, Arc<dyn UnbakedRoot>>,
        resolved_models: HashMap<Identifier, Arc<ModelWrapper>>,
        missing_model: Arc<ModelWrapper>,
    ) -> ModelBakery {
        ModelBakery {
            unbaked_block_state_models,
            resolved_models,
            missing_model,
        }
    }

    // A state hashes its definition's address and its index, which never change: the definition's
    // cell is its registration, set once.
    #[allow(clippy::mutable_key_type)]
    pub fn bake_models(&self, materials: &MaterialBaker<'_>) -> BakingResult {
        let interner = InternerImpl::default();
        let missing_models = MissingModels::bake(&*self.missing_model, materials, &interner);
        let baker = ModelBakerImpl {
            bakery: self,
            materials,
            interner: &interner,
            missing_models: &missing_models,
            operation_cache: RefCell::default(),
        };
        let block_state_models = self
            .unbaked_block_state_models
            .iter()
            .map(|(block_state, model)| (*block_state, model.bake(*block_state, &baker)))
            .collect();
        BakingResult {
            missing_models,
            block_state_models,
        }
    }
}

impl BakingResult {
    pub fn get_block_state_model(&self, block_state: BlockState) -> &Arc<dyn BlockStateModel> {
        self.block_state_models
            .get(&block_state)
            .unwrap_or(&self.missing_models.block)
    }
}

impl MissingModels {
    /// Java throws `IllegalStateException` when the missing model's quads aren't the block
    /// atlas's, or the missing model asks for another.
    pub fn bake(
        unbaked: &dyn ResolvedModel,
        materials: &MaterialBaker<'_>,
        interner: &dyn Interner,
    ) -> MissingModels {
        let missing_model_bakery = MissingModelBaker {
            materials,
            interner,
        };
        let texture_slots = unbaked.get_top_texture_slots();
        let has_ambient_occlusion = unbaked.get_top_ambient_occlusion();
        let geometry = unbaked.bake_top_geometry(
            &texture_slots,
            &missing_model_bakery,
            BlockModelRotation::identity(),
        );
        if let Some(forbidden_sprites) = SimpleModelWrapper::find_non_block_sprites(&geometry) {
            panic!(
                "IllegalStateException: Missing block contains sprites from outside of block atlas: {forbidden_sprites:?}"
            );
        }

        let particle_material =
            unbaked.resolve_particle_material(&texture_slots, &missing_model_bakery);
        let missing_model_part: Arc<dyn BlockStateModelPart> = Arc::new(SimpleModelWrapper {
            quads: geometry,
            use_ambient_occlusion: has_ambient_occlusion,
            particle_material,
        });
        let baked_block_model: Arc<dyn BlockStateModel> =
            Arc::new(SingleVariant::new(missing_model_part.clone()));
        MissingModels {
            block_part: missing_model_part,
            block: baked_block_model,
        }
    }
}

// `MissingModels.bake`'s anonymous baker.
struct MissingModelBaker<'a> {
    materials: &'a MaterialBaker<'a>,
    interner: &'a dyn Interner,
}

impl ModelBaker for MissingModelBaker<'_> {
    fn get_model(&self, location: &Identifier) -> Arc<dyn ResolvedModel> {
        panic!(
            "IllegalStateException: Missing model can't have dependencies, but asked for {location}"
        )
    }

    fn missing_block_model_part(&self) -> Arc<dyn BlockStateModelPart> {
        panic!("IllegalStateException")
    }

    fn materials(&self) -> &MaterialBaker<'_> {
        self.materials
    }

    fn interner(&self) -> &dyn Interner {
        self.interner
    }

    fn compute(&self, _key: usize, operation: &dyn Fn(&dyn ModelBaker) -> Shared) -> Shared {
        operation(self)
    }
}

struct ModelBakerImpl<'a> {
    bakery: &'a ModelBakery,
    materials: &'a MaterialBaker<'a>,
    interner: &'a InternerImpl,
    missing_models: &'a MissingModels,
    operation_cache: RefCell<HashMap<usize, Shared>>,
}

impl ModelBaker for ModelBakerImpl<'_> {
    fn get_model(&self, location: &Identifier) -> Arc<dyn ResolvedModel> {
        match self.bakery.resolved_models.get(location) {
            Some(result) => result.clone(),
            None => {
                eprintln!("Requested a model that was not discovered previously: {location}");
                self.bakery.missing_model.clone()
            }
        }
    }

    fn missing_block_model_part(&self) -> Arc<dyn BlockStateModelPart> {
        self.missing_models.block_part.clone()
    }

    fn materials(&self) -> &MaterialBaker<'_> {
        self.materials
    }

    fn interner(&self) -> &dyn Interner {
        self.interner
    }

    fn compute(&self, key: usize, operation: &dyn Fn(&dyn ModelBaker) -> Shared) -> Shared {
        if let Some(result) = self.operation_cache.borrow().get(&key) {
            return result.clone();
        }
        let result = operation(self);
        self.operation_cache
            .borrow_mut()
            .entry(key)
            .or_insert(result)
            .clone()
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use md5::{Digest, Md5};
    use serde_json::{Value, json};

    use super::{BakingResult, ModelBakery};
    use crate::golden::Golden;
    use crate::net::minecraft::client::renderer::block::dispatch::block_state_model::BlockStateModel;
    use crate::net::minecraft::client::renderer::block::dispatch::block_state_model_part::BlockStateModelPart;
    use crate::net::minecraft::client::renderer::block::dispatch::multipart::multi_part_model::MultiPartModel;
    use crate::net::minecraft::client::renderer::block::dispatch::single_variant::SingleVariant;
    use crate::net::minecraft::client::renderer::block::dispatch::weighted_variants::WeightedVariants;
    use crate::net::minecraft::client::renderer::texture::sprite_loader::{
        Preparations, SpriteLoader,
    };
    use crate::net::minecraft::client::renderer::texture::texture_atlas::TextureAtlas;
    use crate::net::minecraft::client::resources::model::block_state_model_loader::BlockStateModelLoader;
    use crate::net::minecraft::client::resources::model::cuboid::cuboid_model::CuboidModel;
    use crate::net::minecraft::client::resources::model::cuboid::missing_cuboid_model::MissingCuboidModel;
    use crate::net::minecraft::client::resources::model::geometry::baked_quad::BakedQuad;
    use crate::net::minecraft::client::resources::model::model_discovery::ModelDiscovery;
    use crate::net::minecraft::client::resources::model::sprite::material::Baked;
    use crate::net::minecraft::client::resources::model::sprite::material_baker::MaterialBaker;
    use crate::net::minecraft::client::resources::model::unbaked_model::UnbakedModel;
    use crate::net::minecraft::core::direction::Direction;
    use crate::net::minecraft::resources::file_to_id_converter::FileToIdConverter;
    use crate::net::minecraft::resources::identifier::Identifier;
    use crate::net::minecraft::server::packs::resources::multi_pack_resource_manager::tests::vanilla_resources;
    use crate::net::minecraft::server::packs::resources::resource_manager::ResourceManager;
    use crate::net::minecraft::world::level::block::block::Block;

    // `ModelSuites`' options: llvmpipe's texture size, the oracle client's mipmap levels.
    const MAX_TEXTURE_SIZE: i32 = 16384;
    const MAX_MIPMAP_LEVELS: i32 = 4;

    /// The block and item atlases' layouts, which a quad's UVs come from. Mip levels don't
    /// change what models read.
    pub(crate) fn atlases(manager: &dyn ResourceManager) -> (Preparations, Preparations) {
        let stitch = |location: &str, definition: &str, max_mipmap_levels: i32| {
            SpriteLoader::new(
                Identifier::with_default_namespace(location),
                MAX_TEXTURE_SIZE,
            )
            .stitch_layout(
                SpriteLoader::load_sprites(
                    manager,
                    &Identifier::with_default_namespace(definition),
                ),
                max_mipmap_levels,
                0,
            )
            .expect("the atlas fits")
        };
        (
            stitch(TextureAtlas::LOCATION_BLOCKS, "blocks", MAX_MIPMAP_LEVELS),
            stitch(TextureAtlas::LOCATION_ITEMS, "items", 0),
        )
    }

    /// `ModelManager`'s reload as far as block states go: every model file, the block states'
    /// definitions, the models they need found, and the bake.
    pub(crate) fn bake_block_states(
        manager: &dyn ResourceManager,
        materials: &MaterialBaker<'_>,
    ) -> BakingResult {
        let lister = FileToIdConverter::json("models");
        let mut models: HashMap<Identifier, Arc<dyn UnbakedModel>> = HashMap::new();
        for (file, resource) in lister.list_matching_resources(manager) {
            let parsed = resource
                .read_all_as_string()
                .map_err(|error| error.to_string())
                .and_then(|text| CuboidModel::from_stream(&text));
            match parsed {
                Ok(model) => {
                    models.insert(lister.file_to_id(&file), Arc::new(model));
                }
                Err(error) => eprintln!("Failed to load model {file}: {error}"),
            }
        }
        let block_states = BlockStateModelLoader::load_block_states(manager);
        let mut discovery =
            ModelDiscovery::new(models, Arc::new(MissingCuboidModel::missing_model()));
        for root in block_states.models.values() {
            discovery.add_root(&**root);
        }
        let resolved = discovery.resolve();
        ModelBakery::new(block_states.models, resolved, discovery.missing_model())
            .bake_models(materials)
    }

    fn float(value: f32) -> Value {
        Value::from(format!("{:08x}", value.to_bits()))
    }

    fn material(material: &Baked) -> Value {
        json!([
            material.sprite.atlas_location().to_string(),
            material.sprite.contents().name().to_string(),
            material.force_translucent
        ])
    }

    fn quads(quads: &[BakedQuad]) -> Value {
        quads
            .iter()
            .map(|quad| {
                let info = &quad.material_info;
                json!([
                    quad.positions
                        .iter()
                        .map(|position| json!([
                            float(position.x),
                            float(position.y),
                            float(position.z)
                        ]))
                        .collect::<Vec<_>>(),
                    quad.packed_uvs.to_vec(),
                    format!("{:?}", quad.direction),
                    [
                        Value::from(info.sprite.atlas_location().to_string()),
                        Value::from(info.sprite.contents().name().to_string()),
                        Value::from(format!("{:?}", info.layer)),
                        Value::from(info.tint_index),
                        info.shade_direction_override
                            .map_or(Value::Null, |direction| Value::from(format!(
                                "{direction:?}"
                            ))),
                        Value::from(info.light_emission)
                    ]
                ])
            })
            .collect()
    }

    fn part(part: &dyn BlockStateModelPart) -> Value {
        let mut all = vec![quads(part.get_quads(None))];
        all.extend(Direction::VALUES.map(|direction| quads(part.get_quads(Some(direction)))));
        json!([
            part.use_ambient_occlusion(),
            material(part.particle_material()),
            part.material_flags(),
            all
        ])
    }

    /// `ModelSuites`' encoding of a block state's model.
    pub(crate) fn model(model: &dyn BlockStateModel) -> Value {
        let any = model.as_any();
        let (kind, contents) = if let Some(single) = any.downcast_ref::<SingleVariant>() {
            ("single", part(&**single.model()))
        } else if let Some(weighted) = any.downcast_ref::<WeightedVariants>() {
            let entries = weighted
                .list()
                .unwrap()
                .iter()
                .map(|entry| json!([entry.weight(), self::model(&**entry.value())]))
                .collect();
            ("weighted", Value::Array(entries))
        } else if let Some(multi_part) = any.downcast_ref::<MultiPartModel>() {
            let selected = multi_part
                .select_models()
                .iter()
                .map(|selected| self::model(&**selected))
                .collect();
            ("multipart", Value::Array(selected))
        } else {
            panic!("no encoding for this model");
        };
        json!([
            kind,
            material(model.particle_material()),
            model.material_flags(),
            contents
        ])
    }

    fn md5(text: &str) -> String {
        let digest: [u8; 16] = Md5::digest(text.as_bytes()).into();
        digest.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    /// Every block state's baked model against vanilla's bake over the same pack (`ModelSuites`).
    #[test]
    fn bakes_every_block_state_like_vanilla() {
        let golden = Golden::load("net.minecraft.client.resources.model.ModelBakery");
        let manager = vanilla_resources();
        let (blocks, items) = atlases(&manager);
        let materials = MaterialBaker::new(&blocks, &items);
        let result = bake_block_states(&manager, &materials);
        golden.check("missingModels.block", || {
            model(&*result.missing_models.block)
        });
        golden.check("bake", |id: i32| {
            let state = Block::state_by_id(id);
            md5(
                &serde_json::to_string(&model(&**result.get_block_state_model(state)))
                    .expect("JSON"),
            )
        });
        golden.check("bake[full]", |id: i32| {
            model(&**result.get_block_state_model(Block::state_by_id(id)))
        });
        golden.finish();
    }
}
