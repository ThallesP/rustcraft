//! net/minecraft/client/renderer/chunk/SectionCompiler.java
//!
//! A section's blocks and fluids meshed into a buffer per layer, the translucent one's quads sorted
//! from the camera, and which of its faces see each other. Not ported yet: block entities, which
//! the client's chunks don't hold yet, so `Results` has none. `BlockModelLighter`'s cache, which `compile` turns on and clears in
//! Java, isn't ported (PORTING.md, "Meshing"), and a crash report's details are the panic's.

use std::sync::Arc;

use crate::com::mojang::blaze3d::vertex::buffer_builder::BufferBuilder;
use crate::com::mojang::blaze3d::vertex::byte_buffer_builder::ByteBufferBuilder;
use crate::com::mojang::blaze3d::vertex::mesh_data::{MeshData, SortState};
use crate::com::mojang::blaze3d::vertex::quad_instance::QuadInstance;
use crate::com::mojang::blaze3d::vertex::vertex_consumer::VertexConsumer;
use crate::com::mojang::blaze3d::vertex::vertex_sorting::VertexSorting;
use crate::com::mojang::renderpearl::api::pipeline::primitive_topology::PrimitiveTopology;
use crate::net::minecraft::client::color::block::block_colors::BlockColors;
use crate::net::minecraft::client::renderer::block::block_and_tint_getter::BlockAndTintGetter;
use crate::net::minecraft::client::renderer::block::block_quad_output::BlockQuadOutput;
use crate::net::minecraft::client::renderer::block::block_state_model_set::BlockStateModelSet;
use crate::net::minecraft::client::renderer::block::fluid_renderer::{self, FluidRenderer};
use crate::net::minecraft::client::renderer::block::fluid_state_model_set::FluidStateModelSet;
use crate::net::minecraft::client::renderer::block::model_block_renderer::ModelBlockRenderer;
use crate::net::minecraft::client::renderer::chunk::chunk_section_layer::ChunkSectionLayer;
use crate::net::minecraft::client::renderer::chunk::render_section_region::RenderSectionRegion;
use crate::net::minecraft::client::renderer::chunk::vis_graph::VisGraph;
use crate::net::minecraft::client::renderer::chunk::visibility_set::VisibilitySet;
use crate::net::minecraft::client::renderer::section_buffer_builder_pack::SectionBufferBuilderPack;
use crate::net::minecraft::client::resources::model::geometry::baked_quad::BakedQuad;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::section_pos::SectionPos;
use crate::net::minecraft::world::level::block::state::block_behaviour::RenderShape;
use crate::net::minecraft::world::level::block_getter::BlockGetter;

pub struct SectionCompiler {
    ambient_occlusion: bool,
    cutout_leaves: bool,
    block_model_set: Arc<BlockStateModelSet>,
    fluid_model_set: Arc<FluidStateModelSet>,
    block_colors: Arc<BlockColors>,
}

/// `SectionCompiler.Results`.
#[derive(Default)]
pub struct Results {
    /// Each layer that has quads, in `ChunkSectionLayer`'s order (Java's `EnumMap`).
    pub rendered_layers: Vec<(ChunkSectionLayer, MeshData)>,
    pub visibility_set: VisibilitySet,
    pub transparency_state: Option<SortState>,
}

impl SectionCompiler {
    pub fn new(
        ambient_occlusion: bool,
        cutout_leaves: bool,
        block_model_set: Arc<BlockStateModelSet>,
        fluid_model_set: Arc<FluidStateModelSet>,
        block_colors: Arc<BlockColors>,
    ) -> SectionCompiler {
        SectionCompiler {
            ambient_occlusion,
            cutout_leaves,
            block_model_set,
            fluid_model_set,
            block_colors,
        }
    }

