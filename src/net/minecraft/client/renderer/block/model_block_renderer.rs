//! net/minecraft/client/renderer/block/ModelBlockRenderer.java
//!
//! A block's model meshed into quads: flat or with ambient occlusion, each face culled against
//! its neighbour, tinted by the block's tint sources. The random source is the model's seed for
//! the block (`SingleThreadedRandomSource`, Java's `createThreadLocalInstance`).

use std::sync::Arc;

use crate::com::mojang::blaze3d::vertex::quad_instance::QuadInstance;
use crate::net::minecraft::client::color::block::block_colors::BlockColors;
use crate::net::minecraft::client::color::block::block_tint_source::BlockTintSource;
use crate::net::minecraft::client::renderer::block::block_and_tint_getter::BlockAndTintGetter;
use crate::net::minecraft::client::renderer::block::block_model_lighter::{
    BlockModelLighter, CHECK_LIGHT,
};
use crate::net::minecraft::client::renderer::block::block_quad_output::BlockQuadOutput;
use crate::net::minecraft::client::renderer::block::dispatch::block_state_model::BlockStateModel;
use crate::net::minecraft::client::renderer::block::dispatch::block_state_model_part::BlockStateModelPart;
use crate::net::minecraft::client::resources::model::geometry::baked_quad::BakedQuad;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::util::random_source::{self, RandomSource};
use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;

pub struct ModelBlockRenderer<'a> {
    lighter: BlockModelLighter,
    ambient_occlusion: bool,
    cull: bool,
    block_colors: &'a BlockColors,
    random: Box<dyn RandomSource>,
    parts: Vec<Arc<dyn BlockStateModelPart>>,
    quad_instance: QuadInstance,
    tint_cache_index: i32,
    tint_cache_value: i32,
    tint_sources_initialized: bool,
    tint_sources: Vec<Option<Arc<dyn BlockTintSource>>>,
    computed_tint_values: Vec<i32>,
}

