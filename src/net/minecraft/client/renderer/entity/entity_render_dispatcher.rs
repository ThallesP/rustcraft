//! net/minecraft/client/renderer/entity/EntityRenderDispatcher.java
//!
//! Only the players' renderers so far, one for each skin model (`EntityRenderers
//! .createAvatarRenderers`), which the first-person hands draw with. The other entities'
//! renderers, extracting, submitting and the shadows and hit boxes come with entity rendering.

use crate::net::minecraft::client::renderer::entity::player::avatar_renderer::AvatarRenderer;
use crate::net::minecraft::client::renderer::entity::state::avatar_render_state::AvatarRenderState;
use crate::net::minecraft::world::entity::player::player_model_type::PlayerModelType;

pub struct EntityRenderDispatcher {
    wide_player_renderer: AvatarRenderer,
    slim_player_renderer: AvatarRenderer,
}

impl Default for EntityRenderDispatcher {
    fn default() -> EntityRenderDispatcher {
        EntityRenderDispatcher::new()
    }
}

impl EntityRenderDispatcher {
    pub fn new() -> EntityRenderDispatcher {
        EntityRenderDispatcher {
            wide_player_renderer: AvatarRenderer::new(false),
            slim_player_renderer: AvatarRenderer::new(true),
        }
    }

    /// `getRenderer(AvatarRenderState)`: the renderer of the skin's model.
    pub fn get_renderer(&mut self, state: &AvatarRenderState) -> &mut AvatarRenderer {
        match state.skin.model {
            PlayerModelType::WIDE => &mut self.wide_player_renderer,
            PlayerModelType::SLIM => &mut self.slim_player_renderer,
        }
    }
}
