//! src/hotspot/cpu/x86/stubGenerator_x86_64_tan.cpp, the stub HotSpot calls for `Math.tan`.
//!
//! Inputs with 0.1015625 <= |x| < 12800 are reduced by multiples of pi/32 in double arithmetic
//! and go through that breakpoint's degree 15 polynomial in `CTABLE`, plus, near a pole, an
//! accurate reciprocal of the remainder modulo pi/2. Larger ones go through a Payne-Hanek
//! reduction like sin's, then the same path. Smaller ones short-cut: an odd polynomial
//! down to 2^-29, then `(x + 2^55 * x) * 2^-55`, and zeros and subnormals come back unchanged.

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
use super::stub_generator_x86_64_constants::{
    NEG_ZERO, ONE, ONEHALF, PI_INV_TABLE, TWO_POW_55, TWO_POW_M55,
};

static MUL16: [u32; 4] = [0x00000000, 0x40300000, 0x00000000, 0x3ff00000];

static SIGN_MASK: [u32; 4] = [0x00000000, 0x80000000, 0x00000000, 0x80000000];

static PI32INV: [u32; 4] = [0x6dc9c883, 0x3fe45f30, 0x6dc9c883, 0x40245f30];

static P_1: [u32; 4] = [0x54444000, 0x3fb921fb, 0x54440000, 0x3fb921fb];

static P_2: [u32; 4] = [0x67674000, 0xbd32e7b9, 0x4c4c0000, 0x3d468c23];

static P_3: [u32; 4] = [0x3707344a, 0x3aa8a2e0, 0x03707345, 0x3ae98a2e];

