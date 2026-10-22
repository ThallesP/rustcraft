//! src/hotspot/cpu/x86/stubGenerator_x86_64_sin.cpp, the stub HotSpot calls for `Math.sin`.
//!
//! Inputs with |x| < 90112 are reduced by multiples of pi/32 in double arithmetic and looked
//! up in `CTABLE`. Larger ones go through a Payne-Hanek reduction in 64-bit integers against
//! `PI_INV_TABLE`, then the same lookup. Tiny inputs short-cut: subnormals are multiplied by
//! 0.9999999999999999, and every other |x| < 2^-252 comes back unchanged, since the stub
//! computes `2^-55 * (2^55 * x - x)` into xmm3 but returns xmm0.

/*
 * Copyright (c) 2016, 2024, Intel Corporation. All rights reserved.
 * Intel Math Library (LIBM) Source Code
 *
 * DO NOT ALTER OR REMOVE COPYRIGHT NOTICES OR THIS FILE HEADER.
 *
 * This code is free software; you can redistribute it and/or modify it
 * under the terms of the GNU General Public License version 2 only, as
 * published by the Free Software Foundation.
 *
 * This code is distributed in the hope that it will be useful, but WITHOUT
 * ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
 * FITNESS FOR A PARTICULAR PURPOSE.  See the GNU General Public License
 * version 2 for more details (a copy is included in the LICENSE file that
 * accompanied this code).
 *
 * You should have received a copy of the GNU General Public License version
 * 2 along with this work; if not, write to the Free Software Foundation,
 * Inc., 51 Franklin St, Fifth Floor, Boston, MA 02110-1301 USA.
 *
 * Please contact Oracle, 500 Oracle Parkway, Redwood Shores, CA 94065 USA
 * or visit www.oracle.com if you need additional information or have any
 * questions.
 *
 */

use super::assembler::{Address, Gpr, Xmm};
use super::stub_generator_x86_64_constants::*;

static ALL_ONES: [u32; 2] = [0xffffffff, 0x3fefffff];

