//! net/minecraft/world/entity/PositionPath.java
//!
//! Where an entity goes: a position, or steps each with its ticks. Java's sealed interface and its
//! two records are an enum here, and `Type` the enum of their kinds; each record's codec is
//! `LINEAR_STREAM_CODEC` or `STEPPED_STREAM_CODEC`. Its codec is `Type.STREAM_CODEC.dispatch`,
//! written out: the kind's id, then its record's codec.

use std::fmt;
use std::sync::LazyLock;

use crate::io::netty::buffer::byte_buf::ByteBuf;
use crate::net::minecraft::network::codec::byte_buf_codecs::{ByteBufCodecs, List};
use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec, StreamCodec};
use crate::net::minecraft::world::entity::position_step::PositionStep;
use crate::net::minecraft::world::phys::vec3::Vec3;

static STEPS_STREAM_CODEC: LazyLock<List<&'static StaticStreamCodec<ByteBuf, PositionStep>>> =
    LazyLock::new(|| PositionStep::STREAM_CODEC.apply(ByteBufCodecs::list()));

// `Linear.STREAM_CODEC`: its position.
static LINEAR_STREAM_CODEC: StaticStreamCodec<ByteBuf, PositionPath> = LazyLock::new(|| {
    Box::new(stream_codec::of(
        |output: &mut ByteBuf, value: &PositionPath| {
            Vec3::STREAM_CODEC.encode(output, &value.end_position());
        },
        |input: &mut ByteBuf| PositionPath::of(Vec3::STREAM_CODEC.decode(input)),
    ))
});

// `Stepped.STREAM_CODEC`: its steps, the last one's position its end.
static STEPPED_STREAM_CODEC: StaticStreamCodec<ByteBuf, PositionPath> = LazyLock::new(|| {
    Box::new(stream_codec::of(
        |output: &mut ByteBuf, value: &PositionPath| match value {
            PositionPath::Stepped { steps, .. } => STEPS_STREAM_CODEC.encode(output, steps),
            PositionPath::Linear { .. } => panic!("a linear path isn't a Stepped"),
        },
        |input: &mut ByteBuf| PositionPath::stepped(STEPS_STREAM_CODEC.decode(input)),
    ))
});

static STREAM_CODEC: StaticStreamCodec<ByteBuf, PositionPath> = LazyLock::new(|| {
    Box::new(stream_codec::of(
        |output: &mut ByteBuf, value: &PositionPath| {
            ByteBufCodecs::VAR_INT.encode(output, &value.r#type().ordinal());
            value.r#type().stream_codec().encode(output, value);
        },
        |input: &mut ByteBuf| {
            Type::by_id(ByteBufCodecs::VAR_INT.decode(input))
                .stream_codec()
                .decode(input)
        },
    ))
});

/// An entity's path.
#[derive(Clone, Debug, PartialEq)]
pub enum PositionPath {
    /// `PositionPath.Linear`: straight to the position.
    Linear { end_position: Vec3 },
    /// `PositionPath.Stepped`: through the steps, ending where it says (its codec writes only the
    /// steps, and reads the last one's position as the end).
    Stepped {
        end_position: Vec3,
        steps: Vec<PositionStep>,
    },
}

/// `PositionPath.Type`, a path's kind.
#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Type {
    LINEAR,
    STEPPED,
}

impl Type {
    pub const VALUES: [Type; 2] = [Type::LINEAR, Type::STEPPED];

    pub fn ordinal(self) -> i32 {
        self as i32
    }

    /// `streamCodec()`: its record's.
    pub fn stream_codec(self) -> &'static StaticStreamCodec<ByteBuf, PositionPath> {
        match self {
            Type::LINEAR => &LINEAR_STREAM_CODEC,
            Type::STEPPED => &STEPPED_STREAM_CODEC,
        }
    }

    /// `BY_ID`: `ByIdMap.continuous` by ordinal with `ZERO`, so any id past them is `LINEAR`.
    pub fn by_id(id: i32) -> Type {
        usize::try_from(id)
            .ok()
            .and_then(|index| Type::VALUES.get(index).copied())
            .unwrap_or(Type::LINEAR)
    }
}

