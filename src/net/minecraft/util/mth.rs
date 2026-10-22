//! net/minecraft/util/Mth.java
//!
//! Not ported yet, until the types they take are: getInt (NumberUtils, which parses non-ASCII
//! digits), rayIntersectsAABB and mulAndTruncate (commons-lang Fraction).

use std::f64::consts::PI as PI_F64;
use std::sync::LazyLock;

use crate::java::lang::math::Math;
use crate::java::util::uuid::UUID;
use crate::net::minecraft::core::vec3i::Vec3i;
use crate::net::minecraft::util::argb::ARGB;
use crate::net::minecraft::util::random_source::{self, RandomSource};
use crate::net::minecraft::world::phys::vec3::Vec3;
use crate::org::joml::quaternionf::Quaternionf;
use crate::org::joml::vector3f::Vector3f;

const UUID_VERSION: i64 = 61440;
const UUID_VERSION_TYPE_4: i64 = 16384;
const UUID_VARIANT: i64 = -4611686018427387904;
const UUID_VARIANT_2: i64 = i64::MIN;
const SIN_QUANTIZATION: usize = 65536;
const SIN_MASK: i64 = 65535;
const COS_OFFSET: f64 = 16384.0;
const SIN_SCALE: f64 = 10430.378350470453;
static SIN: LazyLock<Box<[f32]>> = LazyLock::new(|| {
    (0..SIN_QUANTIZATION)
        .map(|i| Math::sin(i as f64 / SIN_SCALE) as f32)
        .collect()
});
const MULTIPLY_DE_BRUIJN_BIT_POSITION: [i32; 32] = [
    0, 1, 28, 2, 29, 14, 24, 3, 30, 22, 20, 15, 25, 17, 4, 8, 31, 27, 13, 23, 21, 19, 16, 7, 26,
    12, 18, 6, 11, 5, 10, 9,
];
const ONE_SIXTH: f64 = 0.16666666666666666;
const LUT_SIZE: usize = 257;
const FRAC_BIAS: f64 = f64::from_bits(4805340802404319232);
static ASIN_TAB: LazyLock<[f64; LUT_SIZE]> =
    LazyLock::new(|| std::array::from_fn(|ind| Math::asin(ind as f64 / 256.0)));
static COS_TAB: LazyLock<[f64; LUT_SIZE]> =
    LazyLock::new(|| std::array::from_fn(|ind| Math::cos(ASIN_TAB[ind])));

pub struct Mth;

impl Mth {
    pub const PI: f32 = PI_F64 as f32;
    pub const HALF_PI: f32 = (PI_F64 / 2.0) as f32;
    pub const TWO_PI: f32 = (PI_F64 * 2.0) as f32;
    pub const DEG_TO_RAD: f32 = (PI_F64 / 180.0) as f32;
    pub const RAD_TO_DEG: f32 = 180.0 / PI_F64 as f32;
    pub const EPSILON: f32 = 1.0E-5;
    /// `sqrt(2.0F)` in vanilla, which rounds to the same float.
    pub const SQRT_OF_TWO: f32 = std::f32::consts::SQRT_2;
    pub const Y_AXIS: Vector3f = Vector3f::from_xyz(0.0, 1.0, 0.0);
    pub const X_AXIS: Vector3f = Vector3f::from_xyz(1.0, 0.0, 0.0);
    pub const Z_AXIS: Vector3f = Vector3f::from_xyz(0.0, 0.0, 1.0);

    pub fn sin(i: f64) -> f32 {
        SIN[(((i * SIN_SCALE) as i64) & SIN_MASK) as usize]
    }

    pub fn cos(i: f64) -> f32 {
        SIN[(((i * SIN_SCALE + COS_OFFSET) as i64) & SIN_MASK) as usize]
    }

    pub fn sqrt(x: f32) -> f32 {
        f64::from(x).sqrt() as f32
    }

    pub fn floor_f32(v: f32) -> i32 {
        f64::from(v).floor() as i32
    }

    pub fn floor_f64(v: f64) -> i32 {
        v.floor() as i32
    }

    pub fn lfloor(v: f64) -> i64 {
        v.floor() as i64
    }

    pub fn abs_f32(v: f32) -> f32 {
        v.abs()
    }

    pub fn abs_i32(v: i32) -> i32 {
        Math::abs_i32(v)
    }

    pub fn ceil_f32(v: f32) -> i32 {
        f64::from(v).ceil() as i32
    }

    pub fn ceil_f64(v: f64) -> i32 {
        v.ceil() as i32
    }

    pub fn ceil_long(v: f64) -> i64 {
        v.ceil() as i64
    }

    // Not i32::clamp, which panics when min > max.
    pub fn clamp_i32(value: i32, min: i32, max: i32) -> i32 {
        value.max(min).min(max)
    }

    pub fn clamp_i64(value: i64, min: i64, max: i64) -> i64 {
        value.max(min).min(max)
    }

    pub fn clamp_f32(value: f32, min: f32, max: f32) -> f32 {
        if value < min {
            min
        } else {
            Math::min_f32(value, max)
        }
    }

    pub fn clamp_f64(value: f64, min: f64, max: f64) -> f64 {
        if value < min {
            min
        } else {
            Math::min_f64(value, max)
        }
    }