    /// The section's meshes over a region, whose light and tints are the level's copied or a
    /// test level's.
    pub fn compile<L>(
        &self,
        section_pos: SectionPos,
        region: &RenderSectionRegion<L>,
        vertex_sorting: &VertexSorting,
        builders: &mut SectionBufferBuilderPack,
    ) -> Results
    where
        RenderSectionRegion<L>: BlockAndTintGetter,
    {
        let mut results = Results::default();
        let min_pos = BlockPos::new(
            section_pos.min_block_x(),
            section_pos.min_block_y(),
            section_pos.min_block_z(),
        );
        let max_pos = min_pos.offset_xyz(15, 15, 15);
        let mut vis_graph = VisGraph::new();
        let mut block_renderer =
            ModelBlockRenderer::new(self.ambient_occlusion, true, &self.block_colors);
        let fluid_renderer = FluidRenderer::new(&self.fluid_model_set);
        let mut started_layers = StartedLayers {
            builders: Vec::new(),
            buffers: builders.buffers_mut().map(Some),
        };

        for pos in BlockPos::between_closed(&min_pos, &max_pos) {
            let block_state = region.get_block_state(&pos);
            if block_state.is_air() {
                continue;
            }

            if block_state.is_solid_render() {
                vis_graph.set_opaque(&pos);
            }

            let fluid_state = block_state.get_fluid_state();
            if !fluid_state.is_empty() {
                fluid_renderer.tesselate(
                    region,
                    &pos,
                    &mut started_layers,
                    block_state,
                    fluid_state,
                );
            }

            if block_state.get_render_shape() == RenderShape::MODEL {
                let mut opaque_quad_output;
                let output: &mut dyn BlockQuadOutput =
                    if ModelBlockRenderer::force_opaque(self.cutout_leaves, block_state) {
                        opaque_quad_output = OpaqueQuadOutput(&mut started_layers);
                        &mut opaque_quad_output
                    } else {
                        &mut started_layers
                    };
                block_renderer.tesselate_block(
                    output,
                    SectionPos::section_relative(pos.get_x()) as f32,
                    SectionPos::section_relative(pos.get_y()) as f32,
                    SectionPos::section_relative(pos.get_z()) as f32,
                    region,
                    &pos,
                    block_state,
                    &**self.block_model_set.get(block_state),
                    block_state.get_seed(&pos),
                );
            }
        }

        // The builders borrow the pack's buffers, which sorting builds into again.
        let mut started = started_layers.builders;
        started.sort_by_key(|(layer, _)| *layer);
        let meshes: Vec<(ChunkSectionLayer, Option<MeshData>)> = started
            .into_iter()
            .map(|(layer, mut builder)| (layer, builder.build()))
            .collect();
        for (layer, mesh) in meshes {
            if let Some(mut mesh) = mesh {
                if layer == ChunkSectionLayer::TRANSLUCENT {
                    results.transparency_state =
                        mesh.sort_quads(builders.buffer(layer), vertex_sorting);
                }

                results.rendered_layers.push((layer, mesh));
            }
        }

        results.visibility_set = vis_graph.resolve();
        results
    }
}

/// `getOrBeginLayer` over the started layers: a builder per layer, begun on the layer's buffer
/// when its first quad comes.
struct StartedLayers<'a> {
    builders: Vec<(ChunkSectionLayer, BufferBuilder<'a>)>,
    // Each layer's buffer, by the layer's ordinal, until its builder takes it.
    buffers: [Option<&'a mut ByteBufferBuilder>; 3],
}

impl<'a> StartedLayers<'a> {
    fn get_or_begin_layer(&mut self, layer: ChunkSectionLayer) -> &mut BufferBuilder<'a> {
        let position = match self
            .builders
            .iter()
            .position(|(started, _)| *started == layer)
        {
            Some(position) => position,
            None => {
                let buffer = self.buffers[layer as usize]
                    .take()
                    .expect("each layer begins once");
                self.builders.push((
                    layer,
                    BufferBuilder::new(buffer, PrimitiveTopology::QUADS, layer.vertex_format()),
                ));
                self.builders.len() - 1
            }
        };
        &mut self.builders[position].1
    }
}

/// `fluidOutput`: a fluid's layer's builder.
impl fluid_renderer::Output for StartedLayers<'_> {
    fn get_builder(&mut self, layer: ChunkSectionLayer) -> &mut dyn VertexConsumer {
        self.get_or_begin_layer(layer)
    }
}

/// `quadOutput`: each quad into its material's layer.
impl BlockQuadOutput for StartedLayers<'_> {
    fn put(&mut self, x: f32, y: f32, z: f32, quad: &BakedQuad, instance: &QuadInstance) {
        self.get_or_begin_layer(quad.material_info.layer)
            .put_block_baked_quad(x, y, z, quad, instance);
    }
}

/// `opaqueQuadOutput`: every quad into the solid layer, for leaves drawn solid.
struct OpaqueQuadOutput<'b, 'a>(&'b mut StartedLayers<'a>);

