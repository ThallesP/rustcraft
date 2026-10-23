//! net/minecraft/client/model/geom/builders/MeshDefinition.java
//!
//! Not ported yet: `transformed` and `apply` (`MeshTransformer`), which other models use.

use crate::net::minecraft::client::model::geom::builders::part_definition::PartDefinition;
use crate::net::minecraft::client::model::geom::part_pose::PartPose;

/// A model's parts before they're baked, from its root.
#[derive(Clone, Debug)]
pub struct MeshDefinition {
    root: PartDefinition,
}

impl Default for MeshDefinition {
    fn default() -> MeshDefinition {
        MeshDefinition::new()
    }
}

impl MeshDefinition {
    pub fn new() -> MeshDefinition {
        MeshDefinition {
            root: PartDefinition::new(Vec::new(), PartPose::ZERO),
        }
    }

    pub fn get_root(&mut self) -> &mut PartDefinition {
        &mut self.root
    }

    pub(crate) fn root(&self) -> &PartDefinition {
        &self.root
    }
}
