//! com/mojang/renderpearl/api/commands/RenderPass.java
//!
//! Only opening and closing so far; drawing comes with the first pipeline that draws.

/// A pass over a set of attachments, which ends when it's dropped (Java's `close()`).
pub trait RenderPass {}

/// `RenderPass.RenderArea`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RenderArea {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}
