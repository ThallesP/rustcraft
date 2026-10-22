//! com/mojang/blaze3d/systems/ScissorState.java
//!
//! The copy constructor and `setFrom` are `Clone`.

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ScissorState {
    enabled: bool,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

impl ScissorState {
    pub fn new() -> ScissorState {
        ScissorState::default()
    }

    pub fn enable(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.enabled = true;
        self.x = x;
        self.y = y;
        self.width = width;
        self.height = height;
    }

    pub fn disable(&mut self) {
        self.enabled = false;
    }

    pub fn enabled(&self) -> bool {
        self.enabled
    }

    pub fn x(&self) -> i32 {
        self.x
    }

    pub fn y(&self) -> i32 {
        self.y
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }
}
