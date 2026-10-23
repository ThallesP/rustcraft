//! net/minecraft/client/Camera.java
//!
//! The first-person camera at the player's eye: its position and rotation, field of view,
//! projection and culling frustum, and what it's in (`getFluidInCamera`, which narrows the field
//! of view in water and lava). Not ported yet: the third-person cameras (`detached`, `getMaxZoom`,
//! which clips against the level), riding a minecart, sleeping, the panoramic mode, a captured
//! frustum, waypoints, and the dying player's field of view. Java's back-references (the level, the
//! entity, `Minecraft`) are what `update` and `tick` are given.

use crate::java::lang::math::Math;
use crate::net::minecraft::client::player::local_player::LocalPlayer;
use crate::net::minecraft::client::renderer::culling::frustum::Frustum;
use crate::net::minecraft::client::renderer::projection::Projection;
use crate::net::minecraft::client::renderer::state::level::camera_render_state::CameraRenderState;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::tags::fluid_tags::FluidTags;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::attribute::environment_attribute_layer::LayerLevel;
use crate::net::minecraft::world::attribute::environment_attribute_probe::EnvironmentAttributeProbe;
use crate::net::minecraft::world::entity::ai::attributes::attributes::Attributes;
use crate::net::minecraft::world::level::block::blocks::Blocks;
use crate::net::minecraft::world::level::block_getter::BlockGetter;
use crate::net::minecraft::world::level::material::fog_type::FogType;
use crate::net::minecraft::world::phys::vec3::Vec3;
use crate::org::joml::matrix4f::Matrix4f;
use crate::org::joml::quaternionf::Quaternionf;
use crate::org::joml::vector3f::Vector3f;

const FORWARDS: Vector3f = Vector3f::from_xyz(0.0, 0.0, -1.0);
const UP: Vector3f = Vector3f::from_xyz(0.0, 1.0, 0.0);
const LEFT: Vector3f = Vector3f::from_xyz(-1.0, 0.0, 0.0);

/// What `update` reads of the options and the window.
#[derive(Clone, Copy, Debug)]
pub struct CameraOptions {
    /// `options.getEffectiveRenderDistance()`.
    pub render_distance: i32,
    /// `options.cloudRange().get()`.
    pub cloud_range: i32,
    /// `options.fov().get()`.
    pub fov: i32,
    /// `options.fovEffectScale().get()`.
    pub fov_effect_scale: f32,
    pub window_width: i32,
    pub window_height: i32,
    /// The device's `isZZeroToOne()`.
    pub z_zero_to_one: bool,
}

pub struct Camera {
    initialized: bool,
    has_level: bool,
    position: Vec3,
    block_position: BlockPos,
    forwards: Vector3f,
    up: Vector3f,
    left: Vector3f,
    x_rot: f32,
    y_rot: f32,
    rotation: Quaternionf,
    eye_height: f32,
    eye_height_old: f32,
    projection: Projection,
    cull_frustum: Frustum,
    cached_view_rot_matrix: Matrix4f,
    matrix_properties_dirty: i32,
    fov_modifier: f32,
    old_fov_modifier: f32,
    fov: f32,
    hud_fov: f32,
    depth_far: f32,
    attribute_probe: EnvironmentAttributeProbe,
}

impl Default for Camera {
    fn default() -> Camera {
        Camera::new()
    }
}

impl Camera {
    pub const PROJECTION_Z_NEAR: f32 = 0.05;
    pub const BASE_HUD_FOV: f32 = 70.0;
    const DIRTY_VIEW_ROT: i32 = 1;

    pub fn new() -> Camera {
        Camera {
            initialized: false,
            has_level: false,
            position: Vec3::ZERO,
            block_position: BlockPos::ZERO,
            forwards: FORWARDS,
            up: UP,
            left: LEFT,
            x_rot: 0.0,
            y_rot: 0.0,
            rotation: Quaternionf::new(),
            eye_height: 0.0,
            eye_height_old: 0.0,
            projection: Projection::new(),
            cull_frustum: Frustum::new(&Matrix4f::new(), &Matrix4f::new()),
            cached_view_rot_matrix: Matrix4f::new(),
            matrix_properties_dirty: -1,
            fov_modifier: 0.0,
            old_fov_modifier: 0.0,
            fov: 0.0,
            hud_fov: 0.0,
            depth_far: 0.0,
            attribute_probe: EnvironmentAttributeProbe::new(),
        }
    }

