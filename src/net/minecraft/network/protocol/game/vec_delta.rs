//! net/minecraft/network/protocol/game/VecDelta.java
//!
//! An entity's move from where the client last had it (`VecDeltaCodec`'s base), in 4096ths of a
//! block: one delta, or steps, each with how many ticks it takes. Java's sealed interface and its
//! two records are an enum here, and `Stepped.tryEncode` is `try_encode_stepped`.

use std::fmt;

use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
use crate::net::minecraft::network::protocol::game::vec_delta_codec::VecDeltaCodec;
use crate::net::minecraft::world::entity::position_path::PositionPath;
use crate::net::minecraft::world::entity::position_step::PositionStep;

/// A move from the codec's base.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum VecDelta {
    /// `VecDelta.Linear`: straight to the base plus the delta.
    Linear { xa: i16, ya: i16, za: i16 },
    /// `VecDelta.Stepped`: each step's delta from the one before (the first from the base).
    Stepped(Vec<DeltaStep>),
}

/// `VecDelta.Stepped.DeltaStep`: a step's delta and the ticks it takes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DeltaStep {
    pub xa: i16,
    pub ya: i16,
    pub za: i16,
    pub ticks: i32,
}

impl VecDelta {
    pub const ZERO: VecDelta = VecDelta::Linear {
        xa: 0,
        ya: 0,
        za: 0,
    };
    /// `Stepped.MIN_BYTES_PER_STEP`: a step's VarInt of one byte and its three shorts.
    const MIN_BYTES_PER_STEP: i32 = 7;

    pub fn step_count(&self) -> i32 {
        match self {
            VecDelta::Linear { .. } => 0,
            VecDelta::Stepped(steps) => steps.len() as i32,
        }
    }

    pub fn has_delta_x(&self) -> bool {
        match self {
            VecDelta::Linear { xa, .. } => *xa != 0,
            VecDelta::Stepped(steps) => steps.iter().any(|step| step.xa != 0),
        }
    }

    pub fn has_delta_z(&self) -> bool {
        match self {
            VecDelta::Linear { za, .. } => *za != 0,
            VecDelta::Stepped(steps) => steps.iter().any(|step| step.za != 0),
        }
    }

    /// Where the move goes from the codec's base, which it leaves as it was.
    pub fn decode(&self, position_codec: &mut VecDeltaCodec) -> PositionPath {
        match self {
            VecDelta::Linear { xa, ya, za } => PositionPath::of(position_codec.decode(
                i64::from(*xa),
                i64::from(*ya),
                i64::from(*za),
            )),
            VecDelta::Stepped(steps) => {
                if steps.is_empty() {
                    return PositionPath::of(position_codec.get_base());
                }

                let original_base = position_codec.get_base();
                let mut output = Vec::with_capacity(steps.len());
                for step in steps {
                    let pos = position_codec.decode(
                        i64::from(step.xa),
                        i64::from(step.ya),
                        i64::from(step.za),
                    );
                    output.push(PositionStep::new(pos, step.ticks));
                    position_codec.set_base(pos);
                }
                position_codec.set_base(original_base);
                PositionPath::stepped(output)
            }
        }
    }

    /// A move of so many steps, a linear one for none, which a move packet's properties carry.
    /// Panics where Java throws `DecoderException`, for more steps than the bytes left could hold.
    pub fn read(input: &mut FriendlyByteBuf, step_count: i32) -> VecDelta {
        if step_count <= 0 {
            let xa = input.read_short();
            let ya = input.read_short();
            let za = input.read_short();
            return VecDelta::Linear { xa, ya, za };
        }

        let max_steps = input.readable_bytes() / VecDelta::MIN_BYTES_PER_STEP;
        if step_count > max_steps {
            panic!(
                "DecoderException: VecDelta with size {step_count} is bigger than allowed {max_steps}"
            );
        }

        let mut steps = Vec::with_capacity(step_count as usize);
        for _ in 0..step_count {
            let ticks = input.read_var_int();
            let xa = input.read_short();
            let ya = input.read_short();
            let za = input.read_short();
            steps.push(DeltaStep { xa, ya, za, ticks });
        }
        VecDelta::Stepped(steps)
    }

    pub fn write(output: &mut FriendlyByteBuf, delta: &VecDelta) {
        match delta {
            VecDelta::Linear { xa, ya, za } => {
                output.write_short(i32::from(*xa));
                output.write_short(i32::from(*ya));
                output.write_short(i32::from(*za));
            }
            VecDelta::Stepped(steps) => {
                for step in steps {
                    output.write_var_int(step.ticks);
                    output.write_short(i32::from(step.xa));
                    output.write_short(i32::from(step.ya));
                    output.write_short(i32::from(step.za));
                }
            }
        }
    }

    /// A record's hash: a linear move's shorts', or the steps' list's.
    pub fn hash_code(&self) -> i32 {
        match self {
            VecDelta::Linear { xa, ya, za } => {
                (i32::from(*xa).wrapping_mul(31) + i32::from(*ya)).wrapping_mul(31) + i32::from(*za)
            }
            VecDelta::Stepped(steps) => steps.iter().fold(1i32, |hash, step| {
                hash.wrapping_mul(31).wrapping_add(step.hash_code())
            }),
        }
    }
}

