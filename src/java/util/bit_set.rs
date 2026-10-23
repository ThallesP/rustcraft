//! java/util/BitSet.java
//!
//! The light packets' section masks, which `ByteBufCodecs.BIT_SET` and `FriendlyByteBuf`'s bit
//! set readers and writers carry, and the bits of a `BitSetDiscreteVoxelShape`, the voxels of a
//! block's or an entity's shape (checked through the shapes' goldens too). Its words and
//! `wordsInUse` are Java's, so `equals`, `hashCode`, `length` and the arrays it hands out are too;
//! how its array grows past the words in use (`ensureCapacity`'s doubling, `sizeIsSticky`) only
//! shows in `size()`, which isn't ported. Only the methods the game calls are here.

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

use std::fmt;

/// A growable vector of bits, in 64-bit words.
#[derive(Clone, Debug, Default)]
pub struct BitSet {
    words: Vec<i64>,
    words_in_use: usize,
}

const ADDRESS_BITS_PER_WORD: i32 = 6;
const BITS_PER_WORD: i32 = 1 << ADDRESS_BITS_PER_WORD;
const WORD_MASK: i64 = -1;

fn word_index(bit_index: i32) -> usize {
    (bit_index >> ADDRESS_BITS_PER_WORD) as usize
}

impl BitSet {
    /// `new BitSet()`.
    pub fn new_default() -> BitSet {
        BitSet::new(BITS_PER_WORD)
    }

    /// `new BitSet(nbits)`, room for that many bits. Panics on a negative size, where Java throws
    /// `NegativeArraySizeException`.
    pub fn new(nbits: i32) -> BitSet {
        assert!(nbits >= 0, "nbits < 0: {nbits}");
        BitSet {
            words: vec![0; word_index(nbits - 1).wrapping_add(1)],
            words_in_use: 0,
        }
    }

    fn from_words(words: Vec<i64>) -> BitSet {
        let words_in_use = words.len();
        BitSet {
            words,
            words_in_use,
        }
    }

    /// `valueOf(long[])`: the words, little-endian, without the zeros at the end.
    pub fn value_of_longs(longs: &[i64]) -> BitSet {
        let mut n = longs.len();
        while n > 0 && longs[n - 1] == 0 {
            n -= 1;
        }
        BitSet::from_words(longs[..n].to_vec())
    }

    /// `valueOf(byte[])`: the bytes, little-endian, without the zeros at the end.
    pub fn value_of_bytes(bytes: &[i8]) -> BitSet {
        let mut n = bytes.len();
        while n > 0 && bytes[n - 1] == 0 {
            n -= 1;
        }
        let mut words = vec![0i64; n.div_ceil(8)];
        for (i, &byte) in bytes[..n].iter().enumerate() {
            words[i / 8] |= (byte as i64 & 0xff) << (8 * (i % 8));
        }
        BitSet::from_words(words)
    }

    /// Its bits, little-endian, as few bytes as hold them.
    pub fn to_byte_array(&self) -> Vec<i8> {
        let n = self.words_in_use;
        if n == 0 {
            return Vec::new();
        }
        let mut bytes = Vec::with_capacity(8 * n);
        for word in &self.words[..n - 1] {
            bytes.extend(word.to_le_bytes().map(|byte| byte as i8));
        }
        let mut x = self.words[n - 1];
        while x != 0 {
            bytes.push((x & 0xff) as i8);
            x = ((x as u64) >> 8) as i64;
        }
        bytes
    }

    /// Its words in use.
    pub fn to_long_array(&self) -> Vec<i64> {
        self.words[..self.words_in_use].to_vec()
    }

    fn expand_to(&mut self, word_index: usize) {
        let words_required = word_index + 1;
        if self.words_in_use < words_required {
            if self.words.len() < words_required {
                let request = (2 * self.words.len()).max(words_required);
                self.words.resize(request, 0);
            }
            self.words_in_use = words_required;
        }
    }

    fn recalculate_words_in_use(&mut self) {
        let mut i = self.words_in_use;
        while i > 0 && self.words[i - 1] == 0 {
            i -= 1;
        }
        self.words_in_use = i;
    }

    /// Panics on a negative index, where Java throws `IndexOutOfBoundsException`.
    pub fn set(&mut self, bit_index: i32) {
        assert!(bit_index >= 0, "bitIndex < 0: {bit_index}");
        let word_index = word_index(bit_index);
        self.expand_to(word_index);
        self.words[word_index] |= 1i64.wrapping_shl(bit_index as u32);
    }

    /// `clear(fromIndex, toIndex)`: the bits from the first up to the second, exclusive. Panics on
    /// a negative index or a range that ends before it starts, where Java throws
    /// `IndexOutOfBoundsException`.
    pub fn clear_range(&mut self, from_index: i32, mut to_index: i32) {
        assert!(from_index >= 0, "fromIndex < 0: {from_index}");
        assert!(to_index >= 0, "toIndex < 0: {to_index}");
        assert!(
            from_index <= to_index,
            "fromIndex: {from_index} > toIndex: {to_index}"
        );
        if from_index == to_index {
            return;
        }

        let start_word_index = word_index(from_index);
        if start_word_index >= self.words_in_use {
            return;
        }

        let mut end_word_index = word_index(to_index - 1);
        if end_word_index >= self.words_in_use {
            to_index = self.length();
            end_word_index = self.words_in_use - 1;
        }

        let first_word_mask = WORD_MASK.wrapping_shl(from_index as u32);
        let last_word_mask =
            ((WORD_MASK as u64).wrapping_shr(to_index.wrapping_neg() as u32)) as i64;
        if start_word_index == end_word_index {
            self.words[start_word_index] &= !(first_word_mask & last_word_mask);
        } else {
            self.words[start_word_index] &= !first_word_mask;
            for word in &mut self.words[start_word_index + 1..end_word_index] {
                *word = 0;
            }
            self.words[end_word_index] &= !last_word_mask;
        }

        self.recalculate_words_in_use();
    }

