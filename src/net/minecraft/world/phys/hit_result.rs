//! net/minecraft/world/phys/HitResult.java
//!
//! Not ported yet: `distanceTo`, which takes an `Entity`.
//!
//! `HitResult` is abstract. `BlockHitResult` holds it and derefs to it, and `getType` is its
//! own. `EntityHitResult` comes with entities, and with it how the game holds either kind of
//! hit (`Minecraft.hitResult`).

use crate::net::minecraft::world::phys::vec3::Vec3;

/// Where a pick ended: on a block, on an entity, or nowhere.
#[derive(Clone, Copy, Debug)]
pub struct HitResult {
    pub(super) location: Vec3,
}

impl HitResult {
    pub(super) fn new(location: Vec3) -> HitResult {
        HitResult { location }
    }

    pub fn get_location(&self) -> Vec3 {
        self.location
    }
}

/// `HitResult.Type`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Type {
    MISS,
    BLOCK,
    ENTITY,
}

impl Type {
    /// `values()`: the constants in declaration order, which is also their ordinal.
    pub const VALUES: [Type; 3] = [Type::MISS, Type::BLOCK, Type::ENTITY];

    pub fn values() -> [Type; 3] {
        Type::VALUES
    }

    /// Panics on anything but a constant's name, where Java throws `IllegalArgumentException`.
    pub fn value_of(name: &str) -> Type {
        match name {
            "MISS" => Type::MISS,
            "BLOCK" => Type::BLOCK,
            "ENTITY" => Type::ENTITY,
            _ => panic!("IllegalArgumentException: no HitResult.Type constant {name}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Type;
    use crate::golden::{Golden, encode_names};
    use crate::net::minecraft::world::phys::block_hit_result::BlockHitResult;

    encode_names!(Type {
        MISS,
        BLOCK,
        ENTITY
    });

    /// The receivers are `BlockHitResult`s, the only kind before entities.
    #[test]
    fn hit_result_matches_vanilla() {
        let golden = Golden::load("net.minecraft.world.phys.HitResult");
        golden.pending(
            "distanceTo(Lnet/minecraft/world/entity/Entity;)D",
            "waits for Entity",
        );
        golden.check_method(
            "getLocation()Lnet/minecraft/world/phys/Vec3;",
            |this: &mut BlockHitResult| this.get_location(),
        );
        golden.check_method(
            "getType()Lnet/minecraft/world/phys/HitResult$Type;",
            |this: &mut BlockHitResult| this.get_type(),
        );
        golden.finish();
    }

    #[test]
    fn type_matches_vanilla() {
        let golden = Golden::load("net.minecraft.world.phys.HitResult$Type");
        for constant in Type::VALUES {
            golden.field(&format!("{constant:?}"), constant);
        }
        golden.check(
            "values()[Lnet/minecraft/world/phys/HitResult$Type;",
            Type::values,
        );
        golden.check(
            "valueOf(Ljava/lang/String;)Lnet/minecraft/world/phys/HitResult$Type;",
            |name: String| Type::value_of(&name),
        );
        golden.finish();
    }
}
