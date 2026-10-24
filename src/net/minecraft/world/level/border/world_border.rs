//! net/minecraft/world/level/border/WorldBorder.java
//!
//! The border, still (`StaticBorderExtent`) or moving between two sizes over a number of ticks
//! (`MovingBorderExtent`, which `tick` steps), and what collisions, the client's packets and its
//! renderer ask of it. Not ported yet: its listeners (`BorderChangeListener`, which only the
//! server adds), `setDirty` and its saved data's `CODEC` and `TYPE`, which only the server reads
//! and writes, and `getDistanceToBorder(Entity)`, whose callers pass the entity's `x` and `z`.

use crate::java::lang::math::Math;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::level::border::border_status::BorderStatus;
use crate::net::minecraft::world::level::chunk_pos::ChunkPos;
use crate::net::minecraft::world::phys::aabb::AABB;
use crate::net::minecraft::world::phys::shapes::boolean_op::BooleanOp;
use crate::net::minecraft::world::phys::shapes::shapes::Shapes;
use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
use crate::net::minecraft::world::phys::vec3::Vec3;

/// The square the world ends at.
#[derive(Clone, Debug)]
pub struct WorldBorder {
    settings: Settings,
    initialized: bool,
    damage_per_block: f64,
    safe_zone: f64,
    warning_time: i32,
    warning_blocks: i32,
    center_x: f64,
    center_z: f64,
    absolute_max_size: i32,
    extent: BorderExtent,
}

/// `BorderExtent`, an interface its two classes implement: what the border's bounds and size are
/// now. Java's inner classes read the border's center and maximum size, which are parameters
/// here.
#[derive(Clone, Debug)]
enum BorderExtent {
    Static(StaticBorderExtent),
    Moving(MovingBorderExtent),
}

/// A border that doesn't move, its bounds worked out from the center and size.
#[derive(Clone, Debug)]
struct StaticBorderExtent {
    size: f64,
    min_x: f64,
    min_z: f64,
    max_x: f64,
    max_z: f64,
    shape: VoxelShape,
}

/// A border moving from one size to another, a step each tick.
#[derive(Clone, Debug)]
struct MovingBorderExtent {
    from: f64,
    to: f64,
    lerp_end: i64,
    lerp_begin: i64,
    lerp_duration: f64,
    lerp_progress: i64,
    size: f64,
    previous_size: f64,
}

/// `WorldBorder.Settings`: what a border starts with, which `applyInitialSettings` applies.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    pub center_x: f64,
    pub center_z: f64,
    pub damage_per_block: f64,
    pub safe_zone: f64,
    pub warning_blocks: i32,
    pub warning_time: i32,
    pub size: f64,
    pub lerp_time: i64,
    pub lerp_target: f64,
}

impl Settings {
    pub const DEFAULT: Settings = Settings {
        center_x: 0.0,
        center_z: 0.0,
        damage_per_block: 0.2,
        safe_zone: 5.0,
        warning_blocks: 5,
        warning_time: 300,
        size: 5.999997E7_f32 as f64,
        lerp_time: 0,
        lerp_target: 0.0,
    };

    /// `Settings(WorldBorder)`: the border's settings now.
    pub fn from_border(world_border: &WorldBorder) -> Settings {
        Settings {
            center_x: world_border.center_x,
            center_z: world_border.center_z,
            damage_per_block: world_border.damage_per_block,
            safe_zone: world_border.safe_zone,
            warning_blocks: world_border.warning_blocks,
            warning_time: world_border.warning_time,
            size: world_border.extent.get_size(),
            lerp_time: world_border.extent.get_lerp_time(),
            lerp_target: world_border.extent.get_lerp_target(),
        }
    }
}

