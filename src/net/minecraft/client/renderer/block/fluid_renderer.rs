//! net/minecraft/client/renderer/block/FluidRenderer.java
//!
//! A fluid block meshed into its model's layer: the top at its four corners' heights (each the
//! average of the blocks around the corner, weighted toward full ones), the still sprite when it
//! doesn't flow and the flowing one turned along its flow when it does, the bottom and the sides
//! where no neighbour of the same fluid or face hides them, water's overlay on a side against glass
//! or leaves, each face tinted by the model's tint source and shaded by the dimension's lighting,
//! and a backward copy of the faces seen from inside the fluid.

use crate::com::mojang::blaze3d::vertex::quad_instance::NO_OVERLAY;
use crate::com::mojang::blaze3d::vertex::vertex_consumer::VertexConsumer;
use crate::java::lang::math::Math;
use crate::net::minecraft::client::renderer::block::block_and_tint_getter::BlockAndTintGetter;
use crate::net::minecraft::client::renderer::block::fluid_state_model_set::FluidStateModelSet;
use crate::net::minecraft::client::renderer::chunk::chunk_section_layer::ChunkSectionLayer;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::{Axis, Direction, Plane};
use crate::net::minecraft::util::argb::ARGB;
use crate::net::minecraft::util::light_coords_util::LightCoordsUtil;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::material::fluid_state::FluidState;
use crate::net::minecraft::world::level::material::fluids::Fluids;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;

/// `FluidRenderer.Output`: the builder of a layer.
pub trait Output {
    fn get_builder(&mut self, layer: ChunkSectionLayer) -> &mut dyn VertexConsumer;
}

pub struct FluidRenderer<'a> {
    fluid_models: &'a FluidStateModelSet,
}

impl<'a> FluidRenderer<'a> {
    const MAX_FLUID_HEIGHT: f32 = 0.8888889;

