//! com/mojang/renderpearl/api/pipeline/ShaderType.java

use crate::net::minecraft::resources::file_to_id_converter::FileToIdConverter;
use crate::net::minecraft::resources::identifier::Identifier;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ShaderType {
    VERTEX,
    FRAGMENT,
}

impl ShaderType {
    /// `values()`.
    pub const VALUES: [ShaderType; 2] = [ShaderType::VERTEX, ShaderType::FRAGMENT];
}

use ShaderType::*;

impl ShaderType {
    const TYPES: [ShaderType; 2] = [VERTEX, FRAGMENT];

    pub fn by_location(location: &Identifier) -> Option<ShaderType> {
        ShaderType::TYPES
            .into_iter()
            .find(|shader_type| location.get_path().ends_with(shader_type.extension()))
    }

    pub fn get_name(self) -> &'static str {
        match self {
            VERTEX => "vertex",
            FRAGMENT => "fragment",
        }
    }

    pub fn extension(self) -> &'static str {
        match self {
            VERTEX => ".vsh",
            FRAGMENT => ".fsh",
        }
    }

    pub fn id_converter(self) -> FileToIdConverter {
        FileToIdConverter::new("shaders", self.extension())
    }
}