/// The square's side along one axis, clamped to the largest border.
fn clamp_bound(center: f64, size: f64, absolute_max_size: i32) -> (f64, f64) {
    let max_size = f64::from(absolute_max_size);
    (
        Mth::clamp_f64(center - size / 2.0, -max_size, max_size),
        Mth::clamp_f64(center + size / 2.0, -max_size, max_size),
    )
}

/// Everything outside the bounds, rounded out to whole blocks.
fn outside_shape(min_x: f64, min_z: f64, max_x: f64, max_z: f64) -> VoxelShape {
    Shapes::join(
        &Shapes::infinity(),
        &Shapes::r#box(
            min_x.floor(),
            f64::NEG_INFINITY,
            min_z.floor(),
            max_x.ceil(),
            f64::INFINITY,
            max_z.ceil(),
        ),
        BooleanOp::ONLY_FIRST,
    )
}

impl StaticBorderExtent {
    fn new(size: f64, center_x: f64, center_z: f64, absolute_max_size: i32) -> StaticBorderExtent {
        let mut extent = StaticBorderExtent {
            size,
            min_x: 0.0,
            min_z: 0.0,
            max_x: 0.0,
            max_z: 0.0,
            shape: Shapes::empty(),
        };
        extent.update_box(center_x, center_z, absolute_max_size);
        extent
    }

    fn update_box(&mut self, center_x: f64, center_z: f64, absolute_max_size: i32) {
        (self.min_x, self.max_x) = clamp_bound(center_x, self.size, absolute_max_size);
        (self.min_z, self.max_z) = clamp_bound(center_z, self.size, absolute_max_size);
        self.shape = outside_shape(self.min_x, self.min_z, self.max_x, self.max_z);
    }
}

impl MovingBorderExtent {
    fn new(from: f64, to: f64, duration: i64, game_time: i64) -> MovingBorderExtent {
        let mut extent = MovingBorderExtent {
            from,
            to,
            lerp_duration: duration as f64,
            lerp_progress: duration,
            lerp_begin: game_time,
            lerp_end: game_time.wrapping_add(duration),
            size: 0.0,
            previous_size: 0.0,
        };
        let size = extent.calculate_size();
        extent.size = size;
        extent.previous_size = size;
        extent
    }

    fn calculate_size(&self) -> f64 {
        let progress = (self.lerp_duration - self.lerp_progress as f64) / self.lerp_duration;
        if progress < 1.0 {
            Mth::lerp_f64(progress, self.from, self.to)
        } else {
            self.to
        }
    }

    /// The size between the last tick's and this one's.
    fn size_partial(&self, delta_partial_tick: f32) -> f64 {
        Mth::lerp_f64(f64::from(delta_partial_tick), self.previous_size, self.size)
    }
}

impl BorderExtent {
    fn get_min_x(&self, border: &WorldBorder, delta_partial_tick: f32) -> f64 {
        match self {
            BorderExtent::Static(extent) => extent.min_x,
            BorderExtent::Moving(extent) => {
                let size = extent.size_partial(delta_partial_tick);
                clamp_bound(border.center_x, size, border.absolute_max_size).0
            }
        }
    }

    fn get_max_x(&self, border: &WorldBorder, delta_partial_tick: f32) -> f64 {
        match self {
            BorderExtent::Static(extent) => extent.max_x,
            BorderExtent::Moving(extent) => {
                let size = extent.size_partial(delta_partial_tick);
                clamp_bound(border.center_x, size, border.absolute_max_size).1
            }
        }
    }

    fn get_min_z(&self, border: &WorldBorder, delta_partial_tick: f32) -> f64 {
        match self {
            BorderExtent::Static(extent) => extent.min_z,
            BorderExtent::Moving(extent) => {
                let size = extent.size_partial(delta_partial_tick);
                clamp_bound(border.center_z, size, border.absolute_max_size).0
            }
        }
    }

