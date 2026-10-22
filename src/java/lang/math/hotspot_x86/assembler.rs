//! The x86-64 instructions HotSpot's libm stubs use, as methods on register values, so that a
//! stub ports line for line: `__ mulsd(xmm1, xmm0);` becomes `xmm1.mulsd(xmm0);`.
//!
//! An XMM register is two 64-bit lanes. Scalar instructions (`addsd`) work on the low lane and
//! keep the high one, packed ones (`addpd`) work on both, and loads from memory or a general
//! purpose register (`movq`, `movdl`) zero what they don't write. The stubs park values in the
//! high lanes, so those are modelled too. Float lanes are kept as bits, since the integer
//! instructions (`pand`, `psrlq`, `paddd`) work on the same registers.
//!
//! A general purpose register is a `u64`. As on x86-64, 32-bit instructions (`addl`) zero the
//! upper half of their destination, and shift counts are masked to the operand width.
//!
//! Float arithmetic is Rust's: IEEE 754 with round-to-nearest, no flush-to-zero, and no fused
//! multiply-add, which is what SSE does under the MXCSR the JVM runs with. NaN payloads may
//! differ, which never matters (TESTING.md).
//!
//! Comparisons and jumps stay in the stubs as `if`s, with the signedness of the jump: `above` and
//! `below` compare unsigned, `greater` and `less` signed.

/// An XMM register: the low lane, then the high lane.
#[derive(Clone, Copy, Debug, Default)]
#[repr(transparent)]
pub struct Xmm([u64; 2]);

