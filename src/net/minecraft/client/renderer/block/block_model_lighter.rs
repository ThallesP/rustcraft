//! net/minecraft/client/renderer/block/BlockModelLighter.java
//!
//! A quad's light and shading, flat or with ambient occlusion. Java memoizes the last 100
//! positions' light and shade while a section compiles (`Cache`, on per thread); the level doesn't
//! change during a compile, so the port works them out each time, which gives the same.

use crate::com::mojang::blaze3d::vertex::quad_instance::QuadInstance;
use crate::java::lang::math::Math;
use crate::net::minecraft::client::renderer::block::block_and_tint_getter::BlockAndTintGetter;
use crate::net::minecraft::client::resources::model::geometry::baked_quad::BakedQuad;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Direction;
use crate::net::minecraft::util::argb::ARGB;
use crate::net::minecraft::util::light_coords_util::LightCoordsUtil;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::cardinal_lighting::CardinalLighting;

pub const CHECK_LIGHT: i32 = -1;

/// `SizeInfo`'s indexes into the face's shape.
const DOWN: usize = 0;
const UP: usize = 1;
const NORTH: usize = 2;
const SOUTH: usize = 3;
const WEST: usize = 4;
const EAST: usize = 5;
const FLIP_DOWN: usize = 6;
const FLIP_UP: usize = 7;
const FLIP_NORTH: usize = 8;
const FLIP_SOUTH: usize = 9;
const FLIP_WEST: usize = 10;
const FLIP_EAST: usize = 11;
const SIZE_INFO_COUNT: usize = 12;

/// `AdjacencyInfo`: the four neighbours a face's corners blend, and how a partial face weighs
/// them at each vertex.
struct AdjacencyInfo {
    corners: [Direction; 4],
    do_non_cubic_weight: bool,
    vert0_weights: [usize; 8],
    vert1_weights: [usize; 8],
    vert2_weights: [usize; 8],
    vert3_weights: [usize; 8],
}

