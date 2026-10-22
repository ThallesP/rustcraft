//! net/minecraft/client/renderer/oit/OitPipelineSet.java
//!
//! `Builder.build()` reads `RenderPipelines`' three OIT snippets, which are statics of their own
//! here. `getPipeline(OitStage)` comes with `OitStage`.

use crate::com::mojang::renderpearl::api::pipeline::render_pipeline::{
    self, RenderPipeline, Snippet,
};
use crate::net::minecraft::client::renderer::render_pipelines::RenderPipelines;
use std::sync::Arc;

/// The three passes of one order-independent transparency pipeline.
#[derive(Clone, Debug)]
pub struct OitPipelineSet {
    pub depth_bounds_pipeline: Arc<RenderPipeline>,
    pub transmittance_pipeline: Arc<RenderPipeline>,
    pub accumulate_pipeline: Arc<RenderPipeline>,
}

/// A change to one pass's builder, Java's `Consumer<RenderPipeline.Builder>`.
pub type Modifier = Box<dyn Fn(&mut render_pipeline::Builder)>;

impl OitPipelineSet {
    pub fn builder(location_suffix: &str, builder: &render_pipeline::Builder) -> Builder {
        Builder::new(builder.build_snippet(), location_suffix)
    }
}

/// `OitPipelineSet.Builder`.
pub struct Builder {
    base_snippet: Snippet,
    location_suffix: String,
    depth_bounds_modifier: Option<Modifier>,
    transmittance_modifier: Option<Modifier>,
    accumulate_modifier: Option<Modifier>,
}

impl Builder {
    pub fn new(base_snippet: Snippet, location_suffix: &str) -> Builder {
        Builder {
            base_snippet,
            location_suffix: location_suffix.to_owned(),
            depth_bounds_modifier: None,
            transmittance_modifier: None,
            accumulate_modifier: None,
        }
    }

    pub fn with_depth_bounds_modifier(
        mut self,
        modifier: impl Fn(&mut render_pipeline::Builder) + 'static,
    ) -> Builder {
        self.depth_bounds_modifier =
            Builder::compose_modifiers(self.depth_bounds_modifier, modifier);
        self
    }

    pub fn with_transmittance_modifier(
        mut self,
        modifier: impl Fn(&mut render_pipeline::Builder) + 'static,
    ) -> Builder {
        self.transmittance_modifier =
            Builder::compose_modifiers(self.transmittance_modifier, modifier);
        self
    }

    pub fn with_accumulate_modifier(
        mut self,
        modifier: impl Fn(&mut render_pipeline::Builder) + 'static,
    ) -> Builder {
        self.accumulate_modifier = Builder::compose_modifiers(self.accumulate_modifier, modifier);
        self
    }

    pub fn without_depth_test(self) -> Builder {
        let disable_depth_test = |builder: &mut render_pipeline::Builder| {
            builder.with_depth_stencil_state_optional(None);
        };
        self.with_depth_bounds_modifier(disable_depth_test)
            .with_transmittance_modifier(disable_depth_test)
            .with_accumulate_modifier(disable_depth_test)
    }

    fn compose_modifiers(
        current_modifier: Option<Modifier>,
        new_modifier: impl Fn(&mut render_pipeline::Builder) + 'static,
    ) -> Option<Modifier> {
        Some(match current_modifier {
            Some(current) => Box::new(move |builder| {
                current(builder);
                new_modifier(builder);
            }),
            None => Box::new(new_modifier),
        })
    }

    /// Builds the depth bounds, transmittance and accumulate pipelines, in that order.
    pub fn build(&self) -> OitPipelineSet {
        let pass = |snippet: &Snippet, prefix: &str, modifier: &Option<Modifier>| {
            let mut builder = RenderPipeline::builder(&[&self.base_snippet, snippet]);
            builder.with_location(&format!("pipeline/{prefix}{}", self.location_suffix));
            if let Some(modifier) = modifier {
                modifier(&mut builder);
            }
            Arc::new(builder.build())
        };
        let depth_bounds_pipeline = pass(
            RenderPipelines::OIT_DEPTH_BOUNDS_SNIPPET,
            "oit_depth_bounds_",
            &self.depth_bounds_modifier,
        );
        let transmittance_pipeline = pass(
            RenderPipelines::OIT_TRANSMITTANCE_SNIPPET,
            "oit_transmittance_",
            &self.transmittance_modifier,
        );
        let accumulate_pipeline = pass(
            RenderPipelines::OIT_ACCUMULATE_SNIPPET,
            "oit_accumulate_",
            &self.accumulate_modifier,
        );
        OitPipelineSet {
            depth_bounds_pipeline,
            transmittance_pipeline,
            accumulate_pipeline,
        }
    }
}
