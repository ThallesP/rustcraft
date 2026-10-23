//! net/minecraft/world/level/levelgen/synth/GradientNoise.java
//!
//! What the gradient noises share: their permutation table and offsets, made from a random
//! source, and the gradients. Java's abstract class is a struct its noises hold. Not ported yet:
//! `permuteToGrad`, `gradDot` and `wrap`, which the noises the client doesn't read use, and `Noise`
//! itself, whose 3D and volume methods come with world generation.

use crate::net::minecraft::util::random_source::RandomSource;

/// `GradientNoise.Gradient`: a gradient's direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Gradient {
    x: i32,
    y: i32,
    z: i32,
}

impl Gradient {
    const fn new(x: i32, y: i32, z: i32) -> Gradient {
        Gradient { x, y, z }
    }

    pub fn dot_f64(&self, x: f64, y: f64, z: f64) -> f64 {
        f64::from(self.x) * x + f64::from(self.y) * y + f64::from(self.z) * z
    }
}

/// The permutation and offsets.
#[derive(Clone, Debug)]
pub struct GradientNoise {
    perms: [i8; 256],
    pub(super) offset_x: f64,
    pub(super) offset_y: f64,
    // The 3D noise's, which isn't ported yet.
    #[allow(dead_code)]
    pub(super) offset_z: f64,
}

impl GradientNoise {
    pub const GRADIENT: [Gradient; 16] = [
        Gradient::new(1, 1, 0),
        Gradient::new(-1, 1, 0),
        Gradient::new(1, -1, 0),
        Gradient::new(-1, -1, 0),
        Gradient::new(1, 0, 1),
        Gradient::new(-1, 0, 1),
        Gradient::new(1, 0, -1),
        Gradient::new(-1, 0, -1),
        Gradient::new(0, 1, 1),
        Gradient::new(0, -1, 1),
        Gradient::new(0, 1, -1),
        Gradient::new(0, -1, -1),
        Gradient::new(1, 1, 0),
        Gradient::new(0, -1, 1),
        Gradient::new(-1, 1, 0),
        Gradient::new(0, -1, -1),
    ];

    /// `GradientNoise(random, noiseOffsetScale)`: three offsets, then the permutation shuffled.
    pub fn new(random: &mut dyn RandomSource, noise_offset_scale: f64) -> GradientNoise {
        let offset_x = random.next_double() * noise_offset_scale;
        let offset_y = random.next_double() * noise_offset_scale;
        let offset_z = random.next_double() * noise_offset_scale;
        let mut perms = [0i8; 256];

        for (i, perm) in perms.iter_mut().enumerate() {
            *perm = i as i8;
        }

        for i in 0..256 {
            let offset = random.next_int_bounded(256 - i) as usize;
            perms.swap(i as usize, offset + i as usize);
        }

        GradientNoise {
            perms,
            offset_x,
            offset_y,
            offset_z,
        }
    }

    pub fn permute(&self, x: i32) -> i32 {
        i32::from(self.perms[(x & 0xFF) as usize]) & 0xFF
    }
}
