//! net/minecraft/client/renderer/state/gui/GuiRenderState.java
//!
//! What the GUI draws in a frame, in strata, each a stack of layers: an element goes in the layer
//! above the highest one holding something it overlaps (or above the last element's, when that
//! one encompasses it), so later elements cover earlier ones only where they overlap, and the
//! renderer can sort each layer's elements by pipeline and texture. Java's nodes point at their
//! parent and the node above, which only ever make a chain per stratum; here they're an arena of
//! nodes by index. `reset` keeps the last element's bounds, like Java's, so a frame's first
//! element can go above the first layer.
//!
//! Not ported yet: items (`GuiItemRenderState`), pictures in pictures, the panorama, and the debug
//! rectangles (`SharedConstants.DEBUG_RENDER_UI_LAYERING_RECTANGLES`, off).

use crate::net::minecraft::client::gui::navigation::screen_rectangle::ScreenRectangle;
use crate::net::minecraft::client::renderer::state::gui::gui_element_render_state::GuiElementRenderState;
use crate::net::minecraft::client::renderer::state::gui::gui_text_render_state::GuiTextRenderState;
use crate::net::minecraft::client::renderer::state::gui::panorama_render_state::PanoramaRenderState;
use crate::net::minecraft::client::renderer::state::gui::screen_area::ScreenArea;
use crate::org::joml::vector4f::Vector4f;

pub struct GuiRenderState {
    nodes: Vec<Node>,
    strata: Vec<usize>,
    first_stratum_after_blur: usize,
    current: usize,
    last_element_bounds: Option<ScreenRectangle>,
    /// `panoramaRenderState`: the panorama the GUI draws under everything this frame, if any.
    pub panorama_render_state: Option<PanoramaRenderState>,
    pub clear_color_override: Vector4f,
    pub is_hud_hidden: bool,
}

/// `GuiRenderState.Node`: a layer of a stratum.
#[derive(Default)]
struct Node {
    parent: Option<usize>,
    up: Option<usize>,
    element_states: Vec<Box<dyn GuiElementRenderState>>,
    glyph_states: Vec<Box<dyn GuiElementRenderState>>,
    text_states: Vec<GuiTextRenderState>,
}

/// `GuiRenderState.TraverseRange`.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TraverseRange {
    ALL,
    BEFORE_BLUR,
    AFTER_BLUR,
}

impl Default for GuiRenderState {
    fn default() -> GuiRenderState {
        GuiRenderState::new()
    }
}

impl GuiRenderState {
    pub fn new() -> GuiRenderState {
        let mut state = GuiRenderState {
            nodes: Vec::new(),
            strata: Vec::new(),
            first_stratum_after_blur: usize::MAX,
            current: 0,
            last_element_bounds: None,
            panorama_render_state: None,
            clear_color_override: Vector4f::from_scalar(0.0),
            is_hud_hidden: false,
        };
        state.next_stratum();
        state
    }

    fn new_node(&mut self, parent: Option<usize>) -> usize {
        self.nodes.push(Node {
            parent,
            ..Node::default()
        });
        self.nodes.len() - 1
    }

    pub fn next_stratum(&mut self) {
        self.current = self.new_node(None);
        self.strata.push(self.current);
    }

    /// Whether a stratum asked for the blur under it.
    pub fn has_blur(&self) -> bool {
        self.first_stratum_after_blur != usize::MAX
    }

    /// Java throws `IllegalStateException` for a second blur, the panic.
    pub fn blur_before_this_stratum(&mut self) {
        if self.first_stratum_after_blur != usize::MAX {
            panic!("Can only blur once per frame");
        }
        self.first_stratum_after_blur = self.strata.len() - 1;
    }

    pub fn up(&mut self) {
        let up = match self.nodes[self.current].up {
            Some(up) => up,
            None => {
                let up = self.new_node(Some(self.current));
                self.nodes[self.current].up = Some(up);
                up
            }
        };
        self.current = up;
    }

    pub fn add_gui_element(&mut self, blit_state: Box<dyn GuiElementRenderState>) {
        if self.find_appropriate_node(blit_state.bounds()) {
            self.nodes[self.current].element_states.push(blit_state);
        }
    }

    pub fn add_text(&mut self, text_state: GuiTextRenderState) {
        if self.find_appropriate_node(text_state.bounds()) {
            self.nodes[self.current].text_states.push(text_state);
        }
    }

    fn find_appropriate_node(&mut self, bounds: Option<ScreenRectangle>) -> bool {
        let Some(bounds) = bounds else {
            return false;
        };
        if self
            .last_element_bounds
            .is_some_and(|last| last.encompasses(&bounds))
        {
            self.up();
        } else {
            self.navigate_to_above_highest_element_with_intersecting_bounds(&bounds);
        }
        self.last_element_bounds = Some(bounds);
        true
    }

