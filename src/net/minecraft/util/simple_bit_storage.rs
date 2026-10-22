//! net/minecraft/util/SimpleBitStorage.java
//!
//! Finding the long an index is in divides by the values per long, which Java does with a
//! multiply, an add and a shift from `MAGIC` (one row for each count of values per long), as the
//! port does: `get` is what renderers and physics call for every block.

use crate::net::minecraft::util::bit_storage::{BitStorage, validate_inclusive_between};

/// `[multiplier, addend, shift]` for each count of values per long, from 1 to 64.
#[rustfmt::skip]
const MAGIC: [i32; 192] = [
    -1, -1, 0,
    i32::MIN, 0, 0,
    1431655765, 1431655765, 0,
    i32::MIN, 0, 1,
    858993459, 858993459, 0,
    715827882, 715827882, 0,
    613566756, 613566756, 0,
    i32::MIN, 0, 2,
    477218588, 477218588, 0,
    429496729, 429496729, 0,
    390451572, 390451572, 0,
    357913941, 357913941, 0,
    330382099, 330382099, 0,
    306783378, 306783378, 0,
    286331153, 286331153, 0,
    i32::MIN, 0, 3,
    252645135, 252645135, 0,
    238609294, 238609294, 0,
    226050910, 226050910, 0,
    214748364, 214748364, 0,
    204522252, 204522252, 0,
    195225786, 195225786, 0,
    186737708, 186737708, 0,
    178956970, 178956970, 0,
    171798691, 171798691, 0,
    165191049, 165191049, 0,
    159072862, 159072862, 0,
    153391689, 153391689, 0,
    148102320, 148102320, 0,
    143165576, 143165576, 0,
    138547332, 138547332, 0,
    i32::MIN, 0, 4,
    130150524, 130150524, 0,
    126322567, 126322567, 0,
    122713351, 122713351, 0,
    119304647, 119304647, 0,
    116080197, 116080197, 0,
    113025455, 113025455, 0,
    110127366, 110127366, 0,
    107374182, 107374182, 0,
    104755299, 104755299, 0,
    102261126, 102261126, 0,
    99882960, 99882960, 0,
    97612893, 97612893, 0,
    95443717, 95443717, 0,
    93368854, 93368854, 0,
    91382282, 91382282, 0,
    89478485, 89478485, 0,
    87652393, 87652393, 0,
    85899345, 85899345, 0,
    84215045, 84215045, 0,
    82595524, 82595524, 0,
    81037118, 81037118, 0,
    79536431, 79536431, 0,
    78090314, 78090314, 0,
    76695844, 76695844, 0,
    75350303, 75350303, 0,
    74051160, 74051160, 0,
    72796055, 72796055, 0,
    71582788, 71582788, 0,
    70409299, 70409299, 0,
    69273666, 69273666, 0,
    68174084, 68174084, 0,
    i32::MIN, 0, 5,
];

/// `bits`-bit values packed into longs, `64 / bits` to a long from the low bits up, none
/// straddling two longs.
#[derive(Clone, Debug, PartialEq)]
pub struct SimpleBitStorage {
    data: Vec<i64>,
    bits: i32,
    mask: i64,
    size: i32,
    values_per_long: i32,
    divide_mul: i32,
    divide_add: i32,
    divide_shift: i32,
}

impl SimpleBitStorage {
    /// `SimpleBitStorage(bits, size, int[] values)`: the values packed, each cut to `bits` bits.
    /// Panics where Java throws: bits outside 1 to 32, or fewer values than the size.
    pub fn from_values(bits: i32, size: i32, values: &[i32]) -> SimpleBitStorage {
        let mut storage = SimpleBitStorage::new(bits, size);
        let mut output_index = 0;

        let mut input_offset = 0;
        while input_offset <= size - storage.values_per_long {
            let mut packed_value = 0i64;

            for index_in_long in (0..storage.values_per_long).rev() {
                packed_value <<= bits;
                packed_value |=
                    values[(input_offset + index_in_long) as usize] as i64 & storage.mask;
            }

            storage.data[output_index] = packed_value;
            output_index += 1;
            input_offset += storage.values_per_long;
        }

        let remainder_count = size - input_offset;
        if remainder_count > 0 {
            let mut last_packed_value = 0i64;

            for index_in_long in (0..remainder_count).rev() {
                last_packed_value <<= bits;
                last_packed_value |=
                    values[(input_offset + index_in_long) as usize] as i64 & storage.mask;
            }

            storage.data[output_index] = last_packed_value;
        }

        storage
    }

