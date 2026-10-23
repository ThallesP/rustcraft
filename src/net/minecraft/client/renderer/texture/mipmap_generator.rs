//! net/minecraft/client/renderer/texture/MipmapGenerator.java
//!
//! A sprite's mip levels, each a quarter of the one before: the four pixels' mean in linear light
//! (or `darkenedAlphaBlend` for dark cutouts), and for cutouts the alpha scaled so as much of each
//! level passes the alpha test as of the first. Float math runs in Java's order.

use crate::com::mojang::blaze3d::platform::native_image::NativeImage;
use crate::com::mojang::blaze3d::platform::texture_util::TextureUtil;
use crate::com::mojang::blaze3d::platform::transparency::Transparency;
use crate::java::lang::math::Math;
use crate::net::minecraft::client::renderer::texture::mipmap_strategy::MipmapStrategy;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::util::argb::ARGB;

pub struct MipmapGenerator;

const ITEM_PREFIX: &str = "item/";
const ALPHA_CUTOFF: f32 = 0.5;
const STRICT_ALPHA_CUTOFF: f32 = 0.3;

impl MipmapGenerator {
    fn alpha_test_coverage(image: &NativeImage, alpha_ref: f32, alpha_scale: f32) -> f32 {
        let width = image.get_width();
        let height = image.get_height();
        let mut coverage = 0.0f32;
        let alpha = |x: i32, y: i32| {
            Math::clamp_f32(
                ARGB::alpha_float(image.get_pixel(x, y)) * alpha_scale,
                0.0,
                1.0,
            )
        };
        for y in 0..height - 1 {
            for x in 0..width - 1 {
                let alpha00 = alpha(x, y);
                let alpha10 = alpha(x + 1, y);
                let alpha01 = alpha(x, y + 1);
                let alpha11 = alpha(x + 1, y + 1);
                let mut texel_coverage = 0.0f32;
                for subsample_y in 0..4 {
                    let fy = (subsample_y as f32 + 0.5) / 4.0;
                    for subsample_x in 0..4 {
                        let fx = (subsample_x as f32 + 0.5) / 4.0;
                        let alpha = alpha00 * (1.0 - fx) * (1.0 - fy)
                            + alpha10 * fx * (1.0 - fy)
                            + alpha01 * (1.0 - fx) * fy
                            + alpha11 * fx * fy;
                        if alpha > alpha_ref {
                            texel_coverage += 1.0;
                        }
                    }
                }
                coverage += texel_coverage / 16.0;
            }
        }
        coverage / ((width - 1).wrapping_mul(height - 1)) as f32
    }

    #[allow(clippy::neg_cmp_op_on_partial_ord)]
    fn scale_alpha_to_coverage(
        image: &mut NativeImage,
        desired_coverage: f32,
        alpha_ref: f32,
        alpha_cutoff_bias: f32,
    ) {
        let mut min_alpha_scale = 0.0f32;
        let mut max_alpha_scale = 4.0f32;
        let mut alpha_scale = 1.0f32;
        let mut best_alpha_scale = 1.0f32;
        let mut best_error = f32::MAX;
        let width = image.get_width();
        let height = image.get_height();

        for _ in 0..5 {
            let current_coverage =
                MipmapGenerator::alpha_test_coverage(image, alpha_ref, alpha_scale);
            let error = (current_coverage - desired_coverage).abs();
            if error < best_error {
                best_error = error;
                best_alpha_scale = alpha_scale;
            }
            if current_coverage < desired_coverage {
                min_alpha_scale = alpha_scale;
            } else {
                if !(current_coverage > desired_coverage) {
                    break;
                }
                max_alpha_scale = alpha_scale;
            }
            alpha_scale = (min_alpha_scale + max_alpha_scale) * 0.5;
        }

        for y in 0..height {
            for x in 0..width {
                let pixel = image.get_pixel(x, y);
                let mut alpha = ARGB::alpha_float(pixel);
                alpha = alpha * best_alpha_scale + alpha_cutoff_bias + 0.025;
                alpha = Math::clamp_f32(alpha, 0.0, 1.0);
                image.set_pixel(x, y, ARGB::color_alpha_f32(alpha, pixel));
            }
        }
    }

