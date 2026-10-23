//! net/minecraft/client/model/geom/builders/PartDefinition.java
//!
//! A part's children are a `HashMap` in Java, which a baked part keeps in its iteration order
//! (the order its children render in), so they're the port's `java::util::hash_map::HashMap`.
//! Not ported yet: `clearRecursively`, `clearChild`, `retainPartsAndChildren`,
//! `retainExactParts` and `transformed`, which other models' meshes use.

use crate::java::util::hash_map::HashMap;
use crate::net::minecraft::client::model::geom::builders::cube_definition::CubeDefinition;
use crate::net::minecraft::client::model::geom::builders::cube_list_builder::CubeListBuilder;
use crate::net::minecraft::client::model::geom::model_part::ModelPart;
use crate::net::minecraft::client::model::geom::part_pose::PartPose;

/// A part before it's baked: its cubes, its pose and its children by name.
#[derive(Clone, Debug)]
pub struct PartDefinition {
    cubes: Vec<CubeDefinition>,
    part_pose: PartPose,
    children: HashMap<String, PartDefinition>,
}

impl PartDefinition {
    pub(crate) fn new(cubes: Vec<CubeDefinition>, part_pose: PartPose) -> PartDefinition {
        PartDefinition {
            cubes,
            part_pose,
            children: HashMap::new(),
        }
    }

    /// The child named `name`, made of `cubes` at `part_pose`, in place of any child of that name,
    /// whose own children it takes.
    pub fn add_or_replace_child(
        &mut self,
        name: &str,
        cubes: CubeListBuilder,
        part_pose: PartPose,
    ) -> &mut PartDefinition {
        let child = PartDefinition::new(cubes.get_cubes(), part_pose);
        self.add_or_replace_child_definition(name, child)
    }

    pub fn add_or_replace_child_definition(
        &mut self,
        name: &str,
        child: PartDefinition,
    ) -> &mut PartDefinition {
        let previous = self.children.put(name.to_string(), child);
        let child = self.children.get_mut(name).expect("the child was just put");
        if let Some(previous) = previous {
            for (name, grandchild) in previous.children.iter() {
                child.children.put(name.clone(), grandchild.clone());
            }
        }
        child
    }

    /// The child named `name`. Java answers null for none; the port panics.
    pub fn get_child(&mut self, name: &str) -> &mut PartDefinition {
        self.children
            .get_mut(name)
            .unwrap_or_else(|| panic!("no child part {name}"))
    }

    /// The baked part, with its pose as both its initial and its current pose.
    pub fn bake(&self, tex_scale_x: i32, tex_scale_y: i32) -> ModelPart {
        let baked_children = self
            .children
            .iter()
            .map(|(name, child)| (name.clone(), child.bake(tex_scale_x, tex_scale_y)))
            .collect();
        let baked_cubes = self
            .cubes
            .iter()
            .map(|definition| definition.bake(tex_scale_x, tex_scale_y))
            .collect();
        let mut result = ModelPart::new(baked_cubes, baked_children);
        result.set_initial_pose(self.part_pose);
        result.load_pose(&self.part_pose);
        result
    }
}
