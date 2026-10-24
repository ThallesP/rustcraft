//! net/minecraft/commands/arguments/coordinates/Vec3Argument.java
//!
//! Only its errors, which the coordinates' parsers throw; the argument comes with commands.

use std::sync::LazyLock;

use crate::com::mojang::brigadier::exceptions::simple_command_exception_type::SimpleCommandExceptionType;
use crate::net::minecraft::network::chat::component::Component;

pub static ERROR_NOT_COMPLETE: LazyLock<SimpleCommandExceptionType> = LazyLock::new(|| {
    SimpleCommandExceptionType::new(Component::from(Component::translatable(
        "argument.pos3d.incomplete",
    )))
});
pub static ERROR_MIXED_TYPE: LazyLock<SimpleCommandExceptionType> = LazyLock::new(|| {
    SimpleCommandExceptionType::new(Component::from(Component::translatable(
        "argument.pos.mixed",
    )))
});