    fn navigate_to_above_highest_element_with_intersecting_bounds(
        &mut self,
        bounds: &ScreenRectangle,
    ) {
        let mut node = *self.strata.last().expect("there's always a stratum");
        while let Some(up) = self.nodes[node].up {
            node = up;
        }
        let mut found = false;
        while !found {
            let states = &self.nodes[node];
            found = has_intersection(bounds, &states.element_states)
                || states.text_states.iter().any(|text| {
                    text.bounds()
                        .is_some_and(|existing_bounds| existing_bounds.intersects(bounds))
                });
            match states.parent {
                None => break,
                Some(parent) if !found => node = parent,
                Some(_) => {}
            }
        }
        self.current = node;
        if found {
            self.up();
        }
    }

    /// `forEachText` with `addGlyphToCurrentLayer`, as `GuiRenderer.prepareText` runs them: each
    /// text's glyphs, made elements by `glyph`, in the layer the text is in.
    pub fn prepare_text(
        &mut self,
        mut glyph: impl FnMut(
            &GuiTextRenderState,
            &crate::net::minecraft::client::gui::font::text_renderable::TextRenderable,
        ) -> Box<dyn GuiElementRenderState>,
    ) {
        for stratum in self.stratum_range(TraverseRange::ALL) {
            let mut node = Some(self.strata[stratum]);
            while let Some(index) = node {
                let mut glyphs = Vec::new();
                for text in &self.nodes[index].text_states {
                    text.ensure_prepared()
                        .visit(&mut |renderable| glyphs.push(glyph(text, renderable)));
                }
                self.nodes[index].glyph_states.extend(glyphs);
                node = self.nodes[index].up;
            }
        }
    }

    pub fn add_glyph_to_current_layer(&mut self, glyph_state: Box<dyn GuiElementRenderState>) {
        self.nodes[self.current].glyph_states.push(glyph_state);
    }

    pub fn add_blit_to_current_layer(&mut self, blit_state: Box<dyn GuiElementRenderState>) {
        self.nodes[self.current].element_states.push(blit_state);
    }

    pub fn for_each_element(
        &self,
        mut consumer: impl FnMut(&dyn GuiElementRenderState),
        range: TraverseRange,
    ) {
        self.traverse(range, |node| {
            for element_state in &node.element_states {
                consumer(&**element_state);
            }
            for glyph_state in &node.glyph_states {
                consumer(&**glyph_state);
            }
        });
    }

    /// `sortElements`, a stable sort of each layer's elements (Java's `List.sort`).
    pub fn sort_elements(
        &mut self,
        comparator: impl Fn(
            &dyn GuiElementRenderState,
            &dyn GuiElementRenderState,
        ) -> std::cmp::Ordering,
    ) {
        for stratum in self.stratum_range(TraverseRange::ALL) {
            let mut node = Some(self.strata[stratum]);
            while let Some(index) = node {
                self.nodes[index]
                    .element_states
                    .sort_by(|a, b| comparator(&**a, &**b));
                node = self.nodes[index].up;
            }
        }
    }

    fn stratum_range(&self, range: TraverseRange) -> std::ops::Range<usize> {
        let mut start_index = 0;
        let mut end_index = self.strata.len();
        if range == TraverseRange::BEFORE_BLUR {
            end_index = self.first_stratum_after_blur.min(self.strata.len());
        } else if range == TraverseRange::AFTER_BLUR {
            start_index = self.first_stratum_after_blur;
        }
        start_index..end_index
    }

    fn traverse(&self, range: TraverseRange, mut consumer: impl FnMut(&Node)) {
        for stratum in self.stratum_range(range) {
            let mut node = Some(self.strata[stratum]);
            while let Some(index) = node {
                consumer(&self.nodes[index]);
                node = self.nodes[index].up;
            }
        }
    }

    pub fn reset(&mut self) {
        self.nodes.clear();
        self.strata.clear();
        self.first_stratum_after_blur = usize::MAX;
        self.next_stratum();
        self.panorama_render_state = None;
        self.clear_color_override = Vector4f::from_scalar(0.0);
    }
}

