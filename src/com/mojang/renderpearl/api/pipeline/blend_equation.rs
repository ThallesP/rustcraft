//! com/mojang/renderpearl/api/pipeline/BlendEquation.java

use crate::com::mojang::renderpearl::api::pipeline::blend_factor::BlendFactor;
use crate::com::mojang::renderpearl::api::pipeline::blend_op::BlendOp;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlendEquation {
    pub source_factor: BlendFactor,
    pub dest_factor: BlendFactor,
    pub op: BlendOp,
}

impl BlendEquation {
    pub const fn new(
        source_factor: BlendFactor,
        dest_factor: BlendFactor,
        op: BlendOp,
    ) -> BlendEquation {
        BlendEquation {
            source_factor,
            dest_factor,
            op,
        }
    }
}
