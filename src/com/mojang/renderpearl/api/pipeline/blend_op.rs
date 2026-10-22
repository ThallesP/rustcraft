//! com/mojang/renderpearl/api/pipeline/BlendOp.java

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BlendOp {
    ADD,
    SUBTRACT,
    REVERSE_SUBTRACT,
    MIN,
    MAX,
}
