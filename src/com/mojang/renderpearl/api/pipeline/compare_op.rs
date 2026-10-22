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
