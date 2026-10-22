//! com/mojang/renderpearl/api/pipeline/CompiledRenderPipeline.java
//!
//! What the device hands the game for a pipeline it compiled. The frontend's
//! `FrontendRenderPipeline` is the only one, which a render pass casts it back to through
//! `as_any`. `Pending` is the compile's second half, which `PipelineBuilder::compile_pipeline`
//! runs at once.

use std::any::Any;

pub trait CompiledRenderPipeline {
    fn is_closed(&self) -> bool;

    fn as_any(&self) -> &dyn Any;
}
