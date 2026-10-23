//! net/minecraft/world/entity/Pose.java
//!
//! Not ported yet: `BY_ID`, `CODEC` and `STREAM_CODEC`, which come with the entity data packets.

/// How an entity holds itself, which decides its size.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Pose {
    STANDING,
    FALL_FLYING,
    SLEEPING,
    SWIMMING,
    SPIN_ATTACK,
    CROUCHING,
    LONG_JUMPING,
    DYING,
    CROAKING,
    USING_TONGUE,
    SITTING,
    ROARING,
    SNIFFING,
    EMERGING,
    DIGGING,
    SLIDING,
    SHOOTING,
    INHALING,
}

impl Pose {
    pub fn id(&self) -> i32 {
        *self as i32
    }

    /// The constant's name, `name()`.
    pub fn name(&self) -> &'static str {
        match self {
            Pose::STANDING => "STANDING",
            Pose::FALL_FLYING => "FALL_FLYING",
            Pose::SLEEPING => "SLEEPING",
            Pose::SWIMMING => "SWIMMING",
            Pose::SPIN_ATTACK => "SPIN_ATTACK",
            Pose::CROUCHING => "CROUCHING",
            Pose::LONG_JUMPING => "LONG_JUMPING",
            Pose::DYING => "DYING",
            Pose::CROAKING => "CROAKING",
            Pose::USING_TONGUE => "USING_TONGUE",
            Pose::SITTING => "SITTING",
            Pose::ROARING => "ROARING",
            Pose::SNIFFING => "SNIFFING",
            Pose::EMERGING => "EMERGING",
            Pose::DIGGING => "DIGGING",
            Pose::SLIDING => "SLIDING",
            Pose::SHOOTING => "SHOOTING",
            Pose::INHALING => "INHALING",
        }
    }
}
