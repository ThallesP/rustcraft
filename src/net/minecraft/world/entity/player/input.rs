//! net/minecraft/world/entity/player/Input.java

use std::fmt;
use std::sync::LazyLock;

use crate::net::minecraft::network::codec::stream_codec::{self, StaticStreamCodec};
use crate::net::minecraft::network::friendly_byte_buf::FriendlyByteBuf;

// The keys as one byte of flags, in the components' order.
static STREAM_CODEC: StaticStreamCodec<FriendlyByteBuf, Input> = LazyLock::new(|| {
    Box::new(stream_codec::of(
        |output: &mut FriendlyByteBuf, value: &Input| {
            let mut flags: i8 = 0;
            flags |= if value.forward {
                Input::FLAG_FORWARD
            } else {
                0
            };
            flags |= if value.backward {
                Input::FLAG_BACKWARD
            } else {
                0
            };
            flags |= if value.left { Input::FLAG_LEFT } else { 0 };
            flags |= if value.right { Input::FLAG_RIGHT } else { 0 };
            flags |= if value.jump { Input::FLAG_JUMP } else { 0 };
            flags |= if value.shift { Input::FLAG_SHIFT } else { 0 };
            flags |= if value.sprint { Input::FLAG_SPRINT } else { 0 };
            output.write_byte(i32::from(flags));
        },
        |input: &mut FriendlyByteBuf| {
            let flags = input.read_byte();
            Input {
                forward: flags & Input::FLAG_FORWARD != 0,
                backward: flags & Input::FLAG_BACKWARD != 0,
                left: flags & Input::FLAG_LEFT != 0,
                right: flags & Input::FLAG_RIGHT != 0,
                jump: flags & Input::FLAG_JUMP != 0,
                shift: flags & Input::FLAG_SHIFT != 0,
                sprint: flags & Input::FLAG_SPRINT != 0,
            }
        },
    ))
});

/// The movement keys a player holds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct Input {
    pub forward: bool,
    pub backward: bool,
    pub left: bool,
    pub right: bool,
    pub jump: bool,
    pub shift: bool,
    pub sprint: bool,
}

impl Input {
    const FLAG_FORWARD: i8 = 1;
    const FLAG_BACKWARD: i8 = 2;
    const FLAG_LEFT: i8 = 4;
    const FLAG_RIGHT: i8 = 8;
    const FLAG_JUMP: i8 = 16;
    const FLAG_SHIFT: i8 = 32;
    const FLAG_SPRINT: i8 = 64;
    pub const STREAM_CODEC: &'static StaticStreamCodec<FriendlyByteBuf, Input> = &STREAM_CODEC;
    pub const EMPTY: Input = Input {
        forward: false,
        backward: false,
        left: false,
        right: false,
        jump: false,
        shift: false,
        sprint: false,
    };

    pub fn new(
        forward: bool,
        backward: bool,
        left: bool,
        right: bool,
        jump: bool,
        shift: bool,
        sprint: bool,
    ) -> Input {
        Input {
            forward,
            backward,
            left,
            right,
            jump,
            shift,
            sprint,
        }
    }

    /// A record's: each component's `Boolean.hashCode` (1231 or 1237), added to 31 times the
    /// ones before.
    pub fn hash_code(&self) -> i32 {
        [
            self.forward,
            self.backward,
            self.left,
            self.right,
            self.jump,
            self.shift,
            self.sprint,
        ]
        .into_iter()
        .fold(0i32, |hash, component| {
            hash.wrapping_mul(31)
                .wrapping_add(if component { 1231 } else { 1237 })
        })
    }
}

/// A record's: `Input[forward=true, backward=false, …]`.
impl fmt::Display for Input {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Input[forward={}, backward={}, left={}, right={}, jump={}, shift={}, sprint={}]",
            self.forward, self.backward, self.left, self.right, self.jump, self.shift, self.sprint
        )
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::Input;
    use crate::golden::{Golden, encode_fields};

    // A record: its components.
    encode_fields!(Input {
        forward: bool,
        backward: bool,
        left: bool,
        right: bool,
        jump: bool,
        shift: bool,
        sprint: bool,
    });

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.world.entity.player.Input");
        golden.field("EMPTY", Input::EMPTY);
        golden.check_codec("STREAM_CODEC", || Input::STREAM_CODEC);
        golden.check("<init>(ZZZZZZZ)V", Input::new);
        golden.check_method("forward()Z", |this: &mut Input| this.forward);
        golden.check_method("backward()Z", |this: &mut Input| this.backward);
        golden.check_method("left()Z", |this: &mut Input| this.left);
        golden.check_method("right()Z", |this: &mut Input| this.right);
        golden.check_method("jump()Z", |this: &mut Input| this.jump);
        golden.check_method("shift()Z", |this: &mut Input| this.shift);
        golden.check_method("sprint()Z", |this: &mut Input| this.sprint);
        golden.check_method(
            "equals(Ljava/lang/Object;)Z",
            |this: &mut Input, other: Input| *this == other,
        );
        golden.check_method("hashCode()I", |this: &mut Input| this.hash_code());
        golden.check_method("toString()Ljava/lang/String;", |this: &mut Input| {
            this.to_string()
        });
        golden.finish();
    }
}
