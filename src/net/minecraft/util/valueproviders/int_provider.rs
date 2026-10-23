//! net/minecraft/util/valueproviders/IntProvider.java
//!
//! An int a data file gives as a constant or a distribution. Java's interface is implemented by
//! one record per type of `BuiltInRegistries.INT_PROVIDER_TYPE`; here it's an enum of the ones
//! ported (`ConstantInt`, `UniformInt`). The others (`biased_to_bottom`,
//! `very_biased_to_bottom`, `clamped`, `weighted_list`, `clamped_normal`, `trapezoid`) fail to
//! decode saying so (`IntProviders`), and `sample` comes with its callers.

use std::fmt;

use crate::net::minecraft::util::valueproviders::constant_int::ConstantInt;
use crate::net::minecraft::util::valueproviders::uniform_int::UniformInt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntProvider {
    Constant(ConstantInt),
    Uniform(UniformInt),
}

impl IntProvider {
    pub fn min_inclusive(&self) -> i32 {
        match self {
            IntProvider::Constant(provider) => provider.min_inclusive(),
            IntProvider::Uniform(provider) => provider.min_inclusive(),
        }
    }

    pub fn max_inclusive(&self) -> i32 {
        match self {
            IntProvider::Constant(provider) => provider.max_inclusive(),
            IntProvider::Uniform(provider) => provider.max_inclusive(),
        }
    }
}

impl fmt::Display for IntProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            IntProvider::Constant(provider) => provider.fmt(f),
            IntProvider::Uniform(provider) => provider.fmt(f),
        }
    }
}