    fn get_max_z(&self, border: &WorldBorder, delta_partial_tick: f32) -> f64 {
        match self {
            BorderExtent::Static(extent) => extent.max_z,
            BorderExtent::Moving(extent) => {
                let size = extent.size_partial(delta_partial_tick);
                clamp_bound(border.center_z, size, border.absolute_max_size).1
            }
        }
    }

    fn get_size(&self) -> f64 {
        match self {
            BorderExtent::Static(extent) => extent.size,
            BorderExtent::Moving(extent) => extent.size,
        }
    }

    fn get_lerp_speed(&self) -> f64 {
        match self {
            BorderExtent::Static(_) => 0.0,
            BorderExtent::Moving(extent) => {
                (extent.from - extent.to).abs()
                    / extent.lerp_end.wrapping_sub(extent.lerp_begin) as f64
            }
        }
    }

    fn get_lerp_time(&self) -> i64 {
        match self {
            BorderExtent::Static(_) => 0,
            BorderExtent::Moving(extent) => extent.lerp_progress,
        }
    }

    fn get_lerp_target(&self) -> f64 {
        match self {
            BorderExtent::Static(extent) => extent.size,
            BorderExtent::Moving(extent) => extent.to,
        }
    }

    fn get_status(&self) -> BorderStatus {
        match self {
            BorderExtent::Static(_) => BorderStatus::STATIONARY,
            BorderExtent::Moving(extent) if extent.to < extent.from => BorderStatus::SHRINKING,
            BorderExtent::Moving(_) => BorderStatus::GROWING,
        }
    }

    /// `onCenterChange` and `onAbsoluteMaxSizeChange`: a still border works its bounds out again;
    /// a moving one works them out each time they're asked for.
    fn on_bounds_change(&mut self, center_x: f64, center_z: f64, absolute_max_size: i32) {
        if let BorderExtent::Static(extent) = self {
            extent.update_box(center_x, center_z, absolute_max_size);
        }
    }
}

impl Default for WorldBorder {
    fn default() -> WorldBorder {
        WorldBorder::new()
    }
}

impl WorldBorder {
    pub const MAX_SIZE: f64 = 5.999997E7_f32 as f64;
    pub const MAX_CENTER_COORDINATE: f64 = 2.9999984E7;

    /// `new WorldBorder()`, with `Settings.DEFAULT`: centered on 0, 0, 5.999997E7F across.
    pub fn new() -> WorldBorder {
        WorldBorder::from_settings(Settings::DEFAULT)
    }

    /// `new WorldBorder(Settings)`: the default border, with the settings to apply.
    pub fn from_settings(settings: Settings) -> WorldBorder {
        let absolute_max_size = 29999984;
        WorldBorder {
            settings,
            initialized: false,
            damage_per_block: 0.2,
            safe_zone: 5.0,
            warning_time: 15,
            warning_blocks: 5,
            center_x: 0.0,
            center_z: 0.0,
            absolute_max_size,
            extent: BorderExtent::Static(StaticBorderExtent::new(
                WorldBorder::MAX_SIZE,
                0.0,
                0.0,
                absolute_max_size,
            )),
        }
    }

    /// `isWithinBounds(BlockPos)`.
    pub fn is_within_bounds_block_pos(&self, pos: &BlockPos) -> bool {
        self.is_within_bounds(f64::from(pos.get_x()), f64::from(pos.get_z()))
    }

    /// `isWithinBounds(Vec3)`.
    pub fn is_within_bounds_vec3(&self, pos: &Vec3) -> bool {
        self.is_within_bounds(pos.x, pos.z)
    }

    /// `isWithinBounds(ChunkPos)`: both its corners.
    pub fn is_within_bounds_chunk_pos(&self, pos: &ChunkPos) -> bool {
        self.is_within_bounds(
            f64::from(pos.get_min_block_x()),
            f64::from(pos.get_min_block_z()),
        ) && self.is_within_bounds(
            f64::from(pos.get_max_block_x()),
            f64::from(pos.get_max_block_z()),
        )
    }

