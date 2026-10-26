//! com/mojang/renderpearl/api/pipeline/BlendFactor.java

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BlendFactor {
    CONSTANT_ALPHA,
    CONSTANT_COLOR,
    DST_ALPHA,
    DST_COLOR,
    ONE,
    ONE_MINUS_CONSTANT_ALPHA,
    ONE_MINUS_CONSTANT_COLOR,
    ONE_MINUS_DST_ALPHA,
    ONE_MINUS_DST_COLOR,
    ONE_MINUS_SRC_ALPHA,
    ONE_MINUS_SRC_COLOR,
    SRC_ALPHA,
    SRC_ALPHA_SATURATE,
    SRC_COLOR,
    ZERO,
}

impl BlendFactor {
    /// `values()`.
    pub const VALUES: [BlendFactor; 15] = [
        BlendFactor::CONSTANT_ALPHA,
        BlendFactor::CONSTANT_COLOR,
        BlendFactor::DST_ALPHA,
        BlendFactor::DST_COLOR,
        BlendFactor::ONE,
        BlendFactor::ONE_MINUS_CONSTANT_ALPHA,
        BlendFactor::ONE_MINUS_CONSTANT_COLOR,
        BlendFactor::ONE_MINUS_DST_ALPHA,
        BlendFactor::ONE_MINUS_DST_COLOR,
        BlendFactor::ONE_MINUS_SRC_ALPHA,
        BlendFactor::ONE_MINUS_SRC_COLOR,
        BlendFactor::SRC_ALPHA,
        BlendFactor::SRC_ALPHA_SATURATE,
        BlendFactor::SRC_COLOR,
        BlendFactor::ZERO,
    ];
}
