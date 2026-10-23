//! net/minecraft/client/model/HumanoidModel.java
//!
//! Only the humanoid's mesh (`createMesh`) and its parts so far; its animation (`setupAnim`: the
//! walk, the arms' poses, crouching, swimming, attacking) comes with entity rendering, and so do
//! the armor meshes.

use crate::net::minecraft::client::model::geom::builders::cube_deformation::CubeDeformation;
use crate::net::minecraft::client::model::geom::builders::cube_list_builder::CubeListBuilder;
use crate::net::minecraft::client::model::geom::builders::mesh_definition::MeshDefinition;
use crate::net::minecraft::client::model::geom::model_part::ModelPart;
use crate::net::minecraft::client::model::geom::part_pose::PartPose;

/// The humanoid's root part and the names of its parts.
#[derive(Clone, Debug)]
pub struct HumanoidModel {
    pub(crate) root: ModelPart,
}

impl HumanoidModel {
    pub const HAT: &'static str = "hat";
    pub const HEAD: &'static str = "head";
    pub const BODY: &'static str = "body";
    pub const RIGHT_ARM: &'static str = "right_arm";
    pub const LEFT_ARM: &'static str = "left_arm";
    pub const RIGHT_LEG: &'static str = "right_leg";
    pub const LEFT_LEG: &'static str = "left_leg";

    pub fn new(root: ModelPart) -> HumanoidModel {
        HumanoidModel { root }
    }

    /// A head with a hat half a pixel bigger, a body, two arms and two legs, grown by `g`,
    /// `y_offset` lower.
    pub fn create_mesh(g: CubeDeformation, y_offset: f32) -> MeshDefinition {
        let mut mesh = MeshDefinition::new();
        let root = mesh.get_root();
        let head = root.add_or_replace_child(
            "head",
            CubeListBuilder::create()
                .tex_offs(0, 0)
                .add_box_deformed(-4.0, -8.0, -4.0, 8.0, 8.0, 8.0, g),
            PartPose::offset(0.0, 0.0 + y_offset, 0.0),
        );
        head.add_or_replace_child(
            "hat",
            CubeListBuilder::create().tex_offs(32, 0).add_box_deformed(
                -4.0,
                -8.0,
                -4.0,
                8.0,
                8.0,
                8.0,
                g.extend(0.5),
            ),
            PartPose::ZERO,
        );
        root.add_or_replace_child(
            "body",
            CubeListBuilder::create()
                .tex_offs(16, 16)
                .add_box_deformed(-4.0, 0.0, -2.0, 8.0, 12.0, 4.0, g),
            PartPose::offset(0.0, 0.0 + y_offset, 0.0),
        );
        root.add_or_replace_child(
            "right_arm",
            CubeListBuilder::create()
                .tex_offs(40, 16)
                .add_box_deformed(-3.0, -2.0, -2.0, 4.0, 12.0, 4.0, g),
            PartPose::offset(-5.0, 2.0 + y_offset, 0.0),
        );
        root.add_or_replace_child(
            "left_arm",
            CubeListBuilder::create()
                .tex_offs(40, 16)
                .mirror()
                .add_box_deformed(-1.0, -2.0, -2.0, 4.0, 12.0, 4.0, g),
            PartPose::offset(5.0, 2.0 + y_offset, 0.0),
        );
        root.add_or_replace_child(
            "right_leg",
            CubeListBuilder::create()
                .tex_offs(0, 16)
                .add_box_deformed(-2.0, 0.0, -2.0, 4.0, 12.0, 4.0, g),
            PartPose::offset(-1.9, 12.0 + y_offset, 0.0),
        );
        root.add_or_replace_child(
            "left_leg",
            CubeListBuilder::create()
                .tex_offs(0, 16)
                .mirror()
                .add_box_deformed(-2.0, 0.0, -2.0, 4.0, 12.0, 4.0, g),
            PartPose::offset(1.9, 12.0 + y_offset, 0.0),
        );
        mesh
    }

    pub fn root(&self) -> &ModelPart {
        &self.root
    }

    pub fn root_mut(&mut self) -> &mut ModelPart {
        &mut self.root
    }
}