pub fn libm_sin(x: f64) -> f64 {
    let mut xmm0 = Xmm::new(x);
    let [mut xmm1, mut xmm2, mut xmm3, mut xmm4] = [Xmm::default(); 4];
    let [mut xmm5, mut xmm6, mut xmm7] = [Xmm::default(); 3];
    let [mut rax, mut rbx, mut rcx, mut rdx, mut rsi] = [0u64; 5];
    let [mut rdi, mut r8, mut r9, mut r10, mut r11] = [0u64; 5];

    // movsd(Address(rsp, 8), xmm0), then movl(rax, Address(rsp, 12)) reads its high half.
    let rsp_8 = x.to_bits();
    rax.movl(rsp_8 >> 32);
    xmm1.movq(Address(&PI32INV, 0));
    xmm2.movq(Address(&SHIFTER, 0));
    rax.andl(2147418112);
    rax.subl(808452096);
    if rax as u32 <= 281346048 {
        xmm1.mulsd(xmm0);
        xmm5.movdqu(Address(&ONEHALF, 0));
        xmm4.movq(Address(&SIGN_MASK, 0));
        xmm4.pand(xmm0);
        xmm5.por(xmm4);
        xmm1.addpd(xmm5);
        rdx.cvttsd2sil(xmm1);
        xmm1.cvtsi2sdl(rdx);
        xmm6.movdqu(Address(&P_2, 0));
        r8.mov64(0x3fb921fb54400000);
        xmm3.movdq(r8);
        xmm5.movdqu(Address(&SC_4, 0));
        xmm4.pshufd(xmm0, 68);
        xmm3.mulsd(xmm1);
        xmm1.movddup(xmm1);
        rdx.andl(63);
        rdx.shll(5);
        rax.mov64(0); // lea(rax, ExternalAddress(Ctable))
        rax.addq(rdx);
        xmm6.mulpd(xmm1);
        xmm1.mulsd(Address(&P_3, 0));
        xmm4.subsd(xmm3);
        xmm7.movq(Address(&CTABLE, rax + 8));
        xmm0.subsd(xmm3);
        xmm3.movddup(xmm4);
        xmm4.subsd(xmm6);
        xmm0.pshufd(xmm0, 68);
        xmm2.movdqu(Address(&CTABLE, rax));
        xmm5.mulpd(xmm0);
        xmm0.subpd(xmm6);
        xmm7.mulsd(xmm4);
        xmm3.subsd(xmm4);
        xmm5.mulpd(xmm0);
        xmm0.mulpd(xmm0);
        xmm3.subsd(xmm6);
        xmm6.movdqu(Address(&SC_2, 0));
        xmm1.subsd(xmm3);
        xmm3.movq(Address(&CTABLE, rax + 24));
        xmm2.addsd(xmm3);
        xmm7.subsd(xmm2);
        xmm2.mulsd(xmm4);
        xmm6.mulpd(xmm0);
        xmm3.mulsd(xmm4);
        xmm2.mulpd(xmm0);
        xmm0.mulpd(xmm0);
        xmm5.addpd(Address(&SC_3, 0));
        xmm4.mulsd(Address(&CTABLE, rax));
        xmm6.addpd(Address(&SC_1, 0));
        xmm5.mulpd(xmm0);
        xmm0.movdqu(xmm3);
        xmm3.addsd(Address(&CTABLE, rax + 8));
        xmm1.mulpd(xmm7);
        xmm7.movdqu(xmm4);
        xmm4.addsd(xmm3);
        xmm6.addpd(xmm5);
        xmm5.movq(Address(&CTABLE, rax + 8));
        xmm5.subsd(xmm3);
        xmm3.subsd(xmm4);
        xmm1.addsd(Address(&CTABLE, rax + 16));
        xmm6.mulpd(xmm2);
        xmm5.addsd(xmm0);
        xmm3.addsd(xmm7);
        xmm1.addsd(xmm5);
        xmm1.addsd(xmm3);
        xmm1.addsd(xmm6);
        xmm6.unpckhpd(xmm6);
        xmm0.movdqu(xmm4);
        xmm1.addsd(xmm6);
        xmm0.addsd(xmm1);
        return xmm0.into();
    }

    // L_2TAG_PACKET_0_0_1
    if rax as i32 <= 281346048 {
        rax.shrl(20);
        if rax as u32 == 3325 {
            xmm0.mulsd(Address(&ALL_ONES, 0));
            return xmm0.into();
        }

        // L_2TAG_PACKET_2_0_1
        xmm3.movq(Address(&TWO_POW_55, 0));
        xmm3.mulsd(xmm0);
        xmm3.subsd(xmm0);
        xmm3.mulsd(Address(&TWO_POW_M55, 0));
        return xmm0.into();
    }

    // L_2TAG_PACKET_1_0_1
    rax.pextrw(xmm0, 3);
    rax.andl(32752);
    if rax as u32 == 32752 {
        // L_2TAG_PACKET_3_0_1: infinity and NaN.
        xmm0.movq(rsp_8);
        xmm0.mulsd(Address(&NEG_ZERO, 0));
        return xmm0.into();
    }

    rcx.pextrw(xmm0, 3);
    rcx.andl(32752);
    rcx.subl(16224);
    rcx.shrl(7);
    rcx.andl(65532);
    r11.mov64(0); // lea(r11, ExternalAddress(PI_INV_TABLE))
    rcx.addq(r11);
    rax.movdq(xmm0);
    r10.movl(Address(&PI_INV_TABLE, rcx + 20));
    r8.movl(Address(&PI_INV_TABLE, rcx + 24));
    rdx.movl(rax);
    rax.shrq(21);
    rax.orl(i32::MIN);
    rax.shrl(11);
    r9.movl(r10);
    r10.imulq(rdx);
    r9.imulq(rax);
    r8.imulq(rax);
    rsi.movl(Address(&PI_INV_TABLE, rcx + 16));
    rdi.movl(Address(&PI_INV_TABLE, rcx + 12));
    r11.movl(r10);
    r10.shrq(32);
    r9.addq(r10);
    r11.addq(r8);
    r8.movl(r11);
    r11.shrq(32);
    r9.addq(r11);
    r10.movl(rsi);
    rsi.imulq(rdx);
    r10.imulq(rax);
    r11.movl(rdi);
    rdi.imulq(rdx);
    rbx.movl(rsi);
    rsi.shrq(32);
    r9.addq(rbx);
    rbx.movl(r9);
    r9.shrq(32);
    r10.addq(rsi);
    r10.addq(r9);
    rbx.shlq(32);
    r8.orq(rbx);
    r11.imulq(rax);
    r9.movl(Address(&PI_INV_TABLE, rcx + 8));
    rsi.movl(Address(&PI_INV_TABLE, rcx + 4));
    rbx.movl(rdi);
    rdi.shrq(32);
    r10.addq(rbx);
    rbx.movl(r10);
    r10.shrq(32);
    r11.addq(rdi);
    r11.addq(r10);
    rdi.movq(r9);
    r9.imulq(rdx);
    rdi.imulq(rax);
    r10.movl(r9);
    r9.shrq(32);
    r11.addq(r10);
    r10.movl(r11);
    r11.shrq(32);
    rdi.addq(r9);
    rdi.addq(r11);
    r9.movq(rsi);
    rsi.imulq(rdx);
    r9.imulq(rax);
    r10.shlq(32);
    r10.orq(rbx);
    rax.movl(Address(&PI_INV_TABLE, rcx));
    r11.movl(rsi);
    rsi.shrq(32);
    rdi.addq(r11);
    r11.movl(rdi);
    rdi.shrq(32);
    r9.addq(rsi);
    r9.addq(rdi);
    rdx.imulq(rax);
    rbx.pextrw(xmm0, 3);
    rdi.mov64(0); // lea(rdi, ExternalAddress(PI_INV_TABLE))
    rcx.subq(rdi);
    rcx.addl(rcx);
    rcx.addl(rcx);
    rcx.addl(rcx);
    rcx.addl(19);
    rsi.movl(32768);
    rsi.andl(rbx);
    rbx.shrl(4);
    rbx.andl(2047);
    rbx.subl(1023);
    rcx.subl(rbx);
    r9.addq(rdx);
    rdx.movl(rcx);
    rdx.addl(32);
    if (rcx as i32) < 1 {
        // L_2TAG_PACKET_4_0_1
        rcx.negl();
        r9.shlq(32);
        r9.orq(r11);
        r9.shlq(rcx);
        rdi.movq(r9);
        if r9 as u32 & i32::MIN as u32 != 0 {
            // L_2TAG_PACKET_13_0_1
            r9.shrl(rcx);
            rbx.mov64(0x100000000);
            rbx.shrq(rcx);
            rcx.movl(0);
            r11.movl(0);
            let borrow = rcx.subq(r8);
            let borrow = r11.sbbq(r10, borrow);
            rbx.sbbq(r9, borrow);
            r8.movq(rcx);
            r10.movq(r11);
            r9.movq(rbx);
            rbx.movl(32768);
            rdi.shrq(3);
            rdi.addl(536870912);
        } else {
            r9.shrl(rcx);
            rbx.movl(0);
            rdi.shrq(3);
        }
    } else {
        rcx.negl();
        rcx.addl(29);
        r9.shll(rcx);
        rdi.movl(r9);
        r9.andl(536870911);
        if r9 as u32 & 268435456 != 0 {
            // L_2TAG_PACKET_5_0_1
            r9.shrl(rcx);
            rbx.movl(536870912);
            rbx.shrl(rcx);
            r9.shlq(32);
            r9.orq(r11);
            rbx.shlq(32);
            rdi.addl(536870912);
            rcx.movl(0);
            r11.movl(0);
            let borrow = rcx.subq(r8);
            let borrow = r11.sbbq(r10, borrow);
            rbx.sbbq(r9, borrow);
            r8.movq(rcx);
            r10.movq(r11);
            r9.movq(rbx);
            rbx.movl(32768);
        } else {
            r9.shrl(rcx);
            rbx.movl(0);
            r9.shlq(32);
            r9.orq(r11);
        }
    }

    // L_2TAG_PACKET_6_0_1 and L_2TAG_PACKET_7_0_1
    let mut reduced = true;
    if r9 == 0 {
        // L_2TAG_PACKET_8_0_1: the leading 64 bits are all zero, so shifts in the next ones.
        rdx.addl(64);
        r9.movq(r10);
        r10.movq(r8);
        r8.movl(0);
        if r9 == 0 {
            rdx.addl(64);
            r9.movq(r10);
            r10.movq(r8);
            if r9 == 0 {
                xmm0.xorpd(xmm0);
                xmm6.xorpd(xmm6);
                reduced = false;
            }
        }
    }

    if reduced {
        // L_2TAG_PACKET_9_0_1
        r11.bsrq(r9);
        rcx.movl(29);
        rcx.subl(r11);
        if rcx as i32 > 0 {
            r9.shlq(rcx);
            rax.movq(r10);
            r10.shlq(rcx);
            rdx.addl(rcx);
            rcx.negl();
            rcx.addl(64);
            rax.shrq(rcx);
            r8.shrq(rcx);
            r9.orq(rax);
            r10.orq(r8);
        } else if rcx as u32 != 0 {
            // L_2TAG_PACKET_10_0_1
            rcx.negl();
            r10.shrq(rcx);
            rax.movq(r9);
            r9.shrq(rcx);
            rdx.subl(rcx);
            rcx.negl();
            rcx.addl(64);
            rax.shlq(rcx);
            r10.orq(rax);
        }

        // L_2TAG_PACKET_11_0_1
        xmm0.cvtsi2sdq(r9);
        r10.shrq(1);
        xmm3.cvtsi2sdq(r10);
        xmm4.xorpd(xmm4);
        rdx.shll(4);
        rdx.negl();
        rdx.addl(16368);
        rdx.orl(rsi);
        rdx.xorl(rbx);
        xmm4.pinsrw(rdx, 3);
        xmm2.movq(Address(&PI_4, 0));
        xmm6.movq(Address(&PI_4, 8));
        xmm5.xorpd(xmm5);
        rdx.subl(1008);
        xmm5.pinsrw(rdx, 3);
        xmm0.mulsd(xmm4);
        rsi.shll(16);
        rsi.sarl(31);
        xmm3.mulsd(xmm5);
        xmm1.movdqu(xmm0);
        xmm0.mulsd(xmm2);
        rdi.shrl(29);
        xmm1.addsd(xmm3);
        xmm3.mulsd(xmm2);
        rdi.addl(rsi);
        rdi.xorl(rsi);
        xmm6.mulsd(xmm1);
        rax.movl(rdi);
        xmm6.addsd(xmm3);
        xmm2.movdqu(xmm0);
        xmm0.addsd(xmm6);
        xmm2.subsd(xmm0);
        xmm6.addsd(xmm2);
    }

    // L_2TAG_PACKET_12_0_1: x reduced by multiples of pi/2 to xmm0 + xmm6, quadrant in rax.
    xmm1.movq(Address(&PI32INV, 0));
    xmm1.mulsd(xmm0);
    xmm5.movq(Address(&ONEHALF, 0));
    xmm4.movq(Address(&SIGN_MASK, 0));
    xmm4.pand(xmm0);
    xmm5.por(xmm4);
    xmm1.addpd(xmm5);
    rdx.cvttsd2sil(xmm1);
    xmm1.cvtsi2sdl(rdx);
    xmm3.movq(Address(&P_1, 0));
    xmm2.movdqu(Address(&P_2, 0));
    xmm3.mulsd(xmm1);
    xmm1.unpcklpd(xmm1);
    rax.shll(3);
    rdx.addl(1865216);
    xmm4.movdqu(xmm0);
    rdx.addl(rax);
    rdx.andl(63);
    xmm5.movdqu(Address(&SC_4, 0));
    rax.mov64(0); // lea(rax, ExternalAddress(Ctable))
    rdx.shll(5);
    rax.addq(rdx);
    xmm2.mulpd(xmm1);
    xmm0.subsd(xmm3);
    xmm1.mulsd(Address(&P_3, 0));
    xmm4.subsd(xmm3);
    xmm7.movq(Address(&CTABLE, rax + 8));
    xmm0.unpcklpd(xmm0);
    xmm3.movdqu(xmm4);
    xmm4.subsd(xmm2);
    xmm5.mulpd(xmm0);
    xmm0.subpd(xmm2);
    xmm7.mulsd(xmm4);
    xmm3.subsd(xmm4);
    xmm5.mulpd(xmm0);
    xmm0.mulpd(xmm0);
    xmm3.subsd(xmm2);
    xmm2.movdqu(Address(&CTABLE, rax));
    xmm1.subsd(xmm3);
    xmm3.movq(Address(&CTABLE, rax + 24));
    xmm2.addsd(xmm3);
    xmm7.subsd(xmm2);
    xmm1.subsd(xmm6);
    xmm6.movdqu(Address(&SC_2, 0));
    xmm2.mulsd(xmm4);
    xmm6.mulpd(xmm0);
    xmm3.mulsd(xmm4);
    xmm2.mulpd(xmm0);
    xmm0.mulpd(xmm0);
    xmm5.addpd(Address(&SC_3, 0));
    xmm4.mulsd(Address(&CTABLE, rax));
    xmm6.addpd(Address(&SC_1, 0));
    xmm5.mulpd(xmm0);
    xmm0.movdqu(xmm3);
    xmm3.addsd(Address(&CTABLE, rax + 8));
    xmm1.mulpd(xmm7);
    xmm7.movdqu(xmm4);
    xmm4.addsd(xmm3);
    xmm6.addpd(xmm5);
    xmm5.movq(Address(&CTABLE, rax + 8));
    xmm5.subsd(xmm3);
    xmm3.subsd(xmm4);
    xmm1.addsd(Address(&CTABLE, rax + 16));
    xmm6.mulpd(xmm2);
    xmm5.addsd(xmm0);
    xmm3.addsd(xmm7);
    xmm1.addsd(xmm5);
    xmm1.addsd(xmm3);
    xmm1.addsd(xmm6);
    xmm6.unpckhpd(xmm6);
    xmm0.movdqu(xmm4);
    xmm1.addsd(xmm6);
    xmm0.addsd(xmm1);
    xmm0.into()
}
