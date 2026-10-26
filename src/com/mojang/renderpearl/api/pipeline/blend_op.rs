//! com/mojang/renderpearl/api/pipeline/BlendOp.java

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BlendOp {
    ADD,
    SUBTRACT,
    REVERSE_SUBTRACT,
    MIN,
    MAX,
}

impl BlendOp {
    /// `values()`.
    pub const VALUES: [BlendOp; 5] = [
        BlendOp::ADD,
        BlendOp::SUBTRACT,
        BlendOp::REVERSE_SUBTRACT,
        BlendOp::MIN,
        BlendOp::MAX,
    ];
}
