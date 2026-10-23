//! net/minecraft/world/entity/ai/attributes/AttributeModifier.java
//!
//! Not ported yet: `MAP_CODEC` and `CODEC`, and `Operation.CODEC`, which wait for DataFixerUpper's
//! codecs. Its record's `hashCode` hashes the operation's identity hash, which changes from run to run.
//!
//! `id()` lends the identifier, which `AttributeInstance` keys its modifiers by.

use std::fmt;
use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::java::lang::double::Double;
use crate::net::minecraft::network::codec::byte_buf_codecs::ByteBufCodecs;
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec};
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::util::by_id_map::{ByIdMap, OutOfBoundsStrategy};
use crate::net::minecraft::util::string_representable::StringRepresentable;

static STREAM_CODEC: StaticStreamCodec<ByteBuf, AttributeModifier> = LazyLock::new(|| {
    Box::new(stream_codec::composite3(
        Identifier::STREAM_CODEC,
        |modifier: &AttributeModifier| modifier.id().clone(),
        ByteBufCodecs::DOUBLE,
        AttributeModifier::amount,
        Operation::STREAM_CODEC,
        AttributeModifier::operation,
        AttributeModifier::new,
    ))
});

/// A change to an attribute's value, under an id: an amount added, or a multiple of its base or
/// total.
#[derive(Clone, Debug)]
pub struct AttributeModifier {
    id: Identifier,
    amount: f64,
    operation: Operation,
}

impl AttributeModifier {
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, AttributeModifier> = &STREAM_CODEC;

    pub fn new(id: Identifier, amount: f64, operation: Operation) -> AttributeModifier {
        AttributeModifier {
            id,
            amount,
            operation,
        }
    }

    pub fn id(&self) -> &Identifier {
        &self.id
    }

    pub fn amount(&self) -> f64 {
        self.amount
    }

    pub fn operation(&self) -> Operation {
        self.operation
    }

    pub fn is(&self, id: &Identifier) -> bool {
        *id == self.id
    }
}

/// A record's: the amount compares by its bits, NaNs made one (`Double.compare`).
impl PartialEq for AttributeModifier {
    fn eq(&self, other: &AttributeModifier) -> bool {
        self.id == other.id
            && Double::double_to_long_bits(self.amount) == Double::double_to_long_bits(other.amount)
            && self.operation == other.operation
    }
}

/// `AttributeModifier.Operation`: added to the base, a multiple of the base added, or the total
/// multiplied, in that order.
#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Operation {
    ADD_VALUE,
    ADD_MULTIPLIED_BASE,
    ADD_MULTIPLIED_TOTAL,
}

// By id, the first for any other.
static BY_ID: LazyLock<Box<dyn Fn(i32) -> Operation + Send + Sync>> = LazyLock::new(|| {
    Box::new(ByIdMap::continuous(
        Operation::id,
        &Operation::VALUES,
        OutOfBoundsStrategy::ZERO,
    ))
});

static OPERATION_STREAM_CODEC: StaticStreamCodec<ByteBuf, Operation> =
    LazyLock::new(|| Box::new(ByteBufCodecs::id_mapper(BY_ID.as_ref(), Operation::id)));

impl Operation {
    pub const VALUES: [Operation; 3] = [
        Operation::ADD_VALUE,
        Operation::ADD_MULTIPLIED_BASE,
        Operation::ADD_MULTIPLIED_TOTAL,
    ];
    pub const BY_ID: &'static LazyLock<Box<dyn Fn(i32) -> Operation + Send + Sync>> = &BY_ID;
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, Operation> =
        &OPERATION_STREAM_CODEC;

    pub fn values() -> [Operation; 3] {
        Operation::VALUES
    }

    /// Panics on anything but a constant's name, where Java throws `IllegalArgumentException`.
    pub fn value_of(name: &str) -> Operation {
        Operation::VALUES
            .into_iter()
            .find(|operation| format!("{operation:?}") == name)
            .unwrap_or_else(|| panic!("IllegalArgumentException: no Operation constant {name}"))
    }

