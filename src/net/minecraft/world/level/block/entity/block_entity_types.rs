//! net/minecraft/world/level/block/entity/BlockEntityTypes.java
//!
//! The types, in the order `BlockEntityTypes` registers them in `BuiltInRegistries.BLOCK_ENTITY_TYPE`,
//! which numbers them. Not ported yet: each type's block entity class (`FurnaceBlockEntity::new`)
//! and valid blocks, and `OP_ONLY_CUSTOM_DATA`, which come with block entities.

use crate::net::minecraft::world::level::block::entity::block_entity_type::BlockEntityType;

pub struct BlockEntityTypes;

impl BlockEntityTypes {
    pub const FURNACE: BlockEntityType = BlockEntityType(0);
    pub const CHEST: BlockEntityType = BlockEntityType(1);
    pub const TRAPPED_CHEST: BlockEntityType = BlockEntityType(2);
    pub const ENDER_CHEST: BlockEntityType = BlockEntityType(3);
    pub const JUKEBOX: BlockEntityType = BlockEntityType(4);
    pub const DISPENSER: BlockEntityType = BlockEntityType(5);
    pub const DROPPER: BlockEntityType = BlockEntityType(6);
    pub const SIGN: BlockEntityType = BlockEntityType(7);
    pub const HANGING_SIGN: BlockEntityType = BlockEntityType(8);
    pub const MOB_SPAWNER: BlockEntityType = BlockEntityType(9);
    pub const CREAKING_HEART: BlockEntityType = BlockEntityType(10);
    pub const PISTON: BlockEntityType = BlockEntityType(11);
    pub const BREWING_STAND: BlockEntityType = BlockEntityType(12);
    pub const ENCHANTING_TABLE: BlockEntityType = BlockEntityType(13);
    pub const END_PORTAL: BlockEntityType = BlockEntityType(14);
    pub const BEACON: BlockEntityType = BlockEntityType(15);
    pub const SKULL: BlockEntityType = BlockEntityType(16);
    pub const DAYLIGHT_DETECTOR: BlockEntityType = BlockEntityType(17);
    pub const HOPPER: BlockEntityType = BlockEntityType(18);
    pub const COMPARATOR: BlockEntityType = BlockEntityType(19);
    pub const BANNER: BlockEntityType = BlockEntityType(20);
    pub const STRUCTURE_BLOCK: BlockEntityType = BlockEntityType(21);
    pub const END_GATEWAY: BlockEntityType = BlockEntityType(22);
    pub const COMMAND_BLOCK: BlockEntityType = BlockEntityType(23);
    pub const SHULKER_BOX: BlockEntityType = BlockEntityType(24);
    pub const CONDUIT: BlockEntityType = BlockEntityType(25);
    pub const BARREL: BlockEntityType = BlockEntityType(26);
    pub const SMOKER: BlockEntityType = BlockEntityType(27);
    pub const BLAST_FURNACE: BlockEntityType = BlockEntityType(28);
    pub const LECTERN: BlockEntityType = BlockEntityType(29);
    pub const BELL: BlockEntityType = BlockEntityType(30);
    pub const JIGSAW: BlockEntityType = BlockEntityType(31);
    pub const CAMPFIRE: BlockEntityType = BlockEntityType(32);
    pub const BEEHIVE: BlockEntityType = BlockEntityType(33);
    pub const SCULK_SENSOR: BlockEntityType = BlockEntityType(34);
    pub const CALIBRATED_SCULK_SENSOR: BlockEntityType = BlockEntityType(35);
    pub const SCULK_CATALYST: BlockEntityType = BlockEntityType(36);
    pub const SCULK_SHRIEKER: BlockEntityType = BlockEntityType(37);
    pub const CHISELED_BOOKSHELF: BlockEntityType = BlockEntityType(38);
    pub const SHELF: BlockEntityType = BlockEntityType(39);
    pub const BRUSHABLE_BLOCK: BlockEntityType = BlockEntityType(40);
    pub const DECORATED_POT: BlockEntityType = BlockEntityType(41);
    pub const CRAFTER: BlockEntityType = BlockEntityType(42);
    pub const TRIAL_SPAWNER: BlockEntityType = BlockEntityType(43);
    pub const VAULT: BlockEntityType = BlockEntityType(44);
    pub const TEST_BLOCK: BlockEntityType = BlockEntityType(45);
    pub const TEST_INSTANCE_BLOCK: BlockEntityType = BlockEntityType(46);
    pub const COPPER_GOLEM_STATUE: BlockEntityType = BlockEntityType(47);
    pub const POTENT_SULFUR: BlockEntityType = BlockEntityType(48);
}