    /// `SimpleBitStorage(bits, size)`: all zeros.
    pub fn new(bits: i32, size: i32) -> SimpleBitStorage {
        SimpleBitStorage::from_raw(bits, size, None)
    }

    /// `SimpleBitStorage(bits, size, long[] data)`, which keeps the longs. Panics where Java
    /// throws: bits outside 1 to 32, and longs other than as many as the size needs
    /// (`InitializationException`).
    pub fn from_raw(bits: i32, size: i32, data: Option<Vec<i64>>) -> SimpleBitStorage {
        validate_inclusive_between(1, 32, bits as i64);
        let mask = (1i64 << bits) - 1;
        // Java's `(char)`, which can't change a count from 2 to 64.
        let values_per_long = (64 / bits) as u16 as i32;
        let row = (3 * (values_per_long - 1)) as usize;
        let required_length = size.wrapping_add(values_per_long).wrapping_sub(1) / values_per_long;
        let data = match data {
            Some(data) => {
                if data.len() as i64 != required_length as i64 {
                    panic!(
                        "Invalid length given for storage, got: {} but expected: {required_length}",
                        data.len()
                    );
                }
                data
            }
            None => {
                assert!(required_length >= 0, "{required_length}");
                vec![0; required_length as usize]
            }
        };

        SimpleBitStorage {
            data,
            bits,
            mask,
            size,
            values_per_long,
            divide_mul: MAGIC[row],
            divide_add: MAGIC[row + 1],
            divide_shift: MAGIC[row + 2],
        }
    }

    /// Which long the index is in: the index divided by the values per long.
    fn cell_index(&self, bit_index: i32) -> i32 {
        let mul = self.divide_mul as u32 as i64;
        let add = self.divide_add as u32 as i64;
        ((bit_index as i64).wrapping_mul(mul).wrapping_add(add) >> 32 >> self.divide_shift) as i32
    }

    fn check_index(&self, index: i32) {
        validate_inclusive_between(0, self.size.wrapping_sub(1) as i64, index as i64);
    }
}

impl BitStorage for SimpleBitStorage {
    fn get_and_set(&mut self, index: i32, value: i32) -> i32 {
        self.check_index(index);
        validate_inclusive_between(0, self.mask, value as i64);
        let cell_index = self.cell_index(index);
        let cell_value = self.data[cell_index as usize];
        let bit_index = (index - cell_index * self.values_per_long) * self.bits;
        let old_value = (cell_value >> bit_index & self.mask) as i32;
        self.data[cell_index as usize] =
            cell_value & !(self.mask << bit_index) | (value as i64 & self.mask) << bit_index;
        old_value
    }

    fn set(&mut self, index: i32, value: i32) {
        self.check_index(index);
        validate_inclusive_between(0, self.mask, value as i64);
        let cell_index = self.cell_index(index);
        let cell_value = self.data[cell_index as usize];
        let bit_index = (index - cell_index * self.values_per_long) * self.bits;
        self.data[cell_index as usize] =
            cell_value & !(self.mask << bit_index) | (value as i64 & self.mask) << bit_index;
    }

    /// Every value set to one, by doubling it across each long, which also writes it into the
    /// bits past the long's last whole value (cut off at the top).
    fn fill(&mut self, value: i32) {
        validate_inclusive_between(0, self.mask, value as i64);
        let mut cell_value = value as i64 & self.mask;
        let mut offset = self.bits;

        loop {
            cell_value |= cell_value << offset;
            offset <<= 1;
            if offset >= 64 {
                break;
            }
        }

        self.data.fill(cell_value);
    }

    fn get(&self, index: i32) -> i32 {
        self.check_index(index);
        let cell_index = self.cell_index(index);
        let cell_value = self.data[cell_index as usize];
        let bit_index = (index - cell_index * self.values_per_long) * self.bits;
        (cell_value >> bit_index & self.mask) as i32
    }

    fn get_raw(&self) -> &[i64] {
        &self.data
    }

    fn get_raw_mut(&mut self) -> &mut [i64] {
        &mut self.data
    }

    fn get_size(&self) -> i32 {
        self.size
    }

    fn get_bits(&self) -> i32 {
        self.bits
    }

    fn get_all(&self, output: &mut dyn FnMut(i32)) {
        let mut count = 0;

        for &cell in &self.data {
            let mut cell_value = cell;
            for _ in 0..self.values_per_long {
                output((cell_value & self.mask) as i32);
                cell_value >>= self.bits;
                count += 1;
                if count >= self.size {
                    return;
                }
            }
        }
    }