    /// Panics on a negative index, where Java throws `IndexOutOfBoundsException`.
    pub fn get(&self, bit_index: i32) -> bool {
        assert!(bit_index >= 0, "bitIndex < 0: {bit_index}");
        let word_index = word_index(bit_index);
        word_index < self.words_in_use
            && self.words[word_index] & 1i64.wrapping_shl(bit_index as u32) != 0
    }

    /// The first set bit at or after `from_index`, or -1.
    pub fn next_set_bit(&self, from_index: i32) -> i32 {
        assert!(from_index >= 0, "fromIndex < 0: {from_index}");
        let mut u = word_index(from_index);
        if u >= self.words_in_use {
            return -1;
        }

        let mut word = self.words[u] & WORD_MASK.wrapping_shl(from_index as u32);
        loop {
            if word != 0 {
                return u as i32 * BITS_PER_WORD + word.trailing_zeros() as i32;
            }
            u += 1;
            if u == self.words_in_use {
                return -1;
            }
            word = self.words[u];
        }
    }

    /// The first clear bit at or after `from_index`.
    pub fn next_clear_bit(&self, from_index: i32) -> i32 {
        assert!(from_index >= 0, "fromIndex < 0: {from_index}");
        let mut u = word_index(from_index);
        if u >= self.words_in_use {
            return from_index;
        }

        let mut word = !self.words[u] & WORD_MASK.wrapping_shl(from_index as u32);
        loop {
            if word != 0 {
                return u as i32 * BITS_PER_WORD + word.trailing_zeros() as i32;
            }
            u += 1;
            if u == self.words_in_use {
                return self.words_in_use as i32 * BITS_PER_WORD;
            }
            word = !self.words[u];
        }
    }

    /// One past its highest set bit.
    pub fn length(&self) -> i32 {
        if self.words_in_use == 0 {
            return 0;
        }
        BITS_PER_WORD * (self.words_in_use as i32 - 1)
            + (BITS_PER_WORD - self.words[self.words_in_use - 1].leading_zeros() as i32)
    }

    pub fn is_empty(&self) -> bool {
        self.words_in_use == 0
    }

    pub fn cardinality(&self) -> i32 {
        self.words[..self.words_in_use]
            .iter()
            .map(|word| word.count_ones() as i32)
            .sum()
    }

    pub fn hash_code(&self) -> i32 {
        let mut h: i64 = 1234;
        for i in (0..self.words_in_use).rev() {
            h ^= self.words[i].wrapping_mul(i as i64 + 1);
        }
        ((h >> 32) ^ h) as i32
    }
}

impl PartialEq for BitSet {
    /// The same words in use.
    fn eq(&self, other: &BitSet) -> bool {
        self.words[..self.words_in_use] == other.words[..other.words_in_use]
    }
}

impl Eq for BitSet {}

/// `toString()`: its set bits, `{0, 2, 63}`.
impl fmt::Display for BitSet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("{")?;
        let mut first = true;
        let mut i = self.next_set_bit(0);
        while i >= 0 {
            if !first {
                f.write_str(", ")?;
            }
            write!(f, "{i}")?;
            first = false;
            match i.checked_add(1) {
                Some(next) => i = self.next_set_bit(next),
                None => break,
            }
        }
        f.write_str("}")
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use serde_json::Value;

    use super::BitSet;
    use crate::golden::{Decode, Encode, Golden};

    /// Its words in use.
    impl Encode for BitSet {
        fn encode(&self) -> Value {
            self.to_long_array().encode()
        }
    }

    impl Decode for BitSet {
        fn decode(value: &Value) -> BitSet {
            BitSet::value_of_longs(&Vec::<i64>::decode(value))
        }
    }

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

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("java.util.BitSet");
        golden.check("<init>()V", BitSet::new_default);
        golden.check("<init>(I)V", BitSet::new);
        golden.check("valueOf([J)Ljava/util/BitSet;", |longs: Vec<i64>| {
            BitSet::value_of_longs(&longs)
        });
        golden.check("valueOf([B)Ljava/util/BitSet;", |bytes: Vec<i8>| {
            BitSet::value_of_bytes(&bytes)
        });
        golden.check_method("toByteArray()[B", |bits: &mut BitSet| bits.to_byte_array());
        golden.check_method("toLongArray()[J", |bits: &mut BitSet| bits.to_long_array());
        golden.check_method("get(I)Z", |bits: &mut BitSet, index: i32| bits.get(index));
        golden.check_method("set(I)V", |bits: &mut BitSet, index: i32| bits.set(index));
        golden.check_method("clear(II)V", |bits: &mut BitSet, from: i32, to: i32| {
            bits.clear_range(from, to)
        });
        golden.check_method("nextSetBit(I)I", |bits: &mut BitSet, from: i32| {
            bits.next_set_bit(from)
        });
        golden.check_method("nextClearBit(I)I", |bits: &mut BitSet, from: i32| {
            bits.next_clear_bit(from)
        });
        golden.check_method("length()I", |bits: &mut BitSet| bits.length());
        golden.check_method("isEmpty()Z", |bits: &mut BitSet| bits.is_empty());
        golden.check_method("cardinality()I", |bits: &mut BitSet| bits.cardinality());
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |bits: &mut BitSet, other: BitSet| *bits == other,
        );
        golden.check_method("hashCode()I", |bits: &mut BitSet| bits.hash_code());
        golden.check_method("toString()Ljava/lang/String;", |bits: &mut BitSet| {
            bits.to_string()
        });
        golden.finish();
    }
}
