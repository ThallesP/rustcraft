//! net/minecraft/client/model/geom/builders/MaterialDefinition.java

/// The size of the texture a layer's cubes take their coordinates in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MaterialDefinition {
    pub(crate) x_tex_size: i32,
    pub(crate) y_tex_size: i32,
}

impl MaterialDefinition {
    pub fn new(x_tex_size: i32, y_tex_size: i32) -> MaterialDefinition {
        MaterialDefinition {
            x_tex_size,
            y_tex_size,
        }
    }
}