    /// `tick()`, once a client tick: the eye height eases toward the entity's, the attribute
    /// probe samples the biomes where the camera was last put, and the field of view's modifier
    /// eases toward the player's. `player` is the camera's entity, and `level` what the
    /// attributes read of the level, when there's one.
    pub fn tick(
        &mut self,
        player: Option<&LocalPlayer>,
        level: Option<&dyn LayerLevel>,
        fov_effect_scale: f32,
    ) {
        if self.has_level
            && let (Some(player), Some(level)) = (player, level)
        {
            self.eye_height_old = self.eye_height;
            self.eye_height += (player.get_eye_height() - self.eye_height) * 0.5;
            self.attribute_probe.tick(level, self.position);
        }

        self.tick_fov(player, fov_effect_scale);
    }

    /// `setLevel`: whether there's a level, which is what the camera asks of it.
    pub fn set_level(&mut self, has_level: bool) {
        self.has_level = has_level;
    }

    /// `update(DeltaTracker)`: at the player's eye at `partial_ticks`, with its field of view,
    /// culling frustum and projection.
    pub fn update(
        &mut self,
        player: Option<&LocalPlayer>,
        level: Option<&dyn BlockGetter>,
        partial_ticks: f32,
        options: &CameraOptions,
    ) {
        let render_distance = (options.render_distance * 16) as f32;
        self.depth_far = Math::max_f32(render_distance * 4.0, (options.cloud_range * 16) as f32);
        if let Some(player) = player
            && self.has_level
        {
            self.align_with_entity(player, partial_ticks);
            self.fov = self.calculate_fov(level, partial_ticks, options);
            self.hud_fov = self.modify_fov_based_on_fluid(level, Camera::BASE_HUD_FOV, options);
            let mut view_rotation = Matrix4f::new();
            self.get_view_rotation_matrix(&mut view_rotation);
            let projection_for_culling = self.create_projection_matrix_for_culling(options);
            self.prepare_cull_frustum(&view_rotation, &projection_for_culling);
            self.projection.setup_perspective(
                Camera::PROJECTION_Z_NEAR,
                self.depth_far,
                self.fov,
                options.window_width as f32,
                options.window_height as f32,
            );
            self.initialized = true;
        }
    }

    /// `extractRenderState`, for the first-person camera.
    pub fn extract_render_state(
        &mut self,
        camera_state: &mut CameraRenderState,
        partial_ticks: f32,
        z_zero_to_one: bool,
    ) {
        camera_state.camera_entity_partial_ticks = partial_ticks;
        camera_state.initialized = self.initialized;
        camera_state.is_frustum_captured = false;
        camera_state.is_first_person = true;
        camera_state.smart_cull = true;
        camera_state.pos = self.position;
        camera_state.x_rot = self.x_rot;
        camera_state.y_rot = self.y_rot;
        camera_state.block_pos = self.block_position;
        camera_state.orientation = self.rotation;
        camera_state.cull_frustum = self.cull_frustum.clone();
        camera_state.depth_far = self.depth_far;
        self.projection
            .get_matrix(&mut camera_state.projection_matrix, z_zero_to_one);
        self.get_view_rotation_matrix(&mut camera_state.view_rotation_matrix);
        camera_state.hud_fov = self.hud_fov;
    }

    /// `AbstractClientPlayer.getFieldOfViewModifier` for a first-person camera, without a bow or
    /// a spyglass in use (items aren't ported).
    fn tick_fov(&mut self, player: Option<&LocalPlayer>, effect_scale: f32) {
        let target_fov_modifier = match player {
            Some(player) => {
                let mut modifier = 1.0f32;
                if player.get_abilities().flying {
                    modifier *= 1.1;
                }

                let walking_speed = player.get_abilities().get_walking_speed();
                if walking_speed != 0.0 {
                    let speed_factor = player.get_attribute_value(Attributes::MOVEMENT_SPEED)
                        as f32
                        / walking_speed;
                    modifier *= (speed_factor + 1.0) / 2.0;
                }

                Mth::lerp_f32(effect_scale, 1.0, modifier)
            }
            None => 1.0,
        };
        self.old_fov_modifier = self.fov_modifier;
        self.fov_modifier += (target_fov_modifier - self.fov_modifier) * 0.5;
        self.fov_modifier = Mth::clamp_f32(self.fov_modifier, 0.1, 1.5);
    }

