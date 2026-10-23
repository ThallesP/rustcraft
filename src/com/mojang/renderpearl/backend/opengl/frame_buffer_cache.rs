//! com/mojang/renderpearl/backend/opengl/FrameBufferCache.java
//!
//! The framebuffers render passes and copies draw through, one per set of attachments, made the
//! first time (the command encoder's `get_fbo`). A texture going away takes the framebuffers that
//! hold it with it (`GlTexture.destroyImmediately`'s `destroyFbo`s): GL reuses a deleted
//! texture's name, so a framebuffer still cached under it would draw into a texture that's gone
//! once a texture of that name is made again (a resized main target). Java's attachments keep the
//! keys of their framebuffers; here the cache finds them by the texture's name.

use glow::HasContext;
use std::cell::RefCell;

/// A framebuffer's attachments: each color target's texture and mip level, and the depth's.
pub type FrameBufferKey = (
    Vec<Option<(glow::NativeTexture, i32)>>,
    Option<(glow::NativeTexture, i32)>,
);

#[derive(Default)]
pub struct FrameBufferCache {
    cache: RefCell<Vec<(FrameBufferKey, glow::NativeFramebuffer)>>,
}

impl FrameBufferCache {
    pub fn get(&self, key: &FrameBufferKey) -> Option<glow::NativeFramebuffer> {
        self.cache
            .borrow()
            .iter()
            .find(|(existing, _)| existing == key)
            .map(|(_, fbo)| *fbo)
    }

    pub fn insert(&self, key: FrameBufferKey, fbo: glow::NativeFramebuffer) {
        self.cache.borrow_mut().push((key, fbo));
    }

    /// `destroyFbo` of every framebuffer the texture is attached to.
    pub fn destroy_fbos_of(&self, gl: &glow::Context, texture: glow::NativeTexture) {
        self.cache.borrow_mut().retain(|((colors, depth), fbo)| {
            let attached = colors
                .iter()
                .chain(std::iter::once(depth))
                .any(|attachment| attachment.is_some_and(|(id, _)| id == texture));
            if attached {
                // SAFETY: the framebuffer is the context's, which is current.
                unsafe { gl.delete_framebuffer(*fbo) };
            }
            !attached
        });
    }

    /// Every framebuffer, deleted, while the device's context is still current.
    pub fn close(&self, gl: &glow::Context) {
        for (_, fbo) in self.cache.borrow_mut().drain(..) {
            // SAFETY: the framebuffer is the context's, which is current.
            unsafe { gl.delete_framebuffer(fbo) };
        }
    }
}