    pub fn clamped_lerp_f64(factor: f64, min: f64, max: f64) -> f64 {
        if factor < 0.0 {
            min
        } else if factor > 1.0 {
            max
        } else {
            Mth::lerp_f64(factor, min, max)
        }
    }

    pub fn clamped_lerp_f32(factor: f32, min: f32, max: f32) -> f32 {
        if factor < 0.0 {
            min
        } else if factor > 1.0 {
            max
        } else {
            Mth::lerp_f32(factor, min, max)
        }
    }

    pub fn abs_max_i32(a: i32, b: i32) -> i32 {
        Math::abs_i32(a).max(Math::abs_i32(b))
    }

    pub fn abs_max_f32(a: f32, b: f32) -> f32 {
        Math::max_f32(a.abs(), b.abs())
    }

    pub fn abs_max_f64(a: f64, b: f64) -> f64 {
        Math::max_f64(a.abs(), b.abs())
    }

    pub fn chessboard_distance(x0: i32, z0: i32, x1: i32, z1: i32) -> i32 {
        Mth::abs_max_i32(x1.wrapping_sub(x0), z1.wrapping_sub(z0))
    }

    pub fn floor_div(a: i32, b: i32) -> i32 {
        Math::floor_div_i32(a, b)
    }

    pub fn next_int(random: &mut dyn RandomSource, min_inclusive: i32, max_inclusive: i32) -> i32 {
        if min_inclusive >= max_inclusive {
            min_inclusive
        } else {
            random
                .next_int_bounded(max_inclusive.wrapping_sub(min_inclusive).wrapping_add(1))
                .wrapping_add(min_inclusive)
        }
    }

    pub fn next_float(random: &mut dyn RandomSource, min: f32, max: f32) -> f32 {
        if min >= max {
            min
        } else {
            random.next_float() * (max - min) + min
        }
    }

    pub fn next_double(random: &mut dyn RandomSource, min: f64, max: f64) -> f64 {
        if min >= max {
            min
        } else {
            random.next_double() * (max - min) + min
        }
    }

    pub fn equal_f32(a: f32, b: f32) -> bool {
        (b - a).abs() < 1.0E-5
    }

    /// The epsilon is the float 1.0E-5F, widened.
    pub fn equal_f64(a: f64, b: f64) -> bool {
        (b - a).abs() < f64::from(1.0E-5_f32)
    }

    pub fn positive_modulo_i32(input: i32, modulo: i32) -> i32 {
        Math::floor_mod_i32(input, modulo)
    }

    pub fn positive_modulo_f32(input: f32, modulo: f32) -> f32 {
        (input % modulo + modulo) % modulo
    }

    pub fn positive_modulo_f64(input: f64, modulo: f64) -> f64 {
        (input % modulo + modulo) % modulo
    }

    pub fn is_multiple_of(dividend: i32, divisor: i32) -> bool {
        dividend.wrapping_rem(divisor) == 0
    }

    pub fn pack_degrees(angle: f32) -> i8 {
        Mth::floor_f32(angle * 256.0 / 360.0) as i8
    }

    pub fn unpack_degrees(rot: i8) -> f32 {
        (i32::from(rot) * 360) as f32 / 256.0
    }

    pub fn wrap_degrees_i32(angle: i32) -> i32 {
        let mut normalized_angle = angle % 360;
        if normalized_angle >= 180 {
            normalized_angle -= 360;
        }
        if normalized_angle < -180 {
            normalized_angle += 360;
        }
        normalized_angle
    }

    pub fn wrap_degrees_i64(angle: i64) -> f32 {
        let mut normalized_angle = (angle % 360) as f32;
        if normalized_angle >= 180.0 {
            normalized_angle -= 360.0;
        }
        if normalized_angle < -180.0 {
            normalized_angle += 360.0;
        }
        normalized_angle
    }

    pub fn wrap_degrees_f32(angle: f32) -> f32 {
        let mut normalized_angle = angle % 360.0;
        if normalized_angle >= 180.0 {
            normalized_angle -= 360.0;
        }
        if normalized_angle < -180.0 {
            normalized_angle += 360.0;
        }
        normalized_angle
    }

    pub fn wrap_degrees_f64(angle: f64) -> f64 {
        let mut normalized_angle = angle % 360.0;
        if normalized_angle >= 180.0 {
            normalized_angle -= 360.0;
        }
        if normalized_angle < -180.0 {
            normalized_angle += 360.0;
        }
        normalized_angle
    }

    pub fn wrap_degrees90(angle: f32) -> f32 {
        let mut normalized_angle = angle % 90.0;
        if normalized_angle >= 45.0 {
            normalized_angle -= 90.0;
        }
        if normalized_angle < -45.0 {
            normalized_angle += 90.0;
        }
        normalized_angle
    }

    pub fn degrees_difference(from_angle: f32, to_angle: f32) -> f32 {
        Mth::wrap_degrees_f32(to_angle - from_angle)
    }

    pub fn degrees_difference_abs(angle_a: f32, angle_b: f32) -> f32 {
        Mth::abs_f32(Mth::degrees_difference(angle_a, angle_b))
    }