    pub fn id(&self) -> i32 {
        match self {
            Operation::ADD_VALUE => 0,
            Operation::ADD_MULTIPLIED_BASE => 1,
            Operation::ADD_MULTIPLIED_TOTAL => 2,
        }
    }
}

impl StringRepresentable for Operation {
    fn get_serialized_name(&self) -> &'static str {
        match self {
            Operation::ADD_VALUE => "add_value",
            Operation::ADD_MULTIPLIED_BASE => "add_multiplied_base",
            Operation::ADD_MULTIPLIED_TOTAL => "add_multiplied_total",
        }
    }
}

/// The constant's name, like any enum's `toString`.
impl fmt::Display for Operation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

/// A record's: `AttributeModifier[id=minecraft:sprinting, amount=0.30000001192092896, operation=ADD_MULTIPLIED_TOTAL]`.
impl fmt::Display for AttributeModifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "AttributeModifier[id={}, amount={}, operation={}]",
            self.id,
            Double::to_string(self.amount),
            self.operation
        )
    }
}

#[cfg(test)]
mod tests {
    use super::{AttributeModifier, Operation};
    use crate::golden::{Golden, encode_fields, encode_names};
    use crate::net::minecraft::resources::identifier::Identifier;
    use crate::net::minecraft::util::string_representable::StringRepresentable;

    encode_fields!(AttributeModifier {
        id: Identifier,
        amount: f64,
        operation: Operation,
    });
    encode_names!(Operation {
        ADD_VALUE,
        ADD_MULTIPLIED_BASE,
        ADD_MULTIPLIED_TOTAL
    });

    type This = AttributeModifier;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.world.entity.ai.attributes.AttributeModifier");
        for name in ["CODEC", "MAP_CODEC"] {
            golden.pending(name, "waits for DataFixerUpper's codecs");
        }
        golden.pending("hashCode()I", "hashes the operation's identity hash");
        golden.check_method("toString()Ljava/lang/String;", |this: &mut This| {
            this.to_string()
        });
        golden.check_codec("STREAM_CODEC", || AttributeModifier::STREAM_CODEC);
        golden.check(
            "<init>(Lnet/minecraft/resources/Identifier;DLnet/minecraft/world/entity/ai/attributes/AttributeModifier$Operation;)V",
            AttributeModifier::new,
        );
        golden.check_method(
            "id()Lnet/minecraft/resources/Identifier;",
            |this: &mut This| this.id().clone(),
        );
        golden.check_method("amount()D", |this: &mut This| this.amount());
        golden.check_method(
            "operation()Lnet/minecraft/world/entity/ai/attributes/AttributeModifier$Operation;",
            |this: &mut This| this.operation(),
        );
        golden.check_method(
            "is(Lnet/minecraft/resources/Identifier;)Z",
            |this: &mut This, id: Identifier| this.is(&id),
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut This, other: This| *this == other,
        );
        golden.finish();
    }

    #[test]
    fn operation_matches_vanilla() {
        let golden =
            Golden::load("net.minecraft.world.entity.ai.attributes.AttributeModifier$Operation");
        golden.pending("CODEC", "waits for DataFixerUpper's codecs");
        golden.check_function("BY_ID", || Operation::BY_ID.as_ref());
        golden.check_codec("STREAM_CODEC", || Operation::STREAM_CODEC);
        for constant in Operation::VALUES {
            golden.field(&constant.to_string(), constant);
        }
        golden.check(
            "values()[Lnet/minecraft/world/entity/ai/attributes/AttributeModifier$Operation;",
            Operation::values,
        );
        golden.check(
            "valueOf(Ljava/lang/String;)Lnet/minecraft/world/entity/ai/attributes/AttributeModifier$Operation;",
            |name: String| Operation::value_of(&name),
        );
        golden.check_method("id()I", |this: &mut Operation| this.id());
        golden.check_method(
            "getSerializedName()Ljava/lang/String;",
            |this: &mut Operation| this.get_serialized_name().to_string(),
        );
        golden.finish();
    }
}
