//! net/minecraft/client/renderer/texture/atlas/sources/LazyLoadedImage.java
//!
//! The image is read the first time a permutation asks for it and dropped once every one it was
//! counted for has released it, like Java's; the count and image share a lock instead of Java's
//! atomics.

use std::sync::Mutex;

use crate::com::mojang::blaze3d::platform::native_image::NativeImage;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::server::packs::resources::resource::Resource;

pub struct LazyLoadedImage {
    id: Identifier,
    resource: Resource,
    // The image once read, and the references left.
    state: Mutex<(Option<NativeImage>, i32)>,
}

impl LazyLoadedImage {
    pub fn new(id: Identifier, resource: Resource, count: i32) -> LazyLoadedImage {
        LazyLoadedImage {
            id,
            resource,
            state: Mutex::new((None, count)),
        }
    }

    /// The image through `use_image`; the `Err` is Java's `IOException`.
    pub fn with_image<R>(&self, use_image: impl FnOnce(&NativeImage) -> R) -> Result<R, String> {
        let mut state = self.state.lock().expect("not poisoned");
        if state.0.is_none() {
            let image = self
                .resource
                .open()
                .map_err(|error| error.to_string())
                .and_then(|bytes| NativeImage::read(&bytes))
                .map_err(|error| format!("Failed to load image {}: {error}", self.id))?;
            state.0 = Some(image);
        }
        Ok(use_image(state.0.as_ref().expect("read above")))
    }

    pub fn release(&self) {
        let mut state = self.state.lock().expect("not poisoned");
        state.1 -= 1;
        if state.1 <= 0 {
            state.0 = None;
        }
    }
}