    fn create_projection_matrix_for_culling(&self, options: &CameraOptions) -> Matrix4f {
        let fov_for_culling = Math::max_f32(self.fov, options.fov as f32);
        let mut projection = Matrix4f::new();
        projection.perspective(
            fov_for_culling * (std::f64::consts::PI / 180.0) as f32,
            options.window_width as f32 / options.window_height as f32,
            0.05,
            self.depth_far,
            options.z_zero_to_one,
        );
        projection
    }

    pub fn get_cull_frustum(&self) -> &Frustum {
        &self.cull_frustum
    }

    fn prepare_cull_frustum(
        &mut self,
        model_view_matrix: &Matrix4f,
        projection_matrix_for_culling: &Matrix4f,
    ) {
        self.cull_frustum = Frustum::new(model_view_matrix, projection_matrix_for_culling);
        self.cull_frustum
            .prepare(self.position.x, self.position.y, self.position.z);
    }

    fn calculate_fov(
        &self,
        level: Option<&dyn BlockGetter>,
        partial_ticks: f32,
        options: &CameraOptions,
    ) -> f32 {
        let fov = options.fov as f32
            * Mth::lerp_f32(partial_ticks, self.old_fov_modifier, self.fov_modifier);
        self.modify_fov_based_on_fluid(level, fov, options)
    }

    /// `modifyFovBasedOnDeathOrFluid`: narrower in water or lava, by the options' effect scale.
    /// The player's dying isn't ported, so it never narrows for that.
    fn modify_fov_based_on_fluid(
        &self,
        level: Option<&dyn BlockGetter>,
        mut fov: f32,
        options: &CameraOptions,
    ) -> f32 {
        let state = level.map_or(FogType::NONE, |level| {
            self.get_fluid_in_camera(level, options.fov)
        });
        if state == FogType::LAVA || state == FogType::WATER {
            fov *= Mth::lerp_f32(options.fov_effect_scale, 1.0, 0.85714287);
        }

        fov
    }

    /// The first-person position: the entity's, interpolated, at its interpolated eye height.
    fn align_with_entity(&mut self, entity: &LocalPlayer, partial_ticks: f32) {
        // LocalPlayer.getViewYRot and Entity.getViewXRot: getYRot(a) and getXRot(a).
        let (y_rot, x_rot) = if partial_ticks == 1.0 {
            (entity.get_y_rot(), entity.get_x_rot())
        } else {
            (
                Mth::rot_lerp_f32(partial_ticks, entity.y_rot_o, entity.get_y_rot()),
                Mth::lerp_f32(partial_ticks, entity.x_rot_o, entity.get_x_rot()),
            )
        };
        self.set_rotation(y_rot, x_rot);
        let old = entity.old_position();
        self.set_position(Vec3::new(
            Mth::lerp_f64(f64::from(partial_ticks), old.x, entity.get_x()),
            Mth::lerp_f64(f64::from(partial_ticks), old.y, entity.get_y())
                + f64::from(Mth::lerp_f32(
                    partial_ticks,
                    self.eye_height_old,
                    self.eye_height,
                )),
            Mth::lerp_f64(f64::from(partial_ticks), old.z, entity.get_z()),
        ));
    }

    pub fn get_fov(&self) -> f32 {
        self.fov
    }

    fn set_rotation(&mut self, y_rot: f32, x_rot: f32) {
        self.x_rot = x_rot;
        self.y_rot = y_rot;
        self.rotation.rotation_yxz(
            std::f32::consts::PI - y_rot * (std::f64::consts::PI / 180.0) as f32,
            -x_rot * (std::f64::consts::PI / 180.0) as f32,
            0.0,
        );
        self.rotation.transform_into(&FORWARDS, &mut self.forwards);
        self.rotation.transform_into(&UP, &mut self.up);
        self.rotation.transform_into(&LEFT, &mut self.left);
        self.matrix_properties_dirty |= 3;
    }

    fn set_position(&mut self, position: Vec3) {
        self.position = position;
        self.block_position = BlockPos::containing_xyz(position.x, position.y, position.z);
    }

    pub fn position(&self) -> Vec3 {
        self.position
    }

    pub fn block_position(&self) -> BlockPos {
        self.block_position
    }

    pub fn x_rot(&self) -> f32 {
        self.x_rot
    }

    pub fn y_rot(&self) -> f32 {
        self.y_rot
    }