impl BlockQuadOutput for OpaqueQuadOutput<'_, '_> {
    fn put(&mut self, x: f32, y: f32, z: f32, quad: &BakedQuad, instance: &QuadInstance) {
        self.0
            .get_or_begin_layer(ChunkSectionLayer::SOLID)
            .put_block_baked_quad(x, y, z, quad, instance);
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::sync::Arc;

    use md5::{Digest, Md5};
    use serde_json::{Value, json};

    use super::{Results, SectionCompiler};
    use crate::com::mojang::blaze3d::vertex::vertex_sorting::VertexSorting;
    use crate::golden::Golden;
    use crate::net::minecraft::client::color::block::block_colors::BlockColors;
    use crate::net::minecraft::client::multiplayer::client_level::ClientLevel;
    use crate::net::minecraft::client::renderer::block::block_and_tint_getter::BlockAndTintGetter;
    use crate::net::minecraft::client::renderer::block::block_state_model_set::BlockStateModelSet;
    use crate::net::minecraft::client::renderer::block::fluid_state_model_set::FluidStateModelSet;
    use crate::net::minecraft::client::renderer::block::model_block_renderer::tests::MeshLevel;
    use crate::net::minecraft::client::renderer::chunk::render_region_cache::RenderRegionCache;
    use crate::net::minecraft::client::renderer::chunk::render_section_region::RenderSectionRegion;
    use crate::net::minecraft::client::renderer::chunk::section_copy::SectionCopy;
    use crate::net::minecraft::client::renderer::section_buffer_builder_pack::SectionBufferBuilderPack;
    use crate::net::minecraft::client::resources::model::model_bakery::tests::{
        atlases, bake_block_states,
    };
    use crate::net::minecraft::client::resources::model::sprite::material_baker::MaterialBaker;
    use crate::net::minecraft::core::block_pos::BlockPos;
    use crate::net::minecraft::core::direction::Direction;
    use crate::net::minecraft::core::holder::Holder;
    use crate::net::minecraft::core::registries::registries::Registries;
    use crate::net::minecraft::core::section_pos::SectionPos;
    use crate::net::minecraft::network::codec::stream_codec::StreamCodec;
    use crate::net::minecraft::network::protocol::game::clientbound_level_chunk_with_light_packet::ClientboundLevelChunkWithLightPacket;
    use crate::net::minecraft::network::protocol::game::clientbound_level_chunk_with_light_packet::tests::registry_reader;
    use crate::net::minecraft::resources::identifier::Identifier;
    use crate::net::minecraft::resources::resource_key::ResourceKey;
    use crate::net::minecraft::server::packs::resources::multi_pack_resource_manager::tests::vanilla_resources;
    use crate::net::minecraft::world::level::block::block::Block;
    use crate::net::minecraft::world::level::block::blocks::{self, Blocks};
    use crate::net::minecraft::world::level::block::liquid_block::LiquidBlock;
    use crate::net::minecraft::world::level::block::state::block_state::BlockState;
    use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
    use crate::net::minecraft::world::level::block_and_light_getter::BlockAndLightGetter;
    use crate::net::minecraft::world::level::block_getter::BlockGetter;
    use crate::net::minecraft::world::level::cardinal_lighting::CardinalLighting;
    use crate::net::minecraft::world::level::color_resolver::ColorResolver;
    use crate::net::minecraft::world::level::chunk::level_chunk_section::tests::registries;
    use crate::net::minecraft::world::level::chunk::paletted_container_factory::PalettedContainerFactory;
    use crate::net::minecraft::world::level::light_layer::LightLayer;
    use crate::net::minecraft::world::level::lighting::level_light_engine::LevelLightEngine;
    use crate::net::minecraft::world::level::material::fluid_state::FluidState;
    use crate::rustcraft::oracle::client::replay::FlatLevel;
    use crate::rustcraft::oracle::client::scenario::Scenario;
    use std::path::Path;

    /// What `ModelSuites.GoldenRegion` answers for the level: `MeshLevel`'s light, the default
    /// cardinal lighting, and white tints, or with `tinted` a colour by `MeshLevel`'s hash.
    struct Lighting {
        seed: i32,
        tinted: bool,
        // `LevelLightEngine.EMPTY`: its light is `get_brightness`'s own.
        light_engine: LevelLightEngine,
    }

    /// `GoldenRegion`'s `flatLight`: the light a client has on the flat world, the sky's 15 but
    /// inside opaque blocks, and no block light.
    struct FlatLighting<'a> {
        level: &'a FlatLevel,
        light_engine: LevelLightEngine,
    }

