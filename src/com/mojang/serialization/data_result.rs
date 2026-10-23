//! com/mojang/serialization/DataResult.java (DataFixerUpper 10.0.21)
//!
//! What a codec's read or write gives: a value, or an error with its message and maybe a partial
//! value, either with its `Lifecycle`. Java's is a sealed interface of two records, `Success` and
//! `Error`, so here it's an enum of the two.
//!
//! Java's error message is a `Supplier`, made into the string only when asked for; here it's the
//! string, made when the error is. The two only differ when what a supplier reads changes after
//! the error is made, which no ported caller's does. `ap`, `apply2`, `apply2stable` and `apply3`
//! are DataFixerUpper's applicative (`DataResult.Instance`, whose higher-kinded `App` isn't ported)
//! written out: a curried function applied to each argument in turn, so the errors' messages join
//! in the arguments' reverse order (`appendMessages`). `unbox`, `instance` and `Mu` are that
//! machinery's, and `partialGet` comes with its first caller.

use std::fmt;

use crate::com::mojang::serialization::lifecycle::Lifecycle;

/// A value, or an error.
#[derive(Clone, Debug, PartialEq)]
pub enum DataResult<R> {
    Success(Success<R>),
    Error(Error<R>),
}

/// `DataResult.Success`: the value.
#[derive(Clone, Debug, PartialEq)]
pub struct Success<R> {
    value: R,
    lifecycle: Lifecycle,
}

/// `DataResult.Error`: what went wrong, and what could be read or written anyway.
#[derive(Clone, Debug, PartialEq)]
pub struct Error<R> {
    message: String,
    partial_value: Option<R>,
    lifecycle: Lifecycle,
}

impl<R> Success<R> {
    pub fn value(&self) -> &R {
        &self.value
    }

    pub fn lifecycle(&self) -> Lifecycle {
        self.lifecycle
    }
}

impl<R> Error<R> {
    pub fn message(&self) -> &str {
        &self.message
    }

    pub fn partial_value(&self) -> Option<&R> {
        self.partial_value.as_ref()
    }

    pub fn lifecycle(&self) -> Lifecycle {
        self.lifecycle
    }
}

impl<R> DataResult<R> {
    /// `success(R)`: experimental, like every result made without a lifecycle.
    pub fn success(result: R) -> DataResult<R> {
        DataResult::success_lifecycle(result, Lifecycle::experimental())
    }

    /// `success(R, Lifecycle)`.
    pub fn success_lifecycle(result: R, lifecycle: Lifecycle) -> DataResult<R> {
        DataResult::Success(Success {
            value: result,
            lifecycle,
        })
    }

    /// `error(Supplier)`.
    pub fn error(message: impl FnOnce() -> String) -> DataResult<R> {
        DataResult::error_lifecycle(message, Lifecycle::experimental())
    }

    /// `error(Supplier, R)`: an error with a partial value.
    pub fn error_partial(message: impl FnOnce() -> String, partial_result: R) -> DataResult<R> {
        DataResult::error_partial_lifecycle(message, partial_result, Lifecycle::experimental())
    }

    /// `error(Supplier, R, Lifecycle)`.
    pub fn error_partial_lifecycle(
        message: impl FnOnce() -> String,
        partial_result: R,
        lifecycle: Lifecycle,
    ) -> DataResult<R> {
        DataResult::Error(Error {
            message: message(),
            partial_value: Some(partial_result),
            lifecycle,
        })
    }

    /// `error(Supplier, Lifecycle)`.
    pub fn error_lifecycle(
        message: impl FnOnce() -> String,
        lifecycle: Lifecycle,
    ) -> DataResult<R> {
        DataResult::Error(Error {
            message: message(),
            partial_value: None,
            lifecycle,
        })
    }

    /// `appendMessages`: how errors' messages join.
    pub fn append_messages(first: &str, second: &str) -> String {
        format!("{first}; {second}")
    }

    pub fn result(&self) -> Option<&R> {
        match self {
            DataResult::Success(success) => Some(&success.value),
            DataResult::Error(_) => None,
        }
    }

    /// `result()`, taking the value.
    pub fn into_result(self) -> Option<R> {
        match self {
            DataResult::Success(success) => Some(success.value),
            DataResult::Error(_) => None,
        }
    }

    pub fn error_ref(&self) -> Option<&Error<R>> {
        match self {
            DataResult::Success(_) => None,
            DataResult::Error(error) => Some(error),
        }
    }