/// `Stepped.tryEncode`: the steps' deltas, each from the step before, or none when one is too big
/// for a short. The codec's base is as it was after.
pub fn try_encode_stepped(
    position_codec: &mut VecDeltaCodec,
    steps: &[PositionStep],
) -> Option<Vec<DeltaStep>> {
    let original_base = position_codec.get_base();
    let mut output = Vec::with_capacity(steps.len());
    for step in steps {
        let pos = step.position();
        let xa = position_codec.encode_x(&pos);
        let ya = position_codec.encode_y(&pos);
        let za = position_codec.encode_z(&pos);
        if VecDeltaCodec::is_delta_too_big(xa, ya, za) {
            position_codec.set_base(original_base);
            return None;
        }

        output.push(DeltaStep {
            xa: xa as i16,
            ya: ya as i16,
            za: za as i16,
            ticks: step.tick_offset(),
        });
        position_codec.set_base(pos);
    }
    position_codec.set_base(original_base);
    Some(output)
}

impl DeltaStep {
    /// A record's hash, of its components in order.
    pub fn hash_code(&self) -> i32 {
        ((i32::from(self.xa).wrapping_mul(31) + i32::from(self.ya)).wrapping_mul(31)
            + i32::from(self.za))
        .wrapping_mul(31)
        .wrapping_add(self.ticks)
    }
}

/// A record's: `Linear[xa=1, ya=0, za=-2]`, `Stepped[steps=[DeltaStep[xa=0, ya=0, za=0, ticks=3]]]`.
impl fmt::Display for VecDelta {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VecDelta::Linear { xa, ya, za } => write!(f, "Linear[xa={xa}, ya={ya}, za={za}]"),
            VecDelta::Stepped(steps) => {
                write!(f, "Stepped[steps=[")?;
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

impl fmt::Display for DeltaStep {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "DeltaStep[xa={}, ya={}, za={}, ticks={}]",
            self.xa, self.ya, self.za, self.ticks
        )
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use serde_json::Value;

    use super::{DeltaStep, VecDelta, try_encode_stepped};
    use crate::golden::{Decode, Encode, Golden, elements, encode_fields};
    use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;
    use crate::net::minecraft::network::protocol::game::vec_delta_codec::VecDeltaCodec;
    use crate::net::minecraft::world::entity::position_step::PositionStep;

    encode_fields!(DeltaStep {
        xa: i16,
        ya: i16,
        za: i16,
        ticks: i32,
    });

    /// A `VecDelta.Linear` declared as itself: its components.
    #[derive(Debug)]
    pub(crate) struct Linear(VecDelta);

    impl Decode for Linear {
        fn decode(value: &Value) -> Self {
            let [xa, ya, za] = elements(value);
            Linear(VecDelta::Linear {
                xa: i16::decode(xa),
                ya: i16::decode(ya),
                za: i16::decode(za),
            })
        }
    }

    impl Encode for Linear {
        fn encode(&self) -> Value {
            match &self.0 {
                VecDelta::Linear { xa, ya, za } => {
                    Value::from(vec![xa.encode(), ya.encode(), za.encode()])
                }
                VecDelta::Stepped(_) => panic!("a stepped delta isn't a Linear"),
            }
        }
    }

    /// A `VecDelta.Stepped` declared as itself: its one component, the steps.
    #[derive(Debug)]
    pub(crate) struct Stepped(VecDelta);

    impl Decode for Stepped {
        fn decode(value: &Value) -> Self {
            let [steps] = elements(value);
            Stepped(VecDelta::Stepped(Vec::<DeltaStep>::decode(steps)))
        }
    }

    impl Encode for Stepped {
        fn encode(&self) -> Value {
            match &self.0 {
                VecDelta::Stepped(steps) => Value::from(vec![steps.encode()]),
                VecDelta::Linear { .. } => panic!("a linear delta isn't a Stepped"),
            }
        }
    }

    /// A `VecDelta` declared as the interface: `[class, fields]`, by its record's simple name.
    impl Decode for VecDelta {
        fn decode(value: &Value) -> Self {
            let [class, fields] = elements(value);
            match class.as_str() {
                Some("Linear") => Linear::decode(fields).0,
                Some("Stepped") => Stepped::decode(fields).0,
                _ => panic!("expected a VecDelta, got {value}"),
            }
        }
    }

    impl Encode for VecDelta {
        fn encode(&self) -> Value {
            match self {
                VecDelta::Linear { .. } => {
                    Value::from(vec![Value::from("Linear"), Linear(self.clone()).encode()])
                }
                VecDelta::Stepped(_) => {
                    Value::from(vec![Value::from("Stepped"), Stepped(self.clone()).encode()])
                }
            }
        }
    }

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.network.protocol.game.VecDelta");
        golden.field("ZERO", VecDelta::ZERO);
        golden.check_mut(
            "read(Lnet/minecraft/network/FriendlyByteBuf;I)Lnet/minecraft/network/protocol/game/VecDelta;",
            |input: &mut FriendlyByteBuf, step_count: &mut i32| VecDelta::read(input, *step_count),
        );
        golden.check_mut(
            "write(Lnet/minecraft/network/FriendlyByteBuf;Lnet/minecraft/network/protocol/game/VecDelta;)V",
            |output: &mut FriendlyByteBuf, delta: &mut VecDelta| VecDelta::write(output, delta),
        );
        golden.finish();
    }