    /// `generateMipLevels`: the levels up to `new_mip_level`, keeping those `current_mips` has.
    /// The first level is solidified (or darkened) in place, as Java changes the image it was
    /// given.
    pub fn generate_mip_levels(
        name: &Identifier,
        mut current_mips: Vec<NativeImage>,
        new_mip_level: i32,
        mut mipmap_strategy: MipmapStrategy,
        alpha_cutoff_bias: f32,
        transparency: Transparency,
    ) -> Vec<NativeImage> {
        if mipmap_strategy == MipmapStrategy::AUTO {
            mipmap_strategy = if transparency.has_transparent {
                MipmapStrategy::CUTOUT
            } else {
                MipmapStrategy::MEAN
            };
        }

        if current_mips.len() == 1 && !name.get_path().starts_with(ITEM_PREFIX) {
            if mipmap_strategy == MipmapStrategy::CUTOUT
                || mipmap_strategy == MipmapStrategy::STRICT_CUTOUT
            {
                TextureUtil::solidify(&mut current_mips[0]);
            } else if mipmap_strategy == MipmapStrategy::DARK_CUTOUT {
                TextureUtil::fill_empty_areas_with_dark_color(&mut current_mips[0]);
            }
        }

        if (new_mip_level as usize) < current_mips.len() {
            return current_mips;
        }

        let is_cutout_mip = mipmap_strategy == MipmapStrategy::CUTOUT
            || mipmap_strategy == MipmapStrategy::STRICT_CUTOUT
            || mipmap_strategy == MipmapStrategy::DARK_CUTOUT;
        let cutout_ref = if mipmap_strategy == MipmapStrategy::STRICT_CUTOUT {
            STRICT_ALPHA_CUTOFF
        } else {
            ALPHA_CUTOFF
        };
        let original_coverage = if is_cutout_mip {
            MipmapGenerator::alpha_test_coverage(&current_mips[0], cutout_ref, 1.0)
        } else {
            0.0
        };

        let existing = current_mips.len();
        let mut result = current_mips;
        for level in 1..=new_mip_level as usize {
            if level >= existing {
                let last_data = &result[level - 1];
                let mut data =
                    NativeImage::new(last_data.get_width() >> 1, last_data.get_height() >> 1);
                for x in 0..data.get_width() {
                    for y in 0..data.get_height() {
                        let color1 = last_data.get_pixel(x * 2, y * 2);
                        let color2 = last_data.get_pixel(x * 2 + 1, y * 2);
                        let color3 = last_data.get_pixel(x * 2, y * 2 + 1);
                        let color4 = last_data.get_pixel(x * 2 + 1, y * 2 + 1);
                        let color = if mipmap_strategy == MipmapStrategy::DARK_CUTOUT {
                            MipmapGenerator::darkened_alpha_blend(color1, color2, color3, color4)
                        } else {
                            ARGB::mean_linear(color1, color2, color3, color4)
                        };
                        data.set_pixel(x, y, color);
                    }
                }
                result.push(data);
            }
            if is_cutout_mip {
                MipmapGenerator::scale_alpha_to_coverage(
                    &mut result[level],
                    original_coverage,
                    cutout_ref,
                    alpha_cutoff_bias,
                );
            }
        }
        result
    }

    fn darkened_alpha_blend(color1: i32, color2: i32, color3: i32, color4: i32) -> i32 {
        let mut a_total = 0.0f32;
        let mut r_total = 0.0f32;
        let mut g_total = 0.0f32;
        let mut b_total = 0.0f32;
        for color in [color1, color2, color3, color4] {
            if ARGB::alpha(color) != 0 {
                a_total += ARGB::srgb_to_linear_channel(ARGB::alpha(color));
                r_total += ARGB::srgb_to_linear_channel(ARGB::red(color));
                g_total += ARGB::srgb_to_linear_channel(ARGB::green(color));
                b_total += ARGB::srgb_to_linear_channel(ARGB::blue(color));
            }
        }
        a_total /= 4.0;
        r_total /= 4.0;
        g_total /= 4.0;
        b_total /= 4.0;
        ARGB::color(
            ARGB::linear_to_srgb_channel(a_total),
            ARGB::linear_to_srgb_channel(r_total),
            ARGB::linear_to_srgb_channel(g_total),
            ARGB::linear_to_srgb_channel(b_total),
        )
    }
}
