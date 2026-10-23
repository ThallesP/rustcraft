//! net/minecraft/world/phys/shapes/BooleanOp.java
//!
//! An interface whose only implementations are its sixteen constants, the lambdas of every
//! boolean function of two arguments, so it's an enum (PORTING.md).

/// How two shapes combine, voxel by voxel.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum BooleanOp {
    FALSE,
    NOT_OR,
    ONLY_SECOND,
    NOT_FIRST,
    ONLY_FIRST,
    NOT_SECOND,
    NOT_SAME,
    NOT_AND,
    AND,
    SAME,
    SECOND,
    CAUSES,
    FIRST,
    CAUSED_BY,
    OR,
    TRUE,
}

impl BooleanOp {
    pub const VALUES: [BooleanOp; 16] = [
        BooleanOp::FALSE,
        BooleanOp::NOT_OR,
        BooleanOp::ONLY_SECOND,
        BooleanOp::NOT_FIRST,
        BooleanOp::ONLY_FIRST,
        BooleanOp::NOT_SECOND,
        BooleanOp::NOT_SAME,
        BooleanOp::NOT_AND,
        BooleanOp::AND,
        BooleanOp::SAME,
        BooleanOp::SECOND,
        BooleanOp::CAUSES,
        BooleanOp::FIRST,
        BooleanOp::CAUSED_BY,
        BooleanOp::OR,
        BooleanOp::TRUE,
    ];

    pub fn apply(&self, first: bool, second: bool) -> bool {
        match self {
            BooleanOp::FALSE => false,
            BooleanOp::NOT_OR => !first && !second,
            BooleanOp::ONLY_SECOND => second && !first,
            BooleanOp::NOT_FIRST => !first,
            BooleanOp::ONLY_FIRST => first && !second,
            BooleanOp::NOT_SECOND => !second,
            BooleanOp::NOT_SAME => first != second,
            BooleanOp::NOT_AND => !first || !second,
            BooleanOp::AND => first && second,
            BooleanOp::SAME => first == second,
            BooleanOp::SECOND => second,
            BooleanOp::CAUSES => !first || second,
            BooleanOp::FIRST => first,
            BooleanOp::CAUSED_BY => first || !second,
            BooleanOp::OR => first || second,
            BooleanOp::TRUE => true,
        }
    }

    /// The name of its field in `BooleanOp`.
    pub fn name(&self) -> &'static str {
        match self {
            BooleanOp::FALSE => "FALSE",
            BooleanOp::NOT_OR => "NOT_OR",
            BooleanOp::ONLY_SECOND => "ONLY_SECOND",
            BooleanOp::NOT_FIRST => "NOT_FIRST",
            BooleanOp::ONLY_FIRST => "ONLY_FIRST",
            BooleanOp::NOT_SECOND => "NOT_SECOND",
            BooleanOp::NOT_SAME => "NOT_SAME",
            BooleanOp::NOT_AND => "NOT_AND",
            BooleanOp::AND => "AND",
            BooleanOp::SAME => "SAME",
            BooleanOp::SECOND => "SECOND",
            BooleanOp::CAUSES => "CAUSES",
            BooleanOp::FIRST => "FIRST",
            BooleanOp::CAUSED_BY => "CAUSED_BY",
            BooleanOp::OR => "OR",
            BooleanOp::TRUE => "TRUE",
        }
    }
}
