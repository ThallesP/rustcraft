//! net/minecraft/client/resources/model/cuboid/UnbakedCuboidGeometry.java

use crate::net::minecraft::client::renderer::block::dispatch::model_state::ModelState;
use crate::net::minecraft::client::resources::model::cuboid::cuboid_model_element::CuboidModelElement;
use crate::net::minecraft::client::resources::model::cuboid::face_bakery::FaceBakery;
use crate::net::minecraft::client::resources::model::geometry::quad_collection::{
    Builder, QuadCollection,
};
use crate::net::minecraft::client::resources::model::geometry::unbaked_geometry::UnbakedGeometry;
use crate::net::minecraft::client::resources::model::model_baker::ModelBaker;
use crate::net::minecraft::client::resources::model::model_debug_name::ModelDebugName;
use crate::net::minecraft::client::resources::model::sprite::texture_slots::TextureSlots;
use crate::net::minecraft::core::direction::{Axis, Direction};

pub struct UnbakedCuboidGeometry {
    pub elements: Vec<CuboidModelElement>,
}

impl UnbakedCuboidGeometry {
    pub fn new(elements: Vec<CuboidModelElement>) -> UnbakedCuboidGeometry {
        UnbakedCuboidGeometry { elements }
    }

    /// Each element's faces, a flat element only its two sides, each face culled with the side
    /// it names turned like the model.
    pub fn bake_elements(
        elements: &[CuboidModelElement],
        textures: &TextureSlots,
        model_baker: &dyn ModelBaker,
        model_state: &'static dyn ModelState,
        name: &dyn ModelDebugName,
    ) -> QuadCollection {
        let mut builder = Builder::new();
        for element in elements {
            let mut draw_x_faces = true;
            let mut draw_y_faces = true;
            let mut draw_z_faces = true;
            let from = &element.from;
            let to = &element.to;
            if from.x == to.x {
                draw_y_faces = false;
                draw_z_faces = false;
            }

            if from.y == to.y {
                draw_x_faces = false;
                draw_z_faces = false;
            }

            if from.z == to.z {
                draw_x_faces = false;
                draw_y_faces = false;
            }

            if draw_x_faces || draw_y_faces || draw_z_faces {
                for (facing, face) in &element.faces {
                    let should_draw_face = match facing.get_axis() {
                        Axis::X => draw_x_faces,
                        Axis::Y => draw_y_faces,
                        Axis::Z => draw_z_faces,
                    };
                    if should_draw_face {
                        let material =
                            model_baker
                                .materials()
                                .resolve_slot(textures, &face.texture, name);
                        let quad = FaceBakery::bake_quad(
                            model_baker,
                            from,
                            to,
                            face,
                            &material,
                            *facing,
                            model_state,
                            element.rotation.as_ref(),
                            element.shade_direction_override,
                            element.light_emission,
                        );
                        match face.cull_for_direction {
                            None => builder.add_unculled_face(quad),
                            Some(cull_for_direction) => builder.add_culled_face(
                                Direction::rotate(
                                    model_state.transformation().get_matrix(),
                                    cull_for_direction,
                                ),
                                quad,
                            ),
                        };
                    }
                }
            }
        }

        builder.build()
    }
}

impl UnbakedGeometry for UnbakedCuboidGeometry {
    fn bake(
        &self,
        textures: &TextureSlots,
        model_baker: &dyn ModelBaker,
        model_state: &'static dyn ModelState,
        name: &dyn ModelDebugName,
    ) -> QuadCollection {
        UnbakedCuboidGeometry::bake_elements(
            &self.elements,
            textures,
            model_baker,
            model_state,
            name,
        )
    }
}