    pub fn lifecycle(&self) -> Lifecycle {
        match self {
            DataResult::Success(success) => success.lifecycle,
            DataResult::Error(error) => error.lifecycle,
        }
    }

    pub fn has_result_or_partial(&self) -> bool {
        match self {
            DataResult::Success(_) => true,
            DataResult::Error(error) => error.partial_value.is_some(),
        }
    }

    /// `resultOrPartial(Consumer)`: the value, or the partial one after telling `on_error` the
    /// message.
    pub fn result_or_partial_with(self, on_error: impl FnOnce(&str)) -> Option<R> {
        match self {
            DataResult::Success(success) => Some(success.value),
            DataResult::Error(error) => {
                on_error(&error.message);
                error.partial_value
            }
        }
    }

    /// `resultOrPartial()`.
    pub fn result_or_partial(self) -> Option<R> {
        match self {
            DataResult::Success(success) => Some(success.value),
            DataResult::Error(error) => error.partial_value,
        }
    }

    /// `getOrThrow()`: panics with the message, where Java throws `IllegalStateException`.
    pub fn get_or_throw(self) -> R {
        match self {
            DataResult::Success(success) => success.value,
            DataResult::Error(error) => panic!("IllegalStateException: {}", error.message),
        }
    }

    /// `getPartialOrThrow()`: the value or the partial one, or panics with the message.
    pub fn get_partial_or_throw(self) -> R {
        match self {
            DataResult::Success(success) => success.value,
            DataResult::Error(Error {
                partial_value: Some(partial),
                ..
            }) => partial,
            DataResult::Error(error) => panic!("IllegalStateException: {}", error.message),
        }
    }

    /// The value, or the partial one, mapped.
    pub fn map<T>(self, function: impl FnOnce(R) -> T) -> DataResult<T> {
        match self {
            DataResult::Success(success) => DataResult::Success(Success {
                value: function(success.value),
                lifecycle: success.lifecycle,
            }),
            DataResult::Error(error) => DataResult::Error(Error {
                message: error.message,
                partial_value: error.partial_value.map(function),
                lifecycle: error.lifecycle,
            }),
        }
    }

    pub fn map_or_else<T>(
        self,
        success_function: impl FnOnce(R) -> T,
        error_function: impl FnOnce(Error<R>) -> T,
    ) -> T {
        match self {
            DataResult::Success(success) => success_function(success.value),
            DataResult::Error(error) => error_function(error),
        }
    }

    pub fn if_success(self, if_success: impl FnOnce(&R)) -> DataResult<R> {
        if let DataResult::Success(success) = &self {
            if_success(&success.value);
        }
        self
    }

    pub fn if_error(self, if_error: impl FnOnce(&Error<R>)) -> DataResult<R> {
        if let DataResult::Error(error) = &self {
            if_error(error);
        }
        self
    }

    /// An error with a partial value becomes a success of it, after telling `on_error` the
    /// message.
    pub fn promote_partial(self, on_error: impl FnOnce(&str)) -> DataResult<R> {
        match self {
            DataResult::Success(_) => self,
            DataResult::Error(error) => {
                on_error(&error.message);
                match error.partial_value {
                    Some(value) => DataResult::Success(Success {
                        value,
                        lifecycle: error.lifecycle,
                    }),
                    None => DataResult::Error(error),
                }
            }
        }
    }

    /// The result of the function on the value, or on the partial one, the lifecycles added and
    /// an error's message first.
    pub fn flat_map<R2>(self, function: impl FnOnce(R) -> DataResult<R2>) -> DataResult<R2> {
        match self {
            DataResult::Success(success) => {
                function(success.value).add_lifecycle(success.lifecycle)
            }
            DataResult::Error(Error {
                message,
                partial_value: None,
                lifecycle,
            }) => DataResult::Error(Error {
                message,
                partial_value: None,
                lifecycle,
            }),
            DataResult::Error(Error {
                message,
                partial_value: Some(partial),
                lifecycle,
            }) => {
                let second = function(partial);
                let combined_lifecycle = lifecycle.add(second.lifecycle());
                match second {
                    DataResult::Success(second) => DataResult::Error(Error {
                        message,
                        partial_value: Some(second.value),
                        lifecycle: combined_lifecycle,
                    }),
                    DataResult::Error(second) => DataResult::Error(Error {
                        message: DataResult::<R>::append_messages(&message, &second.message),
                        partial_value: second.partial_value,
                        lifecycle: combined_lifecycle,
                    }),
                }
            }
        }
    }

