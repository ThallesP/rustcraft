//! com/mojang/serialization/ListBuilder.java (DataFixerUpper 10.0.21)
//!
//! What a list codec writes its elements into, then merges into its prefix. Java's fluent
//! methods return the builder; here they change it in place. `withErrorsFrom` takes a result of
//! anything, which only its success, message, partial value and lifecycle matter for, so it takes
//! the result mapped to `()`.

use crate::com::mojang::serialization::data_result::DataResult;
use crate::com::mojang::serialization::dynamic_ops::{DynamicOps, OpsValue};
use crate::com::mojang::serialization::lifecycle::Lifecycle;

pub trait ListBuilder<T: OpsValue> {
    fn ops(&self) -> &dyn DynamicOps<T>;

    fn build(&mut self, prefix: T) -> DataResult<T>;

    fn add(&mut self, value: T);

    /// `add(DataResult)`.
    fn add_result(&mut self, value: DataResult<T>);

    fn with_errors_from(&mut self, result: DataResult<()>);

    fn map_error(&mut self, on_error: &dyn Fn(String) -> String);

    /// `build(DataResult)`.
    fn build_result(&mut self, prefix: DataResult<T>) -> DataResult<T> {
        prefix.flat_map(|prefix| self.build(prefix))
    }
}

/// `ListBuilder.Builder`: the elements in an `ImmutableList.Builder`, a `Vec` here, merged with
/// `mergeToList`.
pub struct Builder<'a, T> {
    ops: &'a dyn DynamicOps<T>,
    builder: DataResult<Vec<T>>,
}

impl<'a, T: OpsValue> Builder<'a, T> {
    pub fn new(ops: &'a dyn DynamicOps<T>) -> Builder<'a, T> {
        Builder {
            ops,
            builder: DataResult::success_lifecycle(Vec::new(), Lifecycle::stable()),
        }
    }

    fn take(&mut self) -> DataResult<Vec<T>> {
        std::mem::replace(
            &mut self.builder,
            DataResult::success_lifecycle(Vec::new(), Lifecycle::stable()),
        )
    }
}

impl<T: OpsValue> ListBuilder<T> for Builder<'_, T> {
    fn ops(&self) -> &dyn DynamicOps<T> {
        self.ops
    }

    fn add(&mut self, value: T) {
        self.builder = self.take().map(|mut b| {
            b.push(value);
            b
        });
    }

    fn add_result(&mut self, value: DataResult<T>) {
        self.builder = self.take().apply2stable(
            |mut b, value| {
                b.push(value);
                b
            },
            value,
        );
    }

    fn with_errors_from(&mut self, result: DataResult<()>) {
        self.builder = self.take().flat_map(|r| result.map(|_| r));
    }

    fn map_error(&mut self, on_error: &dyn Fn(String) -> String) {
        self.builder = self.take().map_error(on_error);
    }

    fn build(&mut self, prefix: T) -> DataResult<T> {
        let ops = self.ops;
        self.take()
            .flat_map(|b| ops.merge_to_list_values(&prefix, b))
    }
}
