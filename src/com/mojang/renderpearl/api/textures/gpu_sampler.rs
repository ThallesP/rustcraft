//! com/mojang/renderpearl/api/textures/GpuSampler.java
//!
//! A sampler a backend made, which each backend casts back to its own through `as_any`. Its GL
//! name is deleted when it's dropped (Java's `close()`).

use crate::com::mojang::renderpearl::api::textures::address_mode::AddressMode;
use crate::com::mojang::renderpearl::api::textures::filter_mode::FilterMode;
use std::any::Any;

pub trait GpuSampler {
    fn get_address_mode_u(&self) -> AddressMode;

    fn get_address_mode_v(&self) -> AddressMode;

    fn get_min_filter(&self) -> FilterMode;

    fn get_mag_filter(&self) -> FilterMode;

    fn get_max_anisotropy(&self) -> i32;

    /// Java's `OptionalDouble`.
    fn get_max_lod(&self) -> Option<f64>;

    fn as_any(&self) -> &dyn Any;
}