    /// The function's result applied to this one: an error's message is this one's, then the
    /// function's.
    pub fn ap<R2, F: FnOnce(R) -> R2>(self, function_result: DataResult<F>) -> DataResult<R2> {
        let combined_lifecycle = self.lifecycle().add(function_result.lifecycle());
        match (self, function_result) {
            (DataResult::Success(this), DataResult::Success(function)) => {
                DataResult::Success(Success {
                    value: (function.value)(this.value),
                    lifecycle: combined_lifecycle,
                })
            }
            (DataResult::Success(this), DataResult::Error(function)) => DataResult::Error(Error {
                message: function.message,
                partial_value: function.partial_value.map(|function| function(this.value)),
                lifecycle: combined_lifecycle,
            }),
            (DataResult::Error(this), DataResult::Success(function)) => DataResult::Error(Error {
                message: this.message,
                partial_value: this.partial_value.map(function.value),
                lifecycle: combined_lifecycle,
            }),
            (DataResult::Error(this), DataResult::Error(function)) => DataResult::Error(Error {
                message: DataResult::<R>::append_messages(&this.message, &function.message),
                partial_value: this
                    .partial_value
                    .and_then(|value| function.partial_value.map(|function| function(value))),
                lifecycle: combined_lifecycle,
            }),
        }
    }

    /// `apply2`: the function of both values, experimental (the applicative's `point`).
    pub fn apply2<R2, S>(
        self,
        function: impl FnOnce(R, R2) -> S,
        second: DataResult<R2>,
    ) -> DataResult<S> {
        apply2_from(Lifecycle::experimental(), function, self, second)
    }

    /// `apply2stable`: the same, from a stable function, so a stable result when both are.
    pub fn apply2stable<R2, S>(
        self,
        function: impl FnOnce(R, R2) -> S,
        second: DataResult<R2>,
    ) -> DataResult<S> {
        apply2_from(Lifecycle::stable(), function, self, second)
    }

    /// `apply3`: the function of the three values, experimental.
    pub fn apply3<R2, R3, S>(
        self,
        function: impl FnOnce(R, R2, R3) -> S,
        second: DataResult<R2>,
        third: DataResult<R3>,
    ) -> DataResult<S> {
        let lifecycle = Lifecycle::experimental()
            .add(self.lifecycle())
            .add(second.lifecycle())
            .add(third.lifecycle());
        let mut messages = Vec::new();
        let (first, second, third) = (self.split(), second.split(), third.split());
        for message in [&third.1, &second.1, &first.1].into_iter().flatten() {
            messages.push(message.clone());
        }
        let value = match (first.0, second.0, third.0) {
            (Some(first), Some(second), Some(third)) => Some(function(first, second, third)),
            _ => None,
        };
        joined(messages, value, lifecycle)
    }

    /// `setPartial(R)`: an error's partial value; a success stays as it is.
    pub fn set_partial(self, partial: R) -> DataResult<R> {
        match self {
            DataResult::Success(_) => self,
            DataResult::Error(error) => DataResult::Error(Error {
                message: error.message,
                partial_value: Some(partial),
                lifecycle: error.lifecycle,
            }),
        }
    }

    /// `setPartial(Supplier)`, which makes the partial value even for a success.
    pub fn set_partial_with(self, partial: impl FnOnce() -> R) -> DataResult<R> {
        match self {
            DataResult::Success(_) => self,
            DataResult::Error(_) => self.set_partial(partial()),
        }
    }

    pub fn map_error(self, function: impl FnOnce(String) -> String) -> DataResult<R> {
        match self {
            DataResult::Success(_) => self,
            DataResult::Error(error) => DataResult::Error(Error {
                message: function(error.message),
                partial_value: error.partial_value,
                lifecycle: error.lifecycle,
            }),
        }
    }

    pub fn set_lifecycle(self, lifecycle: Lifecycle) -> DataResult<R> {
        match self {
            DataResult::Success(success) => DataResult::Success(Success {
                value: success.value,
                lifecycle,
            }),
            DataResult::Error(error) => DataResult::Error(Error {
                message: error.message,
                partial_value: error.partial_value,
                lifecycle,
            }),
        }
    }

    pub fn add_lifecycle(self, lifecycle: Lifecycle) -> DataResult<R> {
        let added = self.lifecycle().add(lifecycle);
        self.set_lifecycle(added)
    }

