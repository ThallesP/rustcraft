//! net/minecraft/world/level/BlockAndLightGetter.java
//!
//! Not ported yet: its methods (`getLightEngine`, `getBrightness`, `getRawBrightness`,
//! `canSeeSky`), which read the level's light engine; the client's keeps the server's light as
//! its section states allow, which comes with it. The trait is here so the renderer can take a
//! level through it now.

use crate::net::minecraft::world::level::block_getter::BlockGetter;

/// Blocks and their light.
pub trait BlockAndLightGetter: BlockGetter {}
