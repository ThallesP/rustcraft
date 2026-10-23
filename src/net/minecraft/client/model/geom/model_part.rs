//! net/minecraft/client/model/geom/ModelPart.java
//!
//! A baked part keeps its children in the order its definition's `HashMap` iterated them
//! (Java's `Object2ObjectArrayMap`), a list of names and parts here. Not ported yet: `rotateBy`,
//! `getExtentsForGui` and `visit` (`Visitor`), `getRandomCube`, `createPartLookup` and the
//! `offset…` methods, which other models and the GUI's entity previews use.

use crate::com::mojang::blaze3d::vertex::pose_stack::{Pose, PoseStack};
use crate::com::mojang::blaze3d::vertex::vertex_consumer::VertexConsumer;
use crate::net::minecraft::client::model::geom::part_pose::PartPose;
use crate::net::minecraft::core::direction::{Axis, Direction};
use crate::org::joml::quaternionf::Quaternionf;
use crate::org::joml::vector3f::Vector3f;

/// A part of a model: its cubes, its children, and its pose, which animations change and
/// `reset_pose` sets back.
#[derive(Clone, Debug)]
pub struct ModelPart {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub x_rot: f32,
    pub y_rot: f32,
    pub z_rot: f32,
    pub x_scale: f32,
    pub y_scale: f32,
    pub z_scale: f32,
    pub visible: bool,
    pub skip_draw: bool,
    cubes: Vec<Cube>,
    children: Vec<(String, ModelPart)>,
    initial_pose: PartPose,
}

impl ModelPart {
    pub const DEFAULT_SCALE: f32 = 1.0;

    pub fn new(cubes: Vec<Cube>, children: Vec<(String, ModelPart)>) -> ModelPart {
        ModelPart {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            x_rot: 0.0,
            y_rot: 0.0,
            z_rot: 0.0,
            x_scale: 1.0,
            y_scale: 1.0,
            z_scale: 1.0,
            visible: true,
            skip_draw: false,
            cubes,
            children,
            initial_pose: PartPose::ZERO,
        }
    }

    pub fn store_pose(&self) -> PartPose {
        PartPose::offset_and_rotation(self.x, self.y, self.z, self.x_rot, self.y_rot, self.z_rot)
    }

    pub fn get_initial_pose(&self) -> PartPose {
        self.initial_pose
    }

    pub fn set_initial_pose(&mut self, initial_pose: PartPose) {
        self.initial_pose = initial_pose;
    }

    pub fn reset_pose(&mut self) {
        let initial_pose = self.initial_pose;
        self.load_pose(&initial_pose);
    }

    pub fn load_pose(&mut self, pose: &PartPose) {
        self.x = pose.x;
        self.y = pose.y;
        self.z = pose.z;
        self.x_rot = pose.x_rot;
        self.y_rot = pose.y_rot;
        self.z_rot = pose.z_rot;
        self.x_scale = pose.x_scale;
        self.y_scale = pose.y_scale;
        self.z_scale = pose.z_scale;
    }

    pub fn has_child(&self, name: &str) -> bool {
        self.children.iter().any(|(child, _)| child == name)
    }

    /// Panics where Java throws `NoSuchElementException`.
    pub fn get_child(&self, name: &str) -> &ModelPart {
        self.children
            .iter()
            .find(|(child, _)| child == name)
            .map(|(_, part)| part)
            .unwrap_or_else(|| panic!("NoSuchElementException: Can't find part {name}"))
    }

    pub fn get_child_mut(&mut self, name: &str) -> &mut ModelPart {
        self.children
            .iter_mut()
            .find(|(child, _)| child == name)
            .map(|(_, part)| part)
            .unwrap_or_else(|| panic!("NoSuchElementException: Can't find part {name}"))
    }

    pub fn children(&self) -> impl Iterator<Item = (&str, &ModelPart)> {
        self.children
            .iter()
            .map(|(name, part)| (name.as_str(), part))
    }

    pub fn set_pos(&mut self, x: f32, y: f32, z: f32) {
        self.x = x;
        self.y = y;
        self.z = z;
    }

    pub fn set_rotation(&mut self, x_rot: f32, y_rot: f32, z_rot: f32) {
        self.x_rot = x_rot;
        self.y_rot = y_rot;
        self.z_rot = z_rot;
    }

    /// `render(poseStack, buffer, lightCoords, overlayCoords)`: white.
    pub fn render(
        &self,
        pose_stack: &mut PoseStack,
        buffer: &mut dyn VertexConsumer,
        light_coords: i32,
        overlay_coords: i32,
    ) {
        self.render_colored(pose_stack, buffer, light_coords, overlay_coords, -1);
    }