    pub fn rotate_if_necessary(base_angle: f32, target_angle: f32, max_angle_diff: f32) -> f32 {
        let delta_angle = Mth::degrees_difference(base_angle, target_angle);
        let delta_angle_clamped = Mth::clamp_f32(delta_angle, -max_angle_diff, max_angle_diff);
        target_angle - delta_angle_clamped
    }

    pub fn approach(current: f32, target: f32, increment: f32) -> f32 {
        let increment = Mth::abs_f32(increment);
        if current < target {
            Mth::clamp_f32(current + increment, current, target)
        } else {
            Mth::clamp_f32(current - increment, target, current)
        }
    }

    pub fn approach_degrees(current: f32, target: f32, increment: f32) -> f32 {
        let difference = Mth::degrees_difference(current, target);
        Mth::approach(current, current + difference, increment)
    }

    pub const fn smallest_encompassing_power_of_two(input: i32) -> i32 {
        let mut result = input.wrapping_sub(1);
        result |= result >> 1;
        result |= result >> 2;
        result |= result >> 4;
        result |= result >> 8;
        result |= result >> 16;
        result.wrapping_add(1)
    }

    pub fn smallest_square_side(item_count: i32) -> i32 {
        assert!(
            item_count >= 0,
            "itemCount must be greater than or equal to zero"
        );
        Mth::ceil_f64(f64::from(item_count).sqrt())
    }

    pub const fn is_power_of_two_i32(input: i32) -> bool {
        input != 0 && (input & input.wrapping_sub(1)) == 0
    }

    pub fn is_power_of_two_i64(input: i64) -> bool {
        input != 0 && (input & input.wrapping_sub(1)) == 0
    }

    pub const fn ceillog2(input: i32) -> i32 {
        let input = if Mth::is_power_of_two_i32(input) {
            input
        } else {
            Mth::smallest_encompassing_power_of_two(input)
        };
        MULTIPLY_DE_BRUIJN_BIT_POSITION
            [((((input as i64).wrapping_mul(125613361) >> 27) as i32) & 31) as usize]
    }

    pub const fn log2(input: i32) -> i32 {
        Mth::ceillog2(input)
            - if Mth::is_power_of_two_i32(input) {
                0
            } else {
                1
            }
    }

    pub fn frac_f32(num: f32) -> f32 {
        num - Mth::floor_f32(num) as f32
    }

    pub fn frac_f64(num: f64) -> f64 {
        num - Mth::lfloor(num) as f64
    }

    pub fn get_seed_vec3i(vec: &Vec3i) -> i64 {
        #[allow(deprecated)]
        Mth::get_seed(vec.get_x(), vec.get_y(), vec.get_z())
    }

    /// `x * 3129871` is an int multiply that wraps before it's widened to long.
    #[deprecated]
    pub fn get_seed(x: i32, y: i32, z: i32) -> i64 {
        let seed = i64::from(x.wrapping_mul(3129871))
            ^ i64::from(z).wrapping_mul(116129781)
            ^ i64::from(y);
        let seed = seed
            .wrapping_mul(seed)
            .wrapping_mul(42317861)
            .wrapping_add(seed.wrapping_mul(11));
        seed >> 16
    }

    pub fn create_insecure_uuid(random: &mut dyn RandomSource) -> UUID {
        let most = (random.next_long() & !UUID_VERSION) | UUID_VERSION_TYPE_4;
        let least = (random.next_long() & !UUID_VARIANT) | UUID_VARIANT_2;
        UUID::new(most, least)
    }

    pub fn inverse_lerp_f64(value: f64, min: f64, max: f64) -> f64 {
        (value - min) / (max - min)
    }

    pub fn inverse_lerp_f32(value: f32, min: f32, max: f32) -> f32 {
        (value - min) / (max - min)
    }

    // A lookup-table atan2. Out-of-range table indexes panic where vanilla throws.
    pub fn atan2(y: f64, x: f64) -> f64 {
        let d2 = x * x + y * y;
        if d2.is_nan() {
            return f64::NAN;
        }

        let neg_y = y < 0.0;
        let y = if neg_y { -y } else { y };
        let neg_x = x < 0.0;
        let x = if neg_x { -x } else { x };
        let steep = y > x;
        let (x, y) = if steep { (y, x) } else { (x, y) };

        #[allow(deprecated)]
        let rinv = Mth::fast_inv_sqrt(d2);
        let x = x * rinv;
        let y = y * rinv;
        let yp = FRAC_BIAS + y;
        let index = yp.to_bits() as i32;
        let phi = ASIN_TAB[index as usize];
        let c_phi = COS_TAB[index as usize];
        let s_phi = yp - FRAC_BIAS;
        let sd = y * c_phi - x * s_phi;
        let d = (6.0 + sd * sd) * sd * ONE_SIXTH;
        let mut theta = phi + d;
        if steep {
            theta = PI_F64 / 2.0 - theta;
        }
        if neg_x {
            theta = PI_F64 - theta;
        }
        if neg_y {
            theta = -theta;
        }
        theta
    }

    pub fn inv_sqrt_f32(x: f32) -> f32 {
        1.0 / (f64::from(x).sqrt() as f32)
    }

    pub fn inv_sqrt_f64(x: f64) -> f64 {
        1.0 / x.sqrt()
    }