    /// `isWithinBounds(AABB)`: its corners, the far one just inside.
    pub fn is_within_bounds_aabb(&self, aabb: &AABB) -> bool {
        self.is_within_bounds(aabb.min_x, aabb.min_z)
            && self.is_within_bounds(
                aabb.max_x - f64::from(1.0E-5_f32),
                aabb.max_z - f64::from(1.0E-5_f32),
            )
    }

    /// `isWithinBounds(double, double)`.
    pub fn is_within_bounds(&self, x: f64, z: f64) -> bool {
        self.is_within_bounds_margin(x, z, 0.0)
    }

    /// `isWithinBounds(double, double, double)`.
    pub fn is_within_bounds_margin(&self, x: f64, z: f64, margin: f64) -> bool {
        x >= self.get_min_x() - margin
            && x < self.get_max_x() + margin
            && z >= self.get_min_z() - margin
            && z < self.get_max_z() + margin
    }

    /// `clampToBounds(BlockPos)`.
    pub fn clamp_to_bounds_block_pos(&self, position: &BlockPos) -> BlockPos {
        self.clamp_to_bounds_xyz(
            f64::from(position.get_x()),
            f64::from(position.get_y()),
            f64::from(position.get_z()),
        )
    }

    /// `clampToBounds(Vec3)`.
    pub fn clamp_to_bounds_vec3(&self, position: &Vec3) -> BlockPos {
        self.clamp_to_bounds_xyz(position.x, position.y, position.z)
    }

    /// `clampToBounds(double, double, double)`: the block holding the clamped position.
    pub fn clamp_to_bounds_xyz(&self, x: f64, y: f64, z: f64) -> BlockPos {
        BlockPos::containing(&self.clamp_vec3_to_bound_xyz(x, y, z))
    }

    /// `clampVec3ToBound(Vec3)`.
    pub fn clamp_vec3_to_bound(&self, position: &Vec3) -> Vec3 {
        self.clamp_vec3_to_bound_xyz(position.x, position.y, position.z)
    }

    /// `clampVec3ToBound(double, double, double)`: inside the bounds, the far sides just inside.
    pub fn clamp_vec3_to_bound_xyz(&self, x: f64, y: f64, z: f64) -> Vec3 {
        Vec3::new(
            Mth::clamp_f64(
                x,
                self.get_min_x(),
                self.get_max_x() - f64::from(1.0E-5_f32),
            ),
            y,
            Mth::clamp_f64(
                z,
                self.get_min_z(),
                self.get_max_z() - f64::from(1.0E-5_f32),
            ),
        )
    }

    /// Everything outside the border, rounded out to whole blocks.
    pub fn get_collision_shape(&self) -> VoxelShape {
        match &self.extent {
            BorderExtent::Static(extent) => extent.shape.clone(),
            BorderExtent::Moving(_) => outside_shape(
                self.get_min_x(),
                self.get_min_z(),
                self.get_max_x(),
                self.get_max_z(),
            ),
        }
    }

    /// `getDistanceToBorder(double, double)`.
    pub fn get_distance_to_border(&self, x: f64, z: f64) -> f64 {
        let from_north = z - self.get_min_z();
        let from_south = self.get_max_z() - z;
        let from_west = x - self.get_min_x();
        let from_east = self.get_max_x() - x;
        let min = Math::min_f64(from_west, from_east);
        let min = Math::min_f64(min, from_north);
        Math::min_f64(min, from_south)
    }

    /// `isInsideCloseToBorder(Entity, AABB)` of an entity at `x`, `z`: inside the border and
    /// within twice the box's size of it.
    pub fn is_inside_close_to_border(&self, x: f64, z: f64, bounding_box: &AABB) -> bool {
        let bb_max = Math::max_f64(
            Mth::abs_max_f64(bounding_box.get_xsize(), bounding_box.get_zsize()),
            1.0,
        );
        self.get_distance_to_border(x, z) < bb_max * 2.0
            && self.is_within_bounds_margin(x, z, bb_max)
    }