    /// The part's cubes at its pose, then its children's, unless it's hidden.
    pub fn render_colored(
        &self,
        pose_stack: &mut PoseStack,
        buffer: &mut dyn VertexConsumer,
        light_coords: i32,
        overlay_coords: i32,
        color: i32,
    ) {
        if !self.visible || (self.cubes.is_empty() && self.children.is_empty()) {
            return;
        }
        pose_stack.push_pose();
        self.translate_and_rotate(pose_stack);
        if !self.skip_draw {
            self.compile(
                pose_stack.last(),
                buffer,
                light_coords,
                overlay_coords,
                color,
            );
        }

        for (_, child) in &self.children {
            child.render_colored(pose_stack, buffer, light_coords, overlay_coords, color);
        }

        pose_stack.pop_pose();
    }

    pub fn translate_and_rotate(&self, pose_stack: &mut PoseStack) {
        pose_stack.translate_f32(self.x / 16.0, self.y / 16.0, self.z / 16.0);
        if self.x_rot != 0.0 || self.y_rot != 0.0 || self.z_rot != 0.0 {
            pose_stack.rotate(Quaternionf::new().rotation_zyx(self.z_rot, self.y_rot, self.x_rot));
        }

        if self.x_scale != 1.0 || self.y_scale != 1.0 || self.z_scale != 1.0 {
            pose_stack.scale(self.x_scale, self.y_scale, self.z_scale);
        }
    }

    fn compile(
        &self,
        pose: &Pose,
        builder: &mut dyn VertexConsumer,
        light_coords: i32,
        overlay_coords: i32,
        color: i32,
    ) {
        for cube in &self.cubes {
            cube.compile(pose, builder, light_coords, overlay_coords, color);
        }
    }

    pub fn is_empty(&self) -> bool {
        self.cubes.is_empty()
    }

    /// `getAllParts`: this part, then its children by name, depth first by level (each part's
    /// children before their own children).
    pub fn get_all_parts(&self) -> Vec<&ModelPart> {
        let mut all_parts = vec![self];
        self.add_all_children(&mut all_parts);
        all_parts
    }

    fn add_all_children<'a>(&'a self, output: &mut Vec<&'a ModelPart>) {
        for (_, part) in &self.children {
            output.push(part);
        }

        for (_, part) in &self.children {
            part.add_all_children(output);
        }
    }

    /// Every part's pose back to its initial pose, like `Model.resetPose` over `getAllParts`.
    pub fn reset_all_poses(&mut self) {
        self.reset_pose();
        for (_, child) in &mut self.children {
            child.reset_all_poses();
        }
    }
}

/// `ModelPart.Cube`: a box's polygons, one for each visible face.
#[derive(Clone, Debug)]
pub struct Cube {
    pub polygons: Vec<Polygon>,
    pub min_x: f32,
    pub min_y: f32,
    pub min_z: f32,
    pub max_x: f32,
    pub max_y: f32,
    pub max_z: f32,
}