    #[deprecated]
    pub fn fast_inv_sqrt(x: f64) -> f64 {
        let xhalf = 0.5 * x;
        let i = 6910469410427058090_i64.wrapping_sub((x.to_bits() as i64) >> 1);
        let x = f64::from_bits(i as u64);
        x * (1.5 - xhalf * x * x)
    }

    /// Reads the bits with `Float.floatToIntBits`, which turns every NaN into the canonical one.
    pub fn fast_inv_cube_root(x: f32) -> f32 {
        let i = if x.is_nan() {
            0x7fc00000
        } else {
            x.to_bits() as i32
        };
        let i = 1419967116 - i / 3;
        let y = f32::from_bits(i as u32);
        let y = 0.6666667 * y + 1.0 / (3.0 * y * y * x);
        0.6666667 * y + 1.0 / (3.0 * y * y * x)
    }

    pub fn hsv_to_rgb(hue: f32, saturation: f32, value: f32) -> i32 {
        Mth::hsv_to_argb(hue, saturation, value, 0)
    }

    pub fn hsv_to_argb(hue: f32, saturation: f32, value: f32, alpha: i32) -> i32 {
        let h = (hue * 6.0) as i32 % 6;
        let f = hue * 6.0 - h as f32;
        let p = value * (1.0 - saturation);
        let q = value * (1.0 - f * saturation);
        let t = value * (1.0 - (1.0 - f) * saturation);
        let (red, green, blue) = match h {
            0 => (value, t, p),
            1 => (q, value, p),
            2 => (p, value, t),
            3 => (p, q, value),
            4 => (t, p, value),
            5 => (value, p, q),
            _ => panic!(
                "Something went wrong when converting from HSV to RGB. Input was {hue}, {saturation}, {value}"
            ),
        };
        ARGB::color(
            alpha,
            Mth::clamp_i32((red * 255.0) as i32, 0, 255),
            Mth::clamp_i32((green * 255.0) as i32, 0, 255),
            Mth::clamp_i32((blue * 255.0) as i32, 0, 255),
        )
    }

    pub fn murmur_hash3_mixer(hash: i32) -> i32 {
        let hash = hash ^ ((hash as u32) >> 16) as i32;
        let hash = hash.wrapping_mul(-2048144789);
        let hash = hash ^ ((hash as u32) >> 13) as i32;
        let hash = hash.wrapping_mul(-1028477387);
        hash ^ ((hash as u32) >> 16) as i32
    }

    pub fn binary_search(from: i32, to: i32, mut condition: impl FnMut(i32) -> bool) -> i32 {
        let mut from = from;
        let mut len = to.wrapping_sub(from);
        while len > 0 {
            let half = len / 2;
            let middle = from.wrapping_add(half);
            if condition(middle) {
                len = half;
            } else {
                from = middle.wrapping_add(1);
                len -= half + 1;
            }
        }
        from
    }

    pub fn lerp_int(alpha1: f32, p0: i32, p1: i32) -> i32 {
        p0.wrapping_add(Mth::floor_f32(alpha1 * p1.wrapping_sub(p0) as f32))
    }

    pub fn lerp_discrete(alpha1: f32, p0: i32, p1: i32) -> i32 {
        let delta = p1.wrapping_sub(p0);
        p0.wrapping_add(Mth::floor_f32(alpha1 * delta.wrapping_sub(1) as f32))
            .wrapping_add(if alpha1 > 0.0 { 1 } else { 0 })
    }

    pub fn lerp_f32(alpha1: f32, p0: f32, p1: f32) -> f32 {
        p0 + alpha1 * (p1 - p0)
    }

