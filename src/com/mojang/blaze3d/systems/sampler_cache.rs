//! com/mojang/blaze3d/systems/SamplerCache.java
//!
//! A sampler for each address mode, filter and whether it reads mipmaps (a maximum LOD of 0 when
//! it doesn't), made once.

use crate::com::mojang::renderpearl::api::device::gpu_device::GpuDevice;
use crate::com::mojang::renderpearl::api::textures::address_mode::AddressMode;
use crate::com::mojang::renderpearl::api::textures::filter_mode::FilterMode;
use crate::com::mojang::renderpearl::api::textures::gpu_sampler::GpuSampler;
use std::rc::Rc;

pub struct SamplerCache {
    samplers: Vec<Rc<dyn GpuSampler>>,
}

const ADDRESS_MODES: [AddressMode; 2] = [AddressMode::REPEAT, AddressMode::CLAMP_TO_EDGE];
const FILTER_MODES: [FilterMode; 2] = [FilterMode::NEAREST, FilterMode::LINEAR];

impl SamplerCache {
    /// `initialize()`, in Java's order.
    pub fn new(device: &dyn GpuDevice) -> SamplerCache {
        let mut samplers: Vec<Option<Rc<dyn GpuSampler>>> = vec![None; 32];
        for address_mode_u in ADDRESS_MODES {
            for address_mode_v in ADDRESS_MODES {
                for min_filter in FILTER_MODES {
                    for mag_filter in FILTER_MODES {
                        for use_mipmaps in [true, false] {
                            samplers[SamplerCache::encode(
                                address_mode_u,
                                address_mode_v,
                                min_filter,
                                mag_filter,
                                use_mipmaps,
                            )] = Some(device.create_sampler(
                                address_mode_u,
                                address_mode_v,
                                min_filter,
                                mag_filter,
                                1,
                                if use_mipmaps { None } else { Some(0.0) },
                            ));
                        }
                    }
                }
            }
        }
        SamplerCache {
            samplers: samplers
                .into_iter()
                .map(|sampler| sampler.expect("every sampler"))
                .collect(),
        }
    }

    pub fn get_sampler(
        &self,
        address_mode_u: AddressMode,
        address_mode_v: AddressMode,
        min_filter: FilterMode,
        mag_filter: FilterMode,
        use_mipmaps: bool,
    ) -> &Rc<dyn GpuSampler> {
        &self.samplers[SamplerCache::encode(
            address_mode_u,
            address_mode_v,
            min_filter,
            mag_filter,
            use_mipmaps,
        )]
    }

    pub fn get_clamp_to_edge(&self, min_mag: FilterMode) -> &Rc<dyn GpuSampler> {
        self.get_clamp_to_edge_mipmaps(min_mag, false)
    }

    pub fn get_clamp_to_edge_mipmaps(
        &self,
        min_mag: FilterMode,
        mipmaps: bool,
    ) -> &Rc<dyn GpuSampler> {
        self.get_sampler(
            AddressMode::CLAMP_TO_EDGE,
            AddressMode::CLAMP_TO_EDGE,
            min_mag,
            min_mag,
            mipmaps,
        )
    }

    pub fn get_repeat(&self, min_mag: FilterMode) -> &Rc<dyn GpuSampler> {
        self.get_sampler(
            AddressMode::REPEAT,
            AddressMode::REPEAT,
            min_mag,
            min_mag,
            false,
        )
    }

    fn encode(
        address_mode_u: AddressMode,
        address_mode_v: AddressMode,
        min_filter: FilterMode,
        mag_filter: FilterMode,
        use_mipmaps: bool,
    ) -> usize {
        let mut result = 0;
        result |= address_mode_u as usize & 1;
        result |= (address_mode_v as usize & 1) << 1;
        result |= (min_filter as usize & 1) << 2;
        result |= (mag_filter as usize & 1) << 3;
        if use_mipmaps {
            result |= 16;
        }
        result
    }
}
