//! net/minecraft/world/level/block/entity/BlockEntityTypes.java
//!
//! The types, in the order `BlockEntityTypes` registers them in `BuiltInRegistries.BLOCK_ENTITY_TYPE`,
//! which numbers them, each with its valid blocks and its class (`register`'s factory, which
//! `BlockEntityType::create` makes the block entity of). Not ported yet: `OP_ONLY_CUSTOM_DATA`,
//! which comes with the server's checks of what players send.

use std::sync::LazyLock;

use crate::net::minecraft::world::level::block::block::Block;
use crate::net::minecraft::world::level::block::blocks::Blocks;
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

/// `register`'s valid blocks of each type, by id, in `BlockEntityTypes`' order.
fn valid_blocks() -> Vec<Vec<Block>> {
    vec![
        vec![Blocks::FURNACE],
        Blocks::COPPER_CHEST
            .as_list()
            .into_iter()
            .copied()
            .chain(std::iter::once(Blocks::CHEST))
            .collect(),
        vec![Blocks::TRAPPED_CHEST],
        vec![Blocks::ENDER_CHEST],
        vec![Blocks::JUKEBOX],
        vec![Blocks::DISPENSER],
        vec![Blocks::DROPPER],
        vec![
            Blocks::OAK_SIGN,
            Blocks::SPRUCE_SIGN,
            Blocks::BIRCH_SIGN,
            Blocks::ACACIA_SIGN,
            Blocks::CHERRY_SIGN,
            Blocks::JUNGLE_SIGN,
            Blocks::DARK_OAK_SIGN,
            Blocks::PALE_OAK_SIGN,
            Blocks::OAK_WALL_SIGN,
            Blocks::SPRUCE_WALL_SIGN,
            Blocks::BIRCH_WALL_SIGN,
            Blocks::ACACIA_WALL_SIGN,
            Blocks::CHERRY_WALL_SIGN,
            Blocks::JUNGLE_WALL_SIGN,
            Blocks::DARK_OAK_WALL_SIGN,
            Blocks::PALE_OAK_WALL_SIGN,
            Blocks::CRIMSON_SIGN,
            Blocks::CRIMSON_WALL_SIGN,
            Blocks::WARPED_SIGN,
            Blocks::WARPED_WALL_SIGN,
            Blocks::MANGROVE_SIGN,
            Blocks::MANGROVE_WALL_SIGN,
            Blocks::BAMBOO_SIGN,
            Blocks::BAMBOO_WALL_SIGN,
            Blocks::POPLAR_SIGN,
            Blocks::POPLAR_WALL_SIGN,
        ],
        vec![
            Blocks::OAK_HANGING_SIGN,
            Blocks::SPRUCE_HANGING_SIGN,
            Blocks::BIRCH_HANGING_SIGN,
            Blocks::ACACIA_HANGING_SIGN,
            Blocks::CHERRY_HANGING_SIGN,
            Blocks::JUNGLE_HANGING_SIGN,
            Blocks::DARK_OAK_HANGING_SIGN,
            Blocks::PALE_OAK_HANGING_SIGN,
            Blocks::CRIMSON_HANGING_SIGN,
            Blocks::WARPED_HANGING_SIGN,
            Blocks::MANGROVE_HANGING_SIGN,
            Blocks::BAMBOO_HANGING_SIGN,
            Blocks::OAK_WALL_HANGING_SIGN,
            Blocks::SPRUCE_WALL_HANGING_SIGN,
            Blocks::BIRCH_WALL_HANGING_SIGN,
            Blocks::ACACIA_WALL_HANGING_SIGN,
            Blocks::CHERRY_WALL_HANGING_SIGN,
            Blocks::JUNGLE_WALL_HANGING_SIGN,
            Blocks::DARK_OAK_WALL_HANGING_SIGN,
            Blocks::PALE_OAK_WALL_HANGING_SIGN,
            Blocks::CRIMSON_WALL_HANGING_SIGN,
            Blocks::WARPED_WALL_HANGING_SIGN,
            Blocks::MANGROVE_WALL_HANGING_SIGN,
            Blocks::BAMBOO_WALL_HANGING_SIGN,
            Blocks::POPLAR_HANGING_SIGN,
            Blocks::POPLAR_WALL_HANGING_SIGN,
        ],
        vec![Blocks::SPAWNER],
        vec![Blocks::CREAKING_HEART],
        vec![Blocks::MOVING_PISTON],
        vec![Blocks::BREWING_STAND],
        vec![Blocks::ENCHANTING_TABLE],
        vec![Blocks::END_PORTAL],
        vec![Blocks::BEACON],
        vec![
            Blocks::SKELETON_SKULL,
            Blocks::SKELETON_WALL_SKULL,
            Blocks::CREEPER_HEAD,
            Blocks::CREEPER_WALL_HEAD,
            Blocks::DRAGON_HEAD,
            Blocks::DRAGON_WALL_HEAD,
            Blocks::ZOMBIE_HEAD,
            Blocks::ZOMBIE_WALL_HEAD,
            Blocks::WITHER_SKELETON_SKULL,
            Blocks::WITHER_SKELETON_WALL_SKULL,
            Blocks::PLAYER_HEAD,
            Blocks::PLAYER_WALL_HEAD,
            Blocks::PIGLIN_HEAD,
            Blocks::PIGLIN_WALL_HEAD,
        ],
        vec![Blocks::DAYLIGHT_DETECTOR],
        vec![Blocks::HOPPER],
        vec![Blocks::COMPARATOR],
        Blocks::BANNER
            .as_list()
            .iter()
            .copied()
            .chain(Blocks::WALL_BANNER.as_list().iter().copied())
            .collect(),
        vec![Blocks::STRUCTURE_BLOCK],
        vec![Blocks::END_GATEWAY],
        vec![
            Blocks::COMMAND_BLOCK,
            Blocks::CHAIN_COMMAND_BLOCK,
            Blocks::REPEATING_COMMAND_BLOCK,
        ],
        Blocks::DYED_SHULKER_BOX
            .as_list()
            .iter()
            .copied()
            .chain(std::iter::once(Blocks::SHULKER_BOX))
            .collect(),
        vec![Blocks::CONDUIT],
        vec![Blocks::BARREL],
        vec![Blocks::SMOKER],
        vec![Blocks::BLAST_FURNACE],
        vec![Blocks::LECTERN],
        vec![Blocks::BELL],
        vec![Blocks::JIGSAW],
        vec![Blocks::CAMPFIRE, Blocks::SOUL_CAMPFIRE],
        vec![Blocks::BEE_NEST, Blocks::BEEHIVE],
        vec![Blocks::SCULK_SENSOR],
        vec![Blocks::CALIBRATED_SCULK_SENSOR],
        vec![Blocks::SCULK_CATALYST],
        vec![Blocks::SCULK_SHRIEKER],
        vec![Blocks::CHISELED_BOOKSHELF],
        vec![
            Blocks::ACACIA_SHELF,
            Blocks::BAMBOO_SHELF,
            Blocks::BIRCH_SHELF,
            Blocks::CHERRY_SHELF,
            Blocks::CRIMSON_SHELF,
            Blocks::DARK_OAK_SHELF,
            Blocks::JUNGLE_SHELF,
            Blocks::MANGROVE_SHELF,
            Blocks::OAK_SHELF,
            Blocks::PALE_OAK_SHELF,
            Blocks::SPRUCE_SHELF,
            Blocks::WARPED_SHELF,
            Blocks::POPLAR_SHELF,
        ],
        vec![Blocks::SUSPICIOUS_SAND, Blocks::SUSPICIOUS_GRAVEL],
        vec![Blocks::DECORATED_POT],
        vec![Blocks::CRAFTER],
        vec![Blocks::TRIAL_SPAWNER],
        vec![Blocks::VAULT],
        vec![Blocks::TEST_BLOCK],
        vec![Blocks::TEST_INSTANCE_BLOCK],
        Blocks::COPPER_GOLEM_STATUE
            .as_list()
            .into_iter()
            .copied()
            .collect(),
        vec![Blocks::POTENT_SULFUR],
    ]
}