/// By `Direction`'s 3D data value.
const ADJACENCY_INFOS: [AdjacencyInfo; 6] = [
    // DOWN
    AdjacencyInfo {
        corners: [
            Direction::WEST,
            Direction::EAST,
            Direction::NORTH,
            Direction::SOUTH,
        ],
        do_non_cubic_weight: true,
        vert0_weights: [
            FLIP_WEST, SOUTH, FLIP_WEST, FLIP_SOUTH, WEST, FLIP_SOUTH, WEST, SOUTH,
        ],
        vert1_weights: [
            FLIP_WEST, NORTH, FLIP_WEST, FLIP_NORTH, WEST, FLIP_NORTH, WEST, NORTH,
        ],
        vert2_weights: [
            FLIP_EAST, NORTH, FLIP_EAST, FLIP_NORTH, EAST, FLIP_NORTH, EAST, NORTH,
        ],
        vert3_weights: [
            FLIP_EAST, SOUTH, FLIP_EAST, FLIP_SOUTH, EAST, FLIP_SOUTH, EAST, SOUTH,
        ],
    },
    // UP
    AdjacencyInfo {
        corners: [
            Direction::EAST,
            Direction::WEST,
            Direction::NORTH,
            Direction::SOUTH,
        ],
        do_non_cubic_weight: true,
        vert0_weights: [
            EAST, SOUTH, EAST, FLIP_SOUTH, FLIP_EAST, FLIP_SOUTH, FLIP_EAST, SOUTH,
        ],
        vert1_weights: [
            EAST, NORTH, EAST, FLIP_NORTH, FLIP_EAST, FLIP_NORTH, FLIP_EAST, NORTH,
        ],
        vert2_weights: [
            WEST, NORTH, WEST, FLIP_NORTH, FLIP_WEST, FLIP_NORTH, FLIP_WEST, NORTH,
        ],
        vert3_weights: [
            WEST, SOUTH, WEST, FLIP_SOUTH, FLIP_WEST, FLIP_SOUTH, FLIP_WEST, SOUTH,
        ],
    },
    // NORTH
    AdjacencyInfo {
        corners: [
            Direction::UP,
            Direction::DOWN,
            Direction::EAST,
            Direction::WEST,
        ],
        do_non_cubic_weight: true,
        vert0_weights: [UP, FLIP_WEST, UP, WEST, FLIP_UP, WEST, FLIP_UP, FLIP_WEST],
        vert1_weights: [UP, FLIP_EAST, UP, EAST, FLIP_UP, EAST, FLIP_UP, FLIP_EAST],
        vert2_weights: [
            DOWN, FLIP_EAST, DOWN, EAST, FLIP_DOWN, EAST, FLIP_DOWN, FLIP_EAST,
        ],
        vert3_weights: [
            DOWN, FLIP_WEST, DOWN, WEST, FLIP_DOWN, WEST, FLIP_DOWN, FLIP_WEST,
        ],
    },
    // SOUTH
    AdjacencyInfo {
        corners: [
            Direction::WEST,
            Direction::EAST,
            Direction::DOWN,
            Direction::UP,
        ],
        do_non_cubic_weight: true,
        vert0_weights: [UP, FLIP_WEST, FLIP_UP, FLIP_WEST, FLIP_UP, WEST, UP, WEST],
        vert1_weights: [
            DOWN, FLIP_WEST, FLIP_DOWN, FLIP_WEST, FLIP_DOWN, WEST, DOWN, WEST,
        ],
        vert2_weights: [
            DOWN, FLIP_EAST, FLIP_DOWN, FLIP_EAST, FLIP_DOWN, EAST, DOWN, EAST,
        ],
        vert3_weights: [UP, FLIP_EAST, FLIP_UP, FLIP_EAST, FLIP_UP, EAST, UP, EAST],
    },
    // WEST
    AdjacencyInfo {
        corners: [
            Direction::UP,
            Direction::DOWN,
            Direction::NORTH,
            Direction::SOUTH,
        ],
        do_non_cubic_weight: true,
        vert0_weights: [
            UP, SOUTH, UP, FLIP_SOUTH, FLIP_UP, FLIP_SOUTH, FLIP_UP, SOUTH,
        ],
        vert1_weights: [
            UP, NORTH, UP, FLIP_NORTH, FLIP_UP, FLIP_NORTH, FLIP_UP, NORTH,
        ],
        vert2_weights: [
            DOWN, NORTH, DOWN, FLIP_NORTH, FLIP_DOWN, FLIP_NORTH, FLIP_DOWN, NORTH,
        ],
        vert3_weights: [
            DOWN, SOUTH, DOWN, FLIP_SOUTH, FLIP_DOWN, FLIP_SOUTH, FLIP_DOWN, SOUTH,
        ],
    },
    // EAST
    AdjacencyInfo {
        corners: [
            Direction::DOWN,
            Direction::UP,
            Direction::NORTH,
            Direction::SOUTH,
        ],
        do_non_cubic_weight: true,
        vert0_weights: [
            FLIP_DOWN, SOUTH, FLIP_DOWN, FLIP_SOUTH, DOWN, FLIP_SOUTH, DOWN, SOUTH,
        ],
        vert1_weights: [
            FLIP_DOWN, NORTH, FLIP_DOWN, FLIP_NORTH, DOWN, FLIP_NORTH, DOWN, NORTH,
        ],
        vert2_weights: [
            FLIP_UP, NORTH, FLIP_UP, FLIP_NORTH, UP, FLIP_NORTH, UP, NORTH,
        ],
        vert3_weights: [
            FLIP_UP, SOUTH, FLIP_UP, FLIP_SOUTH, UP, FLIP_SOUTH, UP, SOUTH,
        ],
    },
];

/// `AmbientVertexRemap`: which vertex each blended corner goes to, by facing.
const AMBIENT_VERTEX_REMAPS: [[usize; 4]; 6] = [
    [0, 1, 2, 3],
    [2, 3, 0, 1],
    [3, 0, 1, 2],
    [0, 1, 2, 3],
    [3, 0, 1, 2],
    [1, 2, 3, 0],
];

pub struct BlockModelLighter {
    face_cubic: bool,
    face_partial: bool,
    face_shape: [f32; SIZE_INFO_COUNT],
}

