//! net/minecraft/util/ZeroBitStorage.java
//!
//! Java's `copy()` hands back the storage itself, which holds nothing but its size; a clone is
//! the same. `RAW`, the empty array every one hands out, is an empty slice.

use crate::net::minecraft::util::bit_storage::{BitStorage, validate_inclusive_between};

/// The storage of a palette with one value: every value is 0, and no longs hold them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ZeroBitStorage {
    size: i32,
}

impl ZeroBitStorage {
    pub const RAW: [i64; 0] = [];

    pub fn new(size: i32) -> ZeroBitStorage {
        ZeroBitStorage { size }
    }

    fn check_index(&self, index: i32) {
        validate_inclusive_between(0, self.size.wrapping_sub(1) as i64, index as i64);
    }
}

impl BitStorage for ZeroBitStorage {
    fn get_and_set(&mut self, index: i32, value: i32) -> i32 {
        self.check_index(index);
        validate_inclusive_between(0, 0, value as i64);
        0
    }

    fn set(&mut self, index: i32, value: i32) {
        self.check_index(index);
        validate_inclusive_between(0, 0, value as i64);
    }

    fn fill(&mut self, value: i32) {
        validate_inclusive_between(0, 0, value as i64);
    }

    fn get(&self, index: i32) -> i32 {
        self.check_index(index);
        0
    }

    fn get_raw(&self) -> &[i64] {
        &ZeroBitStorage::RAW
    }

    fn get_raw_mut(&mut self) -> &mut [i64] {
        &mut []
    }

    fn get_size(&self) -> i32 {
        self.size
    }

    fn get_bits(&self) -> i32 {
        0
    }

    fn get_all(&self, output: &mut dyn FnMut(i32)) {
        for _ in 0..self.size {
            output(0);
        }
    }

    fn unpack(&self, output: &mut [i32]) {
        output[..self.size as usize].fill(0);
    }
}

#[cfg(test)]
mod tests {
    use super::ZeroBitStorage;
    use crate::golden::Golden;
    use crate::net::minecraft::util::bit_storage::BitStorage;
    use crate::net::minecraft::util::simple_bit_storage::tests::get_all;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.util.ZeroBitStorage");
        golden.field("RAW", ZeroBitStorage::RAW.to_vec());
        golden.check("getSize()I", |size: i32| {
            ZeroBitStorage::new(size).get_size()
        });
        golden.check("getBits()I", |size: i32| {
            ZeroBitStorage::new(size).get_bits()
        });
        golden.check("getRaw()[J", |size: i32| {
            ZeroBitStorage::new(size).get_raw().to_vec()
        });
        golden.check("getAll(Ljava/util/function/IntConsumer;)V", |size: i32| {
            get_all(&ZeroBitStorage::new(size))
        });
        golden.check("copy()Lnet/minecraft/util/BitStorage;", |size: i32| {
            let copy = ZeroBitStorage::new(size);
            (copy.get_size(), copy.get_bits())
        });
        golden.check("unpack([I)V", |size: i32| {
            let mut output = vec![7; size as usize + 2];
            ZeroBitStorage::new(size).unpack(&mut output);
            output
        });
        golden.check("fill(I)V", |size: i32, value: i32| {
            ZeroBitStorage::new(size).fill(value);
        });
        golden.check("set(II)V", |size: i32, index: i32, value: i32| {
            ZeroBitStorage::new(size).set(index, value);
        });
        golden.check("getAndSet(II)I", |size: i32, index: i32, value: i32| {
            ZeroBitStorage::new(size).get_and_set(index, value)
        });
        golden.check("get(I)I", |size: i32, index: i32| {
            ZeroBitStorage::new(size).get(index)
        });
        golden.finish();
    }
}
