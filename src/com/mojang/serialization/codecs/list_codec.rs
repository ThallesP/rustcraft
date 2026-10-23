//! com/mojang/serialization/codecs/ListCodec.java (DataFixerUpper 10.0.21)
//!
//! A list of elements of one codec, of a size in a range. Decoding keeps going past an element
//! that fails, keeping what it read (and partial values), and the rest of the input is the list
//! of elements it couldn't read, or that came past the maximum.

use crate::com::mojang::datafixers::util::pair::Pair;
use crate::com::mojang::serialization::data_result::DataResult;
use crate::com::mojang::serialization::decoder::Decoder;
use crate::com::mojang::serialization::dynamic_ops::{DynamicOps, OpsValue};
use crate::com::mojang::serialization::encoder::{Encoder, Valued};
use crate::com::mojang::serialization::lifecycle::Lifecycle;

#[derive(Clone)]
pub struct ListCodec<C> {
    element_codec: C,
    min_size: i32,
    max_size: i32,
}

impl<C> ListCodec<C> {
    pub fn new(element_codec: C, min_size: i32, max_size: i32) -> ListCodec<C> {
        ListCodec {
            element_codec,
            min_size,
            max_size,
        }
    }

    fn create_too_short_error<R>(&self, size: usize) -> DataResult<R> {
        let (min, max) = (self.min_size, self.max_size);
        DataResult::error(|| format!("List is too short: {size}, expected range [{min}-{max}]"))
    }

    fn create_too_long_error<R>(&self, size: usize) -> DataResult<R> {
        let (min, max) = (self.min_size, self.max_size);
        DataResult::error(|| format!("List is too long: {size}, expected range [{min}-{max}]"))
    }
}

impl<C: Valued> Valued for ListCodec<C> {
    type Value = Vec<C::Value>;
}

impl<T: OpsValue, C: Encoder<T>> Encoder<T> for ListCodec<C> {
    fn encode(&self, input: &Vec<C::Value>, ops: &dyn DynamicOps<T>, prefix: T) -> DataResult<T> {
        if (input.len() as i64) < i64::from(self.min_size) {
            return self.create_too_short_error(input.len());
        }
        if (input.len() as i64) > i64::from(self.max_size) {
            return self.create_too_long_error(input.len());
        }
        let mut builder = ops.list_builder();
        for element in input {
            builder.add_result(self.element_codec.encode_start(ops, element));
        }
        builder.build(prefix)
    }
}

/// `decode`, with `DecoderState`'s `accept` for each element and its `build`.
impl<T: OpsValue, C: Decoder<T>> Decoder<T> for ListCodec<C> {
    fn decode(&self, ops: &dyn DynamicOps<T>, input: &T) -> DataResult<Pair<Vec<C::Value>, T>> {
        ops.get_list(input)
            .set_lifecycle(Lifecycle::stable())
            .flat_map(|stream| {
                let mut elements = Vec::new();
                let mut failed = Vec::new();
                let mut result = DataResult::success_lifecycle((), Lifecycle::stable());
                let mut total_count = 0usize;

                for value in stream {
                    total_count += 1;
                    if (elements.len() as i64) >= i64::from(self.max_size) {
                        failed.push(value);
                        continue;
                    }
                    let element_result = self.element_codec.decode(ops, &value);
                    if element_result.is_error() {
                        failed.push(value);
                    }
                    let (pair, status) = element_result.split_status();
                    if let Some(pair) = pair {
                        elements.push(pair.into_first());
                    }
                    result = result.apply2stable(|result, _| result, status);
                }

                if (elements.len() as i64) < i64::from(self.min_size) {
                    return self.create_too_short_error(elements.len());
                }
                let errors = ops.create_list(failed);
                if (total_count as i64) > i64::from(self.max_size) {
                    result = self.create_too_long_error(total_count);
                }
                // `result.map(ignored -> pair).setPartial(pair)`: an error always has the pair.
                let lifecycle = result.lifecycle();
                let pair = Pair::new(elements, errors);
                match result.split().1 {
                    None => DataResult::success_lifecycle(pair, lifecycle),
                    Some(message) => {
                        DataResult::error_partial_lifecycle(|| message, pair, lifecycle)
                    }
                }
            })
    }
}