    fn unpack(&self, output: &mut [i32]) {
        let data_length = self.data.len();
        let mut output_offset = 0;

        for i in 0..data_length.saturating_sub(1) {
            let mut cell_value = self.data[i];

            for index_in_long in 0..self.values_per_long {
                output[(output_offset + index_in_long) as usize] = (cell_value & self.mask) as i32;
                cell_value >>= self.bits;
            }

            output_offset += self.values_per_long;
        }

        let remainder = self.size - output_offset;
        if remainder > 0 {
            let mut cell_value = self.data[data_length - 1];

            for index_in_long in 0..remainder {
                output[(output_offset + index_in_long) as usize] = (cell_value & self.mask) as i32;
                cell_value >>= self.bits;
            }
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use serde_json::Value;

    use super::SimpleBitStorage;
    use crate::golden::{Encode, Golden, Thrown, catch};
    use crate::net::minecraft::util::bit_storage::BitStorage;

    /// Every value, by `get`.
    pub(crate) fn read_all(storage: &dyn BitStorage) -> Vec<i32> {
        (0..storage.get_size())
            .map(|index| storage.get(index))
            .collect()
    }

    pub(crate) fn get_all(storage: &dyn BitStorage) -> Vec<i32> {
        let mut values = Vec::new();
        storage.get_all(&mut |value| values.push(value));
        values
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.util.SimpleBitStorage");
        golden.check("<init>(II)V", |bits: i32, size: i32| {
            let storage = SimpleBitStorage::new(bits, size);
            (
                storage.get_bits(),
                storage.get_size(),
                storage.get_raw().to_vec(),
            )
        });
        golden.check("<init>(II[J)V", |bits: i32, size: i32, data: Vec<i64>| {
            read_all(&SimpleBitStorage::from_raw(bits, size, Some(data)))
        });
        golden.check("<init>(II[I)V", |bits: i32, size: i32, values: Vec<i32>| {
            SimpleBitStorage::from_values(bits, size, &values)
                .get_raw()
                .to_vec()
        });
        golden.check("unpack([I)V", |bits: i32, size: i32, data: Vec<i64>| {
            let storage = SimpleBitStorage::from_raw(bits, size, Some(data));
            let mut output = vec![0; size as usize];
            storage.unpack(&mut output);
            output
        });
        golden.check(
            "getAll(Ljava/util/function/IntConsumer;)V",
            |bits: i32, size: i32, data: Vec<i64>| {
                get_all(&SimpleBitStorage::from_raw(bits, size, Some(data)))
            },
        );
        golden.check(
            "fill(I)V",
            |bits: i32, size: i32, data: Vec<i64>, value: i32| {
                let mut storage = SimpleBitStorage::from_raw(bits, size, Some(data));
                storage.fill(value);
                storage.get_raw().to_vec()
            },
        );
        golden.check(
            "programs",
            |bits: i32, size: i32, data: Vec<i64>, ops: Value| {
                run(SimpleBitStorage::from_raw(bits, size, Some(data)), &ops)
            },
        );
        golden.finish();
    }

    // Mirrors SimpleBitStorageSuite.run in oracle/golden/: each op's output, a panic ending them,
    // and the longs after.
    fn run(mut storage: SimpleBitStorage, ops: &Value) -> (Vec<Box<dyn Encode>>, Vec<i64>) {
        let mut outputs: Vec<Box<dyn Encode>> = Vec::new();
        for op in ops.as_array().expect("a program is an array of ops") {
            let op = op.as_array().expect("an op is an array");
            let arg = |index: usize| op[index].as_i64().expect("ops take ints") as i32;
            let output = catch(|| -> Box<dyn Encode> {
                match op[0].as_str().expect("ops start with their name") {
                    "get" => Box::new(storage.get(arg(1))),
                    "set" => {
                        storage.set(arg(1), arg(2));
                        Box::new(())
                    }
                    "getAndSet" => Box::new(storage.get_and_set(arg(1), arg(2))),
                    "fill" => {
                        storage.fill(arg(1));
                        Box::new(())
                    }
                    "copy" => {
                        storage = storage.clone();
                        Box::new(())
                    }
                    name => panic!("unknown op {name}"),
                }
            });
            match output {
                Ok(output) => outputs.push(output),
                Err(_) => {
                    outputs.push(Box::new(Thrown));
                    break;
                }
            }
        }
        (outputs, storage.get_raw().to_vec())
    }
}
