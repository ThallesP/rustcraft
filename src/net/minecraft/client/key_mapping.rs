//! net/minecraft/client/KeyMapping.java
//!
//! A named action and the key bound to it (its default: SDL's scancode for a keyboard key), held
//! down or not and clicked a number of times since the game last asked (`consumeClick`). The
//! keyboard sets them (`KeyMapping.set`, `click`) and the game reads them. Not ported yet: the
//! options' bindings and rebinding, mouse buttons, categories and toggling (sneak and sprint are
//! held, like vanilla's defaults `toggleCrouch:false` and `toggleSprint:false`).

/// A named action a key can be held for.
#[derive(Clone, Debug)]
pub struct KeyMapping {
    name: &'static str,
    /// The bound key's scancode, none for a mapping without one.
    key: Option<i32>,
    is_down: bool,
    click_count: i32,
}

impl KeyMapping {
    pub fn new(name: &'static str) -> KeyMapping {
        KeyMapping {
            name,
            key: None,
            is_down: false,
            click_count: 0,
        }
    }

    /// `KeyMapping(String, int, Category)`: bound to a key.
    pub fn bound(name: &'static str, key: i32) -> KeyMapping {
        KeyMapping {
            key: Some(key),
            ..KeyMapping::new(name)
        }
    }

    pub fn get_name(&self) -> &'static str {
        self.name
    }

    /// Whether `key` is the key bound to it (`matches`).
    pub fn matches(&self, key: i32) -> bool {
        self.key == Some(key)
    }

    pub fn is_down(&self) -> bool {
        self.is_down
    }

    pub fn set_down(&mut self, is_down: bool) {
        self.is_down = is_down;
    }

    /// `click`: one more click to consume.
    pub fn click(&mut self) {
        self.click_count += 1;
    }

    /// `consumeClick`: whether there was a click, taking it.
    pub fn consume_click(&mut self) -> bool {
        if self.click_count == 0 {
            false
        } else {
            self.click_count -= 1;
            true
        }
    }

    /// `release`: up, with no clicks left (`releaseAll`'s for each mapping).
    pub fn release(&mut self) {
        self.click_count = 0;
        self.set_down(false);
    }
}
