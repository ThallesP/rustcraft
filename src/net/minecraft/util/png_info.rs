//! net/minecraft/util/PngInfo.java
//!
//! Only `validateHeader`, which `NativeImage.read` checks a PNG's signature and header with before
//! stb decodes it. Its `IOException`s are `Err`s with Java's message.

pub struct PngInfo;

impl PngInfo {
    const PNG_HEADER: i64 = -8552249625308161526;
    const IHDR_TYPE: i32 = 1229472850;

    pub fn validate_header(buffer: &[u8]) -> Result<(), String> {
        if buffer.len() < 33 {
            return Err("PNG header missing".to_owned());
        }
        let long_at = |offset: usize| {
            i64::from_be_bytes(buffer[offset..offset + 8].try_into().expect("8 bytes"))
        };
        let int_at = |offset: usize| {
            i32::from_be_bytes(buffer[offset..offset + 4].try_into().expect("4 bytes"))
        };
        if long_at(0) != PngInfo::PNG_HEADER {
            return Err("Bad PNG Signature".to_owned());
        }
        if int_at(8) != 13 {
            return Err("Bad length for IHDR chunk!".to_owned());
        }
        if int_at(12) != PngInfo::IHDR_TYPE {
            return Err("Bad type for IHDR chunk!".to_owned());
        }
        let width = int_at(16);
        let height = int_at(20);
        let bit_depth = buffer[24] as i8;
        if !PngInfo::validate_bit_depth(bit_depth) {
            return Err(format!("Invalid bit depth: {bit_depth}"));
        }
        let color_type = buffer[25] as i8;
        if !PngInfo::validate_color_type(color_type) {
            return Err(format!("Invalid color type: {color_type}"));
        }
        if !PngInfo::validate_bounds(width, height, bit_depth) {
            return Err(format!(
                "Bounds check failed, {width}x{height}@{bit_depth}, type: {color_type}"
            ));
        }
        Ok(())
    }

    fn validate_bit_depth(bit_depth: i8) -> bool {
        matches!(bit_depth, 1 | 2 | 4 | 8 | 16)
    }

    fn validate_color_type(color_type: i8) -> bool {
        matches!(color_type, 0 | 2 | 3 | 4 | 6)
    }

    /// A 16-bit image's RGBA16 buffer has to fit in an int.
    fn validate_bounds(width: i32, height: i32, bit_depth: i8) -> bool {
        if bit_depth == 16 {
            let buffer_estimate = 4i64
                .wrapping_mul(i64::from(width))
                .wrapping_mul(i64::from(height))
                .wrapping_mul(2);
            if buffer_estimate != i64::from(buffer_estimate as i32) {
                return false;
            }
        }
        true
    }
}
