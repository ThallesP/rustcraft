//! net/minecraft/client/renderer/rendertype/RenderTypes.java
//!
//! The game's render types, each made once for its arguments (`Util.memoize`), which the port
//! keeps per render thread. Only `entityTranslucent`, the player's skin's, and `LINES_TRANSLUCENT`,
//! the block outline's, so far; the others come with what draws with them.

use std::cell::RefCell;
use std::rc::Rc;

use crate::net::minecraft::client::renderer::render_pipelines::RenderPipelines;
use crate::net::minecraft::client::renderer::rendertype::layering_transform::LayeringTransform;
use crate::net::minecraft::client::renderer::rendertype::render_setup::{
    OutlineProperty, RenderSetup,
};
use crate::net::minecraft::client::renderer::rendertype::render_type::RenderType;
use crate::net::minecraft::resources::identifier::Identifier;

/// A memoized factory's render types, by their arguments.
type Memo<K> = RefCell<Vec<(K, Rc<RenderType>)>>;

thread_local! {
    static ENTITY_TRANSLUCENT: Memo<(Identifier, bool)> = const { RefCell::new(Vec::new()) };
    static LINES_TRANSLUCENT: Rc<RenderType> = RenderType::create(
        "lines_translucent",
        RenderSetup::builder(&RenderPipelines::get().LINES_TRANSLUCENT)
            .set_layering_transform(LayeringTransform::VIEW_OFFSET_Z_LAYERING)
            .create_render_setup(),
    );
}

pub struct RenderTypes;

impl RenderTypes {
    /// `entityTranslucent(texture, affectsOutline)`. Its OIT pipelines (`OIT_ENTITY`) come with
    /// improved transparency.
    pub fn entity_translucent_outline(
        texture: &Identifier,
        affects_outline: bool,
    ) -> Rc<RenderType> {
        ENTITY_TRANSLUCENT.with_borrow_mut(|memo| {
            let key = (texture.clone(), affects_outline);
            if let Some((_, render_type)) = memo.iter().find(|(cached, _)| *cached == key) {
                return Rc::clone(render_type);
            }
            let state = RenderSetup::builder(&RenderPipelines::get().ENTITY_TRANSLUCENT)
                .with_texture("Sampler0", texture.clone())
                .use_lightmap()
                .use_overlay()
                .affects_crumbling()
                .sort_on_upload()
                .set_outline(if affects_outline {
                    OutlineProperty::AFFECTS_OUTLINE
                } else {
                    OutlineProperty::NONE
                })
                .create_render_setup();
            let render_type = RenderType::create("entity_translucent", state);
            memo.push((key, Rc::clone(&render_type)));
            render_type
        })
    }

    /// `entityTranslucent(texture)`: one that affects outlines.
    pub fn entity_translucent(texture: &Identifier) -> Rc<RenderType> {
        RenderTypes::entity_translucent_outline(texture, true)
    }

    /// `linesTranslucent()`: lines that blend, layered towards the camera.
    pub fn lines_translucent() -> Rc<RenderType> {
        LINES_TRANSLUCENT.with(Rc::clone)
    }
}