impl Default for BlockModelLighter {
    fn default() -> BlockModelLighter {
        BlockModelLighter::new()
    }
}

impl BlockModelLighter {
    pub fn new() -> BlockModelLighter {
        BlockModelLighter {
            face_cubic: false,
            face_partial: false,
            face_shape: [0.0; SIZE_INFO_COUNT],
        }
    }

    pub fn get_light_coords(
        &self,
        state: BlockState,
        level: &dyn BlockAndTintGetter,
        relative_pos: &BlockPos,
    ) -> i32 {
        LightCoordsUtil::get_light_coords(
            &LightCoordsUtil::default_brightness,
            level,
            state,
            relative_pos,
        )
    }

    fn get_shade_brightness(
        state: BlockState,
        level: &dyn BlockAndTintGetter,
        pos: &BlockPos,
    ) -> f32 {
        state.get_shade_brightness(level, pos)
    }

    pub fn prepare_quad_ambient_occlusion(
        &mut self,
        level: &dyn BlockAndTintGetter,
        state: BlockState,
        center_position: &BlockPos,
        quad: &BakedQuad,
        output_instance: &mut QuadInstance,
    ) {
        self.prepare_quad_shape(level, state, center_position, quad, true);
        let direction = quad.direction;
        let base_position = if self.face_cubic {
            center_position.relative(direction)
        } else {
            *center_position
        };
        let info = &ADJACENCY_INFOS[direction.get_3d_data_value() as usize];
        let light_and_shade = |pos: &BlockPos| {
            let state = level.get_block_state(pos);
            (
                self.get_light_coords(state, level, pos),
                BlockModelLighter::get_shade_brightness(state, level, pos),
            )
        };
        let pos0 = base_position.relative(info.corners[0]);
        let (light0, shade0) = light_and_shade(&pos0);
        let pos1 = base_position.relative(info.corners[1]);
        let (light1, shade1) = light_and_shade(&pos1);
        let pos2 = base_position.relative(info.corners[2]);
        let (light2, shade2) = light_and_shade(&pos2);
        let pos3 = base_position.relative(info.corners[3]);
        let (light3, shade3) = light_and_shade(&pos3);
        let light_permeable = |corner: Direction| {
            level
                .get_block_state(&base_position.relative(corner).relative(direction))
                .is_light_permeable()
        };
        let light_permeable0 = light_permeable(info.corners[0]);
        let light_permeable1 = light_permeable(info.corners[1]);
        let light_permeable2 = light_permeable(info.corners[2]);
        let light_permeable3 = light_permeable(info.corners[3]);
        let diagonal =
            |first: usize, second: usize, permeable_first: bool, permeable_second: bool| {
                if !permeable_second && !permeable_first {
                    (shade0, light0)
                } else {
                    let pos = base_position
                        .relative(info.corners[first])
                        .relative(info.corners[second]);
                    let (light, shade) = light_and_shade(&pos);
                    (shade, light)
                }
            };
        let (shade_corner02, light_corner02) = diagonal(0, 2, light_permeable0, light_permeable2);
        let (shade_corner03, light_corner03) = diagonal(0, 3, light_permeable0, light_permeable3);
        let (shade_corner12, light_corner12) = diagonal(1, 2, light_permeable1, light_permeable2);
        let (shade_corner13, light_corner13) = diagonal(1, 3, light_permeable1, light_permeable3);

        let mut light_center = self.get_light_coords(state, level, center_position);
        let next_pos = center_position.relative(direction);
        let next_state = level.get_block_state(&next_pos);
        if self.face_cubic || !next_state.is_solid_render() {
            light_center = self.get_light_coords(next_state, level, &next_pos);
        }

        let shade_center = if self.face_cubic {
            BlockModelLighter::get_shade_brightness(
                level.get_block_state(&base_position),
                level,
                &base_position,
            )
        } else {
            BlockModelLighter::get_shade_brightness(
                level.get_block_state(center_position),
                level,
                center_position,
            )
        };
        let remap = &AMBIENT_VERTEX_REMAPS[direction.get_3d_data_value() as usize];
        if self.face_partial && info.do_non_cubic_weight {
            let temp_shade1 = (shade3 + shade0 + shade_corner03 + shade_center) * 0.25;
            let temp_shade2 = (shade2 + shade0 + shade_corner02 + shade_center) * 0.25;
            let temp_shade3 = (shade2 + shade1 + shade_corner12 + shade_center) * 0.25;
            let temp_shade4 = (shade3 + shade1 + shade_corner13 + shade_center) * 0.25;
            let shape = &self.face_shape;
            let weights = |sizes: &[usize; 8]| {
                [
                    shape[sizes[0]] * shape[sizes[1]],
                    shape[sizes[2]] * shape[sizes[3]],
                    shape[sizes[4]] * shape[sizes[5]],
                    shape[sizes[6]] * shape[sizes[7]],
                ]
            };
            let vert_weights = [
                weights(&info.vert0_weights),
                weights(&info.vert1_weights),
                weights(&info.vert2_weights),
                weights(&info.vert3_weights),
            ];
            for (vertex, weight) in vert_weights.iter().enumerate() {
                output_instance.set_color_at(
                    remap[vertex],
                    ARGB::gray(Math::clamp_f32(
                        temp_shade1 * weight[0]
                            + temp_shade2 * weight[1]
                            + temp_shade3 * weight[2]
                            + temp_shade4 * weight[3],
                        0.0,
                        1.0,
                    )),
                );
            }
            let tc1 = LightCoordsUtil::smooth_blend(light3, light0, light_corner03, light_center);
            let tc2 = LightCoordsUtil::smooth_blend(light2, light0, light_corner02, light_center);
            let tc3 = LightCoordsUtil::smooth_blend(light2, light1, light_corner12, light_center);
            let tc4 = LightCoordsUtil::smooth_blend(light3, light1, light_corner13, light_center);
            for (vertex, weight) in vert_weights.iter().enumerate() {
                output_instance.set_light_coords_at(
                    remap[vertex],
                    LightCoordsUtil::smooth_weighted_blend(
                        tc1, tc2, tc3, tc4, weight[0], weight[1], weight[2], weight[3],
                    ),
                );
            }
        } else {
            let light_level1 = (shade3 + shade0 + shade_corner03 + shade_center) * 0.25;
            let light_level2 = (shade2 + shade0 + shade_corner02 + shade_center) * 0.25;
            let light_level3 = (shade2 + shade1 + shade_corner12 + shade_center) * 0.25;
            let light_level4 = (shade3 + shade1 + shade_corner13 + shade_center) * 0.25;
            output_instance.set_light_coords_at(
                remap[0],
                LightCoordsUtil::smooth_blend(light3, light0, light_corner03, light_center),
            );
            output_instance.set_light_coords_at(
                remap[1],
                LightCoordsUtil::smooth_blend(light2, light0, light_corner02, light_center),
            );
            output_instance.set_light_coords_at(
                remap[2],
                LightCoordsUtil::smooth_blend(light2, light1, light_corner12, light_center),
            );
            output_instance.set_light_coords_at(
                remap[3],
                LightCoordsUtil::smooth_blend(light3, light1, light_corner13, light_center),
            );
            output_instance.set_color_at(remap[0], ARGB::gray(light_level1));
            output_instance.set_color_at(remap[1], ARGB::gray(light_level2));
            output_instance.set_color_at(remap[2], ARGB::gray(light_level3));
            output_instance.set_color_at(remap[3], ARGB::gray(light_level4));
        }

        let cardinal_lighting = level.cardinal_lighting();
        output_instance.scale_color(BlockModelLighter::get_directional_brightness(
            &cardinal_lighting,
            quad,
            direction,
        ));
    }