static CTABLE: [u32; 1408] = [
    0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x882c10fa, 0x3f9664f4, 0x00000000, 0x00000000,
    0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x55e6c23d, 0x3f8226e3, 0x55555555, 0x3fd55555,
    0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x0e157de0, 0x3f6d6d3d, 0x11111111, 0x3fc11111,
    0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x452b75e3, 0x3f57da36, 0x1ba1ba1c, 0x3faba1ba,
    0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x3ff00000, 0x00000000, 0x00000000,
    0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x4e435f9b, 0x3f953f83, 0x00000000, 0x00000000,
    0x3c6e8e46, 0x3f9b74ea, 0x00000000, 0x00000000, 0xda5b7511, 0x3f85ad63, 0xdc230b9b, 0x3fb97558,
    0x26cb3788, 0x3f881308, 0x76fc4985, 0x3fd62ac9, 0x77bb08ba, 0x3f757c85, 0xb6247521, 0x3fb1381e,
    0x5922170c, 0x3f754e95, 0x8746482d, 0x3fc27f83, 0x11055b30, 0x3f64e391, 0x3e666320, 0x3fa3e609,
    0x0de9dae3, 0x3f6301df, 0x1f1dca06, 0x3fafa8ae, 0x8c5b2da2, 0x3fb936bb, 0x4e88f7a5, 0x3c587d05,
    0x00000000, 0x3ff00000, 0xa8935dd9, 0x3f83dde2, 0x00000000, 0x00000000, 0x00000000, 0x00000000,
    0x5a279ea3, 0x3faa3407, 0x00000000, 0x00000000, 0x432d65fa, 0x3fa70153, 0x00000000, 0x00000000,
    0x891a4602, 0x3f9d03ef, 0xd62ca5f8, 0x3fca77d9, 0xb35f4628, 0x3f97a265, 0x433258fa, 0x3fd8cf51,
    0xb58fd909, 0x3f8f88e3, 0x01771cea, 0x3fc2b154, 0xf3562f8e, 0x3f888f57, 0xc028a723, 0x3fc7370f,
    0x20b7f9f0, 0x3f80f44c, 0x214368e9, 0x3fb6dfaa, 0x28891863, 0x3f79b4b6, 0x172dbbf0, 0x3fb6cb8e,
    0xe0553158, 0x3fc975f5, 0x593fe814, 0x3c2ef5d3, 0x00000000, 0x3ff00000, 0x03dec550, 0x3fa44203,
    0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x9314533e, 0x3fbb8ec5, 0x00000000, 0x00000000,
    0x09aa36d0, 0x3fb6d3f4, 0x00000000, 0x00000000, 0xdcb427fd, 0x3fb13950, 0xd87ab0bb, 0x3fd5335e,
    0xce0ae8a5, 0x3fabb382, 0x79143126, 0x3fddba41, 0x5f2b28d4, 0x3fa552f1, 0x59f21a6d, 0x3fd015ab,
    0x22c27d95, 0x3fa0e984, 0xe19fc6aa, 0x3fd0576c, 0x8f2c2950, 0x3f9a4898, 0xc0b3f22c, 0x3fc59462,
    0x1883a4b8, 0x3f94b61c, 0x3f838640, 0x3fc30eb8, 0x355c63dc, 0x3fd36a08, 0x1dce993d, 0xbc6d704d,
    0x00000000, 0x3ff00000, 0x2b82ab63, 0x3fb78e92, 0x00000000, 0x00000000, 0x00000000, 0x00000000,
    0x56f37042, 0x3fccfc56, 0x00000000, 0x00000000, 0xaa563951, 0x3fc90125, 0x00000000, 0x00000000,
    0x3d0e7c5d, 0x3fc50533, 0x9bed9b2e, 0x3fdf0ed9, 0x5fe7c47c, 0x3fc1f250, 0x96c125e5, 0x3fe2edd9,
    0x5a02bbd8, 0x3fbe5c71, 0x86362c20, 0x3fda08b7, 0x4b4435ed, 0x3fb9d342, 0x4b494091, 0x3fd911bd,
    0xb56658be, 0x3fb5e4c7, 0x93a2fd76, 0x3fd3c092, 0xda271794, 0x3fb29910, 0x3303df2b, 0x3fd189be,
    0x99fcef32, 0x3fda8279, 0xb68c1467, 0x3c708b2f, 0x00000000, 0x3ff00000, 0x980c4337, 0x3fc5f619,
    0x00000000, 0x00000000, 0x00000000, 0x00000000, 0xcc03e501, 0x3fdff10f, 0x00000000, 0x00000000,
    0x44a4e845, 0x3fddb63b, 0x00000000, 0x00000000, 0x3768ad9f, 0x3fdb72a4, 0x3dd01cca, 0x3fe5fdb9,
    0xa61d2811, 0x3fd972b2, 0x5645ad0b, 0x3fe977f9, 0xd013b3ab, 0x3fd78ca3, 0xbf0bf914, 0x3fe4f192,
    0x4d53e730, 0x3fd5d060, 0x3f8b9000, 0x3fe49933, 0xe2b82f08, 0x3fd4322a, 0x5936a835, 0x3fe27ae1,
    0xb1c61c9b, 0x3fd2b3fb, 0xef478605, 0x3fe1659e, 0x190834ec, 0x3fe11ab7, 0xcdb625ea, 0xbc8e564b,
    0x00000000, 0x3ff00000, 0xb07217e3, 0x3fd248f1, 0x00000000, 0x00000000, 0x00000000, 0x00000000,
    0x2b2c49d0, 0x3ff2de9c, 0x00000000, 0x00000000, 0x2655bc98, 0x3ff33e58, 0x00000000, 0x00000000,
    0xff691fa2, 0x3ff3972e, 0xe93463bd, 0x3feeed87, 0x070e10a0, 0x3ff3f5b2, 0xf4d790a4, 0x3ff20c10,
    0xa04e8ea3, 0x3ff4541a, 0x386accd3, 0x3ff1369e, 0x222a66dd, 0x3ff4b521, 0x22a9777e, 0x3ff20817,
    0x52a04a6e, 0x3ff5178f, 0xddaa0031, 0x3ff22137, 0x4447d47c, 0x3ff57c01, 0x1e9c7f1d, 0x3ff29311,
    0x2ab7f990, 0x3fe561b8, 0x209c7df1, 0x3c87a8c5, 0x00000000, 0x3ff00000, 0x4170bcc6, 0x3fdc92d8,
    0x00000000, 0x00000000, 0x00000000, 0x00000000, 0xc7ab4d5a, 0x40085e24, 0x00000000, 0x00000000,
    0xe93ea75d, 0x400b963d, 0x00000000, 0x00000000, 0x94a7f25a, 0x400f37e2, 0x4b6261cb, 0x3ff5f984,
    0x5a9dd812, 0x4011aab0, 0x74c30018, 0x3ffaf5a5, 0x7f2ce8e3, 0x4013fe8b, 0xfe8e54fa, 0x3ffd7334,
    0x670d618d, 0x4016a10c, 0x4db97058, 0x4000e012, 0x24df44dd, 0x40199c5f, 0x697d6ece, 0x4003006e,
    0x83298b82, 0x401cfc4d, 0x19d490d6, 0x40058c19, 0x2ae42850, 0x3fea4300, 0x118e20e6, 0xbc7a6db8,
    0x00000000, 0x40000000, 0xe33345b8, 0xbfd4e526, 0x00000000, 0x00000000, 0x00000000, 0x00000000,
    0x65965966, 0x40219659, 0x00000000, 0x00000000, 0x882c10fa, 0x402664f4, 0x00000000, 0x00000000,
    0x83cd3723, 0x402c8342, 0x00000000, 0x40000000, 0x55e6c23d, 0x403226e3, 0x55555555, 0x40055555,
    0x34451939, 0x40371c96, 0xaaaaaaab, 0x400aaaaa, 0x0e157de0, 0x403d6d3d, 0x11111111, 0x40111111,
    0xa738201f, 0x4042bbce, 0x05b05b06, 0x4015b05b, 0x452b75e3, 0x4047da36, 0x1ba1ba1c, 0x401ba1ba,
    0x00000000, 0x3ff00000, 0x00000000, 0x00000000, 0x00000000, 0x40000000, 0x00000000, 0x00000000,
    0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x4f48b8d3, 0xbf33eaf9, 0x00000000, 0x00000000,
    0x0cf7586f, 0x3f20b8ea, 0x00000000, 0x00000000, 0xd0258911, 0xbf0abaf3, 0x23e49fe9, 0xbfab5a8c,
    0x2d53222e, 0x3ef60d15, 0x21169451, 0x3fa172b2, 0xbb254dbc, 0xbee1d3b5, 0xdbf93b8e, 0xbf84c7db,
    0x05b4630b, 0x3ecd3364, 0xee9aada7, 0x3f743924, 0x794a8297, 0xbeb7b7b9, 0xe015f797, 0xbf5d41f5,
    0xe41a4a56, 0x3ea35dfb, 0xe4c2a251, 0x3f49a2ab, 0x5af9e000, 0xbfce49ce, 0x8c743719, 0x3d1eb860,
    0x00000000, 0x00000000, 0x1b4863cf, 0x3fd78294, 0x00000000, 0x3ff00000, 0x00000000, 0xfffffff8,
    0x535ad890, 0xbf2b9320, 0x00000000, 0x00000000, 0x018fdf1f, 0x3f16d61d, 0x00000000, 0x00000000,
    0x0359f1be, 0xbf0139e4, 0xa4317c6d, 0xbfa67e17, 0x82672d0f, 0x3eebb405, 0x2f1b621e, 0x3f9f455b,
    0x51ccf238, 0xbed55317, 0xf437b9ac, 0xbf804bee, 0xc791a2b5, 0x3ec0e993, 0x919a1db2, 0x3f7080c2,
    0x336a5b0e, 0xbeaa48a2, 0x0a268358, 0xbf55a443, 0xdfd978e4, 0x3e94b61f, 0xd7767a58, 0x3f431806,
    0x2aea0000, 0xbfc9bbe8, 0x7723ea61, 0xbd3a2369, 0x00000000, 0x00000000, 0xdf7796ff, 0x3fd6e642,
    0x00000000, 0x3ff00000, 0x00000000, 0xfffffff8, 0xb9ff07ce, 0xbf231c78, 0x00000000, 0x00000000,
    0xa5517182, 0x3f0ff0e0, 0x00000000, 0x00000000, 0x790b4cbc, 0xbef66191, 0x848a46c6, 0xbfa21ac0,
    0xb16435fa, 0x3ee1d3ec, 0x2a1aa832, 0x3f9c71ea, 0xfdd299ef, 0xbec9dd1a, 0x3f8dbaaf, 0xbf793363,
    0x309fc6ea, 0x3eb415d6, 0xbee60471, 0x3f6b83ba, 0x94a0a697, 0xbe9dae11, 0x3e5c67b3, 0xbf4fd07b,
    0x9a8f3e3e, 0x3e86bd75, 0xa4beb7a4, 0x3f3d1eb1, 0x29cfc000, 0xbfc549ce, 0xbf159358, 0xbd397b33,
    0x00000000, 0x00000000, 0x871fee6c, 0x3fd666f0, 0x00000000, 0x3ff00000, 0x00000000, 0xfffffff8,
    0x7d98a556, 0xbf1a3958, 0x00000000, 0x00000000, 0x9d88dc01, 0x3f0704c2, 0x00000000, 0x00000000,
    0x73742a2b, 0xbeed054a, 0x58844587, 0xbf9c2a13, 0x55688a79, 0x3ed7a326, 0xee33f1d6, 0x3f9a48f4,
    0xa8dc9888, 0xbebf8939, 0xaad4b5b8, 0xbf72f746, 0x9102efa1, 0x3ea88f82, 0xdabc29cf, 0x3f678228,
    0x9289afb8, 0xbe90f456, 0x741fb4ed, 0xbf46f3a3, 0xa97f6663, 0x3e79b4bf, 0xca89ff3f, 0x3f36db70,
    0xa8a2a000, 0xbfc0ee13, 0x3da24be1, 0xbd338b9f, 0x00000000, 0x00000000, 0x11cd6c69, 0x3fd601fd,
    0x00000000, 0x3ff00000, 0x00000000, 0xfffffff8, 0x1a154b97, 0xbf116b01, 0x00000000, 0x00000000,
    0x2d427630, 0x3f0147bf, 0x00000000, 0x00000000, 0xb93820c8, 0xbee264d4, 0xbb6cbb18, 0xbf94ab8c,
    0x888d4d92, 0x3ed0568b, 0x60730f7c, 0x3f98b19b, 0xe4b1fb11, 0xbeb2f950, 0x22cf9f74, 0xbf6b21cd,
    0x4a3ff0a6, 0x3e9f499e, 0xfd2b83ce, 0x3f64aad7, 0x637b73af, 0xbe83487c, 0xe522591a, 0xbf3fc092,
    0xa158e8bc, 0x3e6e3aae, 0xe5e82ffa, 0x3f329d2f, 0xd636a000, 0xbfb9477f, 0xc2c2d2bc, 0xbd135ef9,
    0x00000000, 0x00000000, 0xf2fdb123, 0x3fd5b566, 0x00000000, 0x3ff00000, 0x00000000, 0xfffffff8,
    0xc41acb64, 0xbf05448d, 0x00000000, 0x00000000, 0xdbb03d6f, 0x3efb7ad2, 0x00000000, 0x00000000,
    0x9e42962d, 0xbed5aea5, 0x2579f8ef, 0xbf8b2398, 0x288a1ed9, 0x3ec81441, 0xb0198dc5, 0x3f979a3a,
    0x2fdfe253, 0xbea57cd3, 0x5766336f, 0xbf617caa, 0x600944c3, 0x3e954ed6, 0xa4e0aaf8, 0x3f62c646,
    0x6b8fb29c, 0xbe74e3a3, 0xdc4c0409, 0xbf33f952, 0x9bffe365, 0x3e6301ec, 0xb8869e44, 0x3f2fc566,
    0xe1e04000, 0xbfb0cc62, 0x016b907f, 0xbd119cbc, 0x00000000, 0x00000000, 0xe6b9d8fa, 0x3fd57fb3,
    0x00000000, 0x3ff00000, 0x00000000, 0xfffffff8, 0x5daf22a6, 0xbef429d7, 0x00000000, 0x00000000,
    0x06bca545, 0x3ef7a27d, 0x00000000, 0x00000000, 0x7211c19a, 0xbec41c3e, 0x956ed53e, 0xbf7ae3f4,
    0xee750e72, 0x3ec3901b, 0x91d443f5, 0x3f96f713, 0x36661e6c, 0xbe936e09, 0x506f9381, 0xbf5122e8,
    0xcb6dd43f, 0x3e9041b9, 0x6698b2ff, 0x3f61b0c7, 0x576bf12b, 0xbe625a8a, 0xe5a0e9dc, 0xbf23499d,
    0x110384dd, 0x3e5b1c2c, 0x68d43db6, 0x3f2cb899, 0x6ecac000, 0xbfa0c414, 0xcd7dd58c, 0x3d13500f,
    0x00000000, 0x00000000, 0x85a2c8fb, 0x3fd55fe0, 0x00000000, 0x3ff00000, 0x00000000, 0xfffffff8,
    0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x2bf70ebe, 0x3ef66a8f, 0x00000000, 0x00000000,
    0x00000000, 0x00000000, 0x00000000, 0x00000000, 0xd644267f, 0x3ec22805, 0x16c16c17, 0x3f96c16c,
    0x00000000, 0x00000000, 0x00000000, 0x00000000, 0xc4e09162, 0x3e8d6db2, 0xbc011567, 0x3f61566a,
    0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x1f79955c, 0x3e57da4e, 0x9334ef0b, 0x3f2bbd77,
    0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x55555555, 0x3fd55555,
    0x00000000, 0x3ff00000, 0x00000000, 0xfffffff8, 0x5daf22a6, 0x3ef429d7, 0x00000000, 0x00000000,
    0x06bca545, 0x3ef7a27d, 0x00000000, 0x00000000, 0x7211c19a, 0x3ec41c3e, 0x956ed53e, 0x3f7ae3f4,
    0xee750e72, 0x3ec3901b, 0x91d443f5, 0x3f96f713, 0x36661e6c, 0x3e936e09, 0x506f9381, 0x3f5122e8,
    0xcb6dd43f, 0x3e9041b9, 0x6698b2ff, 0x3f61b0c7, 0x576bf12b, 0x3e625a8a, 0xe5a0e9dc, 0x3f23499d,
    0x110384dd, 0x3e5b1c2c, 0x68d43db6, 0x3f2cb899, 0x6ecac000, 0x3fa0c414, 0xcd7dd58c, 0xbd13500f,
    0x00000000, 0x00000000, 0x85a2c8fb, 0x3fd55fe0, 0x00000000, 0x3ff00000, 0x00000000, 0xfffffff8,
    0xc41acb64, 0x3f05448d, 0x00000000, 0x00000000, 0xdbb03d6f, 0x3efb7ad2, 0x00000000, 0x00000000,
    0x9e42962d, 0x3ed5aea5, 0x2579f8ef, 0x3f8b2398, 0x288a1ed9, 0x3ec81441, 0xb0198dc5, 0x3f979a3a,
    0x2fdfe253, 0x3ea57cd3, 0x5766336f, 0x3f617caa, 0x600944c3, 0x3e954ed6, 0xa4e0aaf8, 0x3f62c646,
    0x6b8fb29c, 0x3e74e3a3, 0xdc4c0409, 0x3f33f952, 0x9bffe365, 0x3e6301ec, 0xb8869e44, 0x3f2fc566,
    0xe1e04000, 0x3fb0cc62, 0x016b907f, 0x3d119cbc, 0x00000000, 0x00000000, 0xe6b9d8fa, 0x3fd57fb3,
    0x00000000, 0x3ff00000, 0x00000000, 0xfffffff8, 0x1a154b97, 0x3f116b01, 0x00000000, 0x00000000,
    0x2d427630, 0x3f0147bf, 0x00000000, 0x00000000, 0xb93820c8, 0x3ee264d4, 0xbb6cbb18, 0x3f94ab8c,
    0x888d4d92, 0x3ed0568b, 0x60730f7c, 0x3f98b19b, 0xe4b1fb11, 0x3eb2f950, 0x22cf9f74, 0x3f6b21cd,
    0x4a3ff0a6, 0x3e9f499e, 0xfd2b83ce, 0x3f64aad7, 0x637b73af, 0x3e83487c, 0xe522591a, 0x3f3fc092,
    0xa158e8bc, 0x3e6e3aae, 0xe5e82ffa, 0x3f329d2f, 0xd636a000, 0x3fb9477f, 0xc2c2d2bc, 0x3d135ef9,
    0x00000000, 0x00000000, 0xf2fdb123, 0x3fd5b566, 0x00000000, 0x3ff00000, 0x00000000, 0xfffffff8,
    0x7d98a556, 0x3f1a3958, 0x00000000, 0x00000000, 0x9d88dc01, 0x3f0704c2, 0x00000000, 0x00000000,
    0x73742a2b, 0x3eed054a, 0x58844587, 0x3f9c2a13, 0x55688a79, 0x3ed7a326, 0xee33f1d6, 0x3f9a48f4,
    0xa8dc9888, 0x3ebf8939, 0xaad4b5b8, 0x3f72f746, 0x9102efa1, 0x3ea88f82, 0xdabc29cf, 0x3f678228,
    0x9289afb8, 0x3e90f456, 0x741fb4ed, 0x3f46f3a3, 0xa97f6663, 0x3e79b4bf, 0xca89ff3f, 0x3f36db70,
    0xa8a2a000, 0x3fc0ee13, 0x3da24be1, 0x3d338b9f, 0x00000000, 0x00000000, 0x11cd6c69, 0x3fd601fd,
    0x00000000, 0x3ff00000, 0x00000000, 0xfffffff8, 0xb9ff07ce, 0x3f231c78, 0x00000000, 0x00000000,
    0xa5517182, 0x3f0ff0e0, 0x00000000, 0x00000000, 0x790b4cbc, 0x3ef66191, 0x848a46c6, 0x3fa21ac0,
    0xb16435fa, 0x3ee1d3ec, 0x2a1aa832, 0x3f9c71ea, 0xfdd299ef, 0x3ec9dd1a, 0x3f8dbaaf, 0x3f793363,
    0x309fc6ea, 0x3eb415d6, 0xbee60471, 0x3f6b83ba, 0x94a0a697, 0x3e9dae11, 0x3e5c67b3, 0x3f4fd07b,
    0x9a8f3e3e, 0x3e86bd75, 0xa4beb7a4, 0x3f3d1eb1, 0x29cfc000, 0x3fc549ce, 0xbf159358, 0x3d397b33,
    0x00000000, 0x00000000, 0x871fee6c, 0x3fd666f0, 0x00000000, 0x3ff00000, 0x00000000, 0xfffffff8,
    0x535ad890, 0x3f2b9320, 0x00000000, 0x00000000, 0x018fdf1f, 0x3f16d61d, 0x00000000, 0x00000000,
    0x0359f1be, 0x3f0139e4, 0xa4317c6d, 0x3fa67e17, 0x82672d0f, 0x3eebb405, 0x2f1b621e, 0x3f9f455b,
    0x51ccf238, 0x3ed55317, 0xf437b9ac, 0x3f804bee, 0xc791a2b5, 0x3ec0e993, 0x919a1db2, 0x3f7080c2,
    0x336a5b0e, 0x3eaa48a2, 0x0a268358, 0x3f55a443, 0xdfd978e4, 0x3e94b61f, 0xd7767a58, 0x3f431806,
    0x2aea0000, 0x3fc9bbe8, 0x7723ea61, 0x3d3a2369, 0x00000000, 0x00000000, 0xdf7796ff, 0x3fd6e642,
    0x00000000, 0x3ff00000, 0x00000000, 0xfffffff8, 0x4f48b8d3, 0x3f33eaf9, 0x00000000, 0x00000000,
    0x0cf7586f, 0x3f20b8ea, 0x00000000, 0x00000000, 0xd0258911, 0x3f0abaf3, 0x23e49fe9, 0x3fab5a8c,
    0x2d53222e, 0x3ef60d15, 0x21169451, 0x3fa172b2, 0xbb254dbc, 0x3ee1d3b5, 0xdbf93b8e, 0x3f84c7db,
    0x05b4630b, 0x3ecd3364, 0xee9aada7, 0x3f743924, 0x794a8297, 0x3eb7b7b9, 0xe015f797, 0x3f5d41f5,
    0xe41a4a56, 0x3ea35dfb, 0xe4c2a251, 0x3f49a2ab, 0x5af9e000, 0x3fce49ce, 0x8c743719, 0xbd1eb860,
    0x00000000, 0x00000000, 0x1b4863cf, 0x3fd78294, 0x00000000, 0x3ff00000, 0x00000000, 0xfffffff8,
    0x65965966, 0xc0219659, 0x00000000, 0x00000000, 0x882c10fa, 0x402664f4, 0x00000000, 0x00000000,
    0x83cd3723, 0xc02c8342, 0x00000000, 0xc0000000, 0x55e6c23d, 0x403226e3, 0x55555555, 0x40055555,
    0x34451939, 0xc0371c96, 0xaaaaaaab, 0xc00aaaaa, 0x0e157de0, 0x403d6d3d, 0x11111111, 0x40111111,
    0xa738201f, 0xc042bbce, 0x05b05b06, 0xc015b05b, 0x452b75e3, 0x4047da36, 0x1ba1ba1c, 0x401ba1ba,
    0x00000000, 0xbff00000, 0x00000000, 0x00000000, 0x00000000, 0x40000000, 0x00000000, 0x00000000,
    0x00000000, 0x00000000, 0x00000000, 0x00000000, 0xc7ab4d5a, 0xc0085e24, 0x00000000, 0x00000000,
    0xe93ea75d, 0x400b963d, 0x00000000, 0x00000000, 0x94a7f25a, 0xc00f37e2, 0x4b6261cb, 0xbff5f984,
    0x5a9dd812, 0x4011aab0, 0x74c30018, 0x3ffaf5a5, 0x7f2ce8e3, 0xc013fe8b, 0xfe8e54fa, 0xbffd7334,
    0x670d618d, 0x4016a10c, 0x4db97058, 0x4000e012, 0x24df44dd, 0xc0199c5f, 0x697d6ece, 0xc003006e,
    0x83298b82, 0x401cfc4d, 0x19d490d6, 0x40058c19, 0x2ae42850, 0xbfea4300, 0x118e20e6, 0x3c7a6db8,
    0x00000000, 0x40000000, 0xe33345b8, 0xbfd4e526, 0x00000000, 0x00000000, 0x00000000, 0x00000000,
    0x2b2c49d0, 0xbff2de9c, 0x00000000, 0x00000000, 0x2655bc98, 0x3ff33e58, 0x00000000, 0x00000000,
    0xff691fa2, 0xbff3972e, 0xe93463bd, 0xbfeeed87, 0x070e10a0, 0x3ff3f5b2, 0xf4d790a4, 0x3ff20c10,
    0xa04e8ea3, 0xbff4541a, 0x386accd3, 0xbff1369e, 0x222a66dd, 0x3ff4b521, 0x22a9777e, 0x3ff20817,
    0x52a04a6e, 0xbff5178f, 0xddaa0031, 0xbff22137, 0x4447d47c, 0x3ff57c01, 0x1e9c7f1d, 0x3ff29311,
    0x2ab7f990, 0xbfe561b8, 0x209c7df1, 0xbc87a8c5, 0x00000000, 0x3ff00000, 0x4170bcc6, 0x3fdc92d8,
    0x00000000, 0x00000000, 0x00000000, 0x00000000, 0xcc03e501, 0xbfdff10f, 0x00000000, 0x00000000,
    0x44a4e845, 0x3fddb63b, 0x00000000, 0x00000000, 0x3768ad9f, 0xbfdb72a4, 0x3dd01cca, 0xbfe5fdb9,
    0xa61d2811, 0x3fd972b2, 0x5645ad0b, 0x3fe977f9, 0xd013b3ab, 0xbfd78ca3, 0xbf0bf914, 0xbfe4f192,
    0x4d53e730, 0x3fd5d060, 0x3f8b9000, 0x3fe49933, 0xe2b82f08, 0xbfd4322a, 0x5936a835, 0xbfe27ae1,
    0xb1c61c9b, 0x3fd2b3fb, 0xef478605, 0x3fe1659e, 0x190834ec, 0xbfe11ab7, 0xcdb625ea, 0x3c8e564b,
    0x00000000, 0x3ff00000, 0xb07217e3, 0x3fd248f1, 0x00000000, 0x00000000, 0x00000000, 0x00000000,
    0x56f37042, 0xbfccfc56, 0x00000000, 0x00000000, 0xaa563951, 0x3fc90125, 0x00000000, 0x00000000,
    0x3d0e7c5d, 0xbfc50533, 0x9bed9b2e, 0xbfdf0ed9, 0x5fe7c47c, 0x3fc1f250, 0x96c125e5, 0x3fe2edd9,
    0x5a02bbd8, 0xbfbe5c71, 0x86362c20, 0xbfda08b7, 0x4b4435ed, 0x3fb9d342, 0x4b494091, 0x3fd911bd,
    0xb56658be, 0xbfb5e4c7, 0x93a2fd76, 0xbfd3c092, 0xda271794, 0x3fb29910, 0x3303df2b, 0x3fd189be,
    0x99fcef32, 0xbfda8279, 0xb68c1467, 0xbc708b2f, 0x00000000, 0x3ff00000, 0x980c4337, 0x3fc5f619,
    0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x9314533e, 0xbfbb8ec5, 0x00000000, 0x00000000,
    0x09aa36d0, 0x3fb6d3f4, 0x00000000, 0x00000000, 0xdcb427fd, 0xbfb13950, 0xd87ab0bb, 0xbfd5335e,
    0xce0ae8a5, 0x3fabb382, 0x79143126, 0x3fddba41, 0x5f2b28d4, 0xbfa552f1, 0x59f21a6d, 0xbfd015ab,
    0x22c27d95, 0x3fa0e984, 0xe19fc6aa, 0x3fd0576c, 0x8f2c2950, 0xbf9a4898, 0xc0b3f22c, 0xbfc59462,
    0x1883a4b8, 0x3f94b61c, 0x3f838640, 0x3fc30eb8, 0x355c63dc, 0xbfd36a08, 0x1dce993d, 0x3c6d704d,
    0x00000000, 0x3ff00000, 0x2b82ab63, 0x3fb78e92, 0x00000000, 0x00000000, 0x00000000, 0x00000000,
    0x5a279ea3, 0xbfaa3407, 0x00000000, 0x00000000, 0x432d65fa, 0x3fa70153, 0x00000000, 0x00000000,
    0x891a4602, 0xbf9d03ef, 0xd62ca5f8, 0xbfca77d9, 0xb35f4628, 0x3f97a265, 0x433258fa, 0x3fd8cf51,
    0xb58fd909, 0xbf8f88e3, 0x01771cea, 0xbfc2b154, 0xf3562f8e, 0x3f888f57, 0xc028a723, 0x3fc7370f,
    0x20b7f9f0, 0xbf80f44c, 0x214368e9, 0xbfb6dfaa, 0x28891863, 0x3f79b4b6, 0x172dbbf0, 0x3fb6cb8e,
    0xe0553158, 0xbfc975f5, 0x593fe814, 0xbc2ef5d3, 0x00000000, 0x3ff00000, 0x03dec550, 0x3fa44203,
    0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x4e435f9b, 0xbf953f83, 0x00000000, 0x00000000,
    0x3c6e8e46, 0x3f9b74ea, 0x00000000, 0x00000000, 0xda5b7511, 0xbf85ad63, 0xdc230b9b, 0xbfb97558,
    0x26cb3788, 0x3f881308, 0x76fc4985, 0x3fd62ac9, 0x77bb08ba, 0xbf757c85, 0xb6247521, 0xbfb1381e,
    0x5922170c, 0x3f754e95, 0x8746482d, 0x3fc27f83, 0x11055b30, 0xbf64e391, 0x3e666320, 0xbfa3e609,
    0x0de9dae3, 0x3f6301df, 0x1f1dca06, 0x3fafa8ae, 0x8c5b2da2, 0xbfb936bb, 0x4e88f7a5, 0xbc587d05,
    0x00000000, 0x3ff00000, 0xa8935dd9, 0x3f83dde2, 0x00000000, 0x00000000, 0x00000000, 0x00000000,
];