/// Each type's class, `BlockEntitySupplier`'s, by id: what `getNameForReporting` names.
pub(crate) const CLASS_NAMES: [&str; 49] = [
    "net.minecraft.world.level.block.entity.FurnaceBlockEntity",
    "net.minecraft.world.level.block.entity.ChestBlockEntity",
    "net.minecraft.world.level.block.entity.TrappedChestBlockEntity",
    "net.minecraft.world.level.block.entity.EnderChestBlockEntity",
    "net.minecraft.world.level.block.entity.JukeboxBlockEntity",
    "net.minecraft.world.level.block.entity.DispenserBlockEntity",
    "net.minecraft.world.level.block.entity.DropperBlockEntity",
    "net.minecraft.world.level.block.entity.SignBlockEntity",
    "net.minecraft.world.level.block.entity.HangingSignBlockEntity",
    "net.minecraft.world.level.block.entity.SpawnerBlockEntity",
    "net.minecraft.world.level.block.entity.CreakingHeartBlockEntity",
    "net.minecraft.world.level.block.piston.PistonMovingBlockEntity",
    "net.minecraft.world.level.block.entity.BrewingStandBlockEntity",
    "net.minecraft.world.level.block.entity.EnchantingTableBlockEntity",
    "net.minecraft.world.level.block.entity.TheEndPortalBlockEntity",
    "net.minecraft.world.level.block.entity.BeaconBlockEntity",
    "net.minecraft.world.level.block.entity.SkullBlockEntity",
    "net.minecraft.world.level.block.entity.DaylightDetectorBlockEntity",
    "net.minecraft.world.level.block.entity.HopperBlockEntity",
    "net.minecraft.world.level.block.entity.ComparatorBlockEntity",
    "net.minecraft.world.level.block.entity.BannerBlockEntity",
    "net.minecraft.world.level.block.entity.StructureBlockEntity",
    "net.minecraft.world.level.block.entity.TheEndGatewayBlockEntity",
    "net.minecraft.world.level.block.entity.CommandBlockEntity",
    "net.minecraft.world.level.block.entity.ShulkerBoxBlockEntity",
    "net.minecraft.world.level.block.entity.ConduitBlockEntity",
    "net.minecraft.world.level.block.entity.BarrelBlockEntity",
    "net.minecraft.world.level.block.entity.SmokerBlockEntity",
    "net.minecraft.world.level.block.entity.BlastFurnaceBlockEntity",
    "net.minecraft.world.level.block.entity.LecternBlockEntity",
    "net.minecraft.world.level.block.entity.BellBlockEntity",
    "net.minecraft.world.level.block.entity.JigsawBlockEntity",
    "net.minecraft.world.level.block.entity.CampfireBlockEntity",
    "net.minecraft.world.level.block.entity.BeehiveBlockEntity",
    "net.minecraft.world.level.block.entity.SculkSensorBlockEntity",
    "net.minecraft.world.level.block.entity.CalibratedSculkSensorBlockEntity",
    "net.minecraft.world.level.block.entity.SculkCatalystBlockEntity",
    "net.minecraft.world.level.block.entity.SculkShriekerBlockEntity",
    "net.minecraft.world.level.block.entity.ChiseledBookShelfBlockEntity",
    "net.minecraft.world.level.block.entity.ShelfBlockEntity",
    "net.minecraft.world.level.block.entity.BrushableBlockEntity",
    "net.minecraft.world.level.block.entity.DecoratedPotBlockEntity",
    "net.minecraft.world.level.block.entity.CrafterBlockEntity",
    "net.minecraft.world.level.block.entity.trialspawner.TrialSpawnerBlockEntity",
    "net.minecraft.world.level.block.entity.vault.VaultBlockEntity",
    "net.minecraft.world.level.block.entity.TestBlockEntity",
    "net.minecraft.world.level.block.entity.TestInstanceBlockEntity",
    "net.minecraft.world.level.block.entity.CopperGolemStatueBlockEntity",
    "net.minecraft.world.level.block.entity.PotentSulfurBlockEntity",
];

/// Each type's valid blocks, made on first use (they need the block registry).
pub(crate) static VALID_BLOCKS: LazyLock<Vec<Vec<Block>>> = LazyLock::new(valid_blocks);

/// Each block's type, the one whose valid blocks have it, by block id: what its `newBlockEntity`
/// makes but for the moving piston, which makes none (the piston's block event makes it).
pub(crate) static BY_BLOCK: LazyLock<Vec<Option<BlockEntityType>>> = LazyLock::new(|| {
    let mut by_block =
        vec![None; crate::net::minecraft::world::level::block::blocks::registry().size()];
    for (id, blocks) in VALID_BLOCKS.iter().enumerate() {
        for block in blocks {
            by_block[block.0 as usize] = Some(BlockEntityType(id as u16));
        }
    }
    // MovingPistonBlock.newBlockEntity is null.
    by_block[Blocks::MOVING_PISTON.0 as usize] = None;
    by_block
});

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