    pub fn rotation(&self) -> &Quaternionf {
        &self.rotation
    }

    /// The rotation's inverse as a matrix, cached until the rotation changes.
    pub fn get_view_rotation_matrix<'a>(&mut self, dest: &'a mut Matrix4f) -> &'a mut Matrix4f {
        if self.matrix_properties_dirty & Camera::DIRTY_VIEW_ROT != 0 {
            let mut inverse_rotation = Quaternionf::new();
            self.rotation.conjugate_into(&mut inverse_rotation);
            self.cached_view_rot_matrix.rotation(&inverse_rotation);
            self.matrix_properties_dirty &= -2;
        }

        dest.set(&self.cached_view_rot_matrix)
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// `getNearPlane`: the near plane's middle and half its sides, from the camera, at the
    /// projection the camera last had.
    pub fn get_near_plane(&self, fov: f32) -> NearPlane {
        let aspect_ratio = f64::from(self.projection.width()) / f64::from(self.projection.height());
        let plane_height = Math::tan(f64::from(fov * (std::f64::consts::PI / 180.0) as f32) / 2.0)
            * f64::from(self.projection.z_near());
        let plane_width = plane_height * aspect_ratio;
        let forwards =
            Vec3::from_vector3f(&self.forwards).scale(f64::from(self.projection.z_near()));
        let left = Vec3::from_vector3f(&self.left).scale(plane_width);
        let up = Vec3::from_vector3f(&self.up).scale(plane_height);
        NearPlane {
            forward: forwards,
            left,
            up,
        }
    }

    /// What the camera is in: water at the camera under its surface, else lava at the near
    /// plane's middle or a corner under its surface, else powder snow there. `fov` is the
    /// options', which Java reads from the game.
    pub fn get_fluid_in_camera(&self, level: &dyn BlockGetter, fov: i32) -> FogType {
        if !self.initialized {
            return FogType::NONE;
        }

        let fluid_state1 = level.get_fluid_state(&self.block_position);
        if fluid_state1.is_tag(FluidTags::WATER)
            && self.position.y
                < f64::from(self.block_position.get_y())
                    + f64::from(fluid_state1.get_height_for_camera(level, &self.block_position))
        {
            return FogType::WATER;
        }

        let plane = self.get_near_plane(fov as f32);
        for point in [
            plane.forward,
            plane.get_top_left(),
            plane.get_top_right(),
            plane.get_bottom_left(),
            plane.get_bottom_right(),
        ] {
            let offset_pos = self.position.add(&point);
            let check_pos = BlockPos::containing_xyz(offset_pos.x, offset_pos.y, offset_pos.z);
            let fluid_state = level.get_fluid_state(&check_pos);
            if fluid_state.is_tag(FluidTags::LAVA) {
                if offset_pos.y
                    <= f64::from(fluid_state.get_height_for_camera(level, &check_pos))
                        + f64::from(check_pos.get_y())
                {
                    return FogType::LAVA;
                }
            } else if level.get_block_state(&check_pos).is(Blocks::POWDER_SNOW) {
                return FogType::POWDER_SNOW;
            }
        }

        FogType::NONE
    }

    pub fn forward_vector(&self) -> &Vector3f {
        &self.forwards
    }

    pub fn up_vector(&self) -> &Vector3f {
        &self.up
    }

    pub fn left_vector(&self) -> &Vector3f {
        &self.left
    }

    pub fn attribute_probe_mut(&mut self) -> &mut EnvironmentAttributeProbe {
        &mut self.attribute_probe
    }

    /// `reset()`: no level or entity, not initialized.
    pub fn reset(&mut self) {
        self.has_level = false;
        self.attribute_probe.reset();
        self.initialized = false;
    }
}

/// `Camera.NearPlane`.
pub struct NearPlane {
    forward: Vec3,
    left: Vec3,
    up: Vec3,
}

impl NearPlane {
    pub fn get_top_left(&self) -> Vec3 {
        self.forward.add(&self.up).add(&self.left)
    }

    pub fn get_top_right(&self) -> Vec3 {
        self.forward.add(&self.up).subtract(&self.left)
    }

    pub fn get_bottom_left(&self) -> Vec3 {
        self.forward.subtract(&self.up).add(&self.left)
    }

