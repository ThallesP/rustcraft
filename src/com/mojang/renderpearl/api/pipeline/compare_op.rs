//! com/mojang/renderpearl/api/pipeline/CompareOp.java

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CompareOp {
    ALWAYS_PASS,
    LESS_THAN,
    LESS_THAN_OR_EQUAL,
    EQUAL,
    NOT_EQUAL,
    GREATER_THAN_OR_EQUAL,
    GREATER_THAN,
    NEVER_PASS,
}

impl CompareOp {
    /// `values()`.
    pub const VALUES: [CompareOp; 8] = [
        CompareOp::ALWAYS_PASS,
        CompareOp::LESS_THAN,
        CompareOp::LESS_THAN_OR_EQUAL,
        CompareOp::EQUAL,
        CompareOp::NOT_EQUAL,
        CompareOp::GREATER_THAN_OR_EQUAL,
        CompareOp::GREATER_THAN,
        CompareOp::NEVER_PASS,
    ];
}