    pub fn get_status(&self) -> BorderStatus {
        self.extent.get_status()
    }

    pub fn get_min_x(&self) -> f64 {
        self.get_min_x_partial(0.0)
    }

    /// `getMinX(float)`: between the last tick's bounds and this one's.
    pub fn get_min_x_partial(&self, delta_partial_tick: f32) -> f64 {
        self.extent.get_min_x(self, delta_partial_tick)
    }

    pub fn get_min_z(&self) -> f64 {
        self.get_min_z_partial(0.0)
    }

    pub fn get_min_z_partial(&self, delta_partial_tick: f32) -> f64 {
        self.extent.get_min_z(self, delta_partial_tick)
    }

    pub fn get_max_x(&self) -> f64 {
        self.get_max_x_partial(0.0)
    }

    pub fn get_max_x_partial(&self, delta_partial_tick: f32) -> f64 {
        self.extent.get_max_x(self, delta_partial_tick)
    }

    pub fn get_max_z(&self) -> f64 {
        self.get_max_z_partial(0.0)
    }

    pub fn get_max_z_partial(&self, delta_partial_tick: f32) -> f64 {
        self.extent.get_max_z(self, delta_partial_tick)
    }

    pub fn get_center_x(&self) -> f64 {
        self.center_x
    }

    pub fn get_center_z(&self) -> f64 {
        self.center_z
    }

    pub fn set_center(&mut self, x: f64, z: f64) {
        self.center_x = x;
        self.center_z = z;
        self.extent
            .on_bounds_change(self.center_x, self.center_z, self.absolute_max_size);
    }

    pub fn get_size(&self) -> f64 {
        self.extent.get_size()
    }

    /// The ticks left of a move, 0 for a still border.
    pub fn get_lerp_time(&self) -> i64 {
        self.extent.get_lerp_time()
    }

    /// The size a move ends at, a still border's size.
    pub fn get_lerp_target(&self) -> f64 {
        self.extent.get_lerp_target()
    }

    /// A still border of the size.
    pub fn set_size(&mut self, size: f64) {
        self.extent = BorderExtent::Static(StaticBorderExtent::new(
            size,
            self.center_x,
            self.center_z,
            self.absolute_max_size,
        ));
    }

    /// A border moving from one size to another over `ticks` ticks from `game_time`, or a still
    /// one when they're the same.
    pub fn lerp_size_between(&mut self, from: f64, to: f64, ticks: i64, game_time: i64) {
        self.extent = if from == to {
            BorderExtent::Static(StaticBorderExtent::new(
                to,
                self.center_x,
                self.center_z,
                self.absolute_max_size,
            ))
        } else {
            BorderExtent::Moving(MovingBorderExtent::new(from, to, ticks, game_time))
        };
    }

    pub fn set_absolute_max_size(&mut self, absolute_max_size: i32) {
        self.absolute_max_size = absolute_max_size;
        self.extent
            .on_bounds_change(self.center_x, self.center_z, self.absolute_max_size);
    }

    pub fn get_absolute_max_size(&self) -> i32 {
        self.absolute_max_size
    }

    pub fn get_safe_zone(&self) -> f64 {
        self.safe_zone
    }

    pub fn set_safe_zone(&mut self, safe_zone: f64) {
        self.safe_zone = safe_zone;
    }

    pub fn get_damage_per_block(&self) -> f64 {
        self.damage_per_block
    }

    pub fn set_damage_per_block(&mut self, damage_per_block: f64) {
        self.damage_per_block = damage_per_block;
    }

    /// How far a move's size goes a tick, 0 for a still border.
    pub fn get_lerp_speed(&self) -> f64 {
        self.extent.get_lerp_speed()
    }

    pub fn get_warning_time(&self) -> i32 {
        self.warning_time
    }

