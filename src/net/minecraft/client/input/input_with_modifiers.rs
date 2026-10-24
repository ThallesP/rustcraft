//! net/minecraft/client/input/InputWithModifiers.java
//!
//! A key or a mouse button with the modifiers held: SDL's scancode or button (`input`), the key's
//! keycode in the layout (`shortcutKey`, a key event's own) and SDL's modifier bits. The control
//! key's quirk (`InputQuirks.EDIT_SHORTCUT_KEY_MODIFIER`, the command key on macOS) is control's
//! bits on the other systems, which are the ones the port runs on.

pub trait InputWithModifiers {
    fn input(&self) -> i32;

    fn shortcut_key(&self) -> i32 {
        self.input()
    }

    fn modifiers(&self) -> i32;

    fn is_selection(&self) -> bool {
        self.input() == 40 || self.input() == 44 || self.input() == 88
    }

    fn is_confirmation(&self) -> bool {
        self.input() == 40 || self.input() == 88
    }

    fn is_escape(&self) -> bool {
        self.input() == 41
    }

    fn is_left(&self) -> bool {
        self.input() == 80
    }

    fn is_right(&self) -> bool {
        self.input() == 79
    }

    fn is_up(&self) -> bool {
        self.input() == 82
    }

    fn is_down(&self) -> bool {
        self.input() == 81
    }

    fn is_cycle_focus(&self) -> bool {
        self.input() == 43
    }

    fn has_alt_down(&self) -> bool {
        (self.modifiers() & 768) != 0
    }

    fn has_shift_down(&self) -> bool {
        (self.modifiers() & 3) != 0
    }

    fn has_control_down(&self) -> bool {
        (self.modifiers() & 192) != 0
    }

    fn has_control_down_with_quirk(&self) -> bool {
        self.has_control_down()
    }

    fn is_select_all(&self) -> bool {
        self.shortcut_key() == 97
            && self.has_control_down_with_quirk()
            && !self.has_shift_down()
            && !self.has_alt_down()
    }

    fn is_copy(&self) -> bool {
        self.shortcut_key() == 99
            && self.has_control_down_with_quirk()
            && !self.has_shift_down()
            && !self.has_alt_down()
    }

    fn is_paste(&self) -> bool {
        self.shortcut_key() == 118
            && self.has_control_down_with_quirk()
            && !self.has_shift_down()
            && !self.has_alt_down()
    }

    fn is_cut(&self) -> bool {
        self.shortcut_key() == 120
            && self.has_control_down_with_quirk()
            && !self.has_shift_down()
            && !self.has_alt_down()
    }
}
