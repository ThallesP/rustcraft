//! com/mojang/serialization/RecordBuilder.java (DataFixerUpper 10.0.21)
//!
//! What a map codec writes its fields into, then merges into its prefix. Java's fluent methods
//! return the builder; here they change it in place, which is what every encoder the game's codecs
//! reach does with the builder it's given. `withErrorsFrom` takes a result mapped to `()`, like
//! `ListBuilder`'s.
//!
//! `AbstractStringBuilder` is `StringBuilder`, generic in what it builds (`StringBuilderOps`: a
//! `CompoundTag` for NBT, a `JsonObject` for JSON). `AbstractUniversalBuilder` and `MapBuilder`,
//! `DynamicOps.mapBuilder()`'s default, aren't ported: every ops the game's codecs reach has its
//! own builder.

use crate::com::mojang::serialization::data_result::DataResult;
use crate::com::mojang::serialization::dynamic_ops::{DynamicOps, OpsValue};
use crate::com::mojang::serialization::lifecycle::Lifecycle;

pub trait RecordBuilder<T: OpsValue> {
    fn ops(&self) -> &dyn DynamicOps<T>;

    fn add(&mut self, key: T, value: T);

    /// `add(T, DataResult)`.
    fn add_result(&mut self, key: T, value: DataResult<T>);

    /// `add(DataResult, DataResult)`.
    fn add_results(&mut self, key: DataResult<T>, value: DataResult<T>);

    fn with_errors_from(&mut self, result: DataResult<()>);

    fn set_lifecycle(&mut self, lifecycle: Lifecycle);

    fn map_error(&mut self, on_error: &dyn Fn(String) -> String);

    fn build(&mut self, prefix: T) -> DataResult<T>;

    /// `build(DataResult)`.
    fn build_result(&mut self, prefix: DataResult<T>) -> DataResult<T> {
        prefix.flat_map(|prefix| self.build(prefix))
    }

    /// `add(String, T)`.
    fn add_str(&mut self, key: &str, value: T) {
        let key = self.ops().create_string(key);
        self.add(key, value);
    }

    /// `add(String, DataResult)`.
    fn add_str_result(&mut self, key: &str, value: DataResult<T>) {
        let key = self.ops().create_string(key);
        self.add_result(key, value);
    }
}

/// What an `AbstractStringBuilder` subclass fills in: its builder (`R`), how it starts, how a
/// field goes in, and how it's merged into the prefix.
pub trait StringBuilderOps<T> {
    type Builder;

    fn init_builder(&self) -> Self::Builder;

    fn append(&self, key: String, value: T, builder: Self::Builder) -> Self::Builder;

    fn build(&self, builder: Self::Builder, prefix: T) -> DataResult<T>;
}

/// `RecordBuilder.AbstractStringBuilder`: fields under string keys, a key that isn't one failing
/// the whole builder.
pub struct StringBuilder<'a, T, B: StringBuilderOps<T>> {
    ops: &'a dyn DynamicOps<T>,
    subclass: B,
    builder: DataResult<B::Builder>,
}

impl<'a, T: OpsValue, B: StringBuilderOps<T>> StringBuilder<'a, T, B> {
    pub fn new(ops: &'a dyn DynamicOps<T>, subclass: B) -> StringBuilder<'a, T, B> {
        let builder = DataResult::success_lifecycle(subclass.init_builder(), Lifecycle::stable());
        StringBuilder {
            ops,
            subclass,
            builder,
        }
    }

    fn take(&mut self) -> DataResult<B::Builder> {
        let fresh =
            DataResult::success_lifecycle(self.subclass.init_builder(), Lifecycle::stable());
        std::mem::replace(&mut self.builder, fresh)
    }
}

impl<T: OpsValue, B: StringBuilderOps<T>> RecordBuilder<T> for StringBuilder<'_, T, B> {
    fn ops(&self) -> &dyn DynamicOps<T> {
        self.ops
    }

    fn add_str(&mut self, key: &str, value: T) {
        let subclass = &self.subclass;
        let builder = std::mem::replace(&mut self.builder, DataResult::error(String::new));
        self.builder = builder.map(|b| subclass.append(key.to_owned(), value, b));
    }

    fn add_str_result(&mut self, key: &str, value: DataResult<T>) {
        let subclass = &self.subclass;
        let builder = std::mem::replace(&mut self.builder, DataResult::error(String::new));
        self.builder = builder.apply2stable(|b, v| subclass.append(key.to_owned(), v, b), value);
    }

    // The key's string, then the field added; a key that isn't a string replaces the builder
    // with its error, which has no partial value.
    fn add(&mut self, key: T, value: T) {
        let key = self.ops.get_string_value(&key);
        self.builder = key.flat_map(|k| {
            self.add_str(&k, value);
            self.take()
        });
    }

    fn add_result(&mut self, key: T, value: DataResult<T>) {
        let key = self.ops.get_string_value(&key);
        self.builder = key.flat_map(|k| {
            self.add_str_result(&k, value);
            self.take()
        });
    }

    fn add_results(&mut self, key: DataResult<T>, value: DataResult<T>) {
        let ops = self.ops;
        self.builder = key
            .flat_map(|key| ops.get_string_value(&key))
            .flat_map(|k| {
                self.add_str_result(&k, value);
                self.take()
            });
    }

    fn with_errors_from(&mut self, result: DataResult<()>) {
        self.builder = self.take().flat_map(|v| result.map(|_| v));
    }

    fn set_lifecycle(&mut self, lifecycle: Lifecycle) {
        self.builder = self.take().set_lifecycle(lifecycle);
    }

    fn map_error(&mut self, on_error: &dyn Fn(String) -> String) {
        self.builder = self.take().map_error(on_error);
    }

    fn build(&mut self, prefix: T) -> DataResult<T> {
        let builder = self.take();
        let subclass = &self.subclass;
        builder.flat_map(|b| subclass.build(b, prefix))
    }
}