impl Cube {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        x_tex_offs: i32,
        y_tex_offs: i32,
        mut min_x: f32,
        mut min_y: f32,
        mut min_z: f32,
        width: f32,
        height: f32,
        depth: f32,
        grow_x: f32,
        grow_y: f32,
        grow_z: f32,
        mirror: bool,
        x_tex_size: f32,
        y_tex_size: f32,
        visible_faces: &[Direction],
    ) -> Cube {
        let field_min = (min_x, min_y, min_z);
        let field_max = (min_x + width, min_y + height, min_z + depth);
        let mut max_x = min_x + width;
        let mut max_y = min_y + height;
        let mut max_z = min_z + depth;
        min_x -= grow_x;
        min_y -= grow_y;
        min_z -= grow_z;
        max_x += grow_x;
        max_y += grow_y;
        max_z += grow_z;
        if mirror {
            std::mem::swap(&mut min_x, &mut max_x);
        }

        let t0 = Vertex::new(min_x, min_y, min_z, 0.0, 0.0);
        let t1 = Vertex::new(max_x, min_y, min_z, 0.0, 8.0);
        let t2 = Vertex::new(max_x, max_y, min_z, 8.0, 8.0);
        let t3 = Vertex::new(min_x, max_y, min_z, 8.0, 0.0);
        let l0 = Vertex::new(min_x, min_y, max_z, 0.0, 0.0);
        let l1 = Vertex::new(max_x, min_y, max_z, 0.0, 8.0);
        let l2 = Vertex::new(max_x, max_y, max_z, 8.0, 8.0);
        let l3 = Vertex::new(min_x, max_y, max_z, 8.0, 0.0);
        let x_tex_offs = x_tex_offs as f32;
        let y_tex_offs = y_tex_offs as f32;
        let u0 = x_tex_offs;
        let u1 = x_tex_offs + depth;
        let u2 = x_tex_offs + depth + width;
        let u22 = x_tex_offs + depth + width + width;
        let u3 = x_tex_offs + depth + width + depth;
        let u4 = x_tex_offs + depth + width + depth + width;
        let v0 = y_tex_offs;
        let v1 = y_tex_offs + depth;
        let v2 = y_tex_offs + depth + height;
        let size = (x_tex_size, y_tex_size);
        let mut polygons = Vec::with_capacity(visible_faces.len());
        let faces: [(Direction, [Vertex; 4], [f32; 4]); 6] = [
            (Direction::DOWN, [l1, l0, t0, t1], [u1, v0, u2, v1]),
            (Direction::UP, [t2, t3, l3, l2], [u2, v1, u22, v0]),
            (Direction::WEST, [t0, l0, l3, t3], [u0, v1, u1, v2]),
            (Direction::NORTH, [t1, t0, t3, t2], [u1, v1, u2, v2]),
            (Direction::EAST, [l1, t1, t2, l2], [u2, v1, u3, v2]),
            (Direction::SOUTH, [l0, l1, l2, l3], [u3, v1, u4, v2]),
        ];
        for (facing, vertices, [pu0, pv0, pu1, pv1]) in faces {
            if visible_faces.contains(&facing) {
                polygons.push(Polygon::new(
                    vertices, pu0, pv0, pu1, pv1, size.0, size.1, mirror, facing,
                ));
            }
        }

        Cube {
            polygons,
            min_x: field_min.0,
            min_y: field_min.1,
            min_z: field_min.2,
            max_x: field_max.0,
            max_y: field_max.1,
            max_z: field_max.2,
        }
    }

    /// Each polygon's vertices at the pose, with the polygon's normal through the pose's normal
    /// matrix.
    pub fn compile(
        &self,
        pose: &Pose,
        builder: &mut dyn VertexConsumer,
        light_coords: i32,
        overlay_coords: i32,
        color: i32,
    ) {
        let matrix = pose.pose();
        let mut scratch_vector = Vector3f::new();

        for polygon in &self.polygons {
            let normal = pose.transform_normal(&polygon.normal, &mut scratch_vector);
            let (nx, ny, nz) = (normal.x, normal.y, normal.z);

            for vertex in &polygon.vertices {
                let pos = matrix.transform_position_xyz_into(
                    vertex.world_x(),
                    vertex.world_y(),
                    vertex.world_z(),
                    &mut scratch_vector,
                );
                builder.add_vertex_full(
                    pos.x,
                    pos.y,
                    pos.z,
                    color,
                    vertex.u,
                    vertex.v,
                    overlay_coords,
                    light_coords,
                    nx,
                    ny,
                    nz,
                );
            }
        }
    }
}

/// `ModelPart.Polygon`: a face's four vertices, with their texture coordinates, and its normal.
#[derive(Clone, Debug)]
pub struct Polygon {
    pub vertices: [Vertex; 4],
    pub normal: Vector3f,
}

impl Polygon {
    /// The face's vertices with the texture's rectangle `(u0, v0)` to `(u1, v1)` mapped onto
    /// them, in reverse order when mirrored, whose normal then faces the other way along X.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        mut vertices: [Vertex; 4],
        u0: f32,
        v0: f32,
        u1: f32,
        v1: f32,
        x_tex_size: f32,
        y_tex_size: f32,
        mirror: bool,
        facing: Direction,
    ) -> Polygon {
        let facing = if mirror {
            Polygon::mirror_facing(facing)
        } else {
            facing
        };
        let normal = *facing.get_unit_vec3f();
        let us = 0.0 / x_tex_size;
        let vs = 0.0 / y_tex_size;
        vertices[0] = vertices[0].remap(u1 / x_tex_size - us, v0 / y_tex_size + vs);
        vertices[1] = vertices[1].remap(u0 / x_tex_size + us, v0 / y_tex_size + vs);
        vertices[2] = vertices[2].remap(u0 / x_tex_size + us, v1 / y_tex_size - vs);
        vertices[3] = vertices[3].remap(u1 / x_tex_size - us, v1 / y_tex_size - vs);
        if mirror {
            vertices.reverse();
        }

        Polygon { vertices, normal }
    }

    fn mirror_facing(facing: Direction) -> Direction {
        if facing.get_axis() == Axis::X {
            facing.get_opposite()
        } else {
            facing
        }
    }
}

/// `ModelPart.Vertex`: a corner in sixteenths of a block and its texture coordinates.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Vertex {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub u: f32,
    pub v: f32,
}

impl Vertex {
    pub const SCALE_FACTOR: f32 = 16.0;

    pub fn new(x: f32, y: f32, z: f32, u: f32, v: f32) -> Vertex {
        Vertex { x, y, z, u, v }
    }

    pub fn remap(&self, u: f32, v: f32) -> Vertex {
        Vertex::new(self.x, self.y, self.z, u, v)
    }

