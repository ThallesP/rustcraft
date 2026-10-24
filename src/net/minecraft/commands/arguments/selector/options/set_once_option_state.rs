//! net/minecraft/commands/arguments/selector/options/SetOnceOptionState.java

/// An option a selector takes once.
#[derive(Clone, Copy, Debug, Default)]
pub struct SetOnceOptionState {
    has_value: bool,
}

impl SetOnceOptionState {
    pub fn can_parse(&self) -> bool {
        !self.has_value
    }

    pub fn mark_parsed(&mut self) {
        self.has_value = true;
    }
}
