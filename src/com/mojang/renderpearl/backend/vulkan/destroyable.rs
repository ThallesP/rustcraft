//! com/mojang/renderpearl/backend/vulkan/Destroyable.java
//!
//! What frees a Vulkan object once the GPU is done with it, which the command encoder's
//! destruction queue runs two submits later: a closure.

pub type Destroyable = Box<dyn FnOnce()>;
