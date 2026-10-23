//! net/minecraft/client/KeyMapping.java
//!
//! Only whether a mapping is held down, which the keyboard sets (`KeyMapping.set`) and the input
//! reads. Not ported yet: the keys bound to mappings, clicks, categories and toggling (sneak and
//! sprint are held, like vanilla's defaults `toggleCrouch:false` and `toggleSprint:false`).

/// A named action a key can be held for.
#[derive(Clone, Debug)]
pub struct KeyMapping {
    name: &'static str,
    is_down: bool,
}

impl KeyMapping {
    pub fn new(name: &'static str) -> KeyMapping {
        KeyMapping {
            name,
            is_down: false,
        }
    }

    pub fn get_name(&self) -> &'static str {
        self.name
    }

    pub fn is_down(&self) -> bool {
        self.is_down
    }

    pub fn set_down(&mut self, is_down: bool) {
        self.is_down = is_down;
    }
}
