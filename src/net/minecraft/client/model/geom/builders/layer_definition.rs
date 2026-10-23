//! net/minecraft/client/model/geom/builders/LayerDefinition.java
//!
//! Not ported yet: `apply` (`MeshTransformer`), which other models' layers use.

use crate::net::minecraft::client::model::geom::builders::material_definition::MaterialDefinition;
use crate::net::minecraft::client::model::geom::builders::mesh_definition::MeshDefinition;
use crate::net::minecraft::client::model::geom::model_part::ModelPart;

/// A model layer: its mesh and the size of its texture.
#[derive(Clone, Debug)]
pub struct LayerDefinition {
    mesh: MeshDefinition,
    material: MaterialDefinition,
}

impl LayerDefinition {
    pub fn bake_root(&self) -> ModelPart {
        self.mesh
            .root()
            .bake(self.material.x_tex_size, self.material.y_tex_size)
    }

    pub fn create(mesh: MeshDefinition, x_tex_size: i32, y_tex_size: i32) -> LayerDefinition {
        LayerDefinition {
            mesh,
            material: MaterialDefinition::new(x_tex_size, y_tex_size),
        }
    }
}
