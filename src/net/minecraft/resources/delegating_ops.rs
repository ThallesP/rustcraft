//! net/minecraft/resources/DelegatingOps.java
//!
//! An ops that forwards everything to another, which `RegistryOps` extends with the registries.
//! Java's abstract class is the forwarding `DynamicOps` impl on `RegistryOps` here, its only
//! subclass; this file holds the builders it hands out, which forward to the delegate's builders
//! and answer `ops()` with the delegating ops, and `convertTo`, which leaves a value as it is when
//! it's converted to the delegate.

use crate::com::mojang::serialization::data_result::DataResult;
use crate::com::mojang::serialization::dynamic_ops::{DynamicOps, OpsValue};
use crate::com::mojang::serialization::lifecycle::Lifecycle;
use crate::com::mojang::serialization::list_builder::ListBuilder;
use crate::com::mojang::serialization::record_builder::RecordBuilder;

/// `DelegateListBuilder`.
pub struct DelegateListBuilder<'a, T: OpsValue> {
    pub(crate) original: Box<dyn ListBuilder<T> + 'a>,
    pub(crate) ops: &'a dyn DynamicOps<T>,
}

impl<T: OpsValue> ListBuilder<T> for DelegateListBuilder<'_, T> {
    fn ops(&self) -> &dyn DynamicOps<T> {
        self.ops
    }

    fn build(&mut self, prefix: T) -> DataResult<T> {
        self.original.build(prefix)
    }

    fn add(&mut self, value: T) {
        self.original.add(value);
    }

    fn add_result(&mut self, value: DataResult<T>) {
        self.original.add_result(value);
    }

    fn with_errors_from(&mut self, result: DataResult<()>) {
        self.original.with_errors_from(result);
    }

    fn map_error(&mut self, on_error: &dyn Fn(String) -> String) {
        self.original.map_error(on_error);
    }
}

/// `DelegateRecordBuilder`.
pub struct DelegateRecordBuilder<'a, T: OpsValue> {
    pub(crate) original: Box<dyn RecordBuilder<T> + 'a>,
    pub(crate) ops: &'a dyn DynamicOps<T>,
}

impl<T: OpsValue> RecordBuilder<T> for DelegateRecordBuilder<'_, T> {
    fn ops(&self) -> &dyn DynamicOps<T> {
        self.ops
    }

    fn add(&mut self, key: T, value: T) {
        self.original.add(key, value);
    }

    fn add_result(&mut self, key: T, value: DataResult<T>) {
        self.original.add_result(key, value);
    }

    fn add_results(&mut self, key: DataResult<T>, value: DataResult<T>) {
        self.original.add_results(key, value);
    }

    fn with_errors_from(&mut self, result: DataResult<()>) {
        self.original.with_errors_from(result);
    }

    fn set_lifecycle(&mut self, lifecycle: Lifecycle) {
        self.original.set_lifecycle(lifecycle);
    }

    fn map_error(&mut self, on_error: &dyn Fn(String) -> String) {
        self.original.map_error(on_error);
    }

    fn build(&mut self, prefix: T) -> DataResult<T> {
        self.original.build(prefix)
    }

    fn add_str(&mut self, key: &str, value: T) {
        self.original.add_str(key, value);
    }

    fn add_str_result(&mut self, key: &str, value: DataResult<T>) {
        self.original.add_str_result(key, value);
    }
}