    pub fn world_x(&self) -> f32 {
        self.x / 16.0
    }

    pub fn world_y(&self) -> f32 {
        self.y / 16.0
    }

    pub fn world_z(&self) -> f32 {
        self.z / 16.0
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use crate::com::mojang::blaze3d::vertex::pose_stack::PoseStack;
    use crate::com::mojang::blaze3d::vertex::vertex_consumer::VertexConsumer;
    use crate::golden::Golden;
    use crate::net::minecraft::client::model::geom::builders::cube_deformation::CubeDeformation;
    use crate::net::minecraft::client::model::geom::builders::layer_definition::LayerDefinition;
    use crate::net::minecraft::client::model::player::player_model::PlayerModel;
    use crate::org::joml::quaternionf::Quaternionf;

    fn float(value: f32) -> Value {
        Value::from(format!("{:08x}", value.to_bits()))
    }

    /// Each vertex the eleven-argument `addVertex` gets, like the suite's recorder.
    #[derive(Default)]
    struct Recorder {
        vertices: Vec<Value>,
    }

    impl VertexConsumer for Recorder {
        fn add_vertex(&mut self, _x: f32, _y: f32, _z: f32) -> &mut dyn VertexConsumer {
            unreachable!("model parts add whole vertices")
        }

        fn set_color_rgba(
            &mut self,
            _r: i32,
            _g: i32,
            _b: i32,
            _a: i32,
        ) -> &mut dyn VertexConsumer {
            unreachable!("model parts add whole vertices")
        }

        fn set_color(&mut self, _color: i32) -> &mut dyn VertexConsumer {
            unreachable!("model parts add whole vertices")
        }

        fn set_uv(&mut self, _u: f32, _v: f32) -> &mut dyn VertexConsumer {
            unreachable!("model parts add whole vertices")
        }

        fn set_uv1(&mut self, _u: i32, _v: i32) -> &mut dyn VertexConsumer {
            unreachable!("model parts add whole vertices")
        }

        fn set_uv2(&mut self, _u: i32, _v: i32) -> &mut dyn VertexConsumer {
            unreachable!("model parts add whole vertices")
        }

        fn set_uv3(&mut self, _u: f32, _v: f32) -> &mut dyn VertexConsumer {
            unreachable!("model parts add whole vertices")
        }

        fn set_normal(&mut self, _x: f32, _y: f32, _z: f32) -> &mut dyn VertexConsumer {
            unreachable!("model parts add whole vertices")
        }

        fn set_line_width(&mut self, _width: f32) -> &mut dyn VertexConsumer {
            unreachable!("model parts add whole vertices")
        }

        fn add_vertex_full(
            &mut self,
            x: f32,
            y: f32,
            z: f32,
            color: i32,
            u: f32,
            v: f32,
            overlay_coords: i32,
            light_coords: i32,
            nx: f32,
            ny: f32,
            nz: f32,
        ) {
            self.vertices.push(json!([
                float(x),
                float(y),
                float(z),
                color,
                float(u),
                float(v),
                overlay_coords,
                light_coords,
                float(nx),
                float(ny),
                float(nz)
            ]));
        }
    }

    /// The player's model, wide and slim, baked like `ModelLayers.PLAYER` and `PLAYER_SLIM`, and
    /// a part of it rendered at a pose, vertex for vertex like vanilla's (`EntityModelSuites`).
    #[test]
    fn renders_the_players_parts_like_vanilla() {
        let golden = Golden::load("net.minecraft.client.model.geom.ModelPart");
        golden.check(
            "render",
            |slim: bool, part: String, pose: [f32; 10], sleeves: bool, color: i32| {
                let mut root = LayerDefinition::create(
                    PlayerModel::create_mesh(CubeDeformation::NONE, slim),
                    64,
                    64,
                )
                .bake_root();
                root.get_child_mut("right_arm")
                    .get_child_mut("right_sleeve")
                    .visible = sleeves;
                root.get_child_mut("left_arm")
                    .get_child_mut("left_sleeve")
                    .visible = sleeves;
                let part = if part == "root" {
                    &mut root
                } else {
                    root.get_child_mut(&part)
                };
                part.z_rot = pose[9];
                let mut pose_stack = PoseStack::new();
                pose_stack.translate_f32(pose[0], pose[1], pose[2]);
                pose_stack.rotate(Quaternionf::new().rotation_xyz(pose[3], pose[4], pose[5]));
                pose_stack.scale(pose[6], pose[7], pose[8]);
                let mut recorder = Recorder::default();
                part.render_colored(&mut pose_stack, &mut recorder, 0xF000A0, 0xA0003, color);
                Value::Array(recorder.vertices)
            },
        );
        golden.finish();
    }
}