impl PositionPath {
    pub const STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, PositionPath> = &STREAM_CODEC;
    pub const LINEAR_STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, PositionPath> =
        &LINEAR_STREAM_CODEC;
    pub const STEPPED_STREAM_CODEC: &'static StaticStreamCodec<ByteBuf, PositionPath> =
        &STEPPED_STREAM_CODEC;

    pub fn of(position: Vec3) -> PositionPath {
        PositionPath::Linear {
            end_position: position,
        }
    }

    /// `Stepped(List)`: ending at the last step. Panics where Java throws
    /// `NoSuchElementException`, for no steps.
    pub fn stepped(steps: Vec<PositionStep>) -> PositionPath {
        let end_position = steps
            .last()
            .unwrap_or_else(|| panic!("NoSuchElementException: no steps"))
            .position();
        PositionPath::Stepped {
            end_position,
            steps,
        }
    }

    pub fn end_position(&self) -> Vec3 {
        match self {
            PositionPath::Linear { end_position } | PositionPath::Stepped { end_position, .. } => {
                *end_position
            }
        }
    }

    pub fn r#type(&self) -> Type {
        match self {
            PositionPath::Linear { .. } => Type::LINEAR,
            PositionPath::Stepped { .. } => Type::STEPPED,
        }
    }

    /// A record's hash: a linear path's position's, or the end's and the steps' list's.
    pub fn hash_code(&self) -> i32 {
        match self {
            PositionPath::Linear { end_position } => end_position.hash_code(),
            PositionPath::Stepped {
                end_position,
                steps,
            } => end_position.hash_code().wrapping_mul(31).wrapping_add(
                steps.iter().fold(1i32, |hash, step| {
                    hash.wrapping_mul(31).wrapping_add(step.hash_code())
                }),
            ),
        }
    }
}