fn has_intersection(bounds: &ScreenRectangle, states: &[Box<dyn GuiElementRenderState>]) -> bool {
    states.iter().any(|area| {
        area.bounds()
            .is_some_and(|existing_bounds| existing_bounds.intersects(bounds))
    })
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;

    use super::*;
    use crate::com::mojang::blaze3d::vertex::buffer_builder::BufferBuilder;
    use crate::com::mojang::blaze3d::vertex::byte_buffer_builder::ByteBufferBuilder;
    use crate::golden::{Decode, Encode, Golden, hex};
    use crate::net::minecraft::client::gui::navigation::screen_rectangle::tests::pose;
    use crate::net::minecraft::client::gui::render::gui_renderer::element_sort_comparator;
    use crate::net::minecraft::client::gui::render::texture_setup::TextureSetup;
    use crate::net::minecraft::client::renderer::render_pipelines::pipelines_by_location::pipeline;
    use crate::net::minecraft::client::renderer::state::gui::blit_render_state::BlitRenderState;
    use crate::net::minecraft::client::renderer::state::gui::colored_rectangle_render_state::ColoredRectangleRenderState;
    use crate::net::minecraft::client::renderer::state::gui::tiled_blit_render_state::TiledBlitRenderState;
    use serde_json::{Value, json};

    /// The element an op of `GuiSuites`' programs adds, or none for `nextStratum` and `up`.
    fn element(op: &[Value]) -> Option<Box<dyn GuiElementRenderState>> {
        let int = |index: usize| i32::decode(&op[index]);
        let float = |index: usize| f32::decode(&op[index]);
        let scissor = |index: usize| Option::<ScreenRectangle>::decode(&op[index]);
        let kind = op[0].as_str().expect("an op's kind");
        if kind == "nextStratum" || kind == "up" {
            return None;
        }
        let pipeline = pipeline(op[1].as_str().expect("a pipeline's location"));
        let pose = pose(&op[2]);
        Some(match kind {
            "fill" => Box::new(ColoredRectangleRenderState::new(
                pipeline,
                TextureSetup::no_texture(),
                pose,
                int(3),
                int(4),
                int(5),
                int(6),
                int(7),
                int(8),
                scissor(9),
            )),
            "blit" => Box::new(BlitRenderState::new(
                pipeline,
                TextureSetup::no_texture(),
                pose,
                int(3),
                int(4),
                int(5),
                int(6),
                float(7),
                float(8),
                float(9),
                float(10),
                int(11),
                scissor(12),
            )),
            "tiled" => Box::new(TiledBlitRenderState::new(
                pipeline,
                TextureSetup::no_texture(),
                pose,
                int(3),
                int(4),
                int(5),
                int(6),
                int(7),
                int(8),
                float(9),
                float(10),
                float(11),
                float(12),
                int(13),
                scissor(14),
            )),
            _ => panic!("unknown op {kind}"),
        })
    }

    /// The element's vertices, as a `BufferBuilder` of its pipeline's format writes them.
    fn vertices(element: &dyn GuiElementRenderState) -> Value {
        let pipeline: &Arc<_> = element.pipeline();
        let mut buffer = ByteBufferBuilder::new(1024);
        let mut builder = BufferBuilder::new(
            &mut buffer,
            pipeline.get_primitive_topology(),
            pipeline
                .get_vertex_format_binding(0)
                .expect("a vertex format"),
        );
        element.build_vertices(&mut builder);
        match builder.build() {
            Some(mesh) => {
                let bytes: Vec<i8> = mesh
                    .vertex_buffer()
                    .iter()
                    .map(|&byte| byte as i8)
                    .collect();
                Value::from(hex(&bytes))
            }
            None => Value::Null,
        }
    }

    /// Each program's two frames over one state: the order the renderer draws the elements in
    /// after sorting the layers, every element's bounds, and its vertices. The ops come from
    /// vanilla's result, which the port's frames have to reproduce whole.
    #[test]
    fn places_and_sorts_elements_like_vanilla() {
        let golden = Golden::load("net.minecraft.client.renderer.state.gui.GuiRenderState");
        let cases = golden.cases("program");
        assert_eq!(cases.len(), 400);
        let mut failures = Vec::new();
        for (args, expected) in cases {
            let mut state = GuiRenderState::new();
            let frames = expected.as_array().expect("a program's frames");
            for (frame, expected) in frames.iter().enumerate() {
                state.reset();
                let ops = expected[0].as_array().expect("a frame's ops");
                let actual = run_frame(&mut state, ops);
                if actual != *expected && failures.len() < 3 {
                    failures.push(format!(
                        "program {} frame {frame}:\n expected {expected}\n got      {actual}",
                        args[0]
                    ));
                }
            }
        }
        assert!(failures.is_empty(), "{}", failures.join("\n"));
        golden.finish();
    }

    /// One frame of a program: its ops applied to the state, then what the renderer sees.
    fn run_frame(state: &mut GuiRenderState, ops: &[Value]) -> Value {
        let mut order_by_address = HashMap::new();
        let mut bounds = Vec::new();
        let mut all_vertices = Vec::new();
        let mut index = 0;
        for op in ops {
            let op = op.as_array().expect("an op");
            match element(op) {
                None if op[0] == "nextStratum" => state.next_stratum(),
                None => state.up(),
                Some(element) => {
                    bounds.push(element.bounds().encode());
                    all_vertices.push(vertices(&*element));
                    order_by_address.insert(
                        &*element as *const dyn GuiElementRenderState as *const () as usize,
                        index,
                    );
                    index += 1;
                    state.add_gui_element(element);
                }
            }
        }
        state.sort_elements(element_sort_comparator);
        let mut order = Vec::new();
        state.for_each_element(
            |element| {
                order.push(
                    order_by_address
                        [&(element as *const dyn GuiElementRenderState as *const () as usize)],
                );
            },
            TraverseRange::ALL,
        );
        json!([ops, order, bounds, all_vertices])
    }
}