    pub fn set_warning_time(&mut self, warning_time: i32) {
        self.warning_time = warning_time;
    }

    pub fn get_warning_blocks(&self) -> i32 {
        self.warning_blocks
    }

    pub fn set_warning_blocks(&mut self, warning_blocks: i32) {
        self.warning_blocks = warning_blocks;
    }

    /// A step of a move (`BorderExtent.update`), which ends as a still border of its target.
    pub fn tick(&mut self) {
        if let BorderExtent::Moving(extent) = &mut self.extent {
            extent.lerp_progress = extent.lerp_progress.wrapping_sub(1);
            extent.previous_size = extent.size;
            extent.size = extent.calculate_size();
            if extent.lerp_progress <= 0 {
                let to = extent.to;
                self.extent = BorderExtent::Static(StaticBorderExtent::new(
                    to,
                    self.center_x,
                    self.center_z,
                    self.absolute_max_size,
                ));
            }
        }
    }

    /// The settings it was made with, once.
    pub fn apply_initial_settings(&mut self, game_time: i64) {
        if !self.initialized {
            let settings = self.settings;
            self.set_center(settings.center_x, settings.center_z);
            self.set_damage_per_block(settings.damage_per_block);
            self.set_safe_zone(settings.safe_zone);
            self.set_warning_blocks(settings.warning_blocks);
            self.set_warning_time(settings.warning_time);
            if settings.lerp_time > 0 {
                self.lerp_size_between(
                    settings.size,
                    settings.lerp_target,
                    settings.lerp_time,
                    game_time,
                );
            } else {
                self.set_size(settings.size);
            }

            self.initialized = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::{Settings, WorldBorder};
    use crate::golden::{Decode, Encode, Golden};
    use crate::net::minecraft::core::block_pos::BlockPos;
    use crate::net::minecraft::world::level::chunk_pos::ChunkPos;
    use crate::net::minecraft::world::phys::aabb::AABB;
    use crate::net::minecraft::world::phys::vec3::Vec3;

    fn settings(value: &Value) -> Settings {
        let fields = value.as_array().expect("the settings' fields");
        Settings {
            center_x: f64::decode(&fields[0]),
            center_z: f64::decode(&fields[1]),
            damage_per_block: f64::decode(&fields[2]),
            safe_zone: f64::decode(&fields[3]),
            warning_blocks: i32::decode(&fields[4]),
            warning_time: i32::decode(&fields[5]),
            size: f64::decode(&fields[6]),
            lerp_time: i64::decode(&fields[7]),
            lerp_target: f64::decode(&fields[8]),
        }
    }

    fn encode_settings(settings: &Settings) -> Value {
        Value::from(vec![
            settings.center_x.encode(),
            settings.center_z.encode(),
            settings.damage_per_block.encode(),
            settings.safe_zone.encode(),
            settings.warning_blocks.encode(),
            settings.warning_time.encode(),
            settings.size.encode(),
            settings.lerp_time.encode(),
            settings.lerp_target.encode(),
        ])
    }

    fn state(border: &WorldBorder, partial: f32) -> Value {
        let status = border.get_status();
        Value::from(vec![
            border.get_min_x_partial(partial).encode(),
            border.get_min_z_partial(partial).encode(),
            border.get_max_x_partial(partial).encode(),
            border.get_max_z_partial(partial).encode(),
            border.get_min_x().encode(),
            border.get_min_z().encode(),
            border.get_max_x().encode(),
            border.get_max_z().encode(),
            border.get_center_x().encode(),
            border.get_center_z().encode(),
            border.get_size().encode(),
            border.get_lerp_time().encode(),
            border.get_lerp_target().encode(),
            border.get_lerp_speed().encode(),
            Value::from(format!("{status:?}")),
            status.get_color().encode(),
            border.get_absolute_max_size().encode(),
            border.get_warning_time().encode(),
            border.get_warning_blocks().encode(),
            border.get_safe_zone().encode(),
            border.get_damage_per_block().encode(),
            encode_settings(&Settings::from_border(border)),
        ])
    }

    fn block(pos: &BlockPos) -> Value {
        Value::from(vec![pos.get_x(), pos.get_y(), pos.get_z()])
    }

    fn point(border: &WorldBorder, x: f64, z: f64, margin: f64) -> Value {
        let clamped = border.clamp_to_bounds_xyz(x, 64.5, z);
        let clamped_vec = border.clamp_vec3_to_bound(&Vec3::new(x, 64.5, z));
        let pos = BlockPos::containing_xyz(x, 64.0, z);
        let chunk = ChunkPos::containing(&pos);
        let r#box = AABB::new(x - 0.3, 64.0, z - 0.3, x + 0.3, 65.8, z + 0.3);
        Value::from(vec![
            border.is_within_bounds(x, z).encode(),
            border.is_within_bounds_margin(x, z, margin).encode(),
            border.is_within_bounds_block_pos(&pos).encode(),
            border
                .is_within_bounds_vec3(&Vec3::new(x, 64.0, z))
                .encode(),
            border.is_within_bounds_chunk_pos(&chunk).encode(),
            border.is_within_bounds_aabb(&r#box).encode(),
            border.get_distance_to_border(x, z).encode(),
            block(&clamped),
            Value::from(vec![
                clamped_vec.x.encode(),
                clamped_vec.y.encode(),
                clamped_vec.z.encode(),
            ]),
            block(&border.clamp_to_bounds_block_pos(&pos)),
        ])
    }

    fn run(settings_value: &Value, ops: &Value) -> Value {
        let mut border = if settings_value.is_null() {
            WorldBorder::new()
        } else {
            WorldBorder::from_settings(settings(settings_value))
        };
        let outputs = ops
            .as_array()
            .expect("ops")
            .iter()
            .map(|op| {
                let op = op.as_array().expect("an op is an array");
                let double = |i: usize| f64::decode(&op[i]);
                match op[0].as_str().expect("an op's name") {
                    "setCenter" => border.set_center(double(1), double(2)),
                    "setSize" => border.set_size(double(1)),
                    "lerpSizeBetween" => border.lerp_size_between(
                        double(1),
                        double(2),
                        i64::decode(&op[3]),
                        i64::decode(&op[4]),
                    ),
                    "tick" => border.tick(),
                    "setAbsoluteMaxSize" => border.set_absolute_max_size(i32::decode(&op[1])),
                    "setWarningTime" => border.set_warning_time(i32::decode(&op[1])),
                    "setWarningBlocks" => border.set_warning_blocks(i32::decode(&op[1])),
                    "setSafeZone" => border.set_safe_zone(double(1)),
                    "setDamagePerBlock" => border.set_damage_per_block(double(1)),
                    "applyInitialSettings" => border.apply_initial_settings(i64::decode(&op[1])),
                    "state" => return state(&border, f32::decode(&op[1])),
                    "point" => return point(&border, double(1), double(2), double(3)),
                    "shape" => {
                        return border
                            .get_collision_shape()
                            .to_aabbs()
                            .iter()
                            .map(|r#box| {
                                Value::from(vec![
                                    r#box.min_x.encode(),
                                    r#box.min_y.encode(),
                                    r#box.min_z.encode(),
                                    r#box.max_x.encode(),
                                    r#box.max_y.encode(),
                                    r#box.max_z.encode(),
                                ])
                            })
                            .collect();
                    }
                    name => panic!("unknown op {name}"),
                }
                Value::Null
            })
            .collect::<Vec<_>>();
        Value::from(outputs)
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.world.level.border.WorldBorder");
        golden.check("programs", |settings: Value, ops: Value| {
            run(&settings, &ops)
        });
        golden.finish();
    }
}