    impl BlockGetter for FlatLighting<'_> {
        fn get_block_state(&self, pos: &BlockPos) -> BlockState {
            self.level.get_block_state(pos)
        }

        fn get_fluid_state(&self, pos: &BlockPos) -> FluidState {
            self.level.get_fluid_state(pos)
        }
    }

    impl BlockAndLightGetter for FlatLighting<'_> {
        fn get_light_engine(&self) -> &LevelLightEngine {
            &self.light_engine
        }

        fn get_brightness(&self, layer: LightLayer, pos: &BlockPos) -> i32 {
            if layer == LightLayer::SKY && !self.level.get_block_state(pos).is_solid_render() {
                15
            } else {
                0
            }
        }
    }

    impl BlockAndTintGetter for FlatLighting<'_> {
        fn cardinal_lighting(&self) -> CardinalLighting {
            CardinalLighting::DEFAULT
        }

        fn get_block_tint(&self, _pos: &BlockPos, _color: &'static ColorResolver) -> i32 {
            -1
        }
    }

    impl BlockGetter for Lighting {
        fn get_block_state(&self, _pos: &BlockPos) -> BlockState {
            unreachable!("a region reads its own copies of the blocks")
        }

        fn get_fluid_state(&self, _pos: &BlockPos) -> FluidState {
            unreachable!("a region reads its own copies of the blocks")
        }
    }

    impl BlockAndLightGetter for Lighting {
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

    impl BlockAndTintGetter for Lighting {
        fn cardinal_lighting(&self) -> CardinalLighting {
            CardinalLighting::DEFAULT
        }

        fn get_block_tint(&self, pos: &BlockPos, _color: &'static ColorResolver) -> i32 {
            if self.tinted {
                -16777216
                    | MeshLevel::hash(pos.get_x(), pos.get_y(), pos.get_z(), self.seed + 1)
                        & 0xFFFFFF
            } else {
                -1
            }
        }
    }

    /// `ModelSuites.FluidLevel`: water and lava at every level among blocks, by `MeshLevel`'s
    /// hash.
    struct FluidLevel {
        neighbours: Vec<Block>,
    }

    const NEIGHBOUR_NAMES: [&str; 12] = [
        "stone",
        "grass_block",
        "glass",
        "ice",
        "oak_leaves",
        "oak_slab",
        "oak_stairs",
        "oak_fence",
        "mangrove_roots",
        "soul_sand",
        "glowstone",
        "white_stained_glass",
    ];

    impl FluidLevel {
        fn new() -> FluidLevel {
            let neighbours = NEIGHBOUR_NAMES
                .iter()
                .map(|name| {
                    blocks::registry()
                        .get_value(&Identifier::with_default_namespace(name))
                        .expect("a vanilla block")
                })
                .collect();
            FluidLevel { neighbours }
        }

        fn get_block_state(&self, x: i32, y: i32, z: i32, seed: i32) -> BlockState {
            let h = MeshLevel::hash(x, y, z, seed);
            let kind = h % 7;
            if kind < 2 {
                return Blocks::AIR.default_block_state();
            }
            if kind < 5 {
                let fluid = if h / 7 % 3 == 0 {
                    Blocks::LAVA
                } else {
                    Blocks::WATER
                };
                return fluid
                    .default_block_state()
                    .set_value(LiquidBlock::LEVEL, h / 21 % 16);
            }
            let block = self.neighbours[(h / 7) as usize % self.neighbours.len()];
            let states: Vec<BlockState> =
                block.get_state_definition().get_possible_states().collect();
            states[(h / 7 / self.neighbours.len() as i32) as usize % states.len()]
        }
    }

    fn md5(bytes: &[u8]) -> String {
        let digest: [u8; 16] = Md5::digest(bytes).into();
        digest.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    /// `SectionCompilerSuite.compile`'s encoding of the results.
    fn compile(
        compiler: &SectionCompiler,
        region: &RenderSectionRegion<&dyn BlockAndTintGetter>,
        section_pos: SectionPos,
        camera: [f64; 3],
    ) -> Value {
        let mut buffers = SectionBufferBuilderPack::new();
        // SectionRenderDispatcher.RenderSection.createVertexSorting.
        let sorting = VertexSorting::by_distance_xyz(
            (camera[0] - f64::from(section_pos.min_block_x())) as f32,
            (camera[1] - f64::from(section_pos.min_block_y())) as f32,
            (camera[2] - f64::from(section_pos.min_block_z())) as f32,
        );
        let results: Results = compiler.compile(section_pos, region, &sorting, &mut buffers);
        let layers: Vec<Value> = results
            .rendered_layers
            .iter()
            .map(|(layer, mesh)| {
                json!([
                    format!("{layer:?}"),
                    mesh.draw_state().vertex_count,
                    mesh.draw_state().index_count,
                    md5(mesh.vertex_buffer()),
                    mesh.index_buffer().map(md5)
                ])
            })
            .collect();
        let mut visibility = 0i64;
        for from in Direction::VALUES {
            for to in Direction::VALUES {
                if results.visibility_set.visibility_between(from, to) {
                    visibility |= 1 << (from as u32 + 6 * to as u32);
                }
            }
        }
        json!([layers, visibility])
    }

    /// Sections compiled like vanilla's `SectionCompiler` compiles them (`ModelSuites`'
    /// `SectionCompilerSuite`): every section with blocks of the walk's chunks, put in a level
    /// the way the listener puts them and read through regions the way `RenderRegionCache` makes
    /// them, and sections of `MeshLevel`'s blocks, whose translucent quads sort by the camera.
    #[test]
    fn compiles_sections_like_vanilla() {
        let golden = Golden::load("net.minecraft.client.renderer.chunk.SectionCompiler");
        let manager = vanilla_resources();
        let (blocks, items) = atlases(&manager);
        let materials = MaterialBaker::new(&blocks, &items);
        let baked = bake_block_states(&manager, &materials);
        let models = Arc::new(BlockStateModelSet::new(
            baked.block_state_models,
            baked.missing_models.block,
        ));
        let block_colors = Arc::new(BlockColors::new());
        // `new FluidStateModelSet(Map.of(), null)`: these sections have no fluids.
        let no_fluid_models = Arc::new(FluidStateModelSet::new(
            Vec::new(),
            Arc::clone(&baked.missing_models.fluid),
        ));
        let compilers = [false, true].map(|ambient_occlusion| {
            SectionCompiler::new(
                ambient_occlusion,
                true,
                Arc::clone(&models),
                Arc::clone(&no_fluid_models),
                Arc::clone(&block_colors),
            )
        });
        let fluid_models = Arc::new(FluidStateModelSet::new(
            FluidStateModelSet::bake(&materials),
            Arc::clone(&baked.missing_models.fluid),
        ));
        let default_block_colors = Arc::new(BlockColors::create_default());
        let fluid_compilers = [false, true].map(|ambient_occlusion| {
            SectionCompiler::new(
                ambient_occlusion,
                true,
                Arc::clone(&models),
                Arc::clone(&fluid_models),
                Arc::clone(&default_block_colors),
            )
        });

        // The walk's chunks, put in an overworld level like the chunk packet's handler does.
        let chunks = Golden::load(
            "net.minecraft.network.protocol.game.ClientboundLevelChunkWithLightPacket",
        );
        let registries = registries(&chunks.input("BIOMES"));
        let mut level = ClientLevel::new(
            &registries,
            false,
            true,
            ResourceKey::create(
                Registries::DIMENSION,
                Identifier::with_default_namespace("overworld"),
            ),
            Holder::Reference(ResourceKey::create(
                Registries::DIMENSION_TYPE,
                Identifier::with_default_namespace("overworld"),
            )),
            8,
            8,
            false,
            0,
            -63,
        );
        for (args, _) in chunks.cases("captured") {
            let mut input = registry_reader(crate::golden::unhex(args[0].as_str().expect("hex")));
            let packet = ClientboundLevelChunkWithLightPacket::STREAM_CODEC.decode(&mut input);
            level
                .get_chunk_source_mut()
                .replace_with_packet_data(packet.x(), packet.z(), packet.chunk_data())
                .expect("the walk's chunks are all in range");
        }
        let walk_camera = [0.5, -60.0 + f64::from(1.62f32), 0.5];
        let cache = RefCell::new(RenderRegionCache::new());
        let lighting = Lighting {
            seed: 0,
            tinted: false,
            light_engine: LevelLightEngine::empty(),
        };
        golden.check("walk", |x: i32, y: i32, z: i32, ambient_occlusion: bool| {
            let region = cache.borrow_mut().create_region_over(
                &level,
                &lighting,
                SectionPos::as_long_xyz(x, y, z),
            );
            compile(
                &compilers[usize::from(ambient_occlusion)],
                &region,
                SectionPos::of(x, y, z),
                walk_camera,
            )
        });

        // MeshLevel's blocks in the 27 sections around the origin's, waterlogged states left out.
        let factory = PalettedContainerFactory::create(&registries);
        golden.check(
            "pattern",
            |seed: i32, ambient_occlusion: bool, x: f64, y: f64, z: f64| {
                let pattern = MeshLevel::new(seed);
                let mut sections = Vec::new();
                for section_z in -1..=1 {
                    for section_y in -1..=1 {
                        for section_x in -1..=1 {
                            let mut states = factory.create_for_block_states();
                            for i in 0..4096 {
                                let (bx, bz, by) = (i & 15, i >> 4 & 15, i >> 8);
                                let state = pattern.get_block_state(&BlockPos::new(
                                    16 * section_x + bx,
                                    16 * section_y + by,
                                    16 * section_z + bz,
                                ));
                                if state.get_fluid_state().is_empty() {
                                    states.get_and_set_unchecked(bx, by, bz, &state);
                                }
                            }
                            sections.push(Arc::new(SectionCopy::of_states(Some(states))));
                        }
                    }
                }
                let lighting = Lighting {
                    seed,
                    tinted: false,
                    light_engine: LevelLightEngine::empty(),
                };
                let region = RenderSectionRegion::over(&lighting, -1, -1, -1, sections);
                compile(
                    &compilers[usize::from(ambient_occlusion)],
                    &region,
                    SectionPos::of(0, 0, 0),
                    [x, y, z],
                )
            },
        );

        // FluidLevel's blocks in the 27 sections around the origin's, meshed with the fluids'
        // models, the default block colours and tints by block.
        let fluid_level = FluidLevel::new();
        golden.check(
            "fluids",
            |seed: i32, ambient_occlusion: bool, x: f64, y: f64, z: f64| {
                let mut sections = Vec::new();
                for section_z in -1..=1 {
                    for section_y in -1..=1 {
                        for section_x in -1..=1 {
                            let mut states = factory.create_for_block_states();
                            for i in 0..4096 {
                                let (bx, bz, by) = (i & 15, i >> 4 & 15, i >> 8);
                                let state = fluid_level.get_block_state(
                                    16 * section_x + bx,
                                    16 * section_y + by,
                                    16 * section_z + bz,
                                    seed,
                                );
                                states.get_and_set_unchecked(bx, by, bz, &state);
                            }
                            sections.push(Arc::new(SectionCopy::of_states(Some(states))));
                        }
                    }
                }
                let lighting = Lighting {
                    seed,
                    tinted: true,
                    light_engine: LevelLightEngine::empty(),
                };
                let region = RenderSectionRegion::over(&lighting, -1, -1, -1, sections);
                compile(
                    &fluid_compilers[usize::from(ambient_occlusion)],
                    &region,
                    SectionPos::of(0, 0, 0),
                    [x, y, z],
                )
            },
        );
        // The sections around a scenario's course on the flat world, with the light a client has
        // there (`ModelSuites`' "flat").
        golden.check(
            "flat",
            |scenario: String, x: i32, y: i32, z: i32, ambient_occlusion: bool| {
                let scenario = Scenario::load(Path::new(&format!(
                    "{}/oracle/scenarios/{scenario}.json",
                    env!("CARGO_MANIFEST_DIR")
                )))
                .expect("the scenario");
                let flat = FlatLevel::with_setup(&scenario.setup);
                let mut sections = Vec::new();
                for section_z in z - 1..=z + 1 {
                    for section_y in y - 1..=y + 1 {
                        for section_x in x - 1..=x + 1 {
                            let mut states = factory.create_for_block_states();
                            let mut empty = true;
                            for i in 0..4096 {
                                let (bx, bz, by) = (i & 15, i >> 4 & 15, i >> 8);
                                let state = flat.get_block_state(&BlockPos::new(
                                    16 * section_x + bx,
                                    16 * section_y + by,
                                    16 * section_z + bz,
                                ));
                                states.get_and_set_unchecked(bx, by, bz, &state);
                                empty &= state.is_air();
                            }
                            sections
                                .push(Arc::new(SectionCopy::of_states((!empty).then_some(states))));
                        }
                    }
                }
                let lighting = FlatLighting {
                    level: &flat,
                    light_engine: LevelLightEngine::empty(),
                };
                let region = RenderSectionRegion::over(&lighting, x - 1, y - 1, z - 1, sections);
                compile(
                    &compilers[usize::from(ambient_occlusion)],
                    &region,
                    SectionPos::of(x, y, z),
                    walk_camera,
                )
            },
        );
        golden.finish();
    }
}