/// Memory at a byte offset into one of the stubs' tables, like HotSpot's `Address(rax, 8)`.
/// The port keeps addresses as offsets: `lea(rax, ExternalAddress(Ctable))` sets `rax` to 0.
#[derive(Clone, Copy)]
pub struct Address(pub &'static [u32], pub u64);

/// A 32-bit source operand: the low half of a register, an immediate or memory.
pub trait Dword: Copy {
    fn dword(self) -> u32;
}

/// A 64-bit source operand: a register (an XMM register's low lane), a sign-extended
/// immediate or memory.
pub trait Qword: Copy {
    fn qword(self) -> u64;
}

/// A 128-bit source operand: an XMM register or memory.
pub trait Xmmword: Qword {
    fn xmmword(self) -> Xmm;
}

impl Dword for u64 {
    fn dword(self) -> u32 {
        self as u32
    }
}

impl Dword for i32 {
    fn dword(self) -> u32 {
        self as u32
    }
}

impl Dword for Address {
    fn dword(self) -> u32 {
        self.0[(self.1 / 4) as usize]
    }
}

impl Qword for u64 {
    fn qword(self) -> u64 {
        self
    }
}

impl Qword for i32 {
    fn qword(self) -> u64 {
        i64::from(self) as u64
    }
}

impl Qword for Address {
    fn qword(self) -> u64 {
        u64::from(self.dword()) | u64::from(Address(self.0, self.1 + 4).dword()) << 32
    }
}

impl Qword for Xmm {
    fn qword(self) -> u64 {
        self.0[0]
    }
}

impl Xmmword for Address {
    fn xmmword(self) -> Xmm {
        Xmm([self.qword(), Address(self.0, self.1 + 8).qword()])
    }
}

impl Xmmword for Xmm {
    fn xmmword(self) -> Xmm {
        self
    }
}

impl From<Xmm> for f64 {
    fn from(xmm: Xmm) -> f64 {
        f64::from_bits(xmm.0[0])
    }
}

fn dwords(xmm: Xmm) -> [u32; 4] {
    let [low, high] = xmm.0;
    [
        low as u32,
        (low >> 32) as u32,
        high as u32,
        (high >> 32) as u32,
    ]
}

fn from_dwords(dwords: [u32; 4]) -> Xmm {
    Xmm([
        u64::from(dwords[0]) | u64::from(dwords[1]) << 32,
        u64::from(dwords[2]) | u64::from(dwords[3]) << 32,
    ])
}

/// What `cvttsd2sil` and `cvttpd2dq` make of one lane.
fn truncate_i32(bits: u64) -> i32 {
    let value = f64::from_bits(bits);
    if value > -2147483649.0 && value < 2147483648.0 {
        value as i32
    } else {
        i32::MIN
    }
}

impl Xmm {
    /// A register holding `value` in its low lane. What a caller leaves in the high lane never
    /// reaches a stub's result, so it starts at zero.
    pub fn new(value: f64) -> Xmm {
        Xmm([value.to_bits(), 0])
    }

    fn scalar(&mut self, src: impl Qword, operation: fn(f64, f64) -> f64) {
        self.0[0] = operation(f64::from_bits(self.0[0]), f64::from_bits(src.qword())).to_bits();
    }

    fn packed(&mut self, src: impl Xmmword, operation: fn(f64, f64) -> f64) {
        let src = src.xmmword();
        for lane in 0..2 {
            self.0[lane] =
                operation(f64::from_bits(self.0[lane]), f64::from_bits(src.0[lane])).to_bits();
        }
    }

    fn bitwise(&mut self, src: impl Xmmword, operation: fn(u64, u64) -> u64) {
        let src = src.xmmword();
        for lane in 0..2 {
            self.0[lane] = operation(self.0[lane], src.0[lane]);
        }
    }

    pub fn movq(&mut self, src: impl Qword) {
        *self = Xmm([src.qword(), 0]);
    }

    /// The load form only; stores to the stack become locals in the stubs.
    pub fn movsd(&mut self, src: impl Qword) {
        self.movq(src);
    }

    pub fn movdq(&mut self, src: u64) {
        self.movq(src);
    }

    pub fn movdl(&mut self, src: impl Dword) {
        *self = Xmm([u64::from(src.dword()), 0]);
    }

    pub fn movdqu(&mut self, src: impl Xmmword) {
        *self = src.xmmword();
    }

    pub fn movapd(&mut self, src: impl Xmmword) {
        self.movdqu(src);
    }

    /// HotSpot emits `movlhps` instead on CPUs without SSE3. Both copy the low lane to the high one.
    pub fn movddup(&mut self, src: impl Qword) {
        let low = src.qword();
        *self = Xmm([low, low]);
    }

    pub fn pshufd(&mut self, src: impl Xmmword, order: u32) {
        let src = dwords(src.xmmword());
        *self = from_dwords(std::array::from_fn(|i| {
            src[(order >> (2 * i) & 3) as usize]
        }));
    }

    pub fn unpcklpd(&mut self, src: impl Xmmword) {
        self.0[1] = src.xmmword().0[0];
    }

    pub fn unpckhpd(&mut self, src: impl Xmmword) {
        *self = Xmm([self.0[1], src.xmmword().0[1]]);
    }

    pub fn pinsrw(&mut self, src: impl Dword, index: u32) {
        let shift = 16 * (index % 4);
        let lane = &mut self.0[(index / 4) as usize];
        *lane = *lane & !(0xffff << shift) | u64::from(src.dword() as u16) << shift;
    }

    pub fn pand(&mut self, src: impl Xmmword) {
        self.bitwise(src, |a, b| a & b);
    }

    pub fn andpd(&mut self, src: impl Xmmword) {
        self.pand(src);
    }

    pub fn por(&mut self, src: impl Xmmword) {
        self.bitwise(src, |a, b| a | b);
    }

    pub fn xorpd(&mut self, src: impl Xmmword) {
        self.bitwise(src, |a, b| a ^ b);
    }

    pub fn psllq(&mut self, count: u32) {
        self.0 = self.0.map(|lane| lane.checked_shl(count).unwrap_or(0));
    }

    pub fn psrlq(&mut self, count: u32) {
        self.0 = self.0.map(|lane| lane.checked_shr(count).unwrap_or(0));
    }

    pub fn psrld(&mut self, count: u32) {
        *self = from_dwords(dwords(*self).map(|dword| dword.checked_shr(count).unwrap_or(0)));
    }

    pub fn paddd(&mut self, src: impl Xmmword) {
        let (a, b) = (dwords(*self), dwords(src.xmmword()));
        *self = from_dwords(std::array::from_fn(|i| a[i].wrapping_add(b[i])));
    }

    pub fn paddq(&mut self, src: impl Xmmword) {
        let src = src.xmmword();
        self.0 = std::array::from_fn(|lane| self.0[lane].wrapping_add(src.0[lane]));
    }

    pub fn addsd(&mut self, src: impl Qword) {
        self.scalar(src, |a, b| a + b);
    }

    pub fn subsd(&mut self, src: impl Qword) {
        self.scalar(src, |a, b| a - b);
    }

    pub fn mulsd(&mut self, src: impl Qword) {
        self.scalar(src, |a, b| a * b);
    }

    pub fn divsd(&mut self, src: impl Qword) {
        self.scalar(src, |a, b| a / b);
    }

    /// Correctly rounded, like Rust's `sqrt`.
    pub fn sqrtsd(&mut self, src: impl Qword) {
        self.scalar(src, |_, b| b.sqrt());
    }

    /// Multiplies the lowest float lane and keeps the other three.
    pub fn mulss(&mut self, src: Xmm) {
        let mut dwords = dwords(*self);
        dwords[0] = (f32::from_bits(dwords[0]) * f32::from_bits(src.0[0] as u32)).to_bits();
        *self = from_dwords(dwords);
    }

    pub fn addpd(&mut self, src: impl Xmmword) {
        self.packed(src, |a, b| a + b);
    }

    pub fn subpd(&mut self, src: impl Xmmword) {
        self.packed(src, |a, b| a - b);
    }

    pub fn mulpd(&mut self, src: impl Xmmword) {
        self.packed(src, |a, b| a * b);
    }

    pub fn cvtsi2sdl(&mut self, src: impl Dword) {
        self.0[0] = f64::from(src.dword() as i32).to_bits();
    }

    /// Rounds to nearest like SSE's default rounding mode, as Rust's `as` does.
    pub fn cvtsi2sdq(&mut self, src: impl Qword) {
        self.0[0] = (src.qword() as i64 as f64).to_bits();
    }

    /// Truncates both lanes to 32-bit integers in the low lane, like `cvttsd2sil` does one, and
    /// zeroes the high lane.
    pub fn cvttpd2dq(&mut self, src: impl Xmmword) {
        let [low, high] = src.xmmword().0.map(|lane| truncate_i32(lane) as u32);
        *self = from_dwords([low, high, 0, 0]);
    }

    /// Converts the two lowest 32-bit integers to the two lanes, exactly.
    pub fn cvtdq2pd(&mut self, src: impl Qword) {
        let src = src.qword();
        *self =
            Xmm([src as u32, (src >> 32) as u32].map(|dword| f64::from(dword as i32).to_bits()));
    }

    /// The approximate reciprocal of each float lane. It's only specified to 1.5 * 2^-12, and
    /// Intel's and AMD's CPUs return different bits, so this runs the real instruction: the port
    /// then matches vanilla on whatever CPU both run on.
    #[cfg(target_arch = "x86_64")]
    pub fn rcpps(&mut self, src: impl Xmmword) {
        use std::arch::x86_64::{__m128, _mm_rcp_ps};
        // SAFETY: Xmm and __m128 are both 16 bytes, and any bits are a valid value of either.
        unsafe {
            let reciprocal = _mm_rcp_ps(std::mem::transmute::<Xmm, __m128>(src.xmmword()));
            *self = std::mem::transmute::<__m128, Xmm>(reciprocal);
        }
    }

    /// Elsewhere there's no rcpps to run. A correctly rounded reciprocal, with denormals read as
    /// zero like rcpps does, is within its error bound but won't match any CPU bit for bit.
    #[cfg(not(target_arch = "x86_64"))]
    pub fn rcpps(&mut self, src: impl Xmmword) {
        *self = from_dwords(dwords(src.xmmword()).map(|bits| {
            let value = f32::from_bits(bits);
            let value = if value.is_subnormal() {
                0.0f32.copysign(value)
            } else {
                value
            };
            (1.0 / value).to_bits()
        }));
    }
}

/// The general purpose instructions, on registers held as `u64`.
pub trait Gpr {
    fn mov64(&mut self, imm: u64);
    fn movq(&mut self, src: u64);
    fn movl(&mut self, src: impl Dword);
    /// Writes the low 16 bits and keeps the rest, unlike the 32-bit instructions.
    fn movw(&mut self, src: impl Dword);
    fn movdq(&mut self, src: Xmm);
    fn movdl(&mut self, src: Xmm);
    fn pextrw(&mut self, src: Xmm, index: u32);
    /// Truncates toward zero. NaN and out-of-range values give the "integer indefinite"
    /// `i32::MIN`, where Rust's `as` would saturate.
    fn cvttsd2sil(&mut self, src: impl Qword);
    fn cvttsd2siq(&mut self, src: impl Qword);
    fn addl(&mut self, src: impl Dword);
    fn subl(&mut self, src: impl Dword);
    fn andl(&mut self, src: impl Dword);
    fn orl(&mut self, src: impl Dword);
    fn xorl(&mut self, src: impl Dword);
    fn negl(&mut self);
    fn notl(&mut self);
    fn shll(&mut self, count: impl Dword);
    fn shrl(&mut self, count: impl Dword);
    fn sarl(&mut self, count: impl Dword);
    fn addq(&mut self, src: impl Qword);
    /// Returns the borrow (the carry flag) for a following `sbbq`.
    fn subq(&mut self, src: impl Qword) -> bool;
    fn sbbq(&mut self, src: impl Qword, borrow: bool) -> bool;
    fn andq(&mut self, src: impl Qword);
    fn orq(&mut self, src: impl Qword);
    fn imulq(&mut self, src: impl Qword);
    fn shlq(&mut self, count: impl Dword);
    fn shrq(&mut self, count: impl Dword);
    /// The index of the highest set bit. The stubs never scan zero, where it's undefined.
    fn bsrq(&mut self, src: u64);
    fn bsrl(&mut self, src: u64);
}

impl Gpr for u64 {
    fn mov64(&mut self, imm: u64) {
        *self = imm;
    }

    fn movq(&mut self, src: u64) {
        *self = src;
    }

    fn movl(&mut self, src: impl Dword) {
        *self = u64::from(src.dword());
    }

    fn movw(&mut self, src: impl Dword) {
        *self = *self & !0xffff | u64::from(src.dword() as u16);
    }

    fn movdq(&mut self, src: Xmm) {
        *self = src.0[0];
    }

    fn movdl(&mut self, src: Xmm) {
        *self = u64::from(src.0[0] as u32);
    }

    fn pextrw(&mut self, src: Xmm, index: u32) {
        *self = src.0[(index / 4) as usize] >> (16 * (index % 4)) & 0xffff;
    }

    fn cvttsd2sil(&mut self, src: impl Qword) {
        *self = u64::from(truncate_i32(src.qword()) as u32);
    }

    fn cvttsd2siq(&mut self, src: impl Qword) {
        let value = f64::from_bits(src.qword());
        let truncated = if (-9223372036854775808.0..9223372036854775808.0).contains(&value) {
            value as i64
        } else {
            i64::MIN
        };
        *self = truncated as u64;
    }

    fn addl(&mut self, src: impl Dword) {
        *self = u64::from((*self as u32).wrapping_add(src.dword()));
    }

    fn subl(&mut self, src: impl Dword) {
        *self = u64::from((*self as u32).wrapping_sub(src.dword()));
    }

    fn andl(&mut self, src: impl Dword) {
        *self = u64::from(*self as u32 & src.dword());
    }

    fn orl(&mut self, src: impl Dword) {
        *self = u64::from(*self as u32 | src.dword());
    }

    fn xorl(&mut self, src: impl Dword) {
        *self = u64::from(*self as u32 ^ src.dword());
    }

    fn negl(&mut self) {
        *self = u64::from((*self as u32).wrapping_neg());
    }

    fn notl(&mut self) {
        *self = u64::from(!(*self as u32));
    }

    fn shll(&mut self, count: impl Dword) {
        *self = u64::from((*self as u32).wrapping_shl(count.dword()));
    }

    fn shrl(&mut self, count: impl Dword) {
        *self = u64::from((*self as u32).wrapping_shr(count.dword()));
    }

    fn sarl(&mut self, count: impl Dword) {
        *self = u64::from((*self as i32).wrapping_shr(count.dword()) as u32);
    }

    fn addq(&mut self, src: impl Qword) {
        *self = self.wrapping_add(src.qword());
    }

    fn subq(&mut self, src: impl Qword) -> bool {
        self.sbbq(src, false)
    }

    fn sbbq(&mut self, src: impl Qword, borrow: bool) -> bool {
        let (difference, borrow_out) = self.overflowing_sub(src.qword());
        let (difference, borrow_in_out) = difference.overflowing_sub(u64::from(borrow));
        *self = difference;
        borrow_out || borrow_in_out
    }

    fn andq(&mut self, src: impl Qword) {
        *self &= src.qword();
    }

    fn orq(&mut self, src: impl Qword) {
        *self |= src.qword();
    }

    fn imulq(&mut self, src: impl Qword) {
        *self = self.wrapping_mul(src.qword());
    }

    fn shlq(&mut self, count: impl Dword) {
        *self = self.wrapping_shl(count.dword());
    }

    fn shrq(&mut self, count: impl Dword) {
        *self = self.wrapping_shr(count.dword());
    }

    fn bsrq(&mut self, src: u64) {
        *self = u64::from(63 - src.leading_zeros());
    }

    fn bsrl(&mut self, src: u64) {
        *self = u64::from(31 - (src as u32).leading_zeros());
    }
}
