//! com/mojang/renderpearl/api/pipeline/UniformType.java

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum UniformType {
    COMBINED_IMAGE_SAMPLER,
    UNIFORM_BUFFER,
    TEXEL_BUFFER,
}
