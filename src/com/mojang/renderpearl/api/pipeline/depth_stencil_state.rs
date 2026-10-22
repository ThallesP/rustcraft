//! com/mojang/renderpearl/api/pipeline/DepthStencilState.java

use crate::com::mojang::renderpearl::api::pipeline::compare_op::CompareOp;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct DepthStencilState {
    pub depth_test: CompareOp,
    pub write_depth: bool,
    pub depth_bias_scale_factor: f32,
    pub depth_bias_constant: f32,
}

impl DepthStencilState {
    pub const DEFAULT: DepthStencilState =
        DepthStencilState::new(CompareOp::GREATER_THAN_OR_EQUAL, true);

    /// `DepthStencilState(depthTest, depthWrite)`, without a depth bias.
    pub const fn new(depth_test: CompareOp, write_depth: bool) -> DepthStencilState {
        DepthStencilState::with_bias(depth_test, write_depth, 0.0, 0.0)
    }

    pub const fn with_bias(
        depth_test: CompareOp,
        write_depth: bool,
        depth_bias_scale_factor: f32,
        depth_bias_constant: f32,
    ) -> DepthStencilState {
        DepthStencilState {
            depth_test,
            write_depth,
            depth_bias_scale_factor,
            depth_bias_constant,
        }
    }
}
