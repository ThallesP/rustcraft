//! net/minecraft/client/model/player/PlayerModel.java
//!
//! A player's model: the humanoid's, with a sleeve on each arm, a pants leg on each leg and a
//! jacket, each a quarter pixel bigger, and slim (three pixel) arms for slim skins. Java's fields
//! are references to parts of the root's tree; here they're the parts' paths, reached through the
//! root. Not ported yet: `setupAnim` (with `HumanoidModel`'s), `translateToHand`,
//! `getRandomBodyPart` and the armor meshes.

use crate::net::minecraft::client::model::geom::builders::cube_deformation::CubeDeformation;
use crate::net::minecraft::client::model::geom::builders::cube_list_builder::CubeListBuilder;
use crate::net::minecraft::client::model::geom::builders::mesh_definition::MeshDefinition;
use crate::net::minecraft::client::model::geom::model_part::ModelPart;
use crate::net::minecraft::client::model::geom::part_pose::PartPose;
use crate::net::minecraft::client::model::humanoid_model::HumanoidModel;

#[derive(Clone, Debug)]
pub struct PlayerModel {
    humanoid: HumanoidModel,
    slim: bool,
}

impl PlayerModel {
    pub const LEFT_SLEEVE: &'static str = "left_sleeve";
    pub const RIGHT_SLEEVE: &'static str = "right_sleeve";
    pub const LEFT_PANTS: &'static str = "left_pants";
    pub const RIGHT_PANTS: &'static str = "right_pants";

    /// Panics where Java throws `NoSuchElementException` for a root without the player's parts.
    pub fn new(root: ModelPart, slim: bool) -> PlayerModel {
        root.get_child(HumanoidModel::LEFT_ARM)
            .get_child(PlayerModel::LEFT_SLEEVE);
        root.get_child(HumanoidModel::RIGHT_ARM)
            .get_child(PlayerModel::RIGHT_SLEEVE);
        root.get_child(HumanoidModel::LEFT_LEG)
            .get_child(PlayerModel::LEFT_PANTS);
        root.get_child(HumanoidModel::RIGHT_LEG)
            .get_child(PlayerModel::RIGHT_PANTS);
        root.get_child(HumanoidModel::BODY).get_child("jacket");
        PlayerModel {
            humanoid: HumanoidModel::new(root),
            slim,
        }
    }

    pub fn create_mesh(scale: CubeDeformation, slim: bool) -> MeshDefinition {
        let mut mesh = HumanoidModel::create_mesh(scale, 0.0);
        let root = mesh.get_root();
        if slim {
            root.add_or_replace_child(
                "left_arm",
                CubeListBuilder::create()
                    .tex_offs(32, 48)
                    .add_box_deformed(-1.0, -2.0, -2.0, 3.0, 12.0, 4.0, scale),
                PartPose::offset(5.0, 2.0, 0.0),
            )
            .add_or_replace_child(
                "left_sleeve",
                CubeListBuilder::create().tex_offs(48, 48).add_box_deformed(
                    -1.0,
                    -2.0,
                    -2.0,
                    3.0,
                    12.0,
                    4.0,
                    scale.extend(0.25),
                ),
                PartPose::ZERO,
            );
            root.add_or_replace_child(
                "right_arm",
                CubeListBuilder::create()
                    .tex_offs(40, 16)
                    .add_box_deformed(-2.0, -2.0, -2.0, 3.0, 12.0, 4.0, scale),
                PartPose::offset(-5.0, 2.0, 0.0),
            )
            .add_or_replace_child(
                "right_sleeve",
                CubeListBuilder::create().tex_offs(40, 32).add_box_deformed(
                    -2.0,
                    -2.0,
                    -2.0,
                    3.0,
                    12.0,
                    4.0,
                    scale.extend(0.25),
                ),
                PartPose::ZERO,
            );
        } else {
            root.add_or_replace_child(
                "left_arm",
                CubeListBuilder::create()
                    .tex_offs(32, 48)
                    .add_box_deformed(-1.0, -2.0, -2.0, 4.0, 12.0, 4.0, scale),
                PartPose::offset(5.0, 2.0, 0.0),
            )
            .add_or_replace_child(
                "left_sleeve",
                CubeListBuilder::create().tex_offs(48, 48).add_box_deformed(
                    -1.0,
                    -2.0,
                    -2.0,
                    4.0,
                    12.0,
                    4.0,
                    scale.extend(0.25),
                ),
                PartPose::ZERO,
            );
            root.get_child("right_arm").add_or_replace_child(
                "right_sleeve",
                CubeListBuilder::create().tex_offs(40, 32).add_box_deformed(
                    -3.0,
                    -2.0,
                    -2.0,
                    4.0,
                    12.0,
                    4.0,
                    scale.extend(0.25),
                ),
                PartPose::ZERO,
            );
        }

        root.add_or_replace_child(
            "left_leg",
            CubeListBuilder::create()
                .tex_offs(16, 48)
                .add_box_deformed(-2.0, 0.0, -2.0, 4.0, 12.0, 4.0, scale),
            PartPose::offset(1.9, 12.0, 0.0),
        )
        .add_or_replace_child(
            "left_pants",
            CubeListBuilder::create().tex_offs(0, 48).add_box_deformed(
                -2.0,
                0.0,
                -2.0,
                4.0,
                12.0,
                4.0,
                scale.extend(0.25),
            ),
            PartPose::ZERO,
        );
        root.get_child("right_leg").add_or_replace_child(
            "right_pants",
            CubeListBuilder::create().tex_offs(0, 32).add_box_deformed(
                -2.0,
                0.0,
                -2.0,
                4.0,
                12.0,
                4.0,
                scale.extend(0.25),
            ),
            PartPose::ZERO,
        );
        root.get_child("body").add_or_replace_child(
            "jacket",
            CubeListBuilder::create().tex_offs(16, 32).add_box_deformed(
                -4.0,
                0.0,
                -2.0,
                8.0,
                12.0,
                4.0,
                scale.extend(0.25),
            ),
            PartPose::ZERO,
        );
        mesh
    }

    pub fn slim(&self) -> bool {
        self.slim
    }

    pub fn root(&self) -> &ModelPart {
        self.humanoid.root()
    }

    pub fn left_arm(&self) -> &ModelPart {
        self.root().get_child(HumanoidModel::LEFT_ARM)
    }

    pub fn right_arm(&self) -> &ModelPart {
        self.root().get_child(HumanoidModel::RIGHT_ARM)
    }

    pub fn left_arm_mut(&mut self) -> &mut ModelPart {
        self.humanoid
            .root_mut()
            .get_child_mut(HumanoidModel::LEFT_ARM)
    }

    pub fn right_arm_mut(&mut self) -> &mut ModelPart {
        self.humanoid
            .root_mut()
            .get_child_mut(HumanoidModel::RIGHT_ARM)
    }

    pub fn left_sleeve_mut(&mut self) -> &mut ModelPart {
        self.left_arm_mut().get_child_mut(PlayerModel::LEFT_SLEEVE)
    }

    pub fn right_sleeve_mut(&mut self) -> &mut ModelPart {
        self.right_arm_mut()
            .get_child_mut(PlayerModel::RIGHT_SLEEVE)
    }
}
