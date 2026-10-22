//! net/minecraft/world/level/levelgen/MarsagliaPolarGaussian.java
//!
//! In Java this holds a reference back to the RandomSource that owns it. Here the owner passes
//! itself in, which keeps the struct a plain value.

use crate::java::lang::math::Math;
use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::util::random_source::RandomSource;

#[derive(Clone, Copy, Debug, Default)]
pub struct MarsagliaPolarGaussian {
    next_next_gaussian: f64,
    have_next_next_gaussian: bool,
}

impl MarsagliaPolarGaussian {
    pub fn new() -> MarsagliaPolarGaussian {
        MarsagliaPolarGaussian::default()
    }

    pub fn reset(&mut self) {
        self.have_next_next_gaussian = false;
    }

    pub fn next_gaussian(&mut self, random_source: &mut (impl RandomSource + ?Sized)) -> f64 {
        if self.have_next_next_gaussian {
            self.have_next_next_gaussian = false;
            return self.next_next_gaussian;
        }

        let (x, y, radius_squared) = loop {
            let x = 2.0 * random_source.next_double() - 1.0;
            let y = 2.0 * random_source.next_double() - 1.0;
            let radius_squared = Mth::square_f64(x) + Mth::square_f64(y);
            // Negated like vanilla's loop condition, so NaN exits the loop too.
            #[allow(clippy::neg_cmp_op_on_partial_ord)]
            if !(radius_squared >= 1.0 || radius_squared == 0.0) {
                break (x, y, radius_squared);
            }
        };

        let multiplier = (-2.0 * Math::log(radius_squared) / radius_squared).sqrt();
        self.next_next_gaussian = y * multiplier;
        self.have_next_next_gaussian = true;
        x * multiplier
    }
}