    pub fn is_success(&self) -> bool {
        matches!(self, DataResult::Success(_))
    }

    pub fn is_error(&self) -> bool {
        !self.is_success()
    }

    // The value or the partial one, and an error's message.
    fn split(self) -> (Option<R>, Option<String>) {
        match self {
            DataResult::Success(success) => (Some(success.value), None),
            DataResult::Error(error) => (error.partial_value, Some(error.message)),
        }
    }
}

// The applicative's `ap2` of a function with this lifecycle on the two results: the curried
// function applied to the first, then the second, so an error from both has the second's message
// first.
fn apply2_from<R, R2, S>(
    function_lifecycle: Lifecycle,
    function: impl FnOnce(R, R2) -> S,
    first: DataResult<R>,
    second: DataResult<R2>,
) -> DataResult<S> {
    let lifecycle = function_lifecycle
        .add(first.lifecycle())
        .add(second.lifecycle());
    let (first, second) = (first.split(), second.split());
    let messages = [second.1, first.1].into_iter().flatten().collect();
    let value = match (first.0, second.0) {
        (Some(first), Some(second)) => Some(function(first, second)),
        _ => None,
    };
    joined(messages, value, lifecycle)
}

// A success of the value when there are no messages, and otherwise an error of the messages joined
// in order, with the value as its partial one.
fn joined<S>(messages: Vec<String>, value: Option<S>, lifecycle: Lifecycle) -> DataResult<S> {
    let mut messages = messages.into_iter();
    match messages.next() {
        None => DataResult::Success(Success {
            value: value.expect("every argument succeeded"),
            lifecycle,
        }),
        Some(first) => DataResult::Error(Error {
            message: messages.fold(first, |message, next| {
                DataResult::<S>::append_messages(&message, &next)
            }),
            partial_value: value,
            lifecycle,
        }),
    }
}

