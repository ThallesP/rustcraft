//! net/minecraft/client/gui/components/WidgetSprites.java

use crate::net::minecraft::resources::identifier::Identifier;

/// A widget's sprites, enabled or not, focused or not.
#[derive(Clone, Debug, PartialEq)]
pub struct WidgetSprites {
    pub enabled: Identifier,
    pub disabled: Identifier,
    pub enabled_focused: Identifier,
    pub disabled_focused: Identifier,
}

impl WidgetSprites {
    pub fn new(
        enabled: Identifier,
        disabled: Identifier,
        enabled_focused: Identifier,
        disabled_focused: Identifier,
    ) -> WidgetSprites {
        WidgetSprites {
            enabled,
            disabled,
            enabled_focused,
            disabled_focused,
        }
    }

    /// `WidgetSprites(Identifier)`: one sprite for every state.
    pub fn of(sprite: Identifier) -> WidgetSprites {
        WidgetSprites::new(sprite.clone(), sprite.clone(), sprite.clone(), sprite)
    }

    /// `WidgetSprites(Identifier, Identifier)`.
    pub fn focused(sprite: Identifier, focused: Identifier) -> WidgetSprites {
        WidgetSprites::new(sprite.clone(), sprite, focused.clone(), focused)
    }

    /// `WidgetSprites(Identifier, Identifier, Identifier)`.
    pub fn three(enabled: Identifier, disabled: Identifier, focused: Identifier) -> WidgetSprites {
        WidgetSprites::new(enabled, disabled.clone(), focused, disabled)
    }

    pub fn get(&self, enabled: bool, focused: bool) -> &Identifier {
        match (enabled, focused) {
            (true, true) => &self.enabled_focused,
            (true, false) => &self.enabled,
            (false, true) => &self.disabled_focused,
            (false, false) => &self.disabled,
        }
    }
}