    pub fn new(fluid_models: &'a FluidStateModelSet) -> FluidRenderer<'a> {
        FluidRenderer { fluid_models }
    }

    fn is_neighbor_same_fluid(fluid_state: FluidState, neighbor_fluid_state: FluidState) -> bool {
        neighbor_fluid_state
            .get_type()
            .is_same(fluid_state.get_type())
    }

    fn is_face_occluded_by_state(direction: Direction, height: f32, state: BlockState) -> bool {
        let occluder = state.get_face_occlusion_shape(direction.get_opposite());
        if occluder == Shapes::empty() {
            false
        } else if occluder == Shapes::block() {
            let full_block = height == 1.0;
            direction != Direction::UP || full_block
        } else {
            let shape = Shapes::r#box(0.0, 0.0, 0.0, 1.0, f64::from(height), 1.0);
            Shapes::block_occludes(&shape, &occluder, direction)
        }
    }

    fn is_face_occluded_by_neighbor(
        direction: Direction,
        height: f32,
        neighbor_state: BlockState,
    ) -> bool {
        FluidRenderer::is_face_occluded_by_state(direction, height, neighbor_state)
    }

    fn is_face_occluded_by_self(state: BlockState, direction: Direction) -> bool {
        FluidRenderer::is_face_occluded_by_state(direction.get_opposite(), 1.0, state)
    }

    pub fn should_render_face(
        fluid_state: FluidState,
        block_state: BlockState,
        direction: Direction,
        neighbor_fluid_state: FluidState,
    ) -> bool {
        !FluidRenderer::is_neighbor_same_fluid(fluid_state, neighbor_fluid_state)
            && !FluidRenderer::is_face_occluded_by_self(block_state, direction)
    }

    pub fn tesselate(
        &self,
        level: &dyn BlockAndTintGetter,
        pos: &BlockPos,
        output: &mut dyn Output,
        block_state: BlockState,
        fluid_state: FluidState,
    ) {
        let block_state_down = level.get_block_state(&pos.relative(Direction::DOWN));
        let fluid_state_down = block_state_down.get_fluid_state();
        let block_state_up = level.get_block_state(&pos.relative(Direction::UP));
        let fluid_state_up = block_state_up.get_fluid_state();
        let block_state_north = level.get_block_state(&pos.relative(Direction::NORTH));
        let fluid_state_north = block_state_north.get_fluid_state();
        let block_state_south = level.get_block_state(&pos.relative(Direction::SOUTH));
        let fluid_state_south = block_state_south.get_fluid_state();
        let block_state_west = level.get_block_state(&pos.relative(Direction::WEST));
        let fluid_state_west = block_state_west.get_fluid_state();
        let block_state_east = level.get_block_state(&pos.relative(Direction::EAST));
        let fluid_state_east = block_state_east.get_fluid_state();
        let render_up = !FluidRenderer::is_neighbor_same_fluid(fluid_state, fluid_state_up);
        let render_down = FluidRenderer::should_render_face(
            fluid_state,
            block_state,
            Direction::DOWN,
            fluid_state_down,
        ) && !FluidRenderer::is_face_occluded_by_neighbor(
            Direction::DOWN,
            FluidRenderer::MAX_FLUID_HEIGHT,
            block_state_down,
        );
        let render_north = FluidRenderer::should_render_face(
            fluid_state,
            block_state,
            Direction::NORTH,
            fluid_state_north,
        );
        let render_south = FluidRenderer::should_render_face(
            fluid_state,
            block_state,
            Direction::SOUTH,
            fluid_state_south,
        );
        let render_west = FluidRenderer::should_render_face(
            fluid_state,
            block_state,
            Direction::WEST,
            fluid_state_west,
        );
        let render_east = FluidRenderer::should_render_face(
            fluid_state,
            block_state,
            Direction::EAST,
            fluid_state_east,
        );
        if !(render_up || render_down || render_east || render_west || render_north || render_south)
        {
            return;
        }

        let model = self.fluid_models.get(fluid_state);
        let builder = output.get_builder(model.layer);
        let tint_color = match &model.tint_source {
            Some(tint_source) => tint_source.color_in_world(block_state, level, pos),
            None => -1,
        };
        let cardinal_lighting = level.cardinal_lighting();
        let fluid_type = fluid_state.get_type();
        let height_self =
            FluidRenderer::get_height_with(level, fluid_type, pos, block_state, fluid_state);
        let mut height_north_east;
        let mut height_north_west;
        let mut height_south_east;
        let mut height_south_west;
        if height_self >= 1.0 {
            height_north_east = 1.0;
            height_north_west = 1.0;
            height_south_east = 1.0;
            height_south_west = 1.0;
        } else {
            let height_north = FluidRenderer::get_height_with(
                level,
                fluid_type,
                &pos.north(),
                block_state_north,
                fluid_state_north,
            );
            let height_south = FluidRenderer::get_height_with(
                level,
                fluid_type,
                &pos.south(),
                block_state_south,
                fluid_state_south,
            );
            let height_east = FluidRenderer::get_height_with(
                level,
                fluid_type,
                &pos.east(),
                block_state_east,
                fluid_state_east,
            );
            let height_west = FluidRenderer::get_height_with(
                level,
                fluid_type,
                &pos.west(),
                block_state_west,
                fluid_state_west,
            );
            height_north_east = FluidRenderer::calculate_average_height(
                level,
                fluid_type,
                height_self,
                height_north,
                height_east,
                &pos.relative(Direction::NORTH).relative(Direction::EAST),
            );
            height_north_west = FluidRenderer::calculate_average_height(
                level,
                fluid_type,
                height_self,
                height_north,
                height_west,
                &pos.relative(Direction::NORTH).relative(Direction::WEST),
            );
            height_south_east = FluidRenderer::calculate_average_height(
                level,
                fluid_type,
                height_self,
                height_south,
                height_east,
                &pos.relative(Direction::SOUTH).relative(Direction::EAST),
            );
            height_south_west = FluidRenderer::calculate_average_height(
                level,
                fluid_type,
                height_self,
                height_south,
                height_west,
                &pos.relative(Direction::SOUTH).relative(Direction::WEST),
            );
        }

        let x = (pos.get_x() & 15) as f32;
        let y = (pos.get_y() & 15) as f32;
        let z = (pos.get_z() & 15) as f32;
        let offs = 0.001f32;
        let bottom_offs = if render_down { 0.001f32 } else { 0.0 };
        if render_up
            && !FluidRenderer::is_face_occluded_by_neighbor(
                Direction::UP,
                Math::min_f32(
                    Math::min_f32(height_north_west, height_south_west),
                    Math::min_f32(height_south_east, height_north_east),
                ),
                block_state_up,
            )
        {
            height_north_west -= offs;
            height_south_west -= offs;
            height_south_east -= offs;
            height_north_east -= offs;
            let flow = fluid_state.get_flow(level, pos);
            let u00;
            let u01;
            let u10;
            let u11;
            let v00;
            let v01;
            let v10;
            let v11;
            if flow.x == 0.0 && flow.z == 0.0 {
                let still_sprite = &model.still_material.sprite;
                u00 = still_sprite.get_u0();
                v00 = still_sprite.get_v0();
                u01 = u00;
                v01 = still_sprite.get_v1();
                u10 = still_sprite.get_u1();
                v10 = v01;
                u11 = u10;
                v11 = v00;
            } else {
                let angle = Mth::atan2(flow.z, flow.x) as f32 - (std::f64::consts::PI / 2.0) as f32;
                let s = Mth::sin(f64::from(angle)) * 0.25;
                let c = Mth::cos(f64::from(angle)) * 0.25;
                let flowing_sprite = &model.flowing_material.sprite;
                u00 = flowing_sprite.get_u(0.5 + (-c - s));
                v00 = flowing_sprite.get_v(0.5 + (-c + s));
                u01 = flowing_sprite.get_u(0.5 + (-c + s));
                v01 = flowing_sprite.get_v(0.5 + (c + s));
                u10 = flowing_sprite.get_u(0.5 + (c + s));
                v10 = flowing_sprite.get_v(0.5 + (c - s));
                u11 = flowing_sprite.get_u(0.5 + (c - s));
                v11 = flowing_sprite.get_v(0.5 + (-c - s));
            }

            let top_light_coords = FluidRenderer::get_light_coords(level, pos);
            let top_color = ARGB::scale_rgb(tint_color, cardinal_lighting.up);
            FluidRenderer::add_face(
                builder,
                [
                    (x + 0.0, y + height_north_west, z + 0.0, u00, v00),
                    (x + 0.0, y + height_south_west, z + 1.0, u01, v01),
                    (x + 1.0, y + height_south_east, z + 1.0, u10, v10),
                    (x + 1.0, y + height_north_east, z + 0.0, u11, v11),
                ],
                top_color,
                top_light_coords,
                fluid_state.should_render_backward_up_face(level, &pos.above()),
            );
        }

        if render_down {
            let still_sprite = &model.still_material.sprite;
            let u0 = still_sprite.get_u0();
            let u1 = still_sprite.get_u1();
            let v0 = still_sprite.get_v0();
            let v1 = still_sprite.get_v1();
            let below_light_coords = FluidRenderer::get_light_coords(level, &pos.below());
            let below_color = ARGB::scale_rgb(tint_color, cardinal_lighting.down);
            FluidRenderer::add_face(
                builder,
                [
                    (x, y + bottom_offs, z, u0, v0),
                    (x + 1.0, y + bottom_offs, z, u1, v0),
                    (x + 1.0, y + bottom_offs, z + 1.0, u1, v1),
                    (x, y + bottom_offs, z + 1.0, u0, v1),
                ],
                below_color,
                below_light_coords,
                false,
            );
        }

        let side_light_coords = FluidRenderer::get_light_coords(level, pos);

        for face_dir in Plane::HORIZONTAL.iterator() {
            let (hh0, hh1, x0, z0, x1, z1, render_condition, face_state) = match face_dir {
                Direction::NORTH => (
                    height_north_west,
                    height_north_east,
                    x,
                    z + offs,
                    x + 1.0,
                    z + offs,
                    render_north,
                    block_state_north,
                ),
                Direction::SOUTH => (
                    height_south_east,
                    height_south_west,
                    x + 1.0,
                    z + 1.0 - offs,
                    x,
                    z + 1.0 - offs,
                    render_south,
                    block_state_south,
                ),
                Direction::WEST => (
                    height_south_west,
                    height_north_west,
                    x + offs,
                    z + 1.0,
                    x + offs,
                    z,
                    render_west,
                    block_state_west,
                ),
                Direction::EAST => (
                    height_north_east,
                    height_south_east,
                    x + 1.0 - offs,
                    z,
                    x + 1.0 - offs,
                    z + 1.0,
                    render_east,
                    block_state_east,
                ),
                _ => panic!("UnsupportedOperationException"),
            };

            if render_condition
                && !FluidRenderer::is_face_occluded_by_neighbor(
                    face_dir,
                    Math::max_f32(hh0, hh1),
                    face_state,
                )
            {
                let mut sprite = &model.flowing_material.sprite;
                let mut is_overlay = false;
                if let Some(overlay_material) = &model.overlay_material
                    && (face_state.class_is_half_transparent() || face_state.class_is_leaves())
                {
                    sprite = &overlay_material.sprite;
                    is_overlay = true;
                }

                let u0 = sprite.get_u(0.0);
                let u1 = sprite.get_u(0.5);
                let v01 = sprite.get_v((1.0 - hh0) * 0.5);
                let v02 = sprite.get_v((1.0 - hh1) * 0.5);
                let v1 = sprite.get_v(0.5);
                let shade_side = if face_dir.get_axis() == Axis::Z {
                    cardinal_lighting.north
                } else {
                    cardinal_lighting.west
                };
                let face_color = ARGB::scale_rgb(tint_color, cardinal_lighting.up * shade_side);
                FluidRenderer::add_face(
                    builder,
                    [
                        (x0, y + hh0, z0, u0, v01),
                        (x1, y + hh1, z1, u1, v02),
                        (x1, y + bottom_offs, z1, u1, v1),
                        (x0, y + bottom_offs, z0, u0, v1),
                    ],
                    face_color,
                    side_light_coords,
                    !is_overlay,
                );
            }
        }
    }

    /// `addFace`: the quad's four corners `(x, y, z, u, v)`, then, for a face seen from inside
    /// too, the same corners the other way round.
    fn add_face(
        builder: &mut dyn VertexConsumer,
        corners: [(f32, f32, f32, f32, f32); 4],
        color: i32,
        light_coords: i32,
        add_back_face: bool,
    ) {
        for (x, y, z, u, v) in corners {
            FluidRenderer::vertex(builder, x, y, z, color, u, v, light_coords);
        }
        if add_back_face {
            for index in [0, 3, 2, 1] {
                let (x, y, z, u, v) = corners[index];
                FluidRenderer::vertex(builder, x, y, z, color, u, v, light_coords);
            }
        }
    }

    fn calculate_average_height(
        level: &dyn BlockAndTintGetter,
        fluid_type: Fluids,
        height_self: f32,
        height2: f32,
        height1: f32,
        corner_pos: &BlockPos,
    ) -> f32 {
        if height1 >= 1.0 || height2 >= 1.0 {
            return 1.0;
        }

        let mut weighted_height = [0.0f32; 2];
        if height1 > 0.0 || height2 > 0.0 {
            let height_corner = FluidRenderer::get_height(level, fluid_type, corner_pos);
            if height_corner >= 1.0 {
                return 1.0;
            }

            FluidRenderer::add_weighted_height(&mut weighted_height, height_corner);
        }

        FluidRenderer::add_weighted_height(&mut weighted_height, height_self);
        FluidRenderer::add_weighted_height(&mut weighted_height, height1);
        FluidRenderer::add_weighted_height(&mut weighted_height, height2);
        weighted_height[0] / weighted_height[1]
    }

    fn add_weighted_height(weighted_height: &mut [f32; 2], height: f32) {
        if height >= 0.8 {
            weighted_height[0] += height * 10.0;
            weighted_height[1] += 10.0;
        } else if height >= 0.0 {
            weighted_height[0] += height;
            weighted_height[1] += 1.0;
        }
    }

    fn get_height(level: &dyn BlockAndTintGetter, fluid_type: Fluids, pos: &BlockPos) -> f32 {
        let state = level.get_block_state(pos);
        FluidRenderer::get_height_with(level, fluid_type, pos, state, state.get_fluid_state())
    }

    /// `getHeight(level, fluidType, pos, state, fluidState)`: a whole block under the same fluid,
    /// the fluid's own height, nothing beside air or a block that isn't solid, -1 beside one that
    /// is.
    fn get_height_with(
        level: &dyn BlockAndTintGetter,
        fluid_type: Fluids,
        pos: &BlockPos,
        state: BlockState,
        fluid_state: FluidState,
    ) -> f32 {
        if fluid_type.is_same(fluid_state.get_type()) {
            let above_state = level.get_block_state(&pos.above());
            if fluid_type.is_same(above_state.get_fluid_state().get_type()) {
                1.0
            } else {
                fluid_state.get_own_height()
            }
        } else if !state.is_solid() {
            0.0
        } else {
            -1.0
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn vertex(
        builder: &mut dyn VertexConsumer,
        x: f32,
        y: f32,
        z: f32,
        color: i32,
        u: f32,
        v: f32,
        light_coords: i32,
    ) {
        builder.add_vertex_full(
            x,
            y,
            z,
            color,
            u,
            v,
            NO_OVERLAY,
            light_coords,
            0.0,
            1.0,
            0.0,
        );
    }

    fn get_light_coords(level: &dyn BlockAndTintGetter, pos: &BlockPos) -> i32 {
        LightCoordsUtil::max(
            LightCoordsUtil::get_light_coords_at(level, pos),
            LightCoordsUtil::get_light_coords_at(level, &pos.above()),
        )
    }
}