    pub fn prepare_quad_flat(
        &mut self,
        level: &dyn BlockAndTintGetter,
        state: BlockState,
        pos: &BlockPos,
        light_coords: i32,
        quad: &BakedQuad,
        output_instance: &mut QuadInstance,
    ) {
        if light_coords == CHECK_LIGHT {
            self.prepare_quad_shape(level, state, pos, quad, false);
            let light_pos = if self.face_cubic {
                pos.relative(quad.direction)
            } else {
                *pos
            };
            output_instance.set_light_coords(self.get_light_coords(state, level, &light_pos));
        } else {
            output_instance.set_light_coords(light_coords);
        }

        let cardinal_lighting = level.cardinal_lighting();
        output_instance.set_color(ARGB::gray(BlockModelLighter::get_directional_brightness(
            &cardinal_lighting,
            quad,
            quad.direction,
        )));
    }

    fn prepare_quad_shape(
        &mut self,
        level: &dyn BlockAndTintGetter,
        state: BlockState,
        pos: &BlockPos,
        quad: &BakedQuad,
        ambient_occlusion: bool,
    ) {
        let mut min_x = 32.0f32;
        let mut min_y = 32.0f32;
        let mut min_z = 32.0f32;
        let mut max_x = -32.0f32;
        let mut max_y = -32.0f32;
        let mut max_z = -32.0f32;
        for position in &quad.positions {
            min_x = Math::min_f32(min_x, position.x);
            min_y = Math::min_f32(min_y, position.y);
            min_z = Math::min_f32(min_z, position.z);
            max_x = Math::max_f32(max_x, position.x);
            max_y = Math::max_f32(max_y, position.y);
            max_z = Math::max_f32(max_z, position.z);
        }

        if ambient_occlusion {
            self.face_shape[WEST] = min_x;
            self.face_shape[EAST] = max_x;
            self.face_shape[DOWN] = min_y;
            self.face_shape[UP] = max_y;
            self.face_shape[NORTH] = min_z;
            self.face_shape[SOUTH] = max_z;
            self.face_shape[FLIP_WEST] = 1.0 - min_x;
            self.face_shape[FLIP_EAST] = 1.0 - max_x;
            self.face_shape[FLIP_DOWN] = 1.0 - min_y;
            self.face_shape[FLIP_UP] = 1.0 - max_y;
            self.face_shape[FLIP_NORTH] = 1.0 - min_z;
            self.face_shape[FLIP_SOUTH] = 1.0 - max_z;
        }

        const MIN_EPSILON: f32 = 1.0E-4;
        const MAX_EPSILON: f32 = 0.9999;
        self.face_partial = match quad.direction {
            Direction::DOWN | Direction::UP => {
                min_x >= MIN_EPSILON
                    || min_z >= MIN_EPSILON
                    || max_x <= MAX_EPSILON
                    || max_z <= MAX_EPSILON
            }
            Direction::NORTH | Direction::SOUTH => {
                min_x >= MIN_EPSILON
                    || min_y >= MIN_EPSILON
                    || max_x <= MAX_EPSILON
                    || max_y <= MAX_EPSILON
            }
            Direction::WEST | Direction::EAST => {
                min_y >= MIN_EPSILON
                    || min_z >= MIN_EPSILON
                    || max_y <= MAX_EPSILON
                    || max_z <= MAX_EPSILON
            }
        };

        let full = || state.is_collision_shape_full_block(level, pos);
        self.face_cubic = match quad.direction {
            Direction::DOWN => min_y == max_y && (min_y < MIN_EPSILON || full()),
            Direction::UP => min_y == max_y && (max_y > MAX_EPSILON || full()),
            Direction::NORTH => min_z == max_z && (min_z < MIN_EPSILON || full()),
            Direction::SOUTH => min_z == max_z && (max_z > MAX_EPSILON || full()),
            Direction::WEST => min_x == max_x && (min_x < MIN_EPSILON || full()),
            Direction::EAST => min_x == max_x && (max_x > MAX_EPSILON || full()),
        };
    }

    fn get_directional_brightness(
        cardinal_lighting: &CardinalLighting,
        quad: &BakedQuad,
        actual_direction: Direction,
    ) -> f32 {
        match quad.material_info.shade_direction_override {
            Some(shade_direction_override) => cardinal_lighting.by_face(shade_direction_override),
            None => cardinal_lighting.by_face(actual_direction),
        }
    }
}
