//! com/mojang/blaze3d/buffers/Std140SizeCalculator.java
//!
//! The size of a uniform block laid out like `Std140Builder` writes it.

#[derive(Clone, Copy, Debug, Default)]
pub struct Std140SizeCalculator {
    size: i32,
}

impl Std140SizeCalculator {
    pub const fn new() -> Std140SizeCalculator {
        Std140SizeCalculator { size: 0 }
    }

    pub const fn get(self) -> i32 {
        self.size
    }

    pub const fn align(mut self, alignment: i32) -> Std140SizeCalculator {
        // Mth.roundToward, which a const fn can't call.
        self.size = (self.size + alignment - 1) / alignment * alignment;
        self
    }

    pub const fn put_float(self) -> Std140SizeCalculator {
        self.put(4, 4)
    }

    pub const fn put_int(self) -> Std140SizeCalculator {
        self.put(4, 4)
    }

    pub const fn put_vec2(self) -> Std140SizeCalculator {
        self.put(8, 8)
    }

    pub const fn put_ivec2(self) -> Std140SizeCalculator {
        self.put(8, 8)
    }

    pub const fn put_vec3(self) -> Std140SizeCalculator {
        self.put(16, 12)
    }

    pub const fn put_ivec3(self) -> Std140SizeCalculator {
        self.put(16, 12)
    }

    pub const fn put_vec4(self) -> Std140SizeCalculator {
        self.put(16, 16)
    }

    pub const fn put_ivec4(self) -> Std140SizeCalculator {
        self.put(16, 16)
    }

    pub const fn put_mat4f(self) -> Std140SizeCalculator {
        self.put(16, 64)
    }

    const fn put(self, alignment: i32, size: i32) -> Std140SizeCalculator {
        let mut aligned = self.align(alignment);
        aligned.size += size;
        aligned
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::net::minecraft::util::mth::Mth;

    /// `align` is `Mth.roundToward`, spelled out for a const fn.
    #[test]
    fn aligns_like_round_toward() {
        for size in 0..100 {
            for alignment in [4, 8, 16] {
                let calculator = Std140SizeCalculator { size }.align(alignment);
                assert_eq!(calculator.get(), Mth::round_toward_i32(size, alignment));
            }
        }
    }
}
