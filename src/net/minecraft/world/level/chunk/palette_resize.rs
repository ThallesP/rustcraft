//! net/minecraft/world/level/chunk/PaletteResize.java

/// What a full palette asks to make room for a value: its container makes a palette of `bits`
/// bits, copies its values over, and gives back the value's id there.
pub trait PaletteResize<T> {
    fn on_resize(&mut self, bits: i32, last_added_value: &T) -> i32;
}

/// `noResizeExpected()`'s handler, for palettes that must not fill up.
pub struct NoResizeExpected;

impl<T> PaletteResize<T> for NoResizeExpected {
    /// Panics, where Java throws `IllegalArgumentException`.
    fn on_resize(&mut self, bits: i32, _last_added_value: &T) -> i32 {
        panic!("Unexpected palette resize, bits = {bits}")
    }
}

pub fn no_resize_expected() -> NoResizeExpected {
    NoResizeExpected
}
