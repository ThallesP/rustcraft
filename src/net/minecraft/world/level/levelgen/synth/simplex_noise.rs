//! net/minecraft/world/level/levelgen/synth/SimplexNoise.java
//!
//! Not ported yet: the 3D `get`, `range` and `Noise`'s volumes, which come with world generation;
//! the client reads the 2D noise (the swamp's grass, `Biome.BIOME_INFO_NOISE`).

use std::sync::LazyLock;

use crate::net::minecraft::util::mth::Mth;
use crate::net::minecraft::util::random_source::RandomSource;
use crate::net::minecraft::world::level::levelgen::synth::gradient_noise::GradientNoise;

// Java's `Math.sqrt` is correctly rounded, like Rust's (`Math`'s golden suite checks it).
static SQRT_3: LazyLock<f64> = LazyLock::new(|| 3.0f64.sqrt());
static F2: LazyLock<f64> = LazyLock::new(|| 0.5 * (*SQRT_3 - 1.0));
static G2: LazyLock<f64> = LazyLock::new(|| (3.0 - *SQRT_3) / 6.0);

/// Simplex noise over a permutation of 256.
#[derive(Clone, Debug)]
pub struct SimplexNoise {
    base: GradientNoise,
}

impl SimplexNoise {
    pub const STANDARD_DEVIATION: f64 = 0.42544;

    /// `SimplexNoise(random)`: offsets up to 256.
    pub fn new(random: &mut dyn RandomSource) -> SimplexNoise {
        SimplexNoise {
            base: GradientNoise::new(random, 256.0),
        }
    }

    /// `SimplexNoise(random, true)`: no offsets, though their randoms are still drawn.
    pub fn new_discarding_offset(random: &mut dyn RandomSource) -> SimplexNoise {
        SimplexNoise {
            base: GradientNoise::new(random, 0.0),
        }
    }

    fn get_corner_noise_3d(index: i32, x: f64, y: f64, z: f64, base: f64) -> f64 {
        let mut t0 = base - x * x - y * y - z * z;
        if t0 < 0.0 {
            0.0
        } else {
            t0 *= t0;
            t0 * t0 * GradientNoise::GRADIENT[index as usize].dot_f64(x, y, z)
        }
    }

    /// `get(double, double)`.
    pub fn get_2d(&self, x_in: f64, y_in: f64) -> f32 {
        let xin = x_in + self.base.offset_x;
        let yin = y_in + self.base.offset_y;
        let s = (xin + yin) * *F2;
        let i = Mth::floor_f64(xin + s);
        let j = Mth::floor_f64(yin + s);
        let t = f64::from(i.wrapping_add(j)) * *G2;
        let x0_origin = f64::from(i) - t;
        let y0_origin = f64::from(j) - t;
        let x0 = xin - x0_origin;
        let y0 = yin - y0_origin;
        let (i1, j1) = if x0 > y0 { (1, 0) } else { (0, 1) };

        let x1 = x0 - f64::from(i1) + *G2;
        let y1 = y0 - f64::from(j1) + *G2;
        let x2 = x0 - 1.0 + 2.0 * *G2;
        let y2 = y0 - 1.0 + 2.0 * *G2;
        let ii = i & 0xFF;
        let jj = j & 0xFF;
        let gi0 = self.base.permute(ii + self.base.permute(jj)) % 12;
        let gi1 = self.base.permute(ii + i1 + self.base.permute(jj + j1)) % 12;
        let gi2 = self.base.permute(ii + 1 + self.base.permute(jj + 1)) % 12;
        let n0 = SimplexNoise::get_corner_noise_3d(gi0, x0, y0, 0.0, 0.5);
        let n1 = SimplexNoise::get_corner_noise_3d(gi1, x1, y1, 0.0, 0.5);
        let n2 = SimplexNoise::get_corner_noise_3d(gi2, x2, y2, 0.0, 0.5);
        (70.0 * (n0 + n1 + n2)) as f32
    }
}