    pub fn lerp2_f32(alpha1: f32, alpha2: f32, x00: f32, x10: f32, x01: f32, x11: f32) -> f32 {
        Mth::lerp_f32(
            alpha2,
            Mth::lerp_f32(alpha1, x00, x10),
            Mth::lerp_f32(alpha1, x01, x11),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn lerp3_f32(
        alpha1: f32,
        alpha2: f32,
        alpha3: f32,
        x000: f32,
        x100: f32,
        x010: f32,
        x110: f32,
        x001: f32,
        x101: f32,
        x011: f32,
        x111: f32,
    ) -> f32 {
        Mth::lerp_f32(
            alpha3,
            Mth::lerp2_f32(alpha1, alpha2, x000, x100, x010, x110),
            Mth::lerp2_f32(alpha1, alpha2, x001, x101, x011, x111),
        )
    }

    pub fn lerp_vec3(alpha: f64, p1: &Vec3, p2: &Vec3) -> Vec3 {
        Vec3::new(
            Mth::lerp_f64(alpha, p1.x, p2.x),
            Mth::lerp_f64(alpha, p1.y, p2.y),
            Mth::lerp_f64(alpha, p1.z, p2.z),
        )
    }

    pub fn lerp_f64(alpha1: f64, p0: f64, p1: f64) -> f64 {
        p0 + alpha1 * (p1 - p0)
    }

    pub fn lerp2_f64(alpha1: f64, alpha2: f64, x00: f64, x10: f64, x01: f64, x11: f64) -> f64 {
        Mth::lerp_f64(
            alpha2,
            Mth::lerp_f64(alpha1, x00, x10),
            Mth::lerp_f64(alpha1, x01, x11),
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn lerp3_f64(
        alpha1: f64,
        alpha2: f64,
        alpha3: f64,
        x000: f64,
        x100: f64,
        x010: f64,
        x110: f64,
        x001: f64,
        x101: f64,
        x011: f64,
        x111: f64,
    ) -> f64 {
        Mth::lerp_f64(
            alpha3,
            Mth::lerp2_f64(alpha1, alpha2, x000, x100, x010, x110),
            Mth::lerp2_f64(alpha1, alpha2, x001, x101, x011, x111),
        )
    }

    pub fn catmullrom(alpha: f32, p0: f32, p1: f32, p2: f32, p3: f32) -> f32 {
        0.5 * (2.0 * p1
            + (p2 - p0) * alpha
            + (2.0 * p0 - 5.0 * p1 + 4.0 * p2 - p3) * alpha * alpha
            + (3.0 * p1 - p0 - 3.0 * p2 + p3) * alpha * alpha * alpha)
    }

    pub fn smoothstep(x: f32) -> f32 {
        x * x * x * (x * (x * 6.0 - 15.0) + 10.0)
    }

    pub fn smoothstep_derivative(x: f32) -> f32 {
        30.0 * x * x * (x - 1.0) * (x - 1.0)
    }

    pub fn sign(number: f64) -> i32 {
        if number == 0.0 {
            0
        } else if number > 0.0 {
            1
        } else {
            -1
        }
    }

    pub fn rot_lerp_f32(a: f32, from: f32, to: f32) -> f32 {
        from + a * Mth::wrap_degrees_f32(to - from)
    }

    pub fn rot_lerp_f64(a: f64, from: f64, to: f64) -> f64 {
        from + a * Mth::wrap_degrees_f64(to - from)
    }

    /// Never returns once `to - from` is so large that adding 2π no longer changes it, like vanilla.
    pub fn rot_lerp_rad(a: f32, from: f32, to: f32) -> f32 {
        let mut diff = to - from;
        while diff < -Mth::PI {
            diff += Mth::TWO_PI;
        }
        while diff >= Mth::PI {
            diff -= Mth::TWO_PI;
        }
        from + a * diff
    }

    pub fn triangle_wave(index: f32, period: f32) -> f32 {
        ((index % period - period * 0.5).abs() - period * 0.25) / (period * 0.25)
    }

    pub fn square_f32(x: f32) -> f32 {
        x * x
    }

    pub fn cube(x: f32) -> f32 {
        x * x * x
    }

    pub fn square_f64(x: f64) -> f64 {
        x * x
    }

    pub fn square_i32(x: i32) -> i32 {
        x.wrapping_mul(x)
    }

    pub fn square_i64(x: i64) -> i64 {
        x.wrapping_mul(x)
    }

    pub fn clamped_map_f64(
        value: f64,
        from_min: f64,
        from_max: f64,
        to_min: f64,
        to_max: f64,
    ) -> f64 {
        Mth::clamped_lerp_f64(
            Mth::inverse_lerp_f64(value, from_min, from_max),
            to_min,
            to_max,
        )
    }

    pub fn clamped_map_f32(
        value: f32,
        from_min: f32,
        from_max: f32,
        to_min: f32,
        to_max: f32,
    ) -> f32 {
        Mth::clamped_lerp_f32(
            Mth::inverse_lerp_f32(value, from_min, from_max),
            to_min,
            to_max,
        )
    }

    pub fn map_f64(value: f64, from_min: f64, from_max: f64, to_min: f64, to_max: f64) -> f64 {
        Mth::lerp_f64(
            Mth::inverse_lerp_f64(value, from_min, from_max),
            to_min,
            to_max,
        )
    }

    pub fn map_f32(value: f32, from_min: f32, from_max: f32, to_min: f32, to_max: f32) -> f32 {
        Mth::lerp_f32(
            Mth::inverse_lerp_f32(value, from_min, from_max),
            to_min,
            to_max,
        )
    }

    pub fn wobble(coord: f64) -> f64 {
        let mut random = random_source::create_thread_local_instance_with_seed(i64::from(
            Mth::floor_f64(coord * 3000.0),
        ));
        coord + (2.0 * random.next_double() - 1.0) * 1.0E-7 / 2.0
    }

    pub fn round_toward_i32(input: i32, multiple: i32) -> i32 {
        Mth::positive_ceil_div_i32(input, multiple).wrapping_mul(multiple)
    }

    pub fn round_toward_i64(input: i64, multiple: i64) -> i64 {
        Mth::positive_ceil_div_i64(input, multiple).wrapping_mul(multiple)
    }

    pub fn positive_ceil_div_i32(input: i32, divisor: i32) -> i32 {
        Math::floor_div_i32(input.wrapping_neg(), divisor).wrapping_neg()
    }

    pub fn positive_ceil_div_i64(input: i64, divisor: i64) -> i64 {
        Math::floor_div_i64(input.wrapping_neg(), divisor).wrapping_neg()
    }

    pub fn random_between_inclusive(
        random: &mut dyn RandomSource,
        min: i32,
        max_inclusive: i32,
    ) -> i32 {
        random
            .next_int_bounded(max_inclusive.wrapping_sub(min).wrapping_add(1))
            .wrapping_add(min)
    }

    pub fn random_between(random: &mut dyn RandomSource, min: f32, max_exclusive: f32) -> f32 {
        random.next_float() * (max_exclusive - min) + min
    }

    pub fn normal(random: &mut dyn RandomSource, mean: f32, deviation: f32) -> f32 {
        mean + random.next_gaussian() as f32 * deviation
    }

    pub fn length_squared_f64(x: f64, y: f64) -> f64 {
        x * x + y * y
    }

    pub fn length_squared_f32(x: f32, y: f32) -> f32 {
        x * x + y * y
    }

    pub fn length_f64(x: f64, y: f64) -> f64 {
        Mth::length_squared_f64(x, y).sqrt()
    }

    pub fn length_f32(x: f32, y: f32) -> f32 {
        f64::from(Mth::length_squared_f32(x, y)).sqrt() as f32
    }

    pub fn length_squared3_f64(x: f64, y: f64, z: f64) -> f64 {
        x * x + y * y + z * z
    }

    pub fn length3_f64(x: f64, y: f64, z: f64) -> f64 {
        Mth::length_squared3_f64(x, y, z).sqrt()
    }

    pub fn length3_f32(x: f32, y: f32, z: f32) -> f32 {
        Mth::sqrt(Mth::length_squared3_f32(x, y, z))
    }

    pub fn length_squared3_f32(x: f32, y: f32, z: f32) -> f32 {
        x * x + y * y + z * z
    }

    pub fn quantize(value: f64, quantize_resolution: i32) -> i32 {
        Mth::floor_f64(value / f64::from(quantize_resolution)).wrapping_mul(quantize_resolution)
    }

    pub fn out_from_origin(
        origin: i32,
        lower_bound: i32,
        upper_bound: i32,
    ) -> impl Iterator<Item = i32> {
        Mth::out_from_origin_step(origin, lower_bound, upper_bound, 1)
    }

    /// Checks its arguments up front and yields lazily, like vanilla's IntStream.iterate.
    pub fn out_from_origin_step(
        origin: i32,
        lower_bound: i32,
        upper_bound: i32,
        step_size: i32,
    ) -> impl Iterator<Item = i32> {
        assert!(
            lower_bound <= upper_bound,
            "upperBound {upper_bound} expected to be > lowerBound {lower_bound}"
        );
        assert!(
            step_size >= 1,
            "step size expected to be >= 1, was {step_size}"
        );

        let clamped_origin = Mth::clamp_i32(origin, lower_bound, upper_bound);
        std::iter::successors(Some(clamped_origin), move |&cursor| {
            let previous_was_negative = cursor <= clamped_origin;
            let current_distance = Math::abs_i32(clamped_origin.wrapping_sub(cursor));
            let can_move_positive = clamped_origin
                .wrapping_add(current_distance)
                .wrapping_add(step_size)
                <= upper_bound;
            if !previous_was_negative || !can_move_positive {
                let attempted_step = clamped_origin
                    .wrapping_sub(current_distance)
                    .wrapping_sub(if previous_was_negative { step_size } else { 0 });
                if attempted_step >= lower_bound {
                    return Some(attempted_step);
                }
            }
            Some(
                clamped_origin
                    .wrapping_add(current_distance)
                    .wrapping_add(step_size),
            )
        })
        .take_while(move |&cursor| {
            let current_distance = Math::abs_i32(clamped_origin.wrapping_sub(cursor));
            clamped_origin.wrapping_sub(current_distance) >= lower_bound
                || clamped_origin.wrapping_add(current_distance) <= upper_bound
        })
    }

    pub fn rotation_around_axis<'a>(
        axis: &Vector3f,
        rotation: &Quaternionf,
        result: &'a mut Quaternionf,
    ) -> &'a mut Quaternionf {
        let projected_length = axis.dot_xyz(rotation.x, rotation.y, rotation.z);
        result
            .set_xyzw(
                axis.x * projected_length,
                axis.y * projected_length,
                axis.z * projected_length,
                rotation.w,
            )
            .normalize()
    }
}

#[cfg(test)]
mod tests {
    use super::Mth;
    use crate::golden::{Golden, STREAM_LIMIT};
    use crate::net::minecraft::core::vec3i::Vec3i;
    use crate::net::minecraft::world::phys::vec3::Vec3;
    use crate::org::joml::quaternionf::Quaternionf;
    use crate::org::joml::vector3f::Vector3f;

    #[test]
    #[allow(deprecated)]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.util.Mth");
        golden.field("PI", Mth::PI);
        golden.field("HALF_PI", Mth::HALF_PI);
        golden.field("TWO_PI", Mth::TWO_PI);
        golden.field("DEG_TO_RAD", Mth::DEG_TO_RAD);
        golden.field("RAD_TO_DEG", Mth::RAD_TO_DEG);
        golden.field("EPSILON", Mth::EPSILON);
        golden.field("SQRT_OF_TWO", Mth::SQRT_OF_TWO);

        golden.check("sin(D)F", Mth::sin);
        golden.check("cos(D)F", Mth::cos);
        golden.check("sqrt(F)F", Mth::sqrt);
        golden.check("floor(F)I", Mth::floor_f32);
        golden.check("floor(D)I", Mth::floor_f64);
        golden.check("lfloor(D)J", Mth::lfloor);
        golden.check("abs(F)F", Mth::abs_f32);
        golden.check("abs(I)I", Mth::abs_i32);
        golden.check("ceil(F)I", Mth::ceil_f32);
        golden.check("ceil(D)I", Mth::ceil_f64);
        golden.check("ceilLong(D)J", Mth::ceil_long);
        golden.check("clamp(III)I", Mth::clamp_i32);
        golden.check("clamp(JJJ)J", Mth::clamp_i64);
        golden.check("clamp(FFF)F", Mth::clamp_f32);
        golden.check("clamp(DDD)D", Mth::clamp_f64);
        golden.check("clampedLerp(DDD)D", Mth::clamped_lerp_f64);
        golden.check("clampedLerp(FFF)F", Mth::clamped_lerp_f32);
        golden.check("absMax(II)I", Mth::abs_max_i32);
        golden.check("absMax(FF)F", Mth::abs_max_f32);
        golden.check("absMax(DD)D", Mth::abs_max_f64);
        golden.check("chessboardDistance(IIII)I", Mth::chessboard_distance);
        golden.check("floorDiv(II)I", Mth::floor_div);
        golden.check_random(
            "nextInt(Lnet/minecraft/util/RandomSource;II)I",
            Mth::next_int,
        );
        golden.check_random(
            "nextFloat(Lnet/minecraft/util/RandomSource;FF)F",
            Mth::next_float,
        );
        golden.check_random(
            "nextDouble(Lnet/minecraft/util/RandomSource;DD)D",
            Mth::next_double,
        );
        golden.check("equal(FF)Z", Mth::equal_f32);
        golden.check("equal(DD)Z", Mth::equal_f64);
        golden.check("positiveModulo(II)I", Mth::positive_modulo_i32);
        golden.check("positiveModulo(FF)F", Mth::positive_modulo_f32);
        golden.check("positiveModulo(DD)D", Mth::positive_modulo_f64);
        golden.check("isMultipleOf(II)Z", Mth::is_multiple_of);
        golden.check("packDegrees(F)B", Mth::pack_degrees);
        golden.check("unpackDegrees(B)F", Mth::unpack_degrees);
        golden.check("wrapDegrees(I)I", Mth::wrap_degrees_i32);
        golden.check("wrapDegrees(J)F", Mth::wrap_degrees_i64);
        golden.check("wrapDegrees(F)F", Mth::wrap_degrees_f32);
        golden.check("wrapDegrees(D)D", Mth::wrap_degrees_f64);
        golden.check("wrapDegrees90(F)F", Mth::wrap_degrees90);
        golden.check("degreesDifference(FF)F", Mth::degrees_difference);
        golden.check("degreesDifferenceAbs(FF)F", Mth::degrees_difference_abs);
        golden.check("rotateIfNecessary(FFF)F", Mth::rotate_if_necessary);
        golden.check("approach(FFF)F", Mth::approach);
        golden.check("approachDegrees(FFF)F", Mth::approach_degrees);
        golden.check(
            "smallestEncompassingPowerOfTwo(I)I",
            Mth::smallest_encompassing_power_of_two,
        );
        golden.check("smallestSquareSide(I)I", Mth::smallest_square_side);
        golden.check("isPowerOfTwo(I)Z", Mth::is_power_of_two_i32);
        golden.check("isPowerOfTwo(J)Z", Mth::is_power_of_two_i64);
        golden.check("ceillog2(I)I", Mth::ceillog2);
        golden.check("log2(I)I", Mth::log2);
        golden.check("frac(F)F", Mth::frac_f32);
        golden.check("frac(D)D", Mth::frac_f64);
        golden.check("getSeed(III)J", Mth::get_seed);
        golden.check_random(
            "createInsecureUUID(Lnet/minecraft/util/RandomSource;)Ljava/util/UUID;",
            Mth::create_insecure_uuid,
        );
        golden.check("inverseLerp(DDD)D", Mth::inverse_lerp_f64);
        golden.check("inverseLerp(FFF)F", Mth::inverse_lerp_f32);
        golden.check("atan2(DD)D", Mth::atan2);
        golden.check("invSqrt(F)F", Mth::inv_sqrt_f32);
        golden.check("invSqrt(D)D", Mth::inv_sqrt_f64);
        golden.check("fastInvSqrt(D)D", Mth::fast_inv_sqrt);
        golden.check("fastInvCubeRoot(F)F", Mth::fast_inv_cube_root);
        golden.check("hsvToRgb(FFF)I", Mth::hsv_to_rgb);
        golden.check("hsvToArgb(FFFI)I", Mth::hsv_to_argb);
        golden.check("murmurHash3Mixer(I)I", Mth::murmur_hash3_mixer);
        golden.check("lerpInt(FII)I", Mth::lerp_int);
        golden.check("lerpDiscrete(FII)I", Mth::lerp_discrete);
        golden.check("lerp(FFF)F", Mth::lerp_f32);
        golden.check("lerp2(FFFFFF)F", Mth::lerp2_f32);
        golden.check("lerp3(FFFFFFFFFFF)F", Mth::lerp3_f32);
        golden.check("lerp(DDD)D", Mth::lerp_f64);
        golden.check("lerp2(DDDDDD)D", Mth::lerp2_f64);
        golden.check("lerp3(DDDDDDDDDDD)D", Mth::lerp3_f64);
        golden.check("catmullrom(FFFFF)F", Mth::catmullrom);
        golden.check("smoothstep(F)F", Mth::smoothstep);
        golden.check("smoothstepDerivative(F)F", Mth::smoothstep_derivative);
        golden.check("sign(D)I", Mth::sign);
        golden.check("rotLerp(FFF)F", Mth::rot_lerp_f32);
        golden.check("rotLerp(DDD)D", Mth::rot_lerp_f64);
        golden.check("rotLerpRad(FFF)F", Mth::rot_lerp_rad);
        golden.check("triangleWave(FF)F", Mth::triangle_wave);
        golden.check("square(F)F", Mth::square_f32);
        golden.check("cube(F)F", Mth::cube);
        golden.check("square(D)D", Mth::square_f64);
        golden.check("square(I)I", Mth::square_i32);
        golden.check("square(J)J", Mth::square_i64);
        golden.check("clampedMap(DDDDD)D", Mth::clamped_map_f64);
        golden.check("clampedMap(FFFFF)F", Mth::clamped_map_f32);
        golden.check("map(DDDDD)D", Mth::map_f64);
        golden.check("map(FFFFF)F", Mth::map_f32);
        golden.check("wobble(D)D", Mth::wobble);
        golden.check("roundToward(II)I", Mth::round_toward_i32);
        golden.check("roundToward(JJ)J", Mth::round_toward_i64);
        golden.check("positiveCeilDiv(II)I", Mth::positive_ceil_div_i32);
        golden.check("positiveCeilDiv(JJ)J", Mth::positive_ceil_div_i64);
        golden.check_random(
            "randomBetweenInclusive(Lnet/minecraft/util/RandomSource;II)I",
            Mth::random_between_inclusive,
        );
        golden.check_random(
            "randomBetween(Lnet/minecraft/util/RandomSource;FF)F",
            Mth::random_between,
        );
        golden.check_random("normal(Lnet/minecraft/util/RandomSource;FF)F", Mth::normal);
        golden.check("lengthSquared(DD)D", Mth::length_squared_f64);
        golden.check("lengthSquared(FF)F", Mth::length_squared_f32);
        golden.check("length(DD)D", Mth::length_f64);
        golden.check("length(FF)F", Mth::length_f32);
        golden.check("lengthSquared(DDD)D", Mth::length_squared3_f64);
        golden.check("length(DDD)D", Mth::length3_f64);
        golden.check("length(FFF)F", Mth::length3_f32);
        golden.check("lengthSquared(FFF)F", Mth::length_squared3_f32);
        golden.check("quantize(DI)I", Mth::quantize);
        golden.check(
            "outFromOrigin(III)Ljava/util/stream/IntStream;",
            |origin: i32, lower: i32, upper: i32| {
                Mth::out_from_origin(origin, lower, upper)
                    .take(STREAM_LIMIT)
                    .collect::<Vec<_>>()
            },
        );
        golden.check(
            "outFromOrigin(IIII)Ljava/util/stream/IntStream;",
            |origin: i32, lower: i32, upper: i32, step: i32| {
                Mth::out_from_origin_step(origin, lower, upper, step)
                    .take(STREAM_LIMIT)
                    .collect::<Vec<_>>()
            },
        );

        golden.pending(
            "binarySearch(IILjava/util/function/IntPredicate;)I",
            "vanilla can't generate predicates yet",
        );
        golden.pending(
            "getInt(Ljava/lang/String;I)I",
            "needs Integer.parseInt, which takes non-ASCII digits",
        );
        golden.check("getSeed(Lnet/minecraft/core/Vec3i;)J", |vec: Vec3i| {
            Mth::get_seed_vec3i(&vec)
        });
        golden.pending(
            "rayIntersectsAABB(Lnet/minecraft/world/phys/Vec3;Lnet/minecraft/world/phys/Vec3;Lnet/minecraft/world/phys/AABB;)Z",
            "AABB isn't ported",
        );
        golden.check(
            "lerp(DLnet/minecraft/world/phys/Vec3;Lnet/minecraft/world/phys/Vec3;)Lnet/minecraft/world/phys/Vec3;",
            |alpha, p1: Vec3, p2: Vec3| Mth::lerp_vec3(alpha, &p1, &p2),
        );
        golden.check(
            "rotationAroundAxis(Lorg/joml/Vector3fc;Lorg/joml/Quaternionf;Lorg/joml/Quaternionf;)Lorg/joml/Quaternionf;",
            |axis: Vector3f, rotation: Quaternionf, mut result: Quaternionf| {
                *Mth::rotation_around_axis(&axis, &rotation, &mut result)
            },
        );
        golden.pending(
            "mulAndTruncate(Lorg/apache/commons/lang3/math/Fraction;I)I",
            "commons-lang Fraction isn't ported",
        );
        golden.field("X_AXIS", Mth::X_AXIS);
        golden.field("Y_AXIS", Mth::Y_AXIS);
        golden.field("Z_AXIS", Mth::Z_AXIS);
        golden.finish();
    }
}
