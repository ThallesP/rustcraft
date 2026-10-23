//! java/util/BitSet.java
//!
//! The bits of a `BitSetDiscreteVoxelShape`, the voxels of a block's or an entity's shape. Only
//! what the shapes call is ported: `new BitSet(nbits)`, `get`, `set`, `clear(from, to)`,
//! `nextClearBit`, `isEmpty` and `clone`. Its words grow like Java's (`ensureCapacity` doubles
//! them), which nothing outside the set can see, so they're a `Vec` that grows as it needs.
//! Checked through the shapes' goldens (`VoxelShape`, `Shapes`).

/*
 * Copyright (c) 1995, 2024, Oracle and/or its affiliates. All rights reserved.
 * DO NOT ALTER OR REMOVE COPYRIGHT NOTICES OR THIS FILE HEADER.
 *
 * This code is free software; you can redistribute it and/or modify it
 * under the terms of the GNU General Public License version 2 only, as
 * published by the Free Software Foundation.  Oracle designates this
 * particular file as subject to the "Classpath" exception as provided
 * by Oracle in the LICENSE file that accompanied this code.
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
 */

/// A growable vector of bits, in 64-bit words.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct BitSet {
    words: Vec<u64>,
}

impl BitSet {
    const ADDRESS_BITS_PER_WORD: u32 = 6;

    fn word_index(bit_index: i32) -> usize {
        (bit_index >> BitSet::ADDRESS_BITS_PER_WORD) as usize
    }

    /// `new BitSet(nbits)`, which throws `NegativeArraySizeException` for a negative size.
    pub fn new(nbits: i32) -> BitSet {
        assert!(nbits >= 0, "nbits < 0: {nbits}");
        BitSet {
            words: vec![0; (((nbits - 1) >> BitSet::ADDRESS_BITS_PER_WORD) + 1) as usize],
        }
    }

    pub fn get(&self, bit_index: i32) -> bool {
        assert!(bit_index >= 0, "bitIndex < 0: {bit_index}");
        let word_index = BitSet::word_index(bit_index);
        word_index < self.words.len() && self.words[word_index] & (1 << (bit_index & 63)) != 0
    }

    pub fn set(&mut self, bit_index: i32) {
        assert!(bit_index >= 0, "bitIndex < 0: {bit_index}");
        let word_index = BitSet::word_index(bit_index);
        if word_index >= self.words.len() {
            self.words.resize(word_index + 1, 0);
        }
        self.words[word_index] |= 1 << (bit_index & 63);
    }

    /// `clear(fromIndex, toIndex)`: the bits from the first up to the second, exclusive.
    pub fn clear_range(&mut self, from_index: i32, to_index: i32) {
        assert!(from_index >= 0, "fromIndex < 0: {from_index}");
        assert!(to_index >= 0, "toIndex < 0: {to_index}");
        assert!(
            from_index <= to_index,
            "fromIndex: {from_index} > toIndex: {to_index}"
        );
        for bit_index in from_index..to_index {
            let word_index = BitSet::word_index(bit_index);
            if word_index >= self.words.len() {
                break;
            }
            self.words[word_index] &= !(1 << (bit_index & 63));
        }
    }

    /// The first clear bit at or after `from_index`, which is past the last word when every bit
    /// from there on is set.
    pub fn next_clear_bit(&self, from_index: i32) -> i32 {
        assert!(from_index >= 0, "fromIndex < 0: {from_index}");
        let mut bit_index = from_index;
        while BitSet::word_index(bit_index) < self.words.len() && self.get(bit_index) {
            bit_index += 1;
        }
        bit_index
    }

    pub fn is_empty(&self) -> bool {
        self.words.iter().all(|&word| word == 0)
    }
}

#[cfg(test)]
mod tests {
    use super::BitSet;

    #[test]
    fn sets_clears_and_finds_clear_bits_like_java() {
        let mut bits = BitSet::new(0);
        assert!(bits.is_empty());
        assert_eq!(bits.next_clear_bit(0), 0);
        for index in [0, 1, 2, 63, 64, 65, 200] {
            bits.set(index);
        }
        assert!(bits.get(64) && !bits.get(66) && !bits.get(10_000));
        assert_eq!(bits.next_clear_bit(0), 3);
        assert_eq!(bits.next_clear_bit(63), 66);
        bits.clear_range(1, 65);
        assert!(bits.get(0) && !bits.get(1) && !bits.get(64) && bits.get(65));
        bits.clear_range(0, 10_000);
        assert!(bits.is_empty());
        bits.clear_range(5, 5);
    }
}