static MASK_35: [u32; 4] = [0xfffc0000, 0xffffffff, 0x00000000, 0x00000000];

static Q_11: [u32; 2] = [0xb8fe4d77, 0x3f82609a];

static Q_9: [u32; 2] = [0xbf847a43, 0x3f9664a0];

static Q_7: [u32; 2] = [0x52c4c8ab, 0x3faba1ba];

static Q_5: [u32; 2] = [0x11092746, 0x3fc11111];

static Q_3: [u32; 2] = [0x55555612, 0x3fd55555];

static PI_4: [u32; 4] = [0x00000000, 0x3fe921fb, 0x4611a626, 0x3e85110b];

static QQ_2: [u32; 2] = [0x676733af, 0x3d32e7b9];

pub fn libm_tan(x: f64) -> f64 {
    let mut xmm0 = Xmm::new(x);
    let [mut xmm1, mut xmm2, mut xmm3, mut xmm4] = [Xmm::default(); 4];
    let [mut xmm5, mut xmm6, mut xmm7] = [Xmm::default(); 3];
    let [mut rax, mut rbx, mut rcx, mut rdx, mut rsi] = [0u64; 5];
    let [mut rdi, mut r8, mut r9, mut r10, mut r11] = [0u64; 5];

    let rsp_8 = x.to_bits(); // movsd(Address(rsp, 8), xmm0)
    rax.pextrw(xmm0, 3);
    rax.andl(32767);
    rax.subl(16314);
    if rax as u32 <= 270 {
        xmm5.movdqu(Address(&ONEHALF, 0));
        xmm6.movdqu(Address(&MUL16, 0));
        xmm0.unpcklpd(xmm0);
        xmm4.movdqu(Address(&SIGN_MASK, 0));
        xmm4.andpd(xmm0);
        xmm1.movdqu(Address(&PI32INV, 0));
        xmm1.mulpd(xmm0);
        xmm5.por(xmm4);
        xmm1.addpd(xmm5);
        xmm7.movdqu(xmm1);
        xmm7.unpckhpd(xmm7);
        rdx.cvttsd2sil(xmm7);
        xmm1.cvttpd2dq(xmm1);
        xmm1.cvtdq2pd(xmm1);
        xmm1.mulpd(xmm6);
        xmm3.movdqu(Address(&P_1, 0));
        xmm5.movq(Address(&QQ_2, 0));
        rdx.addq(469248);
        xmm4.movdqu(Address(&P_2, 0));
        xmm3.mulpd(xmm1);
        rdx.andq(31);
        xmm5.mulsd(xmm1);
        rcx.movq(rdx);
        xmm4.mulpd(xmm1);
        rcx.shlq(1);
        xmm0.subpd(xmm3);
        xmm1.mulpd(Address(&P_3, 0));
        rdx.addq(rcx);
        rcx.shlq(2);
        rdx.addq(rcx);
        xmm5.addsd(xmm0);
        xmm2.movdqu(xmm0);
        xmm0.subpd(xmm4);
        xmm6.movq(Address(&ONE, 0));
        rdx.shlq(4);
        rax.mov64(0); // lea(rax, ExternalAddress(Ctable))
        xmm5.andpd(Address(&MASK_35, 0));
        xmm3.movdqu(xmm0);
        rax.addq(rdx);
        xmm2.subpd(xmm0);
        xmm0.unpckhpd(xmm0);
        xmm6.divsd(xmm5);
        xmm2.subpd(xmm4);
        xmm7.movdqu(Address(&CTABLE, rax + 16));
        xmm3.subsd(xmm5);
        xmm7.mulpd(xmm0);
        xmm2.subpd(xmm1);
        xmm1.movdqu(Address(&CTABLE, rax + 48));
        xmm1.mulpd(xmm0);
        xmm4.movdqu(Address(&CTABLE, rax + 96));
        xmm4.mulpd(xmm0);
        xmm2.addsd(xmm3);
        xmm3.movdqu(xmm0);
        xmm0.mulpd(xmm0);
        xmm7.addpd(Address(&CTABLE, rax));
        xmm1.addpd(Address(&CTABLE, rax + 32));
        xmm1.mulpd(xmm0);
        xmm4.addpd(Address(&CTABLE, rax + 80));
        xmm7.addpd(xmm1);
        xmm1.movdqu(Address(&CTABLE, rax + 112));
        xmm1.mulpd(xmm0);
        xmm0.mulpd(xmm0);
        xmm4.addpd(xmm1);
        xmm1.movdqu(Address(&CTABLE, rax + 64));
        xmm1.mulpd(xmm0);
        xmm7.addpd(xmm1);
        xmm1.movdqu(xmm3);
        xmm3.mulpd(xmm0);
        xmm0.mulsd(xmm0);
        xmm1.mulpd(Address(&CTABLE, rax + 144));
        xmm4.mulpd(xmm3);
        xmm3.movdqu(xmm1);
        xmm7.addpd(xmm4);
        xmm4.movdqu(xmm1);
        xmm0.mulsd(xmm7);
        xmm7.unpckhpd(xmm7);
        xmm0.addsd(xmm7);
        xmm1.unpckhpd(xmm1);
        xmm3.addsd(xmm1);
        xmm4.subsd(xmm3);
        xmm1.addsd(xmm4);
        xmm4.movdqu(xmm2);
        xmm7.movq(Address(&CTABLE, rax + 144));
        xmm2.unpckhpd(xmm2);
        xmm7.addsd(Address(&CTABLE, rax + 152));
        xmm7.mulsd(xmm2);
        xmm7.addsd(Address(&CTABLE, rax + 136));
        xmm7.addsd(xmm1);
        xmm0.addsd(xmm7);
        xmm7.movq(Address(&ONE, 0));
        xmm4.mulsd(xmm6);
        xmm2.movq(Address(&CTABLE, rax + 168));
        xmm2.andpd(xmm6);
        xmm5.mulsd(xmm2);
        xmm6.mulsd(Address(&CTABLE, rax + 160));
        xmm7.subsd(xmm5);
        xmm2.subsd(Address(&CTABLE, rax + 128));
        xmm7.subsd(xmm4);
        xmm7.mulsd(xmm6);
        xmm4.movdqu(xmm3);
        xmm3.subsd(xmm2);
        xmm2.addsd(xmm3);
        xmm4.subsd(xmm2);
        xmm0.addsd(xmm4);
        xmm0.subsd(xmm7);
        xmm0.addsd(xmm3);
        return xmm0.into();
    }

    // L_2TAG_PACKET_0_0_1
    if rax as i32 <= 270 {
        rax.pextrw(xmm0, 3);
        rdx.movl(rax);
        rax.andl(32752);
        if rax as u32 != 0 {
            rdx.andl(32767);
            if rdx as u32 >= 15904 {
                xmm2.movdqu(xmm0);
                xmm3.movdqu(xmm0);
                xmm1.movq(Address(&Q_11, 0));
                xmm2.mulsd(xmm0);
                xmm3.mulsd(xmm2);
                xmm1.mulsd(xmm2);
                xmm1.addsd(Address(&Q_9, 0));
                xmm1.mulsd(xmm2);
                xmm1.addsd(Address(&Q_7, 0));
                xmm1.mulsd(xmm2);
                xmm1.addsd(Address(&Q_5, 0));
                xmm1.mulsd(xmm2);
                xmm1.addsd(Address(&Q_3, 0));
                xmm1.mulsd(xmm3);
                xmm0.addsd(xmm1);
                return xmm0.into();
            }

            // L_2TAG_PACKET_3_0_1
            xmm3.movq(Address(&TWO_POW_55, 0));
            xmm3.mulsd(xmm0);
            xmm0.addsd(xmm3);
            xmm0.mulsd(Address(&TWO_POW_M55, 0));
            return xmm0.into();
        }

        // L_2TAG_PACKET_2_0_1: zeros and subnormals.
        xmm1.movdqu(xmm0);
        xmm1.mulsd(xmm1);
        return xmm0.into();
    }

    // L_2TAG_PACKET_1_0_1
    rax.pextrw(xmm0, 3);
    rax.andl(32752);
    if rax as u32 == 32752 {
        // L_2TAG_PACKET_4_0_1: infinity and NaN. Also stores the result at rsp, where nothing
        // reads it.
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
    if (rcx as i32) < 0 {
        // L_2TAG_PACKET_5_0_1
        rcx.notl();
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
            rdi.shrq(2);
            rdi.addl(1073741824);
        } else {
            r9.shrl(rcx);
            rbx.movl(0);
            rdi.shrq(2);
        }
    } else {
        rcx.negl();
        rcx.addl(29);
        r9.shll(rcx);
        rdi.movl(r9);
        r9.andl(1073741823);
        if r9 as u32 & 536870912 != 0 {
            // L_2TAG_PACKET_6_0_1
            r9.shrl(rcx);
            rbx.movl(1073741824);
            rbx.shrl(rcx);
            r9.shlq(32);
            r9.orq(r11);
            rbx.shlq(32);
            rdi.addl(1073741824);
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

    // L_2TAG_PACKET_7_0_1 and L_2TAG_PACKET_8_0_1
    if r9 == 0 {
        // L_2TAG_PACKET_9_0_1: the leading 64 bits are all zero, so shifts in the next ones.
        rdx.addl(64);
        r9.movq(r10);
        r10.movq(r8);
        r8.movl(0);
        if r9 == 0 {
            rdx.addl(64);
            r9.movq(r10);
            r10.movq(r8);
        }
    }

    if r9 != 0 {
        // L_2TAG_PACKET_10_0_1
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
            // L_2TAG_PACKET_11_0_1
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
    }

    // L_2TAG_PACKET_12_0_1: x reduced by multiples of pi/2 to xmm0 + xmm7, quadrant in rax.
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
    xmm7.movq(Address(&PI_4, 8));
    xmm5.xorpd(xmm5);
    rdx.subl(1008);
    xmm5.pinsrw(rdx, 3);
    xmm0.mulsd(xmm4);
    rsi.shll(16);
    rsi.sarl(31);
    xmm3.mulsd(xmm5);
    xmm1.movdqu(xmm0);
    xmm0.mulsd(xmm2);
    rdi.shrl(30);
    xmm1.addsd(xmm3);
    xmm3.mulsd(xmm2);
    rdi.addl(rsi);
    rdi.xorl(rsi);
    xmm7.mulsd(xmm1);
    rax.movl(rdi);
    xmm7.addsd(xmm3);
    xmm2.movdqu(xmm0);
    xmm0.addsd(xmm7);
    xmm2.subsd(xmm0);
    xmm7.addsd(xmm2);
    xmm1.movdqu(Address(&PI32INV, 0));
    xmm0.movddup(xmm0);
    xmm4.movdqu(Address(&SIGN_MASK, 0));
    xmm4.andpd(xmm0);
    xmm1.mulpd(xmm0);
    xmm7.movddup(xmm7);
    xmm5.movdqu(Address(&ONEHALF, 0));
    xmm6.movdqu(Address(&MUL16, 0));
    xmm5.por(xmm4);
    xmm1.addpd(xmm5);
    xmm5.movdqu(xmm1);
    xmm5.unpckhpd(xmm5);
    rdx.cvttsd2sil(xmm5);
    xmm1.cvttpd2dq(xmm1);
    xmm1.cvtdq2pd(xmm1);
    xmm1.mulpd(xmm6);
    xmm3.movdqu(Address(&P_1, 0));
    xmm5.movq(Address(&QQ_2, 0));
    rax.shll(4);
    rdx.addl(469248);
    xmm4.movdqu(Address(&P_2, 0));
    xmm3.mulpd(xmm1);
    rdx.addl(rax);
    rdx.andl(31);
    xmm5.mulsd(xmm1);
    rcx.movl(rdx);
    xmm4.mulpd(xmm1);
    rcx.shll(1);
    xmm0.subpd(xmm3);
    xmm1.mulpd(Address(&P_3, 0));
    rdx.addl(rcx);
    rcx.shll(2);
    rdx.addl(rcx);
    xmm5.addsd(xmm0);
    xmm2.movdqu(xmm0);
    xmm0.subpd(xmm4);
    xmm6.movq(Address(&ONE, 0));
    rdx.shll(4);
    rax.mov64(0); // lea(rax, ExternalAddress(Ctable))
    xmm5.andpd(Address(&MASK_35, 0));
    xmm3.movdqu(xmm0);
    rax.addq(rdx);
    xmm2.subpd(xmm0);
    xmm0.unpckhpd(xmm0);
    xmm6.divsd(xmm5);
    xmm2.subpd(xmm4);
    xmm3.subsd(xmm5);
    xmm2.subpd(xmm1);
    xmm1.movdqu(Address(&CTABLE, rax + 48));
    xmm2.addpd(xmm7);
    xmm7.movdqu(Address(&CTABLE, rax + 16));
    xmm7.mulpd(xmm0);
    xmm4.movdqu(Address(&CTABLE, rax + 96));
    xmm1.mulpd(xmm0);
    xmm4.mulpd(xmm0);
    xmm2.addsd(xmm3);
    xmm3.movdqu(xmm0);
    xmm0.mulpd(xmm0);
    xmm7.addpd(Address(&CTABLE, rax));
    xmm1.addpd(Address(&CTABLE, rax + 32));
    xmm1.mulpd(xmm0);
    xmm4.addpd(Address(&CTABLE, rax + 80));
    xmm7.addpd(xmm1);
    xmm1.movdqu(Address(&CTABLE, rax + 112));
    xmm1.mulpd(xmm0);
    xmm0.mulpd(xmm0);
    xmm4.addpd(xmm1);
    xmm1.movdqu(Address(&CTABLE, rax + 64));
    xmm1.mulpd(xmm0);
    xmm7.addpd(xmm1);
    xmm1.movdqu(xmm3);
    xmm3.mulpd(xmm0);
    xmm0.mulsd(xmm0);
    xmm1.mulpd(Address(&CTABLE, rax + 144));
    xmm4.mulpd(xmm3);
    xmm3.movdqu(xmm1);
    xmm7.addpd(xmm4);
    xmm4.movdqu(xmm1);
    xmm0.mulsd(xmm7);
    xmm7.unpckhpd(xmm7);
    xmm0.addsd(xmm7);
    xmm1.unpckhpd(xmm1);
    xmm3.addsd(xmm1);
    xmm4.subsd(xmm3);
    xmm1.addsd(xmm4);
    xmm4.movdqu(xmm2);
    xmm7.movq(Address(&CTABLE, rax + 144));
    xmm2.unpckhpd(xmm2);
    xmm7.addsd(Address(&CTABLE, rax + 152));
    xmm7.mulsd(xmm2);
    xmm7.addsd(Address(&CTABLE, rax + 136));
    xmm7.addsd(xmm1);
    xmm0.addsd(xmm7);
    xmm7.movq(Address(&ONE, 0));
    xmm4.mulsd(xmm6);
    xmm2.movq(Address(&CTABLE, rax + 168));
    xmm2.andpd(xmm6);
    xmm5.mulsd(xmm2);
    xmm6.mulsd(Address(&CTABLE, rax + 160));
    xmm7.subsd(xmm5);
    xmm2.subsd(Address(&CTABLE, rax + 128));
    xmm7.subsd(xmm4);
    xmm7.mulsd(xmm6);
    xmm4.movdqu(xmm3);
    xmm3.subsd(xmm2);
    xmm2.addsd(xmm3);
    xmm4.subsd(xmm2);
    xmm0.addsd(xmm4);
    xmm0.subsd(xmm7);
    xmm0.addsd(xmm3);
    xmm0.into()
}