    #[test]
    fn linear_matches_vanilla() {
        let golden = Golden::load("net.minecraft.network.protocol.game.VecDelta$Linear");
        golden.check("<init>(SSS)V", |xa: i16, ya: i16, za: i16| {
            Linear(VecDelta::Linear { xa, ya, za })
        });
        golden.check_method(
            "decode(Lnet/minecraft/network/protocol/game/VecDeltaCodec;)Lnet/minecraft/world/entity/PositionPath;",
            |this: &mut Linear, mut codec: VecDeltaCodec| this.0.decode(&mut codec),
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut Linear, other: Linear| this.0 == other.0,
        );
        golden.check_method("hasDeltaX()Z", |this: &mut Linear| this.0.has_delta_x());
        golden.check_method("hasDeltaZ()Z", |this: &mut Linear| this.0.has_delta_z());
        golden.check_method("hashCode()I", |this: &mut Linear| this.0.hash_code());
        golden.check_method("stepCount()I", |this: &mut Linear| this.0.step_count());
        golden.check_method("toString()Ljava/lang/String;", |this: &mut Linear| {
            this.0.to_string()
        });
        let component = |this: &Linear, index: usize| match this.0 {
            VecDelta::Linear { xa, ya, za } => [xa, ya, za][index],
            VecDelta::Stepped(_) => unreachable!(),
        };
        golden.check_method("xa()S", |this: &mut Linear| component(this, 0));
        golden.check_method("ya()S", |this: &mut Linear| component(this, 1));
        golden.check_method("za()S", |this: &mut Linear| component(this, 2));
        golden.finish();
    }

    #[test]
    fn stepped_matches_vanilla() {
        let golden = Golden::load("net.minecraft.network.protocol.game.VecDelta$Stepped");
        golden.pending(
            "<init>(Ljava/util/List;)V",
            "takes a list of steps, which the harness has no generator for",
        );
        golden.check_method(
            "decode(Lnet/minecraft/network/protocol/game/VecDeltaCodec;)Lnet/minecraft/world/entity/PositionPath;",
            |this: &mut Stepped, mut codec: VecDeltaCodec| this.0.decode(&mut codec),
        );
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut Stepped, other: Stepped| this.0 == other.0,
        );
        golden.check_method("hasDeltaX()Z", |this: &mut Stepped| this.0.has_delta_x());
        golden.check_method("hasDeltaZ()Z", |this: &mut Stepped| this.0.has_delta_z());
        golden.check_method("hashCode()I", |this: &mut Stepped| this.0.hash_code());
        golden.check_method("stepCount()I", |this: &mut Stepped| this.0.step_count());
        golden.check_method(
            "steps()Ljava/util/List;",
            |this: &mut Stepped| match &this.0 {
                VecDelta::Stepped(steps) => steps.clone(),
                VecDelta::Linear { .. } => unreachable!(),
            },
        );
        golden.check_method("toString()Ljava/lang/String;", |this: &mut Stepped| {
            this.0.to_string()
        });
        golden.check(
            "tryEncode(Lnet/minecraft/network/protocol/game/VecDeltaCodec;Ljava/util/List;)Lnet/minecraft/network/protocol/game/VecDelta$Stepped;",
            |mut codec: VecDeltaCodec, steps: Vec<PositionStep>| {
                try_encode_stepped(&mut codec, &steps).map(|steps| Stepped(VecDelta::Stepped(steps)))
            },
        );
        golden.finish();
    }

    #[test]
    fn delta_step_matches_vanilla() {
        let golden = Golden::load("net.minecraft.network.protocol.game.VecDelta$Stepped$DeltaStep");
        golden.check("<init>(SSSI)V", |xa: i16, ya: i16, za: i16, ticks: i32| {
            DeltaStep { xa, ya, za, ticks }
        });
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut DeltaStep, other: DeltaStep| *this == other,
        );
        golden.check_method("hashCode()I", |this: &mut DeltaStep| this.hash_code());
        golden.check_method("ticks()I", |this: &mut DeltaStep| this.ticks);
        golden.check_method("toString()Ljava/lang/String;", |this: &mut DeltaStep| {
            this.to_string()
        });
        golden.check_method("xa()S", |this: &mut DeltaStep| this.xa);
        golden.check_method("ya()S", |this: &mut DeltaStep| this.ya);
        golden.check_method("za()S", |this: &mut DeltaStep| this.za);
        golden.finish();
    }
}
