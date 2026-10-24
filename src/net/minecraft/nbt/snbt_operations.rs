//! net/minecraft/nbt/SnbtOperations.java
//!
//! SNBT's operations, a name with arguments in parentheses: `bool(x)`, a number (not zero) or a
//! boolean as a boolean, and `uuid(s)`, a string `UUID.fromString` takes as its four ints. Each
//! fails, storing its error, on arguments it can't use.
//!
//! Not ported: `BUILTIN_IDS`, the names the command UI suggests. `BUILTIN_OPERATIONS`' anonymous
//! classes are `BuiltinOperation`'s variants, and its map a `HashMap`, which only `get` reads.

use std::collections::HashMap;
use std::fmt;
use std::sync::LazyLock;

use crate::com::mojang::brigadier::exceptions::simple_command_exception_type::SimpleCommandExceptionType;
use crate::com::mojang::brigadier::string_reader::StringReader;
use crate::com::mojang::serialization::dynamic_ops::{DynamicOps, OpsValue};
use crate::java::util::uuid::UUID;
use crate::net::minecraft::core::uuid_util::UUIDUtil;
use crate::net::minecraft::network::chat::component::Component;
use crate::net::minecraft::util::parsing::packrat::delayed_exception::{self, DelayedException};
use crate::net::minecraft::util::parsing::packrat::parse_state::ParseState;

static ERROR_EXPECTED_STRING_UUID: LazyLock<DelayedException> = LazyLock::new(|| {
    delayed_exception::create(SimpleCommandExceptionType::new(Component::from(
        Component::translatable("snbt.parser.expected_string_uuid"),
    )))
});
static ERROR_EXPECTED_NUMBER_OR_BOOLEAN: LazyLock<DelayedException> = LazyLock::new(|| {
    delayed_exception::create(SimpleCommandExceptionType::new(Component::from(
        Component::translatable("snbt.parser.expected_number_or_boolean"),
    )))
});

pub const BUILTIN_TRUE: &str = "true";
pub const BUILTIN_FALSE: &str = "false";

pub static BUILTIN_OPERATIONS: LazyLock<HashMap<BuiltinKey, BuiltinOperation>> =
    LazyLock::new(|| {
        HashMap::from([
            (
                BuiltinKey::new("bool".to_owned(), 1),
                BuiltinOperation::Bool,
            ),
            (
                BuiltinKey::new("uuid".to_owned(), 1),
                BuiltinOperation::Uuid,
            ),
        ])
    });

/// An operation's name and how many arguments it takes.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BuiltinKey {
    id: String,
    arg_count: i32,
}

impl BuiltinKey {
    pub fn new(id: String, arg_count: i32) -> BuiltinKey {
        BuiltinKey { id, arg_count }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn arg_count(&self) -> i32 {
        self.arg_count
    }
}

/// `id/argCount`.
impl fmt::Display for BuiltinKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}/{}", self.id, self.arg_count)
    }
}

/// What an operation does with its arguments.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuiltinOperation {
    Bool,
    Uuid,
}

impl BuiltinOperation {
    /// The operation's value, `None` once it stored its error, where Java's is `null`.
    pub fn run<T: OpsValue>(
        &self,
        ops: &dyn DynamicOps<T>,
        arguments: &[T],
        state: &mut dyn ParseState<StringReader>,
    ) -> Option<T> {
        match self {
            BuiltinOperation::Bool => match convert(ops, &arguments[0]) {
                Some(result) => Some(ops.create_boolean(result)),
                None => {
                    let mark = state.mark();
                    state
                        .error_collector()
                        .store(mark, &ERROR_EXPECTED_NUMBER_OR_BOOLEAN);
                    None
                }
            },
            BuiltinOperation::Uuid => {
                let uuid = ops
                    .get_string_value(&arguments[0])
                    .into_result()
                    .and_then(|arg| UUID::from_string(&arg).ok());
                match uuid {
                    Some(uuid) => Some(ops.create_int_list(&UUIDUtil::uuid_to_int_array(&uuid))),
                    None => {
                        let mark = state.mark();
                        state
                            .error_collector()
                            .store(mark, &ERROR_EXPECTED_STRING_UUID);
                        None
                    }
                }
            }
        }
    }
}

// `bool`'s argument as a boolean: the ops' own, or whether it's a number other than zero.
fn convert<T: OpsValue>(ops: &dyn DynamicOps<T>, arg: &T) -> Option<bool> {
    if let Some(as_boolean) = ops.get_boolean_value(arg).into_result() {
        return Some(as_boolean);
    }
    ops.get_number_value(arg)
        .into_result()
        .map(|as_number| as_number.double_value() != 0.0)
}