    pub fn get_bottom_right(&self) -> Vec3 {
        self.forward.subtract(&self.up).subtract(&self.left)
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use serde_json::{Value, json};

    use super::Camera;
    use crate::golden::Golden;
    use crate::net::minecraft::client::renderer::culling::frustum::Frustum;
    use crate::net::minecraft::world::level::levelgen::structure::bounding_box::BoundingBox;
    use crate::net::minecraft::world::phys::aabb::AABB;
    use crate::org::joml::matrix4f::Matrix4f;
    use crate::org::joml::vector3f::Vector3f;

    fn float(value: f32) -> Value {
        Value::from(format!("{:08x}", value.to_bits()))
    }

    fn double(value: f64) -> Value {
        Value::from(format!("{:016x}", value.to_bits()))
    }

    fn matrix(matrix: &Matrix4f) -> Value {
        let mut values: Vec<Value> = matrix.get_column_major().into_iter().map(float).collect();
        values.push(Value::from(matrix.properties()));
        Value::Array(values)
    }

    fn vector(vector: &Vector3f) -> Value {
        json!([float(vector.x), float(vector.y), float(vector.z)])
    }

    pub(crate) fn camera(y_rot: f32, x_rot: f32) -> Camera {
        let mut camera = Camera::new();
        camera.set_rotation(y_rot, x_rot);
        camera
    }

    /// The camera's rotation and its view rotation matrix like vanilla's (`CameraSuites`).
    #[test]
    fn rotates_like_vanilla() {
        let golden = Golden::load("net.minecraft.client.Camera");
        golden.check("setRotation", |y_rot: f32, x_rot: f32| {
            let mut camera = camera(y_rot, x_rot);
            let mut view_rotation = Matrix4f::new();
            camera.get_view_rotation_matrix(&mut view_rotation);
            json!([
                matrix(&view_rotation),
                vector(camera.forward_vector()),
                vector(camera.up_vector()),
                vector(camera.left_vector())
            ])
        });
        golden.finish();
    }

    /// `Camera.update`'s culling frustum, tested against the sections around the camera, and
    /// moved back to take in the camera's cube, like vanilla's (`CameraSuites`).
    #[test]
    fn culls_like_vanilla() {
        let golden = Golden::load("net.minecraft.client.renderer.culling.Frustum");
        golden.check(
            "cull",
            |x: f64,
             y: f64,
             z: f64,
             y_rot: f32,
             x_rot: f32,
             fov: f32,
             aspect: f32,
             depth_far: f32| {
                let mut camera = camera(y_rot, x_rot);
                let mut projection = Matrix4f::new();
                projection.perspective(
                    fov * (std::f64::consts::PI / 180.0) as f32,
                    aspect,
                    0.05,
                    depth_far,
                    true,
                );
                let mut view_rotation = Matrix4f::new();
                camera.get_view_rotation_matrix(&mut view_rotation);
                let mut frustum = Frustum::new(&view_rotation, &projection);
                frustum.prepare(x, y, z);
                let (section_x, section_y, section_z) = (
                    (x.floor() as i32).div_euclid(16),
                    (y.floor() as i32).div_euclid(16),
                    (z.floor() as i32).div_euclid(16),
                );
                let mut cubes = Vec::new();
                let mut boxes = Vec::new();
                for dx in -4..=4 {
                    for dy in -2..=2 {
                        for dz in -4..=4 {
                            let (min_x, min_y, min_z) = (
                                (section_x + dx) * 16,
                                (section_y + dy) * 16,
                                (section_z + dz) * 16,
                            );
                            cubes.push(Value::from(frustum.cube_in_frustum(&BoundingBox::new(
                                min_x,
                                min_y,
                                min_z,
                                min_x + 15,
                                min_y + 15,
                                min_z + 15,
                            ))));
                            boxes.push(Value::from(frustum.is_visible(&AABB::new(
                                f64::from(min_x) + 0.33,
                                f64::from(min_y) + 0.33,
                                f64::from(min_z) + 0.33,
                                f64::from(min_x) + 16.33,
                                f64::from(min_y) + 16.33,
                                f64::from(min_z) + 16.33,
                            ))));
                        }
                    }
                }
                let mut offset = frustum.clone();
                offset.offset_to_fully_include_camera_cube(8);
                json!([
                    cubes,
                    boxes,
                    [
                        double(offset.get_cam_x()),
                        double(offset.get_cam_y()),
                        double(offset.get_cam_z())
                    ]
                ])
            },
        );
        golden.finish();
    }
}