impl<'a> ModelBlockRenderer<'a> {
    pub fn new(
        ambient_occlusion: bool,
        cull: bool,
        block_colors: &'a BlockColors,
    ) -> ModelBlockRenderer<'a> {
        ModelBlockRenderer {
            lighter: BlockModelLighter::new(),
            ambient_occlusion,
            cull,
            block_colors,
            random: random_source::create_thread_local_instance_with_seed(0),
            parts: Vec::new(),
            quad_instance: QuadInstance::new(),
            tint_cache_index: -1,
            tint_cache_value: 0,
            tint_sources_initialized: false,
            tint_sources: Vec::new(),
            computed_tint_values: Vec::new(),
        }
    }

    /// Whether leaves are drawn solid, for fast leaves.
    pub fn force_opaque(cutout_leaves: bool, block_state: BlockState) -> bool {
        !cutout_leaves && block_state.class_is_leaves()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn tesselate_block(
        &mut self,
        output: &mut dyn BlockQuadOutput,
        x: f32,
        y: f32,
        z: f32,
        level: &dyn BlockAndTintGetter,
        pos: &BlockPos,
        block_state: BlockState,
        model: &dyn BlockStateModel,
        seed: i64,
    ) {
        self.random.set_seed(seed);
        let mut parts = std::mem::take(&mut self.parts);
        model.collect_parts(&mut *self.random, &mut parts);
        if !parts.is_empty() {
            let offset = block_state.get_offset(pos);
            let (x, y, z) = (
                x + offset.x as f32,
                y + offset.y as f32,
                z + offset.z as f32,
            );
            if self.ambient_occlusion
                && block_state.get_light_emission() == 0
                && parts[0].use_ambient_occlusion()
            {
                self.tesselate_ambient_occlusion(output, x, y, z, &parts, level, block_state, pos);
            } else {
                self.tesselate_flat(output, x, y, z, &parts, level, block_state, pos);
            }
            parts.clear();
            self.reset_tint_cache();
        }
        self.parts = parts;
    }

    fn configure_tint_cache(&mut self, block_state: BlockState) {
        let tint_sources = self.block_colors.get_tint_sources(block_state);
        if !tint_sources.is_empty() {
            self.tint_sources
                .extend(tint_sources.iter().cloned().map(Some));
            self.computed_tint_values
                .extend(std::iter::repeat_n(-1, tint_sources.len()));
        }
    }

    fn reset_tint_cache(&mut self) {
        self.tint_cache_index = -1;
        if self.tint_sources_initialized {
            self.tint_sources.clear();
            self.computed_tint_values.clear();
            self.tint_sources_initialized = false;
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn tesselate_ambient_occlusion(
        &mut self,
        output: &mut dyn BlockQuadOutput,
        x: f32,
        y: f32,
        z: f32,
        parts: &[Arc<dyn BlockStateModelPart>],
        level: &dyn BlockAndTintGetter,
        state: BlockState,
        pos: &BlockPos,
    ) {
        let mut cache_valid = 0;
        let mut should_render_face_cache = 0;
        for part in parts {
            for direction in Direction::VALUES {
                let cache_mask = 1 << direction as u32;
                let valid_cache_for_direction = (cache_valid & cache_mask) != 0;
                let mut should_render_face = (should_render_face_cache & cache_mask) != 0;
                if !valid_cache_for_direction || should_render_face {
                    let culled_quads = part.get_quads(Some(direction));
                    if !culled_quads.is_empty() {
                        if !valid_cache_for_direction {
                            should_render_face = self.should_render_face(
                                level,
                                state,
                                direction,
                                &pos.relative(direction),
                            );
                            cache_valid |= cache_mask;
                            if should_render_face {
                                should_render_face_cache |= cache_mask;
                            }
                        }

                        if should_render_face {
                            for quad in culled_quads {
                                self.lighter.prepare_quad_ambient_occlusion(
                                    level,
                                    state,
                                    pos,
                                    quad,
                                    &mut self.quad_instance,
                                );
                                self.put_quad_with_tint(output, x, y, z, level, state, pos, quad);
                            }
                        }
                    }
                }
            }

            for quad in part.get_quads(None) {
                self.lighter.prepare_quad_ambient_occlusion(
                    level,
                    state,
                    pos,
                    quad,
                    &mut self.quad_instance,
                );
                self.put_quad_with_tint(output, x, y, z, level, state, pos, quad);
            }
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn tesselate_flat(
        &mut self,
        output: &mut dyn BlockQuadOutput,
        x: f32,
        y: f32,
        z: f32,
        parts: &[Arc<dyn BlockStateModelPart>],
        level: &dyn BlockAndTintGetter,
        state: BlockState,
        pos: &BlockPos,
    ) {
        let mut cache_valid = 0;
        let mut should_render_face_cache = 0;
        for part in parts {
            for direction in Direction::VALUES {
                let cache_mask = 1 << direction as u32;
                let valid_cache_for_direction = (cache_valid & cache_mask) != 0;
                let mut should_render_face = (should_render_face_cache & cache_mask) != 0;
                if !valid_cache_for_direction || should_render_face {
                    let culled_quads = part.get_quads(Some(direction));
                    if !culled_quads.is_empty() {
                        let relative_pos = pos.relative(direction);
                        if !valid_cache_for_direction {
                            should_render_face =
                                self.should_render_face(level, state, direction, &relative_pos);
                            cache_valid |= cache_mask;
                            if should_render_face {
                                should_render_face_cache |= cache_mask;
                            }
                        }

                        if should_render_face {
                            let light_coords =
                                self.lighter.get_light_coords(state, level, &relative_pos);
                            for quad in culled_quads {
                                self.lighter.prepare_quad_flat(
                                    level,
                                    state,
                                    pos,
                                    light_coords,
                                    quad,
                                    &mut self.quad_instance,
                                );
                                self.put_quad_with_tint(output, x, y, z, level, state, pos, quad);
                            }
                        }
                    }
                }
            }

            for quad in part.get_quads(None) {
                self.lighter.prepare_quad_flat(
                    level,
                    state,
                    pos,
                    CHECK_LIGHT,
                    quad,
                    &mut self.quad_instance,
                );
                self.put_quad_with_tint(output, x, y, z, level, state, pos, quad);
            }
        }
    }

    fn should_render_face(
        &self,
        level: &dyn BlockAndTintGetter,
        state: BlockState,
        direction: Direction,
        neighbor_pos: &BlockPos,
    ) -> bool {
        if !self.cull {
            return true;
        }

        let neighbor_state = level.get_block_state(neighbor_pos);
        Block::should_render_face(state, neighbor_state, direction)
    }

    #[allow(clippy::too_many_arguments)]
    fn put_quad_with_tint(
        &mut self,
        output: &mut dyn BlockQuadOutput,
        x: f32,
        y: f32,
        z: f32,
        level: &dyn BlockAndTintGetter,
        state: BlockState,
        pos: &BlockPos,
        quad: &BakedQuad,
    ) {
        let tint_index = quad.material_info.tint_index;
        if tint_index != -1 {
            let tint_color = self.get_tint_color(level, state, pos, tint_index);
            self.quad_instance.multiply_color(tint_color);
        }

        output.put(x, y, z, quad, &self.quad_instance);
    }

    fn get_tint_color(
        &mut self,
        level: &dyn BlockAndTintGetter,
        state: BlockState,
        pos: &BlockPos,
        tint_index: i32,
    ) -> i32 {
        if self.tint_cache_index == tint_index {
            return self.tint_cache_value;
        }

        let tint_color = self.compute_tint_color(level, state, pos, tint_index);
        self.tint_cache_index = tint_index;
        self.tint_cache_value = tint_color;
        tint_color
    }

    fn compute_tint_color(
        &mut self,
        level: &dyn BlockAndTintGetter,
        state: BlockState,
        pos: &BlockPos,
        tint_index: i32,
    ) -> i32 {
        if !self.tint_sources_initialized {
            self.configure_tint_cache(state);
            self.tint_sources_initialized = true;
        }

        // A negative index other than -1 throws in Java (`List.set`), like the index here.
        let index = tint_index as usize;
        if tint_index >= self.tint_sources.len() as i32 {
            return -1;
        }
        match self.tint_sources[index].take() {
            Some(tint_source) => {
                let computed_tint_value = tint_source.color_in_world(state, level, pos);
                self.computed_tint_values[index] = computed_tint_value;
                computed_tint_value
            }
            None => self.computed_tint_values[index],
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use md5::{Digest, Md5};
    use serde_json::{Value, json};

    use super::ModelBlockRenderer;
    use crate::com::mojang::blaze3d::vertex::buffer_builder::BufferBuilder;
    use crate::com::mojang::blaze3d::vertex::byte_buffer_builder::ByteBufferBuilder;
    use crate::com::mojang::blaze3d::vertex::quad_instance::QuadInstance;
    use crate::com::mojang::blaze3d::vertex::vertex_consumer::VertexConsumer;
    use crate::com::mojang::renderpearl::api::pipeline::primitive_topology::PrimitiveTopology;
    use crate::golden::Golden;
    use crate::net::minecraft::client::color::block::block_colors::BlockColors;
    use crate::net::minecraft::client::renderer::block::block_and_tint_getter::BlockAndTintGetter;
    use crate::net::minecraft::client::renderer::block::block_quad_output::BlockQuadOutput;
    use crate::net::minecraft::client::renderer::chunk::chunk_section_layer::ChunkSectionLayer;
    use crate::net::minecraft::client::resources::model::geometry::baked_quad::BakedQuad;
    use crate::net::minecraft::client::resources::model::model_bakery::BakingResult;
    use crate::net::minecraft::client::resources::model::model_bakery::tests::{
        atlases, bake_block_states,
    };
    use crate::net::minecraft::client::resources::model::sprite::material_baker::MaterialBaker;
    use crate::net::minecraft::core::block_pos::BlockPos;
    use crate::net::minecraft::resources::identifier::Identifier;
    use crate::net::minecraft::server::packs::resources::multi_pack_resource_manager::tests::vanilla_resources;
    use crate::net::minecraft::world::level::block::block::Block;
    use crate::net::minecraft::world::level::block::blocks::{self, Blocks};
    use crate::net::minecraft::world::level::block::state::block_behaviour::RenderShape;
    use crate::net::minecraft::world::level::block::state::block_state::BlockState;
    use crate::net::minecraft::world::level::block_and_light_getter::BlockAndLightGetter;
    use crate::net::minecraft::world::level::block_getter::BlockGetter;
    use crate::net::minecraft::world::level::cardinal_lighting::CardinalLighting;
    use crate::net::minecraft::world::level::color_resolver::ColorResolver;
    use crate::net::minecraft::world::level::light_layer::LightLayer;
    use crate::net::minecraft::world::level::lighting::level_light_engine::LevelLightEngine;
    use crate::net::minecraft::world::level::material::fluid_state::FluidState;

    /// `ModelSuites.MeshLevel`: a pattern of whole blocks, light that varies, white tints.
    pub(crate) struct MeshLevel {
        seed: i32,
        blocks: Vec<Block>,
        // `LevelLightEngine.EMPTY`, like the suite's level: its light is `get_brightness`'s own.
        light_engine: LevelLightEngine,
    }

    const BLOCK_NAMES: [&str; 24] = [
        "stone",
        "dirt",
        "grass_block",
        "bedrock",
        "cobblestone",
        "oak_planks",
        "oak_log",
        "glass",
        "white_stained_glass",
        "tinted_glass",
        "ice",
        "oak_leaves",
        "glowstone",
        "magma_block",
        "crafting_table",
        "furnace",
        "bookshelf",
        "barrier",
        "quartz_pillar",
        "snow_block",
        "slime_block",
        "beacon",
        "vault",
        "mangrove_roots",
    ];

    impl MeshLevel {
        pub(crate) fn new(seed: i32) -> MeshLevel {
            let blocks = BLOCK_NAMES
                .iter()
                .map(|name| {
                    blocks::registry()
                        .get_value(&Identifier::with_default_namespace(name))
                        .expect("a vanilla block")
                })
                .collect();
            MeshLevel {
                seed,
                blocks,
                light_engine: LevelLightEngine::empty(),
            }
        }

        fn hash(x: i32, y: i32, z: i32, seed: i32) -> i32 {
            let mut h = x.wrapping_mul(73856093)
                ^ y.wrapping_mul(19349663)
                ^ z.wrapping_mul(83492791)
                ^ seed.wrapping_mul(2654435);
            h ^= ((h as u32) >> 13) as i32;
            h = h.wrapping_mul(0x5bd1e995);
            h ^= ((h as u32) >> 15) as i32;
            h & i32::MAX
        }
    }

    impl BlockGetter for MeshLevel {
        fn get_block_state(&self, pos: &BlockPos) -> BlockState {
            let h = MeshLevel::hash(pos.get_x(), pos.get_y(), pos.get_z(), self.seed);
            if h % 5 < 2 {
                return Blocks::AIR.default_block_state();
            }
            let block = self.blocks[(h / 5) as usize % self.blocks.len()];
            let states: Vec<BlockState> =
                block.get_state_definition().get_possible_states().collect();
            states[(h / 5 / self.blocks.len() as i32) as usize % states.len()]
        }

        fn get_fluid_state(&self, pos: &BlockPos) -> FluidState {
            self.get_block_state(pos).get_fluid_state()
        }
    }

    impl BlockAndLightGetter for MeshLevel {
        fn get_light_engine(&self) -> &LevelLightEngine {
            &self.light_engine
        }

        fn get_brightness(&self, layer: LightLayer, pos: &BlockPos) -> i32 {
            let (x, y, z) = (pos.get_x(), pos.get_y(), pos.get_z());
            match layer {
                LightLayer::SKY => (3 * x + 5 * y + 7 * z + self.seed).rem_euclid(16),
                LightLayer::BLOCK => (7 * x + 11 * y + 13 * z).rem_euclid(16),
            }
        }
    }

    impl BlockAndTintGetter for MeshLevel {
        fn cardinal_lighting(&self) -> CardinalLighting {
            CardinalLighting::DEFAULT
        }

        /// What `BlockAndTintGetter.EMPTY` answers: the suite meshes with an empty `BlockColors`,
        /// so nothing asks.
        fn get_block_tint(&self, _pos: &BlockPos, _color: &'static ColorResolver) -> i32 {
            -1
        }
    }

    /// `SectionCompiler.compile`'s quad output: a buffer per layer, started when its first quad
    /// comes, in the layers' order.
    struct Layers<'a> {
        builders: Vec<(ChunkSectionLayer, BufferBuilder<'a>)>,
        // Each layer's buffer, by the layer's ordinal, until its builder takes it.
        buffers: Vec<Option<&'a mut ByteBufferBuilder>>,
    }

    impl BlockQuadOutput for Layers<'_> {
        fn put(&mut self, x: f32, y: f32, z: f32, quad: &BakedQuad, instance: &QuadInstance) {
            let layer = quad.material_info.layer;
            if !self.builders.iter().any(|(started, _)| *started == layer) {
                let buffer = self.buffers[layer as usize]
                    .take()
                    .expect("each layer starts once");
                self.builders.push((
                    layer,
                    BufferBuilder::new(buffer, PrimitiveTopology::QUADS, layer.vertex_format()),
                ));
            }
            let builder = &mut self
                .builders
                .iter_mut()
                .find(|(started, _)| *started == layer)
                .expect("started above")
                .1;
            builder.put_block_baked_quad(x, y, z, quad, instance);
        }
    }

    fn compile(level: &MeshLevel, models: &BakingResult, ambient_occlusion: bool) -> Value {
        let block_colors = BlockColors::new();
        let mut renderer = ModelBlockRenderer::new(ambient_occlusion, true, &block_colors);
        let mut buffers: Vec<ByteBufferBuilder> = ChunkSectionLayer::VALUES
            .iter()
            .map(|layer| ByteBufferBuilder::new(layer.buffer_size()))
            .collect();
        let mut output = Layers {
            builders: Vec::new(),
            buffers: buffers.iter_mut().map(Some).collect(),
        };
        for pos in BlockPos::between_closed(&BlockPos::new(0, 0, 0), &BlockPos::new(15, 15, 15)) {
            let state = level.get_block_state(&pos);
            if !state.is_air() && state.get_render_shape() == RenderShape::MODEL {
                renderer.tesselate_block(
                    &mut output,
                    (pos.get_x() & 15) as f32,
                    (pos.get_y() & 15) as f32,
                    (pos.get_z() & 15) as f32,
                    level,
                    &pos,
                    state,
                    &**models.get_block_state_model(state),
                    state.get_seed(&pos),
                );
            }
        }
        let mut builders = output.builders;
        builders.sort_by_key(|(layer, _)| *layer);
        let layers: Vec<Value> = builders
            .iter_mut()
            .filter_map(|(layer, builder)| {
                let mesh = builder.build()?;
                let digest: [u8; 16] = Md5::digest(mesh.vertex_buffer()).into();
                let md5: String = digest.iter().map(|byte| format!("{byte:02x}")).collect();
                Some(json!([
                    format!("{layer:?}"),
                    mesh.draw_state().vertex_count,
                    md5
                ]))
            })
            .collect();
        Value::Array(layers)
    }

    /// Sections of whole blocks meshed like vanilla's `ModelBlockRenderer` meshes them, flat and
    /// with ambient occlusion, vertex buffer by vertex buffer (`ModelSuites`' `MeshSuite`).
    #[test]
    fn meshes_sections_like_vanilla() {
        let golden = Golden::load("net.minecraft.client.renderer.block.ModelBlockRenderer");
        let manager = vanilla_resources();
        let (blocks, items) = atlases(&manager);
        let materials = MaterialBaker::new(&blocks, &items);
        let models = bake_block_states(&manager, &materials);
        golden.check("compileSection", |seed: i32, ambient_occlusion: bool| {
            compile(&MeshLevel::new(seed), &models, ambient_occlusion)
        });
        golden.finish();
    }
}
