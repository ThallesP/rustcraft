//! com/mojang/renderpearl/util/ShaderCompileException.java

use std::fmt;

/// A pipeline's shaders failed to compile, reflect or link up, with vanilla's message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ShaderCompileException(pub String);

impl fmt::Display for ShaderCompileException {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for ShaderCompileException {}