/// `DataResult.Success[value]` and `DataResult.Error['message': partial]`.
impl<R: fmt::Display> fmt::Display for DataResult<R> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DataResult::Success(success) => write!(f, "DataResult.Success[{}]", success.value),
            DataResult::Error(error) => {
                write!(f, "DataResult.Error['{}'", error.message)?;
                if let Some(partial) = &error.partial_value {
                    write!(f, ": {partial}")?;
                }
                f.write_str("]")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::panic::{self, AssertUnwindSafe};

    use serde_json::Value;

    use super::DataResult;
    use crate::com::mojang::serialization::lifecycle::Lifecycle;
    use crate::golden::Golden;

    fn lifecycle(spec: &Value) -> Lifecycle {
        match spec {
            Value::Array(deprecated) => {
                Lifecycle::deprecated(deprecated[1].as_i64().unwrap() as i32)
            }
            Value::String(name) if name == "Stable" => Lifecycle::stable(),
            _ => Lifecycle::experimental(),
        }
    }

    fn encode_lifecycle(lifecycle: Lifecycle) -> Value {
        match lifecycle {
            Lifecycle::Deprecated(since) => {
                Value::from(vec![Value::from("Deprecated"), Value::from(since)])
            }
            other => Value::from(other.to_string()),
        }
    }

    // One of the six ways to make a result (DataResultSuite).
    fn build(spec: &Value) -> DataResult<i32> {
        let spec = spec.as_array().unwrap();
        let int = |index: usize| spec[index].as_i64().unwrap() as i32;
        let message = |index: usize| spec[index].as_str().unwrap().to_owned();
        match spec[0].as_str().unwrap() {
            "success" => DataResult::success(int(1)),
            "successLifecycle" => DataResult::success_lifecycle(int(1), lifecycle(&spec[2])),
            "error" => DataResult::error(|| message(1)),
            "errorPartial" => DataResult::error_partial(|| message(1), int(2)),
            "errorPartialLifecycle" => {
                DataResult::error_partial_lifecycle(|| message(1), int(2), lifecycle(&spec[3]))
            }
            _ => DataResult::error_lifecycle(|| message(1), lifecycle(&spec[2])),
        }
    }

    fn encode(result: &DataResult<i32>) -> Value {
        let mut array = match result {
            DataResult::Success(success) => {
                vec![Value::from("success"), Value::from(*success.value())]
            }
            DataResult::Error(error) => vec![
                Value::from("error"),
                Value::from(error.message()),
                error
                    .partial_value()
                    .map_or(Value::Null, |partial| Value::from(*partial)),
            ],
        };
        array.push(encode_lifecycle(result.lifecycle()));
        Value::from(array)
    }

    // The thrown message, as the suite records it.
    fn get_or_throw(get: impl FnOnce() -> i32) -> Value {
        match panic::catch_unwind(AssertUnwindSafe(get)) {
            Ok(value) => Value::from(value),
            Err(payload) => {
                let message = payload
                    .downcast_ref::<String>()
                    .cloned()
                    .unwrap_or_default();
                Value::from(format!(
                    "throws {}",
                    message.trim_start_matches("IllegalStateException: ")
                ))
            }
        }
    }

    fn run(start: &Value, ops: &Value) -> Value {
        let mut result = build(start);
        let mut outputs = Vec::new();
        for op in ops.as_array().unwrap() {
            let op = op.as_array().unwrap();
            let mut output = Value::Null;
            result = match op[0].as_str().unwrap() {
                "map" => result.map(|x| x.wrapping_mul(2).wrapping_add(1)),
                "flatMap" => {
                    result.flat_map(|x| build(&op[1]).map(|y| x.wrapping_mul(31).wrapping_add(y)))
                }
                "ap" => {
                    let function =
                        build(&op[1]).map(|y| move |x: i32| x.wrapping_mul(7).wrapping_add(y));
                    result.ap(function)
                }
                "apply2" => {
                    result.apply2(|a, b: i32| a.wrapping_mul(3).wrapping_add(b), build(&op[1]))
                }
                "apply2stable" => result
                    .apply2stable(|a, b: i32| a.wrapping_mul(3).wrapping_add(b), build(&op[1])),
                "apply3" => result.apply3(
                    |a, b: i32, c: i32| {
                        a.wrapping_mul(5)
                            .wrapping_add(b.wrapping_mul(3))
                            .wrapping_add(c)
                    },
                    build(&op[1]),
                    build(&op[2]),
                ),
                "setPartial" => result.set_partial(op[1].as_i64().unwrap() as i32),
                "mapError" => result.map_error(|message| format!("[{message}]")),
                "setPartialSupplier" => result.set_partial_with(|| op[1].as_i64().unwrap() as i32),
                "mapOrElse" => {
                    output = Value::from(result.clone().map_or_else(
                        |value| format!("success {value}"),
                        |error| format!("error {}", error.message()),
                    ));
                    result
                }
                "setLifecycle" => result.set_lifecycle(lifecycle(&op[1])),
                "addLifecycle" => result.add_lifecycle(lifecycle(&op[1])),
                "promotePartial" => {
                    let mut messages = Vec::new();
                    let promoted =
                        result.promote_partial(|message| messages.push(message.to_owned()));
                    output = Value::from(messages);
                    promoted
                }
                "resultOrPartial" => {
                    let mut messages = Vec::new();
                    let value = result
                        .clone()
                        .result_or_partial_with(|message| messages.push(message.to_owned()));
                    output = Value::from(vec![
                        value.map_or(Value::Null, Value::from),
                        Value::from(messages),
                    ]);
                    result
                }
                "getOrThrow" => {
                    let copy = result.clone();
                    output = get_or_throw(move || copy.get_or_throw());
                    result
                }
                "getPartialOrThrow" => {
                    let copy = result.clone();
                    output = get_or_throw(move || copy.get_partial_or_throw());
                    result
                }
                "hasResultOrPartial" => {
                    output = Value::from(result.has_result_or_partial());
                    result
                }
                "isSuccess" => {
                    output = Value::from(result.is_success());
                    result
                }
                _ => {
                    output = Value::from(result.to_string());
                    result
                }
            };
            outputs.push(Value::from(vec![output, encode(&result)]));
        }
        Value::from(vec![encode(&build(start)), Value::from(outputs)])
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("com.mojang.serialization.DataResult");
        golden.check("programs", |start: Value, ops: Value| run(&start, &ops));
        golden.finish();
    }

    #[test]
    fn lifecycles_add_like_java() {
        let deprecated = Lifecycle::deprecated;
        assert_eq!(
            Lifecycle::stable().add(Lifecycle::stable()),
            Lifecycle::stable()
        );
        assert_eq!(deprecated(2).add(Lifecycle::stable()), deprecated(2));
        assert_eq!(deprecated(2).add(deprecated(1)), deprecated(1));
        assert_eq!(
            deprecated(1).add(Lifecycle::experimental()),
            Lifecycle::experimental()
        );
    }
}