/// A record's: `Linear[endPosition=(0.5, -60.0, 0.5)]`, `Stepped[endPosition=…, steps=[…]]`.
impl fmt::Display for PositionPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PositionPath::Linear { end_position } => {
                write!(f, "Linear[endPosition={end_position}]")
            }
            PositionPath::Stepped {
                end_position,
                steps,
            } => {
                write!(f, "Stepped[endPosition={end_position}, steps=[")?;
                for (index, step) in steps.iter().enumerate() {
                    if index > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{step}")?;
                }
                write!(f, "]]")
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::PositionPath;
    use crate::golden::{Decode, Encode, Golden, elements};
    use crate::net::minecraft::network::codec::stream_codec::StreamCodec;
    use crate::net::minecraft::world::entity::position_step::PositionStep;
    use crate::net::minecraft::world::phys::vec3::Vec3;

    /// A `PositionPath.Linear` declared as itself: its position.
    #[derive(Debug, Clone)]
    struct Linear(PositionPath);

    impl Decode for Linear {
        fn decode(value: &Value) -> Self {
            let [end_position] = elements(value);
            Linear(PositionPath::of(Vec3::decode(end_position)))
        }
    }

    impl Encode for Linear {
        fn encode(&self) -> Value {
            Value::from(vec![self.0.end_position().encode()])
        }
    }

    /// A `PositionPath.Stepped` declared as itself: its end and its steps.
    #[derive(Debug, Clone)]
    struct Stepped(PositionPath);

    impl Decode for Stepped {
        fn decode(value: &Value) -> Self {
            let [end_position, steps] = elements(value);
            Stepped(PositionPath::Stepped {
                end_position: Vec3::decode(end_position),
                steps: Vec::<PositionStep>::decode(steps),
            })
        }
    }

    impl Encode for Stepped {
        fn encode(&self) -> Value {
            match &self.0 {
                PositionPath::Stepped {
                    end_position,
                    steps,
                } => Value::from(vec![end_position.encode(), steps.encode()]),
                PositionPath::Linear { .. } => panic!("a linear path isn't a Stepped"),
            }
        }
    }

    /// A `PositionPath` declared as the interface: `[class, fields]`, by its record's simple name.
    impl Decode for PositionPath {
        fn decode(value: &Value) -> Self {
            let [class, fields] = elements(value);
            match class.as_str() {
                Some("Linear") => Linear::decode(fields).0,
                Some("Stepped") => Stepped::decode(fields).0,
                _ => panic!("expected a PositionPath, got {value}"),
            }
        }
    }

    impl Encode for PositionPath {
        fn encode(&self) -> Value {
            match self {
                PositionPath::Linear { .. } => {
                    Value::from(vec![Value::from("Linear"), Linear(self.clone()).encode()])
                }
                PositionPath::Stepped { .. } => {
                    Value::from(vec![Value::from("Stepped"), Stepped(self.clone()).encode()])
                }
            }
        }
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.world.entity.PositionPath");
        golden.check_codec("STREAM_CODEC", || PositionPath::STREAM_CODEC);
        golden.check(
            "of(Lnet/minecraft/world/phys/Vec3;)Lnet/minecraft/world/entity/PositionPath;",
            PositionPath::of,
        );
        golden.pending(
            "stepped(Ljava/util/List;)Lnet/minecraft/world/entity/PositionPath;",
            "takes a list of steps, which the harness has no generator for as an argument here",
        );
        golden.finish();
    }

    #[test]
    fn linear_matches_vanilla() {
        let golden = Golden::load("net.minecraft.world.entity.PositionPath$Linear");
        golden.check_codec("STREAM_CODEC", || {
            PositionPath::LINEAR_STREAM_CODEC.map(Linear, |linear: &Linear| linear.0.clone())
        });
        golden.check(
            "<init>(Lnet/minecraft/world/phys/Vec3;)V",
            |position: Vec3| Linear(PositionPath::of(position)),
        );
        golden.check_method(
            "endPosition()Lnet/minecraft/world/phys/Vec3;",
            |this: &mut Linear| this.0.end_position(),
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut Linear, other: Linear| this.0 == other.0,
        );
        golden.check_method("hashCode()I", |this: &mut Linear| this.0.hash_code());
        golden.check_method("toString()Ljava/lang/String;", |this: &mut Linear| {
            this.0.to_string()
        });
        golden.pending(
            "type()Lnet/minecraft/world/entity/PositionPath$Type;",
            "the harness has no encoder for the kinds",
        );
        golden.finish();
    }

    #[test]
    fn stepped_matches_vanilla() {
        let golden = Golden::load("net.minecraft.world.entity.PositionPath$Stepped");
        golden.check_codec("STREAM_CODEC", || {
            PositionPath::STEPPED_STREAM_CODEC.map(Stepped, |stepped: &Stepped| stepped.0.clone())
        });
        golden.check("<init>(Ljava/util/List;)V", |steps: Vec<PositionStep>| {
            Stepped(PositionPath::stepped(steps))
        });
        golden.check(
            "<init>(Lnet/minecraft/world/phys/Vec3;Ljava/util/List;)V",
            |end_position: Vec3, steps: Vec<PositionStep>| {
                Stepped(PositionPath::Stepped {
                    end_position,
                    steps,
                })
            },
        );
        golden.check_method(
            "endPosition()Lnet/minecraft/world/phys/Vec3;",
            |this: &mut Stepped| this.0.end_position(),
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut Stepped, other: Stepped| this.0 == other.0,
        );
        golden.check_method("hashCode()I", |this: &mut Stepped| this.0.hash_code());
        golden.check_method(
            "steps()Ljava/util/List;",
            |this: &mut Stepped| match &this.0 {
                PositionPath::Stepped { steps, .. } => steps.clone(),
                PositionPath::Linear { .. } => unreachable!(),
            },
        );
        golden.check_method("toString()Ljava/lang/String;", |this: &mut Stepped| {
            this.0.to_string()
        });
        golden.pending(
            "type()Lnet/minecraft/world/entity/PositionPath$Type;",
            "the harness has no encoder for the kinds",
        );
        golden.finish();
    }
}
