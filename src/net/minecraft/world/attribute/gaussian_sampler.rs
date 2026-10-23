//! net/minecraft/world/attribute/GaussianSampler.java
//!
//! A 6 by 6 by 6 sample of a grid around a position, each point weighted by a binomial kernel
//! that slides with the position's fraction. `Sampler` and `Accumulator` are closures.

use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::world::phys::vec3::Vec3;

pub struct GaussianSampler;

impl GaussianSampler {
    const GAUSSIAN_SAMPLE_RADIUS: i32 = 2;
    const GAUSSIAN_SAMPLE_BREADTH: usize = 6;
    const GAUSSIAN_SAMPLE_KERNEL: [f64; 7] = [0.0, 1.0, 4.0, 6.0, 4.0, 1.0, 0.0];

    pub fn sample<V>(
        position: &Vec3,
        sampler: impl Fn(i32, i32, i32) -> V,
        mut accumulator: impl FnMut(f64, V),
    ) {
        // position.subtract(0.5, 0.5, 0.5).
        let (x, y, z) = (position.x - 0.5, position.y - 0.5, position.z - 0.5);
        let integral_x = Mth::floor_f64(x);
        let integral_y = Mth::floor_f64(y);
        let integral_z = Mth::floor_f64(z);
        let relative_x = x - f64::from(integral_x);
        let relative_y = y - f64::from(integral_y);
        let relative_z = z - f64::from(integral_z);
        let kernel = GaussianSampler::GAUSSIAN_SAMPLE_KERNEL;
        let radius = GaussianSampler::GAUSSIAN_SAMPLE_RADIUS;

        for z in 0..GaussianSampler::GAUSSIAN_SAMPLE_BREADTH {
            let weight_z = Mth::lerp_f64(relative_z, kernel[z + 1], kernel[z]);
            let sample_z = integral_z - radius + z as i32;

            for x in 0..GaussianSampler::GAUSSIAN_SAMPLE_BREADTH {
                let weight_x = Mth::lerp_f64(relative_x, kernel[x + 1], kernel[x]);
                let sample_x = integral_x - radius + x as i32;

                for y in 0..GaussianSampler::GAUSSIAN_SAMPLE_BREADTH {
                    let weight_y = Mth::lerp_f64(relative_y, kernel[y + 1], kernel[y]);
                    let sample_y = integral_y - radius + y as i32;
                    let sample_weight = weight_x * weight_y * weight_z;
                    let value = sampler(sample_x, sample_y, sample_z);
                    accumulator(sample_weight, value);
                }
            }
        }
    }
}