/// Each type's name in the registry, by id.
pub(crate) const KEYS: [&str; 49] = [
    "minecraft:furnace",
    "minecraft:chest",
    "minecraft:trapped_chest",
    "minecraft:ender_chest",
    "minecraft:jukebox",
    "minecraft:dispenser",
    "minecraft:dropper",
    "minecraft:sign",
    "minecraft:hanging_sign",
    "minecraft:mob_spawner",
    "minecraft:creaking_heart",
    "minecraft:piston",
    "minecraft:brewing_stand",
    "minecraft:enchanting_table",
    "minecraft:end_portal",
    "minecraft:beacon",
    "minecraft:skull",
    "minecraft:daylight_detector",
    "minecraft:hopper",
    "minecraft:comparator",
    "minecraft:banner",
    "minecraft:structure_block",
    "minecraft:end_gateway",
    "minecraft:command_block",
    "minecraft:shulker_box",
    "minecraft:conduit",
    "minecraft:barrel",
    "minecraft:smoker",
    "minecraft:blast_furnace",
    "minecraft:lectern",
    "minecraft:bell",
    "minecraft:jigsaw",
    "minecraft:campfire",
    "minecraft:beehive",
    "minecraft:sculk_sensor",
    "minecraft:calibrated_sculk_sensor",
    "minecraft:sculk_catalyst",
    "minecraft:sculk_shrieker",
    "minecraft:chiseled_bookshelf",
    "minecraft:shelf",
    "minecraft:brushable_block",
    "minecraft:decorated_pot",
    "minecraft:crafter",
    "minecraft:trial_spawner",
    "minecraft:vault",
    "minecraft:test_block",
    "minecraft:test_instance_block",
    "minecraft:copper_golem_statue",
    "minecraft:potent_sulfur",
];

#[cfg(test)]
mod tests {
    use super::BlockEntityTypes;
    use crate::golden::Golden;
    use crate::net::minecraft::world::level::block::entity::block_entity_type::BlockEntityType;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.world.level.block.entity.BlockEntityTypes");
        let check = |name: &str, r#type: BlockEntityType| {
            golden.field(name, (r#type.id(), r#type.key().to_owned()));
        };
        check("FURNACE", BlockEntityTypes::FURNACE);
        check("CHEST", BlockEntityTypes::CHEST);
        check("TRAPPED_CHEST", BlockEntityTypes::TRAPPED_CHEST);
        check("ENDER_CHEST", BlockEntityTypes::ENDER_CHEST);
        check("JUKEBOX", BlockEntityTypes::JUKEBOX);
        check("DISPENSER", BlockEntityTypes::DISPENSER);
        check("DROPPER", BlockEntityTypes::DROPPER);
        check("SIGN", BlockEntityTypes::SIGN);
        check("HANGING_SIGN", BlockEntityTypes::HANGING_SIGN);
        check("MOB_SPAWNER", BlockEntityTypes::MOB_SPAWNER);
        check("CREAKING_HEART", BlockEntityTypes::CREAKING_HEART);
        check("PISTON", BlockEntityTypes::PISTON);
        check("BREWING_STAND", BlockEntityTypes::BREWING_STAND);
        check("ENCHANTING_TABLE", BlockEntityTypes::ENCHANTING_TABLE);
        check("END_PORTAL", BlockEntityTypes::END_PORTAL);
        check("BEACON", BlockEntityTypes::BEACON);
        check("SKULL", BlockEntityTypes::SKULL);
        check("DAYLIGHT_DETECTOR", BlockEntityTypes::DAYLIGHT_DETECTOR);
        check("HOPPER", BlockEntityTypes::HOPPER);
        check("COMPARATOR", BlockEntityTypes::COMPARATOR);
        check("BANNER", BlockEntityTypes::BANNER);
        check("STRUCTURE_BLOCK", BlockEntityTypes::STRUCTURE_BLOCK);
        check("END_GATEWAY", BlockEntityTypes::END_GATEWAY);
        check("COMMAND_BLOCK", BlockEntityTypes::COMMAND_BLOCK);
        check("SHULKER_BOX", BlockEntityTypes::SHULKER_BOX);
        check("CONDUIT", BlockEntityTypes::CONDUIT);
        check("BARREL", BlockEntityTypes::BARREL);
        check("SMOKER", BlockEntityTypes::SMOKER);
        check("BLAST_FURNACE", BlockEntityTypes::BLAST_FURNACE);
        check("LECTERN", BlockEntityTypes::LECTERN);
        check("BELL", BlockEntityTypes::BELL);
        check("JIGSAW", BlockEntityTypes::JIGSAW);
        check("CAMPFIRE", BlockEntityTypes::CAMPFIRE);
        check("BEEHIVE", BlockEntityTypes::BEEHIVE);
        check("SCULK_SENSOR", BlockEntityTypes::SCULK_SENSOR);
        check(
            "CALIBRATED_SCULK_SENSOR",
            BlockEntityTypes::CALIBRATED_SCULK_SENSOR,
        );
        check("SCULK_CATALYST", BlockEntityTypes::SCULK_CATALYST);
        check("SCULK_SHRIEKER", BlockEntityTypes::SCULK_SHRIEKER);
        check("CHISELED_BOOKSHELF", BlockEntityTypes::CHISELED_BOOKSHELF);
        check("SHELF", BlockEntityTypes::SHELF);
        check("BRUSHABLE_BLOCK", BlockEntityTypes::BRUSHABLE_BLOCK);
        check("DECORATED_POT", BlockEntityTypes::DECORATED_POT);
        check("CRAFTER", BlockEntityTypes::CRAFTER);
        check("TRIAL_SPAWNER", BlockEntityTypes::TRIAL_SPAWNER);
        check("VAULT", BlockEntityTypes::VAULT);
        check("TEST_BLOCK", BlockEntityTypes::TEST_BLOCK);
        check("TEST_INSTANCE_BLOCK", BlockEntityTypes::TEST_INSTANCE_BLOCK);
        check("COPPER_GOLEM_STATUE", BlockEntityTypes::COPPER_GOLEM_STATUE);
        check("POTENT_SULFUR", BlockEntityTypes::POTENT_SULFUR);
        golden.field("size", BlockEntityType::registry_size());
        golden.finish();
    }
}
