//! net/minecraft/references/BlockItemIds.java
//!
//! The ids of the blocks that have an item, and their items'. Each is made on first use, like
//! `Registries`' keys.

use std::sync::LazyLock;

use crate::net::minecraft::references::block_item_id::BlockItemId;
use crate::net::minecraft::world::level::block::color_collection::ColorCollection;
use crate::net::minecraft::world::level::block::weathering_copper_collection::{
    ByState, WeatheringCopperCollection,
};

static AIR: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("air"));
static DIRT: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("dirt"));
static PUMPKIN: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("pumpkin"));
static MELON: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("melon"));
static STONE: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("stone"));
static GRANITE: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("granite"));
static POLISHED_GRANITE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("polished_granite"));
static DIORITE: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("diorite"));
static POLISHED_DIORITE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("polished_diorite"));
static ANDESITE: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("andesite"));
static POLISHED_ANDESITE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("polished_andesite"));
static DEEPSLATE: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("deepslate"));
static COBBLED_DEEPSLATE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("cobbled_deepslate"));
static POLISHED_DEEPSLATE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("polished_deepslate"));
static CALCITE: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("calcite"));
static TUFF: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("tuff"));
static TUFF_SLAB: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("tuff_slab"));
static TUFF_STAIRS: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("tuff_stairs"));
static TUFF_WALL: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("tuff_wall"));
static CHISELED_TUFF: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("chiseled_tuff"));
static POLISHED_TUFF: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("polished_tuff"));
static POLISHED_TUFF_SLAB: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("polished_tuff_slab"));
static POLISHED_TUFF_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("polished_tuff_stairs"));
static POLISHED_TUFF_WALL: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("polished_tuff_wall"));
static TUFF_BRICKS: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("tuff_bricks"));
static TUFF_BRICK_SLAB: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("tuff_brick_slab"));
static TUFF_BRICK_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("tuff_brick_stairs"));
static TUFF_BRICK_WALL: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("tuff_brick_wall"));
static CHISELED_TUFF_BRICKS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("chiseled_tuff_bricks"));
static SULFUR: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("sulfur"));
static POTENT_SULFUR: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("potent_sulfur"));
static SULFUR_SLAB: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("sulfur_slab"));
static SULFUR_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("sulfur_stairs"));
static SULFUR_WALL: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("sulfur_wall"));
static POLISHED_SULFUR: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("polished_sulfur"));
static POLISHED_SULFUR_SLAB: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("polished_sulfur_slab"));
static POLISHED_SULFUR_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("polished_sulfur_stairs"));
static POLISHED_SULFUR_WALL: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("polished_sulfur_wall"));
static SULFUR_BRICKS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("sulfur_bricks"));
static SULFUR_BRICK_SLAB: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("sulfur_brick_slab"));
static SULFUR_BRICK_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("sulfur_brick_stairs"));
static SULFUR_BRICK_WALL: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("sulfur_brick_wall"));
static CHISELED_SULFUR: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("chiseled_sulfur"));
static CINNABAR: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("cinnabar"));
static CINNABAR_SLAB: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("cinnabar_slab"));
static CINNABAR_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("cinnabar_stairs"));
static CINNABAR_WALL: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("cinnabar_wall"));
static POLISHED_CINNABAR: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("polished_cinnabar"));
static POLISHED_CINNABAR_SLAB: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("polished_cinnabar_slab"));
static POLISHED_CINNABAR_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("polished_cinnabar_stairs"));
static POLISHED_CINNABAR_WALL: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("polished_cinnabar_wall"));
static CINNABAR_BRICKS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("cinnabar_bricks"));
static CINNABAR_BRICK_SLAB: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("cinnabar_brick_slab"));
static CINNABAR_BRICK_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("cinnabar_brick_stairs"));
static CINNABAR_BRICK_WALL: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("cinnabar_brick_wall"));
static CHISELED_CINNABAR: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("chiseled_cinnabar"));
static DRIPSTONE_BLOCK: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("dripstone_block"));
static GRASS_BLOCK: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("grass_block"));
static COARSE_DIRT: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("coarse_dirt"));
static PODZOL: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("podzol"));
static ROOTED_DIRT: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("rooted_dirt"));
static MUD: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("mud"));
static CRIMSON_NYLIUM: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("crimson_nylium"));
static WARPED_NYLIUM: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("warped_nylium"));
static COBBLESTONE: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("cobblestone"));
static OAK_PLANKS: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("oak_planks"));
static SPRUCE_PLANKS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("spruce_planks"));
static BIRCH_PLANKS: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("birch_planks"));
static JUNGLE_PLANKS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("jungle_planks"));
static ACACIA_PLANKS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("acacia_planks"));
static CHERRY_PLANKS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("cherry_planks"));
static DARK_OAK_PLANKS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("dark_oak_planks"));
static PALE_OAK_PLANKS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("pale_oak_planks"));
static MANGROVE_PLANKS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("mangrove_planks"));
static POPLAR_PLANKS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("poplar_planks"));
static BAMBOO_PLANKS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("bamboo_planks"));
static CRIMSON_PLANKS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("crimson_planks"));
static WARPED_PLANKS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("warped_planks"));
static BAMBOO_MOSAIC: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("bamboo_mosaic"));
static OAK_SAPLING: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("oak_sapling"));
static SPRUCE_SAPLING: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("spruce_sapling"));
static BIRCH_SAPLING: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("birch_sapling"));
static JUNGLE_SAPLING: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("jungle_sapling"));
static ACACIA_SAPLING: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("acacia_sapling"));
static CHERRY_SAPLING: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("cherry_sapling"));
static DARK_OAK_SAPLING: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("dark_oak_sapling"));
static PALE_OAK_SAPLING: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("pale_oak_sapling"));
static MANGROVE_PROPAGULE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("mangrove_propagule"));
static POPLAR_SAPLING: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("poplar_sapling"));
static BEDROCK: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("bedrock"));
static SAND: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("sand"));
static SUSPICIOUS_SAND: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("suspicious_sand"));
static SUSPICIOUS_GRAVEL: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("suspicious_gravel"));
static RED_SAND: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("red_sand"));
static GRAVEL: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("gravel"));
static COAL_ORE: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("coal_ore"));
static DEEPSLATE_COAL_ORE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("deepslate_coal_ore"));
static IRON_ORE: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("iron_ore"));
static DEEPSLATE_IRON_ORE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("deepslate_iron_ore"));
static COPPER_ORE: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("copper_ore"));
static DEEPSLATE_COPPER_ORE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("deepslate_copper_ore"));
static GOLD_ORE: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("gold_ore"));
static DEEPSLATE_GOLD_ORE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("deepslate_gold_ore"));
static REDSTONE_ORE: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("redstone_ore"));
static DEEPSLATE_REDSTONE_ORE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("deepslate_redstone_ore"));
static EMERALD_ORE: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("emerald_ore"));
static DEEPSLATE_EMERALD_ORE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("deepslate_emerald_ore"));
static LAPIS_ORE: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("lapis_ore"));
static DEEPSLATE_LAPIS_ORE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("deepslate_lapis_ore"));
static DIAMOND_ORE: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("diamond_ore"));
static DEEPSLATE_DIAMOND_ORE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("deepslate_diamond_ore"));
static NETHER_GOLD_ORE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("nether_gold_ore"));
static NETHER_QUARTZ_ORE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("nether_quartz_ore"));
static ANCIENT_DEBRIS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("ancient_debris"));
static COAL_BLOCK: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("coal_block"));
static RAW_IRON_BLOCK: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("raw_iron_block"));
static RAW_COPPER_BLOCK: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("raw_copper_block"));
static RAW_GOLD_BLOCK: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("raw_gold_block"));
static HEAVY_CORE: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("heavy_core"));
static AMETHYST_BLOCK: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("amethyst_block"));
static BUDDING_AMETHYST: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("budding_amethyst"));
static IRON_BLOCK: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("iron_block"));
static GOLD_BLOCK: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("gold_block"));
static DIAMOND_BLOCK: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("diamond_block"));
static NETHERITE_BLOCK: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("netherite_block"));
static OAK_LOG: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("oak_log"));
static SPRUCE_LOG: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("spruce_log"));
static BIRCH_LOG: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("birch_log"));
static JUNGLE_LOG: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("jungle_log"));
static ACACIA_LOG: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("acacia_log"));
static CHERRY_LOG: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("cherry_log"));
static PALE_OAK_LOG: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("pale_oak_log"));
static DARK_OAK_LOG: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("dark_oak_log"));
static MANGROVE_LOG: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("mangrove_log"));
static POPLAR_LOG: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("poplar_log"));
static MANGROVE_ROOTS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("mangrove_roots"));
static MUDDY_MANGROVE_ROOTS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("muddy_mangrove_roots"));
static CRIMSON_STEM: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("crimson_stem"));
static WARPED_STEM: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("warped_stem"));
static BAMBOO_BLOCK: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("bamboo_block"));
static STRIPPED_OAK_LOG: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("stripped_oak_log"));
static STRIPPED_SPRUCE_LOG: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("stripped_spruce_log"));
static STRIPPED_BIRCH_LOG: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("stripped_birch_log"));
static STRIPPED_JUNGLE_LOG: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("stripped_jungle_log"));
static STRIPPED_ACACIA_LOG: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("stripped_acacia_log"));
static STRIPPED_CHERRY_LOG: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("stripped_cherry_log"));
static STRIPPED_DARK_OAK_LOG: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("stripped_dark_oak_log"));
static STRIPPED_PALE_OAK_LOG: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("stripped_pale_oak_log"));
static STRIPPED_MANGROVE_LOG: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("stripped_mangrove_log"));
static STRIPPED_POPLAR_LOG: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("stripped_poplar_log"));
static STRIPPED_CRIMSON_STEM: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("stripped_crimson_stem"));
static STRIPPED_WARPED_STEM: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("stripped_warped_stem"));
static STRIPPED_OAK_WOOD: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("stripped_oak_wood"));
static STRIPPED_SPRUCE_WOOD: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("stripped_spruce_wood"));
static STRIPPED_BIRCH_WOOD: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("stripped_birch_wood"));
static STRIPPED_JUNGLE_WOOD: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("stripped_jungle_wood"));
static STRIPPED_ACACIA_WOOD: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("stripped_acacia_wood"));
static STRIPPED_CHERRY_WOOD: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("stripped_cherry_wood"));
static STRIPPED_DARK_OAK_WOOD: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("stripped_dark_oak_wood"));
static STRIPPED_PALE_OAK_WOOD: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("stripped_pale_oak_wood"));
static STRIPPED_MANGROVE_WOOD: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("stripped_mangrove_wood"));
static STRIPPED_POPLAR_WOOD: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("stripped_poplar_wood"));
static STRIPPED_CRIMSON_HYPHAE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("stripped_crimson_hyphae"));
static STRIPPED_WARPED_HYPHAE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("stripped_warped_hyphae"));
static STRIPPED_BAMBOO_BLOCK: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("stripped_bamboo_block"));
static OAK_WOOD: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("oak_wood"));
static SPRUCE_WOOD: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("spruce_wood"));
static BIRCH_WOOD: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("birch_wood"));
static JUNGLE_WOOD: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("jungle_wood"));
static ACACIA_WOOD: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("acacia_wood"));
static CHERRY_WOOD: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("cherry_wood"));
static PALE_OAK_WOOD: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("pale_oak_wood"));
static DARK_OAK_WOOD: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("dark_oak_wood"));
static MANGROVE_WOOD: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("mangrove_wood"));
static POPLAR_WOOD: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("poplar_wood"));
static CRIMSON_HYPHAE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("crimson_hyphae"));
static WARPED_HYPHAE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("warped_hyphae"));
static OAK_LEAVES: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("oak_leaves"));
static SPRUCE_LEAVES: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("spruce_leaves"));
static BIRCH_LEAVES: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("birch_leaves"));
static JUNGLE_LEAVES: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("jungle_leaves"));
static ACACIA_LEAVES: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("acacia_leaves"));
static CHERRY_LEAVES: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("cherry_leaves"));
static DARK_OAK_LEAVES: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("dark_oak_leaves"));
static PALE_OAK_LEAVES: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("pale_oak_leaves"));
static MANGROVE_LEAVES: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("mangrove_leaves"));
static RED_POPLAR_LEAVES: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("red_poplar_leaves"));
static ORANGE_POPLAR_LEAVES: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("orange_poplar_leaves"));
static YELLOW_POPLAR_LEAVES: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("yellow_poplar_leaves"));
static AZALEA_LEAVES: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("azalea_leaves"));
static FLOWERING_AZALEA_LEAVES: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("flowering_azalea_leaves"));
static SPONGE: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("sponge"));
static WET_SPONGE: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("wet_sponge"));
static GLASS: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("glass"));
static TINTED_GLASS: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("tinted_glass"));
static LAPIS_BLOCK: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("lapis_block"));
static SANDSTONE: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("sandstone"));
static CHISELED_SANDSTONE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("chiseled_sandstone"));
static CUT_SANDSTONE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("cut_sandstone"));
static COBWEB: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("cobweb"));
static SHORT_GRASS: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("short_grass"));
static FERN: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("fern"));
static BUSH: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("bush"));
static RED_SHRUB: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("red_shrub"));
static AZALEA: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("azalea"));
static FLOWERING_AZALEA: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("flowering_azalea"));
static DEAD_BUSH: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("dead_bush"));
static FIREFLY_BUSH: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("firefly_bush"));
static SHORT_DRY_GRASS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("short_dry_grass"));
static TALL_DRY_GRASS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("tall_dry_grass"));
static SEAGRASS: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("seagrass"));
static SEA_PICKLE: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("sea_pickle"));
static DANDELION: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("dandelion"));
static GOLDEN_DANDELION: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("golden_dandelion"));
static OPEN_EYEBLOSSOM: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("open_eyeblossom"));
static CLOSED_EYEBLOSSOM: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("closed_eyeblossom"));
static POPPY: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("poppy"));
static BLUE_ORCHID: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("blue_orchid"));
static ALLIUM: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("allium"));
static AZURE_BLUET: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("azure_bluet"));
static RED_TULIP: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("red_tulip"));
static ORANGE_TULIP: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("orange_tulip"));
static WHITE_TULIP: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("white_tulip"));
static PINK_TULIP: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("pink_tulip"));
static OXEYE_DAISY: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("oxeye_daisy"));
static CORNFLOWER: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("cornflower"));
static LILY_OF_THE_VALLEY: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("lily_of_the_valley"));
static WITHER_ROSE: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("wither_rose"));
static TORCHFLOWER: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("torchflower"));
static PITCHER_PLANT: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("pitcher_plant"));
static SPORE_BLOSSOM: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("spore_blossom"));
static BROWN_MUSHROOM: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("brown_mushroom"));
static RED_MUSHROOM: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("red_mushroom"));
static SHELF_MUSHROOM: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("shelf_mushroom"));
static CRIMSON_FUNGUS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("crimson_fungus"));
static WARPED_FUNGUS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("warped_fungus"));
static CRIMSON_ROOTS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("crimson_roots"));
static WARPED_ROOTS: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("warped_roots"));
static NETHER_SPROUTS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("nether_sprouts"));
static WEEPING_VINES: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("weeping_vines"));
static TWISTING_VINES: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("twisting_vines"));
static SUGAR_CANE: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("sugar_cane"));
static KELP: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("kelp"));
static PINK_PETALS: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("pink_petals"));
static WILDFLOWERS: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("wildflowers"));
static LEAF_LITTER: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("leaf_litter"));
static MOSS_CARPET: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("moss_carpet"));
static MOSS_BLOCK: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("moss_block"));
static PALE_MOSS_CARPET: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("pale_moss_carpet"));
static PALE_HANGING_MOSS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("pale_hanging_moss"));
static PALE_MOSS_BLOCK: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("pale_moss_block"));
static HANGING_ROOTS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("hanging_roots"));
static BIG_DRIPLEAF: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("big_dripleaf"));
static SMALL_DRIPLEAF: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("small_dripleaf"));
static BAMBOO: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("bamboo"));
static OAK_SLAB: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("oak_slab"));
static SPRUCE_SLAB: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("spruce_slab"));
static BIRCH_SLAB: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("birch_slab"));
static JUNGLE_SLAB: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("jungle_slab"));
static ACACIA_SLAB: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("acacia_slab"));
static CHERRY_SLAB: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("cherry_slab"));
static DARK_OAK_SLAB: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("dark_oak_slab"));
static PALE_OAK_SLAB: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("pale_oak_slab"));
static MANGROVE_SLAB: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("mangrove_slab"));
static POPLAR_SLAB: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("poplar_slab"));
static BAMBOO_SLAB: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("bamboo_slab"));
static BAMBOO_MOSAIC_SLAB: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("bamboo_mosaic_slab"));
static CRIMSON_SLAB: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("crimson_slab"));
static WARPED_SLAB: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("warped_slab"));
static STONE_SLAB: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("stone_slab"));
static SMOOTH_STONE_SLAB: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("smooth_stone_slab"));
static SANDSTONE_SLAB: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("sandstone_slab"));
static CUT_SANDSTONE_SLAB: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("cut_sandstone_slab"));
static PETRIFIED_OAK_SLAB: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("petrified_oak_slab"));
static COBBLESTONE_SLAB: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("cobblestone_slab"));
static BRICK_SLAB: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("brick_slab"));
static STONE_BRICK_SLAB: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("stone_brick_slab"));
static MUD_BRICK_SLAB: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("mud_brick_slab"));
static NETHER_BRICK_SLAB: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("nether_brick_slab"));
static QUARTZ_SLAB: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("quartz_slab"));
static RED_SANDSTONE_SLAB: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("red_sandstone_slab"));
static CUT_RED_SANDSTONE_SLAB: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("cut_red_sandstone_slab"));
static PURPUR_SLAB: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("purpur_slab"));
static PRISMARINE_SLAB: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("prismarine_slab"));
static PRISMARINE_BRICK_SLAB: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("prismarine_brick_slab"));
static DARK_PRISMARINE_SLAB: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("dark_prismarine_slab"));
static SMOOTH_QUARTZ: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("smooth_quartz"));
static SMOOTH_RED_SANDSTONE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("smooth_red_sandstone"));
static SMOOTH_SANDSTONE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("smooth_sandstone"));
static SMOOTH_STONE: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("smooth_stone"));
static BRICKS: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("bricks"));
static ACACIA_SHELF: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("acacia_shelf"));
static BAMBOO_SHELF: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("bamboo_shelf"));
static BIRCH_SHELF: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("birch_shelf"));
static CHERRY_SHELF: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("cherry_shelf"));
static CRIMSON_SHELF: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("crimson_shelf"));
static DARK_OAK_SHELF: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("dark_oak_shelf"));
static JUNGLE_SHELF: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("jungle_shelf"));
static MANGROVE_SHELF: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("mangrove_shelf"));
static POPLAR_SHELF: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("poplar_shelf"));
static OAK_SHELF: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("oak_shelf"));
static PALE_OAK_SHELF: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("pale_oak_shelf"));
static SPRUCE_SHELF: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("spruce_shelf"));
static WARPED_SHELF: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("warped_shelf"));
static BOOKSHELF: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("bookshelf"));
static CHISELED_BOOKSHELF: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("chiseled_bookshelf"));
static DECORATED_POT: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("decorated_pot"));
static MOSSY_COBBLESTONE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("mossy_cobblestone"));
static OBSIDIAN: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("obsidian"));
static TORCH: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("torch"));
static END_ROD: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("end_rod"));
static CHORUS_PLANT: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("chorus_plant"));
static CHORUS_FLOWER: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("chorus_flower"));
static PURPUR_BLOCK: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("purpur_block"));
static PURPUR_PILLAR: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("purpur_pillar"));
static PURPUR_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("purpur_stairs"));
static SPAWNER: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("spawner"));
static CREAKING_HEART: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("creaking_heart"));
static CHEST: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("chest"));
static CRAFTING_TABLE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("crafting_table"));
static FARMLAND: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("farmland"));
static FURNACE: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("furnace"));
static LADDER: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("ladder"));
static COBBLESTONE_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("cobblestone_stairs"));
static SNOW: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("snow"));
static ICE: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("ice"));
static SNOW_BLOCK: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("snow_block"));
static CACTUS: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("cactus"));
static CACTUS_FLOWER: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("cactus_flower"));
static CLAY: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("clay"));
static JUKEBOX: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("jukebox"));
static OAK_FENCE: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("oak_fence"));
static SPRUCE_FENCE: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("spruce_fence"));
static BIRCH_FENCE: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("birch_fence"));
static JUNGLE_FENCE: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("jungle_fence"));
static ACACIA_FENCE: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("acacia_fence"));
static CHERRY_FENCE: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("cherry_fence"));
static DARK_OAK_FENCE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("dark_oak_fence"));
static PALE_OAK_FENCE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("pale_oak_fence"));
static MANGROVE_FENCE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("mangrove_fence"));
static POPLAR_FENCE: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("poplar_fence"));
static BAMBOO_FENCE: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("bamboo_fence"));
static CRIMSON_FENCE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("crimson_fence"));
static WARPED_FENCE: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("warped_fence"));
static CARVED_PUMPKIN: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("carved_pumpkin"));
static JACK_O_LANTERN: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("jack_o_lantern"));
static NETHERRACK: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("netherrack"));
static SOUL_SAND: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("soul_sand"));
static SOUL_SOIL: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("soul_soil"));
static BASALT: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("basalt"));
static POLISHED_BASALT: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("polished_basalt"));
static SMOOTH_BASALT: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("smooth_basalt"));
static SOUL_TORCH: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("soul_torch"));
static COPPER_TORCH: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("copper_torch"));
static GLOWSTONE: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("glowstone"));
static INFESTED_STONE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("infested_stone"));
static INFESTED_COBBLESTONE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("infested_cobblestone"));
static INFESTED_STONE_BRICKS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("infested_stone_bricks"));
static INFESTED_MOSSY_STONE_BRICKS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("infested_mossy_stone_bricks"));
static INFESTED_CRACKED_STONE_BRICKS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("infested_cracked_stone_bricks"));
static INFESTED_CHISELED_STONE_BRICKS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("infested_chiseled_stone_bricks"));
static INFESTED_DEEPSLATE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("infested_deepslate"));
static STONE_BRICKS: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("stone_bricks"));
static MOSSY_STONE_BRICKS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("mossy_stone_bricks"));
static CRACKED_STONE_BRICKS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("cracked_stone_bricks"));
static CHISELED_STONE_BRICKS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("chiseled_stone_bricks"));
static PACKED_MUD: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("packed_mud"));
static MUD_BRICKS: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("mud_bricks"));
static DEEPSLATE_BRICKS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("deepslate_bricks"));
static CRACKED_DEEPSLATE_BRICKS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("cracked_deepslate_bricks"));
static DEEPSLATE_TILES: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("deepslate_tiles"));
static CRACKED_DEEPSLATE_TILES: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("cracked_deepslate_tiles"));
static CHISELED_DEEPSLATE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("chiseled_deepslate"));
static REINFORCED_DEEPSLATE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("reinforced_deepslate"));
static BROWN_MUSHROOM_BLOCK: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("brown_mushroom_block"));
static RED_MUSHROOM_BLOCK: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("red_mushroom_block"));
static MUSHROOM_STEM: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("mushroom_stem"));
static IRON_BARS: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("iron_bars"));
static IRON_CHAIN: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("iron_chain"));
static GLASS_PANE: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("glass_pane"));
static VINE: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("vine"));
static GLOW_LICHEN: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("glow_lichen"));
static RESIN_CLUMP: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("resin_clump"));
static RESIN_BLOCK: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("resin_block"));
static RESIN_BRICKS: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("resin_bricks"));
static RESIN_BRICK_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("resin_brick_stairs"));
static RESIN_BRICK_SLAB: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("resin_brick_slab"));
static RESIN_BRICK_WALL: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("resin_brick_wall"));
static CHISELED_RESIN_BRICKS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("chiseled_resin_bricks"));
static BRICK_STAIRS: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("brick_stairs"));
static STONE_BRICK_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("stone_brick_stairs"));
static MUD_BRICK_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("mud_brick_stairs"));
static MYCELIUM: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("mycelium"));
static LILY_PAD: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("lily_pad"));
static NETHER_BRICKS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("nether_bricks"));
static CRACKED_NETHER_BRICKS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("cracked_nether_bricks"));
static CHISELED_NETHER_BRICKS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("chiseled_nether_bricks"));
static NETHER_BRICK_FENCE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("nether_brick_fence"));
static NETHER_BRICK_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("nether_brick_stairs"));
static SCULK: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("sculk"));
static SCULK_VEIN: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("sculk_vein"));
static SCULK_CATALYST: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("sculk_catalyst"));
static SCULK_SHRIEKER: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("sculk_shrieker"));
static ENCHANTING_TABLE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("enchanting_table"));
static END_PORTAL_FRAME: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("end_portal_frame"));
static END_STONE: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("end_stone"));
static END_STONE_BRICKS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("end_stone_bricks"));
static DRAGON_EGG: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("dragon_egg"));
static SANDSTONE_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("sandstone_stairs"));
static ENDER_CHEST: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("ender_chest"));
static EMERALD_BLOCK: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("emerald_block"));
static OAK_STAIRS: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("oak_stairs"));
static SPRUCE_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("spruce_stairs"));
static BIRCH_STAIRS: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("birch_stairs"));
static JUNGLE_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("jungle_stairs"));
static ACACIA_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("acacia_stairs"));
static CHERRY_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("cherry_stairs"));
static DARK_OAK_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("dark_oak_stairs"));
static PALE_OAK_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("pale_oak_stairs"));
static MANGROVE_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("mangrove_stairs"));
static POPLAR_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("poplar_stairs"));
static BAMBOO_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("bamboo_stairs"));
static BAMBOO_MOSAIC_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("bamboo_mosaic_stairs"));
static CRIMSON_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("crimson_stairs"));
static WARPED_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("warped_stairs"));
static COMMAND_BLOCK: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("command_block"));
static BEACON: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("beacon"));
static COBBLESTONE_WALL: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("cobblestone_wall"));
static MOSSY_COBBLESTONE_WALL: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("mossy_cobblestone_wall"));
static BRICK_WALL: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("brick_wall"));
static PRISMARINE_WALL: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("prismarine_wall"));
static RED_SANDSTONE_WALL: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("red_sandstone_wall"));
static MOSSY_STONE_BRICK_WALL: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("mossy_stone_brick_wall"));
static GRANITE_WALL: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("granite_wall"));
static STONE_BRICK_WALL: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("stone_brick_wall"));
static MUD_BRICK_WALL: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("mud_brick_wall"));
static NETHER_BRICK_WALL: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("nether_brick_wall"));
static ANDESITE_WALL: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("andesite_wall"));
static RED_NETHER_BRICK_WALL: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("red_nether_brick_wall"));
static SANDSTONE_WALL: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("sandstone_wall"));
static END_STONE_BRICK_WALL: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("end_stone_brick_wall"));
static DIORITE_WALL: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("diorite_wall"));
static BLACKSTONE_WALL: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("blackstone_wall"));
static POLISHED_BLACKSTONE_WALL: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("polished_blackstone_wall"));
static POLISHED_BLACKSTONE_BRICK_WALL: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("polished_blackstone_brick_wall"));
static COBBLED_DEEPSLATE_WALL: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("cobbled_deepslate_wall"));
static POLISHED_DEEPSLATE_WALL: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("polished_deepslate_wall"));
static DEEPSLATE_BRICK_WALL: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("deepslate_brick_wall"));
static DEEPSLATE_TILE_WALL: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("deepslate_tile_wall"));
static ANVIL: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("anvil"));
static CHIPPED_ANVIL: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("chipped_anvil"));
static DAMAGED_ANVIL: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("damaged_anvil"));
static CHISELED_QUARTZ_BLOCK: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("chiseled_quartz_block"));
static QUARTZ_BLOCK: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("quartz_block"));
static QUARTZ_BRICKS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("quartz_bricks"));
static QUARTZ_PILLAR: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("quartz_pillar"));
static QUARTZ_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("quartz_stairs"));
static BARRIER: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("barrier"));
static LIGHT: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("light"));
static HAY_BLOCK: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("hay_block"));
static TERRACOTTA: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("terracotta"));
static PACKED_ICE: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("packed_ice"));
static DIRT_PATH: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("dirt_path"));
static SUNFLOWER: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("sunflower"));
static LILAC: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("lilac"));
static ROSE_BUSH: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("rose_bush"));
static PEONY: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("peony"));
static TALL_GRASS: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("tall_grass"));
static LARGE_FERN: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("large_fern"));
static PRISMARINE: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("prismarine"));
static PRISMARINE_BRICKS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("prismarine_bricks"));
static DARK_PRISMARINE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("dark_prismarine"));
static PRISMARINE_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("prismarine_stairs"));
static PRISMARINE_BRICK_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("prismarine_brick_stairs"));
static DARK_PRISMARINE_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("dark_prismarine_stairs"));
static SEA_LANTERN: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("sea_lantern"));
static RED_SANDSTONE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("red_sandstone"));
static CHISELED_RED_SANDSTONE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("chiseled_red_sandstone"));
static CUT_RED_SANDSTONE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("cut_red_sandstone"));
static RED_SANDSTONE_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("red_sandstone_stairs"));
static REPEATING_COMMAND_BLOCK: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("repeating_command_block"));
static CHAIN_COMMAND_BLOCK: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("chain_command_block"));
static MAGMA_BLOCK: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("magma_block"));
static NETHER_WART_BLOCK: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("nether_wart_block"));
static WARPED_WART_BLOCK: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("warped_wart_block"));
static RED_NETHER_BRICKS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("red_nether_bricks"));
static BONE_BLOCK: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("bone_block"));
static STRUCTURE_VOID: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("structure_void"));
static SHULKER_BOX: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("shulker_box"));
static TURTLE_EGG: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("turtle_egg"));
static SNIFFER_EGG: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("sniffer_egg"));
static DRIED_GHAST: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("dried_ghast"));
static DEAD_TUBE_CORAL_BLOCK: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("dead_tube_coral_block"));
static DEAD_BRAIN_CORAL_BLOCK: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("dead_brain_coral_block"));
static DEAD_BUBBLE_CORAL_BLOCK: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("dead_bubble_coral_block"));
static DEAD_FIRE_CORAL_BLOCK: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("dead_fire_coral_block"));
static DEAD_HORN_CORAL_BLOCK: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("dead_horn_coral_block"));
static TUBE_CORAL_BLOCK: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("tube_coral_block"));
static BRAIN_CORAL_BLOCK: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("brain_coral_block"));
static BUBBLE_CORAL_BLOCK: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("bubble_coral_block"));
static FIRE_CORAL_BLOCK: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("fire_coral_block"));
static HORN_CORAL_BLOCK: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("horn_coral_block"));
static TUBE_CORAL: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("tube_coral"));
static BRAIN_CORAL: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("brain_coral"));
static BUBBLE_CORAL: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("bubble_coral"));
static FIRE_CORAL: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("fire_coral"));
static HORN_CORAL: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("horn_coral"));
static DEAD_BRAIN_CORAL: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("dead_brain_coral"));
static DEAD_BUBBLE_CORAL: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("dead_bubble_coral"));
static DEAD_FIRE_CORAL: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("dead_fire_coral"));
static DEAD_HORN_CORAL: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("dead_horn_coral"));
static DEAD_TUBE_CORAL: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("dead_tube_coral"));
static TUBE_CORAL_FAN: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("tube_coral_fan"));
static BRAIN_CORAL_FAN: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("brain_coral_fan"));
static BUBBLE_CORAL_FAN: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("bubble_coral_fan"));
static FIRE_CORAL_FAN: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("fire_coral_fan"));
static HORN_CORAL_FAN: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("horn_coral_fan"));
static DEAD_TUBE_CORAL_FAN: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("dead_tube_coral_fan"));
static DEAD_BRAIN_CORAL_FAN: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("dead_brain_coral_fan"));
static DEAD_BUBBLE_CORAL_FAN: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("dead_bubble_coral_fan"));
static DEAD_FIRE_CORAL_FAN: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("dead_fire_coral_fan"));
static DEAD_HORN_CORAL_FAN: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("dead_horn_coral_fan"));
static BLUE_ICE: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("blue_ice"));
static CONDUIT: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("conduit"));
static POLISHED_GRANITE_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("polished_granite_stairs"));
static SMOOTH_RED_SANDSTONE_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("smooth_red_sandstone_stairs"));
static MOSSY_STONE_BRICK_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("mossy_stone_brick_stairs"));
static POLISHED_DIORITE_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("polished_diorite_stairs"));
static MOSSY_COBBLESTONE_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("mossy_cobblestone_stairs"));
static END_STONE_BRICK_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("end_stone_brick_stairs"));
static STONE_STAIRS: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("stone_stairs"));
static SMOOTH_SANDSTONE_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("smooth_sandstone_stairs"));
static SMOOTH_QUARTZ_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("smooth_quartz_stairs"));
static GRANITE_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("granite_stairs"));
static ANDESITE_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("andesite_stairs"));
static RED_NETHER_BRICK_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("red_nether_brick_stairs"));
static POLISHED_ANDESITE_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("polished_andesite_stairs"));
static DIORITE_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("diorite_stairs"));
static COBBLED_DEEPSLATE_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("cobbled_deepslate_stairs"));
static POLISHED_DEEPSLATE_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("polished_deepslate_stairs"));
static DEEPSLATE_BRICK_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("deepslate_brick_stairs"));
static DEEPSLATE_TILE_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("deepslate_tile_stairs"));
static POLISHED_GRANITE_SLAB: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("polished_granite_slab"));
static SMOOTH_RED_SANDSTONE_SLAB: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("smooth_red_sandstone_slab"));
static MOSSY_STONE_BRICK_SLAB: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("mossy_stone_brick_slab"));
static POLISHED_DIORITE_SLAB: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("polished_diorite_slab"));
static MOSSY_COBBLESTONE_SLAB: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("mossy_cobblestone_slab"));
static END_STONE_BRICK_SLAB: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("end_stone_brick_slab"));
static SMOOTH_SANDSTONE_SLAB: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("smooth_sandstone_slab"));
static SMOOTH_QUARTZ_SLAB: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("smooth_quartz_slab"));
static GRANITE_SLAB: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("granite_slab"));
static ANDESITE_SLAB: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("andesite_slab"));
static RED_NETHER_BRICK_SLAB: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("red_nether_brick_slab"));
static POLISHED_ANDESITE_SLAB: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("polished_andesite_slab"));
static DIORITE_SLAB: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("diorite_slab"));
static COBBLED_DEEPSLATE_SLAB: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("cobbled_deepslate_slab"));
static POLISHED_DEEPSLATE_SLAB: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("polished_deepslate_slab"));
static DEEPSLATE_BRICK_SLAB: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("deepslate_brick_slab"));
static DEEPSLATE_TILE_SLAB: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("deepslate_tile_slab"));
static SCAFFOLDING: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("scaffolding"));
static REDSTONE_TORCH: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("redstone_torch"));
static REDSTONE_BLOCK: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("redstone_block"));
static REPEATER: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("repeater"));
static COMPARATOR: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("comparator"));
static PISTON: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("piston"));
static STICKY_PISTON: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("sticky_piston"));
static SLIME_BLOCK: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("slime_block"));
static HONEY_BLOCK: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("honey_block"));
static OBSERVER: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("observer"));
static HOPPER: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("hopper"));
static DISPENSER: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("dispenser"));
static DROPPER: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("dropper"));
static LECTERN: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("lectern"));
static TARGET: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("target"));
static LEVER: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("lever"));
static DAYLIGHT_DETECTOR: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("daylight_detector"));
static SCULK_SENSOR: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("sculk_sensor"));
static CALIBRATED_SCULK_SENSOR: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("calibrated_sculk_sensor"));
static TRIPWIRE_HOOK: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("tripwire_hook"));
static TRAPPED_CHEST: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("trapped_chest"));
static TNT: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("tnt"));
static REDSTONE_LAMP: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("redstone_lamp"));
static NOTE_BLOCK: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("note_block"));
static STONE_BUTTON: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("stone_button"));
static POLISHED_BLACKSTONE_BUTTON: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("polished_blackstone_button"));
static OAK_BUTTON: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("oak_button"));
static SPRUCE_BUTTON: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("spruce_button"));
static BIRCH_BUTTON: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("birch_button"));
static JUNGLE_BUTTON: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("jungle_button"));
static ACACIA_BUTTON: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("acacia_button"));
static CHERRY_BUTTON: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("cherry_button"));
static DARK_OAK_BUTTON: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("dark_oak_button"));
static PALE_OAK_BUTTON: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("pale_oak_button"));
static MANGROVE_BUTTON: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("mangrove_button"));
static POPLAR_BUTTON: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("poplar_button"));
static BAMBOO_BUTTON: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("bamboo_button"));
static CRIMSON_BUTTON: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("crimson_button"));
static WARPED_BUTTON: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("warped_button"));
static STONE_PRESSURE_PLATE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("stone_pressure_plate"));
static POLISHED_BLACKSTONE_PRESSURE_PLATE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("polished_blackstone_pressure_plate"));
static LIGHT_WEIGHTED_PRESSURE_PLATE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("light_weighted_pressure_plate"));
static HEAVY_WEIGHTED_PRESSURE_PLATE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("heavy_weighted_pressure_plate"));
static OAK_PRESSURE_PLATE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("oak_pressure_plate"));
static SPRUCE_PRESSURE_PLATE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("spruce_pressure_plate"));
static BIRCH_PRESSURE_PLATE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("birch_pressure_plate"));
static JUNGLE_PRESSURE_PLATE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("jungle_pressure_plate"));
static ACACIA_PRESSURE_PLATE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("acacia_pressure_plate"));
static CHERRY_PRESSURE_PLATE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("cherry_pressure_plate"));
static DARK_OAK_PRESSURE_PLATE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("dark_oak_pressure_plate"));
static PALE_OAK_PRESSURE_PLATE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("pale_oak_pressure_plate"));
static MANGROVE_PRESSURE_PLATE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("mangrove_pressure_plate"));
static POPLAR_PRESSURE_PLATE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("poplar_pressure_plate"));
static BAMBOO_PRESSURE_PLATE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("bamboo_pressure_plate"));
static CRIMSON_PRESSURE_PLATE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("crimson_pressure_plate"));
static WARPED_PRESSURE_PLATE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("warped_pressure_plate"));
static IRON_DOOR: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("iron_door"));
static OAK_DOOR: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("oak_door"));
static SPRUCE_DOOR: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("spruce_door"));
static BIRCH_DOOR: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("birch_door"));
static JUNGLE_DOOR: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("jungle_door"));
static ACACIA_DOOR: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("acacia_door"));
static CHERRY_DOOR: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("cherry_door"));
static DARK_OAK_DOOR: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("dark_oak_door"));
static PALE_OAK_DOOR: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("pale_oak_door"));
static MANGROVE_DOOR: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("mangrove_door"));
static POPLAR_DOOR: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("poplar_door"));
static BAMBOO_DOOR: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("bamboo_door"));
static CRIMSON_DOOR: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("crimson_door"));
static WARPED_DOOR: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("warped_door"));
static IRON_TRAPDOOR: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("iron_trapdoor"));
static OAK_TRAPDOOR: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("oak_trapdoor"));
static SPRUCE_TRAPDOOR: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("spruce_trapdoor"));
static BIRCH_TRAPDOOR: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("birch_trapdoor"));
static JUNGLE_TRAPDOOR: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("jungle_trapdoor"));
static ACACIA_TRAPDOOR: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("acacia_trapdoor"));
static CHERRY_TRAPDOOR: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("cherry_trapdoor"));
static DARK_OAK_TRAPDOOR: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("dark_oak_trapdoor"));
static PALE_OAK_TRAPDOOR: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("pale_oak_trapdoor"));
static MANGROVE_TRAPDOOR: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("mangrove_trapdoor"));
static POPLAR_TRAPDOOR: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("poplar_trapdoor"));
static BAMBOO_TRAPDOOR: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("bamboo_trapdoor"));
static CRIMSON_TRAPDOOR: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("crimson_trapdoor"));
static WARPED_TRAPDOOR: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("warped_trapdoor"));
static OAK_FENCE_GATE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("oak_fence_gate"));
static SPRUCE_FENCE_GATE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("spruce_fence_gate"));
static BIRCH_FENCE_GATE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("birch_fence_gate"));
static JUNGLE_FENCE_GATE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("jungle_fence_gate"));
static ACACIA_FENCE_GATE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("acacia_fence_gate"));
static CHERRY_FENCE_GATE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("cherry_fence_gate"));
static DARK_OAK_FENCE_GATE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("dark_oak_fence_gate"));
static PALE_OAK_FENCE_GATE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("pale_oak_fence_gate"));
static MANGROVE_FENCE_GATE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("mangrove_fence_gate"));
static POPLAR_FENCE_GATE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("poplar_fence_gate"));
static BAMBOO_FENCE_GATE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("bamboo_fence_gate"));
static CRIMSON_FENCE_GATE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("crimson_fence_gate"));
static WARPED_FENCE_GATE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("warped_fence_gate"));
static POWERED_RAIL: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("powered_rail"));
static DETECTOR_RAIL: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("detector_rail"));
static RAIL: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("rail"));
static ACTIVATOR_RAIL: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("activator_rail"));
static STRUCTURE_BLOCK: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("structure_block"));
static JIGSAW: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("jigsaw"));
static TEST_BLOCK: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("test_block"));
static TEST_INSTANCE_BLOCK: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("test_instance_block"));
static OAK_SIGN: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("oak_sign"));
static SPRUCE_SIGN: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("spruce_sign"));
static BIRCH_SIGN: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("birch_sign"));
static JUNGLE_SIGN: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("jungle_sign"));
static ACACIA_SIGN: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("acacia_sign"));
static CHERRY_SIGN: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("cherry_sign"));
static DARK_OAK_SIGN: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("dark_oak_sign"));
static PALE_OAK_SIGN: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("pale_oak_sign"));
static MANGROVE_SIGN: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("mangrove_sign"));
static POPLAR_SIGN: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("poplar_sign"));
static BAMBOO_SIGN: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("bamboo_sign"));
static CRIMSON_SIGN: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("crimson_sign"));
static WARPED_SIGN: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("warped_sign"));
static OAK_HANGING_SIGN: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("oak_hanging_sign"));
static SPRUCE_HANGING_SIGN: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("spruce_hanging_sign"));
static BIRCH_HANGING_SIGN: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("birch_hanging_sign"));
static JUNGLE_HANGING_SIGN: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("jungle_hanging_sign"));
static ACACIA_HANGING_SIGN: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("acacia_hanging_sign"));
static CHERRY_HANGING_SIGN: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("cherry_hanging_sign"));
static DARK_OAK_HANGING_SIGN: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("dark_oak_hanging_sign"));
static PALE_OAK_HANGING_SIGN: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("pale_oak_hanging_sign"));
static MANGROVE_HANGING_SIGN: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("mangrove_hanging_sign"));
static POPLAR_HANGING_SIGN: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("poplar_hanging_sign"));
static BAMBOO_HANGING_SIGN: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("bamboo_hanging_sign"));
static CRIMSON_HANGING_SIGN: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("crimson_hanging_sign"));
static WARPED_HANGING_SIGN: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("warped_hanging_sign"));
static DRIED_KELP_BLOCK: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("dried_kelp_block"));
static CAKE: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("cake"));
static CRAFTER: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("crafter"));
static NETHER_WART: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("nether_wart"));
static BREWING_STAND: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("brewing_stand"));
static CAULDRON: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("cauldron"));
static FLOWER_POT: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("flower_pot"));
static SKELETON_SKULL: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("skeleton_skull"));
static WITHER_SKELETON_SKULL: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("wither_skeleton_skull"));
static PLAYER_HEAD: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("player_head"));
static ZOMBIE_HEAD: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("zombie_head"));
static CREEPER_HEAD: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("creeper_head"));
static DRAGON_HEAD: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("dragon_head"));
static PIGLIN_HEAD: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("piglin_head"));
static LOOM: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("loom"));
static COMPOSTER: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("composter"));
static BARREL: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("barrel"));
static SMOKER: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("smoker"));
static BLAST_FURNACE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("blast_furnace"));
static CARTOGRAPHY_TABLE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("cartography_table"));
static FLETCHING_TABLE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("fletching_table"));
static GRINDSTONE: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("grindstone"));
static SMITHING_TABLE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("smithing_table"));
static STONECUTTER: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("stonecutter"));
static BELL: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("bell"));
static LANTERN: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("lantern"));
static SOUL_LANTERN: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("soul_lantern"));
static CAMPFIRE: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("campfire"));
static SOUL_CAMPFIRE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("soul_campfire"));
static SHROOMLIGHT: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("shroomlight"));
static BEE_NEST: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("bee_nest"));
static BEEHIVE: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("beehive"));
static HONEYCOMB_BLOCK: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("honeycomb_block"));
static LODESTONE: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("lodestone"));
static CRYING_OBSIDIAN: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("crying_obsidian"));
static BLACKSTONE: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("blackstone"));
static BLACKSTONE_SLAB: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("blackstone_slab"));
static BLACKSTONE_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("blackstone_stairs"));
static GILDED_BLACKSTONE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("gilded_blackstone"));
static POLISHED_BLACKSTONE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("polished_blackstone"));
static POLISHED_BLACKSTONE_SLAB: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("polished_blackstone_slab"));
static POLISHED_BLACKSTONE_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("polished_blackstone_stairs"));
static CHISELED_POLISHED_BLACKSTONE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("chiseled_polished_blackstone"));
static POLISHED_BLACKSTONE_BRICKS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("polished_blackstone_bricks"));
static POLISHED_BLACKSTONE_BRICK_SLAB: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("polished_blackstone_brick_slab"));
static POLISHED_BLACKSTONE_BRICK_STAIRS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("polished_blackstone_brick_stairs"));
static CRACKED_POLISHED_BLACKSTONE_BRICKS: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("cracked_polished_blackstone_bricks"));
static RESPAWN_ANCHOR: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("respawn_anchor"));
static CANDLE: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("candle"));
static SMALL_AMETHYST_BUD: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("small_amethyst_bud"));
static MEDIUM_AMETHYST_BUD: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("medium_amethyst_bud"));
static LARGE_AMETHYST_BUD: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("large_amethyst_bud"));
static AMETHYST_CLUSTER: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("amethyst_cluster"));
static POINTED_DRIPSTONE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("pointed_dripstone"));
static SULFUR_SPIKE: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("sulfur_spike"));
static OCHRE_FROGLIGHT: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("ochre_froglight"));
static VERDANT_FROGLIGHT: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("verdant_froglight"));
static PEARLESCENT_FROGLIGHT: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("pearlescent_froglight"));
static FROGSPAWN: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("frogspawn"));
static TRIAL_SPAWNER: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create("trial_spawner"));
static VAULT: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("vault"));
static STRAW_BED: LazyLock<BlockItemId> = LazyLock::new(|| BlockItemId::create("straw_bed"));
static REDSTONE_DUST: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create_named("redstone_wire", "redstone"));
static TRIPWIRE: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create_named("tripwire", "string"));
static POWDER_SNOW: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create_named("powder_snow", "powder_snow_bucket"));
static WHEAT_CROP: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create_named("wheat", "wheat_seeds"));
static COCOA_CROP: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create_named("cocoa", "cocoa_beans"));
static CARROT_CROP: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create_named("carrots", "carrot"));
static POTATO_CROP: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create_named("potatoes", "potato"));
static TORCHFLOWER_CROP: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create_named("torchflower_crop", "torchflower_seeds"));
static PITCHER_CROP: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create_named("pitcher_crop", "pitcher_pod"));
static BEETROOT_CROP: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create_named("beetroots", "beetroot_seeds"));
static SWEET_BERRY_CROP: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create_named("sweet_berry_bush", "sweet_berries"));
static GLOW_BERRY_CROP: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create_named("cave_vines", "glow_berries"));
static PUMPKIN_CROP: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create_named("pumpkin_stem", "pumpkin_seeds"));
static MELON_CROP: LazyLock<BlockItemId> =
    LazyLock::new(|| BlockItemId::create_named("melon_stem", "melon_seeds"));
static BED: LazyLock<ColorCollection<BlockItemId>> = LazyLock::new(|| create_simple_colored("bed"));
static WOOL: LazyLock<ColorCollection<BlockItemId>> =
    LazyLock::new(|| create_simple_colored("wool"));
static WOOL_STAIRS: LazyLock<ColorCollection<BlockItemId>> =
    LazyLock::new(|| create_simple_colored("wool_stairs"));
static WOOL_SLAB: LazyLock<ColorCollection<BlockItemId>> =
    LazyLock::new(|| create_simple_colored("wool_slab"));
static STAINED_GLASS: LazyLock<ColorCollection<BlockItemId>> =
    LazyLock::new(|| create_simple_colored("stained_glass"));
static DYED_TERRACOTTA: LazyLock<ColorCollection<BlockItemId>> =
    LazyLock::new(|| create_simple_colored("terracotta"));
static STAINED_GLASS_PANE: LazyLock<ColorCollection<BlockItemId>> =
    LazyLock::new(|| create_simple_colored("stained_glass_pane"));
static CARPET: LazyLock<ColorCollection<BlockItemId>> =
    LazyLock::new(|| create_simple_colored("carpet"));
static BANNER: LazyLock<ColorCollection<BlockItemId>> =
    LazyLock::new(|| create_simple_colored("banner"));
static DYED_SHULKER_BOX: LazyLock<ColorCollection<BlockItemId>> =
    LazyLock::new(|| create_simple_colored("shulker_box"));
static GLAZED_TERRACOTTA: LazyLock<ColorCollection<BlockItemId>> =
    LazyLock::new(|| create_simple_colored("glazed_terracotta"));
static CONCRETE: LazyLock<ColorCollection<BlockItemId>> =
    LazyLock::new(|| create_simple_colored("concrete"));
static CONCRETE_STAIRS: LazyLock<ColorCollection<BlockItemId>> =
    LazyLock::new(|| create_simple_colored("concrete_stairs"));
static CONCRETE_SLAB: LazyLock<ColorCollection<BlockItemId>> =
    LazyLock::new(|| create_simple_colored("concrete_slab"));
static CONCRETE_POWDER: LazyLock<ColorCollection<BlockItemId>> =
    LazyLock::new(|| create_simple_colored("concrete_powder"));
static DYED_CANDLE: LazyLock<ColorCollection<BlockItemId>> =
    LazyLock::new(|| create_simple_colored("candle"));
// `COPPER_BLOCK_SPECIAL_NAMES`: the unaffected block is `copper_block`, the others `copper`.
static COPPER_BLOCK: LazyLock<WeatheringCopperCollection<BlockItemId>> = LazyLock::new(|| {
    WeatheringCopperCollection::prefix_with_state(&WeatheringCopperCollection::same(
        ByState::new("copper_block", "copper", "copper", "copper").map(|name| name.to_string()),
    ))
    .map(|name| BlockItemId::create(name))
});
static COPPER_BARS: LazyLock<WeatheringCopperCollection<BlockItemId>> =
    LazyLock::new(|| create_simple_copper("copper_bars"));
static COPPER_CHAIN: LazyLock<WeatheringCopperCollection<BlockItemId>> =
    LazyLock::new(|| create_simple_copper("copper_chain"));
static COPPER_LANTERN: LazyLock<WeatheringCopperCollection<BlockItemId>> =
    LazyLock::new(|| create_simple_copper("copper_lantern"));
static CUT_COPPER: LazyLock<WeatheringCopperCollection<BlockItemId>> =
    LazyLock::new(|| create_simple_copper("cut_copper"));
static CHISELED_COPPER: LazyLock<WeatheringCopperCollection<BlockItemId>> =
    LazyLock::new(|| create_simple_copper("chiseled_copper"));
static CUT_COPPER_STAIRS: LazyLock<WeatheringCopperCollection<BlockItemId>> =
    LazyLock::new(|| create_simple_copper("cut_copper_stairs"));
static CUT_COPPER_SLAB: LazyLock<WeatheringCopperCollection<BlockItemId>> =
    LazyLock::new(|| create_simple_copper("cut_copper_slab"));
static COPPER_DOOR: LazyLock<WeatheringCopperCollection<BlockItemId>> =
    LazyLock::new(|| create_simple_copper("copper_door"));
static COPPER_TRAPDOOR: LazyLock<WeatheringCopperCollection<BlockItemId>> =
    LazyLock::new(|| create_simple_copper("copper_trapdoor"));
static COPPER_GRATE: LazyLock<WeatheringCopperCollection<BlockItemId>> =
    LazyLock::new(|| create_simple_copper("copper_grate"));
static COPPER_BULB: LazyLock<WeatheringCopperCollection<BlockItemId>> =
    LazyLock::new(|| create_simple_copper("copper_bulb"));
static COPPER_CHEST: LazyLock<WeatheringCopperCollection<BlockItemId>> =
    LazyLock::new(|| create_simple_copper("copper_chest"));
static COPPER_GOLEM_STATUE: LazyLock<WeatheringCopperCollection<BlockItemId>> =
    LazyLock::new(|| create_simple_copper("copper_golem_statue"));
static LIGHTNING_ROD: LazyLock<WeatheringCopperCollection<BlockItemId>> =
    LazyLock::new(|| create_simple_copper("lightning_rod"));

pub struct BlockItemIds;

impl BlockItemIds {
    pub const AIR: &'static LazyLock<BlockItemId> = &AIR;
    pub const DIRT: &'static LazyLock<BlockItemId> = &DIRT;
    pub const PUMPKIN: &'static LazyLock<BlockItemId> = &PUMPKIN;
    pub const MELON: &'static LazyLock<BlockItemId> = &MELON;
    pub const STONE: &'static LazyLock<BlockItemId> = &STONE;
    pub const GRANITE: &'static LazyLock<BlockItemId> = &GRANITE;
    pub const POLISHED_GRANITE: &'static LazyLock<BlockItemId> = &POLISHED_GRANITE;
    pub const DIORITE: &'static LazyLock<BlockItemId> = &DIORITE;
    pub const POLISHED_DIORITE: &'static LazyLock<BlockItemId> = &POLISHED_DIORITE;
    pub const ANDESITE: &'static LazyLock<BlockItemId> = &ANDESITE;
    pub const POLISHED_ANDESITE: &'static LazyLock<BlockItemId> = &POLISHED_ANDESITE;
    pub const DEEPSLATE: &'static LazyLock<BlockItemId> = &DEEPSLATE;
    pub const COBBLED_DEEPSLATE: &'static LazyLock<BlockItemId> = &COBBLED_DEEPSLATE;
    pub const POLISHED_DEEPSLATE: &'static LazyLock<BlockItemId> = &POLISHED_DEEPSLATE;
    pub const CALCITE: &'static LazyLock<BlockItemId> = &CALCITE;
    pub const TUFF: &'static LazyLock<BlockItemId> = &TUFF;
    pub const TUFF_SLAB: &'static LazyLock<BlockItemId> = &TUFF_SLAB;
    pub const TUFF_STAIRS: &'static LazyLock<BlockItemId> = &TUFF_STAIRS;
    pub const TUFF_WALL: &'static LazyLock<BlockItemId> = &TUFF_WALL;
    pub const CHISELED_TUFF: &'static LazyLock<BlockItemId> = &CHISELED_TUFF;
    pub const POLISHED_TUFF: &'static LazyLock<BlockItemId> = &POLISHED_TUFF;
    pub const POLISHED_TUFF_SLAB: &'static LazyLock<BlockItemId> = &POLISHED_TUFF_SLAB;
    pub const POLISHED_TUFF_STAIRS: &'static LazyLock<BlockItemId> = &POLISHED_TUFF_STAIRS;
    pub const POLISHED_TUFF_WALL: &'static LazyLock<BlockItemId> = &POLISHED_TUFF_WALL;
    pub const TUFF_BRICKS: &'static LazyLock<BlockItemId> = &TUFF_BRICKS;
    pub const TUFF_BRICK_SLAB: &'static LazyLock<BlockItemId> = &TUFF_BRICK_SLAB;
    pub const TUFF_BRICK_STAIRS: &'static LazyLock<BlockItemId> = &TUFF_BRICK_STAIRS;
    pub const TUFF_BRICK_WALL: &'static LazyLock<BlockItemId> = &TUFF_BRICK_WALL;
    pub const CHISELED_TUFF_BRICKS: &'static LazyLock<BlockItemId> = &CHISELED_TUFF_BRICKS;
    pub const SULFUR: &'static LazyLock<BlockItemId> = &SULFUR;
    pub const POTENT_SULFUR: &'static LazyLock<BlockItemId> = &POTENT_SULFUR;
    pub const SULFUR_SLAB: &'static LazyLock<BlockItemId> = &SULFUR_SLAB;
    pub const SULFUR_STAIRS: &'static LazyLock<BlockItemId> = &SULFUR_STAIRS;
    pub const SULFUR_WALL: &'static LazyLock<BlockItemId> = &SULFUR_WALL;
    pub const POLISHED_SULFUR: &'static LazyLock<BlockItemId> = &POLISHED_SULFUR;
    pub const POLISHED_SULFUR_SLAB: &'static LazyLock<BlockItemId> = &POLISHED_SULFUR_SLAB;
    pub const POLISHED_SULFUR_STAIRS: &'static LazyLock<BlockItemId> = &POLISHED_SULFUR_STAIRS;
    pub const POLISHED_SULFUR_WALL: &'static LazyLock<BlockItemId> = &POLISHED_SULFUR_WALL;
    pub const SULFUR_BRICKS: &'static LazyLock<BlockItemId> = &SULFUR_BRICKS;
    pub const SULFUR_BRICK_SLAB: &'static LazyLock<BlockItemId> = &SULFUR_BRICK_SLAB;
    pub const SULFUR_BRICK_STAIRS: &'static LazyLock<BlockItemId> = &SULFUR_BRICK_STAIRS;
    pub const SULFUR_BRICK_WALL: &'static LazyLock<BlockItemId> = &SULFUR_BRICK_WALL;
    pub const CHISELED_SULFUR: &'static LazyLock<BlockItemId> = &CHISELED_SULFUR;
    pub const CINNABAR: &'static LazyLock<BlockItemId> = &CINNABAR;
    pub const CINNABAR_SLAB: &'static LazyLock<BlockItemId> = &CINNABAR_SLAB;
    pub const CINNABAR_STAIRS: &'static LazyLock<BlockItemId> = &CINNABAR_STAIRS;
    pub const CINNABAR_WALL: &'static LazyLock<BlockItemId> = &CINNABAR_WALL;
    pub const POLISHED_CINNABAR: &'static LazyLock<BlockItemId> = &POLISHED_CINNABAR;
    pub const POLISHED_CINNABAR_SLAB: &'static LazyLock<BlockItemId> = &POLISHED_CINNABAR_SLAB;
    pub const POLISHED_CINNABAR_STAIRS: &'static LazyLock<BlockItemId> = &POLISHED_CINNABAR_STAIRS;
    pub const POLISHED_CINNABAR_WALL: &'static LazyLock<BlockItemId> = &POLISHED_CINNABAR_WALL;
    pub const CINNABAR_BRICKS: &'static LazyLock<BlockItemId> = &CINNABAR_BRICKS;
    pub const CINNABAR_BRICK_SLAB: &'static LazyLock<BlockItemId> = &CINNABAR_BRICK_SLAB;
    pub const CINNABAR_BRICK_STAIRS: &'static LazyLock<BlockItemId> = &CINNABAR_BRICK_STAIRS;
    pub const CINNABAR_BRICK_WALL: &'static LazyLock<BlockItemId> = &CINNABAR_BRICK_WALL;
    pub const CHISELED_CINNABAR: &'static LazyLock<BlockItemId> = &CHISELED_CINNABAR;
    pub const DRIPSTONE_BLOCK: &'static LazyLock<BlockItemId> = &DRIPSTONE_BLOCK;
    pub const GRASS_BLOCK: &'static LazyLock<BlockItemId> = &GRASS_BLOCK;
    pub const COARSE_DIRT: &'static LazyLock<BlockItemId> = &COARSE_DIRT;
    pub const PODZOL: &'static LazyLock<BlockItemId> = &PODZOL;
    pub const ROOTED_DIRT: &'static LazyLock<BlockItemId> = &ROOTED_DIRT;
    pub const MUD: &'static LazyLock<BlockItemId> = &MUD;
    pub const CRIMSON_NYLIUM: &'static LazyLock<BlockItemId> = &CRIMSON_NYLIUM;
    pub const WARPED_NYLIUM: &'static LazyLock<BlockItemId> = &WARPED_NYLIUM;
    pub const COBBLESTONE: &'static LazyLock<BlockItemId> = &COBBLESTONE;
    pub const OAK_PLANKS: &'static LazyLock<BlockItemId> = &OAK_PLANKS;
    pub const SPRUCE_PLANKS: &'static LazyLock<BlockItemId> = &SPRUCE_PLANKS;
    pub const BIRCH_PLANKS: &'static LazyLock<BlockItemId> = &BIRCH_PLANKS;
    pub const JUNGLE_PLANKS: &'static LazyLock<BlockItemId> = &JUNGLE_PLANKS;
    pub const ACACIA_PLANKS: &'static LazyLock<BlockItemId> = &ACACIA_PLANKS;
    pub const CHERRY_PLANKS: &'static LazyLock<BlockItemId> = &CHERRY_PLANKS;
    pub const DARK_OAK_PLANKS: &'static LazyLock<BlockItemId> = &DARK_OAK_PLANKS;
    pub const PALE_OAK_PLANKS: &'static LazyLock<BlockItemId> = &PALE_OAK_PLANKS;
    pub const MANGROVE_PLANKS: &'static LazyLock<BlockItemId> = &MANGROVE_PLANKS;
    pub const POPLAR_PLANKS: &'static LazyLock<BlockItemId> = &POPLAR_PLANKS;
    pub const BAMBOO_PLANKS: &'static LazyLock<BlockItemId> = &BAMBOO_PLANKS;
    pub const CRIMSON_PLANKS: &'static LazyLock<BlockItemId> = &CRIMSON_PLANKS;
    pub const WARPED_PLANKS: &'static LazyLock<BlockItemId> = &WARPED_PLANKS;
    pub const BAMBOO_MOSAIC: &'static LazyLock<BlockItemId> = &BAMBOO_MOSAIC;
    pub const OAK_SAPLING: &'static LazyLock<BlockItemId> = &OAK_SAPLING;
    pub const SPRUCE_SAPLING: &'static LazyLock<BlockItemId> = &SPRUCE_SAPLING;
    pub const BIRCH_SAPLING: &'static LazyLock<BlockItemId> = &BIRCH_SAPLING;
    pub const JUNGLE_SAPLING: &'static LazyLock<BlockItemId> = &JUNGLE_SAPLING;
    pub const ACACIA_SAPLING: &'static LazyLock<BlockItemId> = &ACACIA_SAPLING;
    pub const CHERRY_SAPLING: &'static LazyLock<BlockItemId> = &CHERRY_SAPLING;
    pub const DARK_OAK_SAPLING: &'static LazyLock<BlockItemId> = &DARK_OAK_SAPLING;
    pub const PALE_OAK_SAPLING: &'static LazyLock<BlockItemId> = &PALE_OAK_SAPLING;
    pub const MANGROVE_PROPAGULE: &'static LazyLock<BlockItemId> = &MANGROVE_PROPAGULE;
    pub const POPLAR_SAPLING: &'static LazyLock<BlockItemId> = &POPLAR_SAPLING;
    pub const BEDROCK: &'static LazyLock<BlockItemId> = &BEDROCK;
    pub const SAND: &'static LazyLock<BlockItemId> = &SAND;
    pub const SUSPICIOUS_SAND: &'static LazyLock<BlockItemId> = &SUSPICIOUS_SAND;
    pub const SUSPICIOUS_GRAVEL: &'static LazyLock<BlockItemId> = &SUSPICIOUS_GRAVEL;
    pub const RED_SAND: &'static LazyLock<BlockItemId> = &RED_SAND;
    pub const GRAVEL: &'static LazyLock<BlockItemId> = &GRAVEL;
    pub const COAL_ORE: &'static LazyLock<BlockItemId> = &COAL_ORE;
    pub const DEEPSLATE_COAL_ORE: &'static LazyLock<BlockItemId> = &DEEPSLATE_COAL_ORE;
    pub const IRON_ORE: &'static LazyLock<BlockItemId> = &IRON_ORE;
    pub const DEEPSLATE_IRON_ORE: &'static LazyLock<BlockItemId> = &DEEPSLATE_IRON_ORE;
    pub const COPPER_ORE: &'static LazyLock<BlockItemId> = &COPPER_ORE;
    pub const DEEPSLATE_COPPER_ORE: &'static LazyLock<BlockItemId> = &DEEPSLATE_COPPER_ORE;
    pub const GOLD_ORE: &'static LazyLock<BlockItemId> = &GOLD_ORE;
    pub const DEEPSLATE_GOLD_ORE: &'static LazyLock<BlockItemId> = &DEEPSLATE_GOLD_ORE;
    pub const REDSTONE_ORE: &'static LazyLock<BlockItemId> = &REDSTONE_ORE;
    pub const DEEPSLATE_REDSTONE_ORE: &'static LazyLock<BlockItemId> = &DEEPSLATE_REDSTONE_ORE;
    pub const EMERALD_ORE: &'static LazyLock<BlockItemId> = &EMERALD_ORE;
    pub const DEEPSLATE_EMERALD_ORE: &'static LazyLock<BlockItemId> = &DEEPSLATE_EMERALD_ORE;
    pub const LAPIS_ORE: &'static LazyLock<BlockItemId> = &LAPIS_ORE;
    pub const DEEPSLATE_LAPIS_ORE: &'static LazyLock<BlockItemId> = &DEEPSLATE_LAPIS_ORE;
    pub const DIAMOND_ORE: &'static LazyLock<BlockItemId> = &DIAMOND_ORE;
    pub const DEEPSLATE_DIAMOND_ORE: &'static LazyLock<BlockItemId> = &DEEPSLATE_DIAMOND_ORE;
    pub const NETHER_GOLD_ORE: &'static LazyLock<BlockItemId> = &NETHER_GOLD_ORE;
    pub const NETHER_QUARTZ_ORE: &'static LazyLock<BlockItemId> = &NETHER_QUARTZ_ORE;
    pub const ANCIENT_DEBRIS: &'static LazyLock<BlockItemId> = &ANCIENT_DEBRIS;
    pub const COAL_BLOCK: &'static LazyLock<BlockItemId> = &COAL_BLOCK;
    pub const RAW_IRON_BLOCK: &'static LazyLock<BlockItemId> = &RAW_IRON_BLOCK;
    pub const RAW_COPPER_BLOCK: &'static LazyLock<BlockItemId> = &RAW_COPPER_BLOCK;
    pub const RAW_GOLD_BLOCK: &'static LazyLock<BlockItemId> = &RAW_GOLD_BLOCK;
    pub const HEAVY_CORE: &'static LazyLock<BlockItemId> = &HEAVY_CORE;
    pub const AMETHYST_BLOCK: &'static LazyLock<BlockItemId> = &AMETHYST_BLOCK;
    pub const BUDDING_AMETHYST: &'static LazyLock<BlockItemId> = &BUDDING_AMETHYST;
    pub const IRON_BLOCK: &'static LazyLock<BlockItemId> = &IRON_BLOCK;
    pub const GOLD_BLOCK: &'static LazyLock<BlockItemId> = &GOLD_BLOCK;
    pub const DIAMOND_BLOCK: &'static LazyLock<BlockItemId> = &DIAMOND_BLOCK;
    pub const NETHERITE_BLOCK: &'static LazyLock<BlockItemId> = &NETHERITE_BLOCK;
    pub const OAK_LOG: &'static LazyLock<BlockItemId> = &OAK_LOG;
    pub const SPRUCE_LOG: &'static LazyLock<BlockItemId> = &SPRUCE_LOG;
    pub const BIRCH_LOG: &'static LazyLock<BlockItemId> = &BIRCH_LOG;
    pub const JUNGLE_LOG: &'static LazyLock<BlockItemId> = &JUNGLE_LOG;
    pub const ACACIA_LOG: &'static LazyLock<BlockItemId> = &ACACIA_LOG;
    pub const CHERRY_LOG: &'static LazyLock<BlockItemId> = &CHERRY_LOG;
    pub const PALE_OAK_LOG: &'static LazyLock<BlockItemId> = &PALE_OAK_LOG;
    pub const DARK_OAK_LOG: &'static LazyLock<BlockItemId> = &DARK_OAK_LOG;
    pub const MANGROVE_LOG: &'static LazyLock<BlockItemId> = &MANGROVE_LOG;
    pub const POPLAR_LOG: &'static LazyLock<BlockItemId> = &POPLAR_LOG;
    pub const MANGROVE_ROOTS: &'static LazyLock<BlockItemId> = &MANGROVE_ROOTS;
    pub const MUDDY_MANGROVE_ROOTS: &'static LazyLock<BlockItemId> = &MUDDY_MANGROVE_ROOTS;
    pub const CRIMSON_STEM: &'static LazyLock<BlockItemId> = &CRIMSON_STEM;
    pub const WARPED_STEM: &'static LazyLock<BlockItemId> = &WARPED_STEM;
    pub const BAMBOO_BLOCK: &'static LazyLock<BlockItemId> = &BAMBOO_BLOCK;
    pub const STRIPPED_OAK_LOG: &'static LazyLock<BlockItemId> = &STRIPPED_OAK_LOG;
    pub const STRIPPED_SPRUCE_LOG: &'static LazyLock<BlockItemId> = &STRIPPED_SPRUCE_LOG;
    pub const STRIPPED_BIRCH_LOG: &'static LazyLock<BlockItemId> = &STRIPPED_BIRCH_LOG;
    pub const STRIPPED_JUNGLE_LOG: &'static LazyLock<BlockItemId> = &STRIPPED_JUNGLE_LOG;
    pub const STRIPPED_ACACIA_LOG: &'static LazyLock<BlockItemId> = &STRIPPED_ACACIA_LOG;
    pub const STRIPPED_CHERRY_LOG: &'static LazyLock<BlockItemId> = &STRIPPED_CHERRY_LOG;
    pub const STRIPPED_DARK_OAK_LOG: &'static LazyLock<BlockItemId> = &STRIPPED_DARK_OAK_LOG;
    pub const STRIPPED_PALE_OAK_LOG: &'static LazyLock<BlockItemId> = &STRIPPED_PALE_OAK_LOG;
    pub const STRIPPED_MANGROVE_LOG: &'static LazyLock<BlockItemId> = &STRIPPED_MANGROVE_LOG;
    pub const STRIPPED_POPLAR_LOG: &'static LazyLock<BlockItemId> = &STRIPPED_POPLAR_LOG;
    pub const STRIPPED_CRIMSON_STEM: &'static LazyLock<BlockItemId> = &STRIPPED_CRIMSON_STEM;
    pub const STRIPPED_WARPED_STEM: &'static LazyLock<BlockItemId> = &STRIPPED_WARPED_STEM;
    pub const STRIPPED_OAK_WOOD: &'static LazyLock<BlockItemId> = &STRIPPED_OAK_WOOD;
    pub const STRIPPED_SPRUCE_WOOD: &'static LazyLock<BlockItemId> = &STRIPPED_SPRUCE_WOOD;
    pub const STRIPPED_BIRCH_WOOD: &'static LazyLock<BlockItemId> = &STRIPPED_BIRCH_WOOD;
    pub const STRIPPED_JUNGLE_WOOD: &'static LazyLock<BlockItemId> = &STRIPPED_JUNGLE_WOOD;
    pub const STRIPPED_ACACIA_WOOD: &'static LazyLock<BlockItemId> = &STRIPPED_ACACIA_WOOD;
    pub const STRIPPED_CHERRY_WOOD: &'static LazyLock<BlockItemId> = &STRIPPED_CHERRY_WOOD;
    pub const STRIPPED_DARK_OAK_WOOD: &'static LazyLock<BlockItemId> = &STRIPPED_DARK_OAK_WOOD;
    pub const STRIPPED_PALE_OAK_WOOD: &'static LazyLock<BlockItemId> = &STRIPPED_PALE_OAK_WOOD;
    pub const STRIPPED_MANGROVE_WOOD: &'static LazyLock<BlockItemId> = &STRIPPED_MANGROVE_WOOD;
    pub const STRIPPED_POPLAR_WOOD: &'static LazyLock<BlockItemId> = &STRIPPED_POPLAR_WOOD;
    pub const STRIPPED_CRIMSON_HYPHAE: &'static LazyLock<BlockItemId> = &STRIPPED_CRIMSON_HYPHAE;
    pub const STRIPPED_WARPED_HYPHAE: &'static LazyLock<BlockItemId> = &STRIPPED_WARPED_HYPHAE;
    pub const STRIPPED_BAMBOO_BLOCK: &'static LazyLock<BlockItemId> = &STRIPPED_BAMBOO_BLOCK;
    pub const OAK_WOOD: &'static LazyLock<BlockItemId> = &OAK_WOOD;
    pub const SPRUCE_WOOD: &'static LazyLock<BlockItemId> = &SPRUCE_WOOD;
    pub const BIRCH_WOOD: &'static LazyLock<BlockItemId> = &BIRCH_WOOD;
    pub const JUNGLE_WOOD: &'static LazyLock<BlockItemId> = &JUNGLE_WOOD;
    pub const ACACIA_WOOD: &'static LazyLock<BlockItemId> = &ACACIA_WOOD;
    pub const CHERRY_WOOD: &'static LazyLock<BlockItemId> = &CHERRY_WOOD;
    pub const PALE_OAK_WOOD: &'static LazyLock<BlockItemId> = &PALE_OAK_WOOD;
    pub const DARK_OAK_WOOD: &'static LazyLock<BlockItemId> = &DARK_OAK_WOOD;
    pub const MANGROVE_WOOD: &'static LazyLock<BlockItemId> = &MANGROVE_WOOD;
    pub const POPLAR_WOOD: &'static LazyLock<BlockItemId> = &POPLAR_WOOD;
    pub const CRIMSON_HYPHAE: &'static LazyLock<BlockItemId> = &CRIMSON_HYPHAE;
    pub const WARPED_HYPHAE: &'static LazyLock<BlockItemId> = &WARPED_HYPHAE;
    pub const OAK_LEAVES: &'static LazyLock<BlockItemId> = &OAK_LEAVES;
    pub const SPRUCE_LEAVES: &'static LazyLock<BlockItemId> = &SPRUCE_LEAVES;
    pub const BIRCH_LEAVES: &'static LazyLock<BlockItemId> = &BIRCH_LEAVES;
    pub const JUNGLE_LEAVES: &'static LazyLock<BlockItemId> = &JUNGLE_LEAVES;
    pub const ACACIA_LEAVES: &'static LazyLock<BlockItemId> = &ACACIA_LEAVES;
    pub const CHERRY_LEAVES: &'static LazyLock<BlockItemId> = &CHERRY_LEAVES;
    pub const DARK_OAK_LEAVES: &'static LazyLock<BlockItemId> = &DARK_OAK_LEAVES;
    pub const PALE_OAK_LEAVES: &'static LazyLock<BlockItemId> = &PALE_OAK_LEAVES;
    pub const MANGROVE_LEAVES: &'static LazyLock<BlockItemId> = &MANGROVE_LEAVES;
    pub const RED_POPLAR_LEAVES: &'static LazyLock<BlockItemId> = &RED_POPLAR_LEAVES;
    pub const ORANGE_POPLAR_LEAVES: &'static LazyLock<BlockItemId> = &ORANGE_POPLAR_LEAVES;
    pub const YELLOW_POPLAR_LEAVES: &'static LazyLock<BlockItemId> = &YELLOW_POPLAR_LEAVES;
    pub const AZALEA_LEAVES: &'static LazyLock<BlockItemId> = &AZALEA_LEAVES;
    pub const FLOWERING_AZALEA_LEAVES: &'static LazyLock<BlockItemId> = &FLOWERING_AZALEA_LEAVES;
    pub const SPONGE: &'static LazyLock<BlockItemId> = &SPONGE;
    pub const WET_SPONGE: &'static LazyLock<BlockItemId> = &WET_SPONGE;
    pub const GLASS: &'static LazyLock<BlockItemId> = &GLASS;
    pub const TINTED_GLASS: &'static LazyLock<BlockItemId> = &TINTED_GLASS;
    pub const LAPIS_BLOCK: &'static LazyLock<BlockItemId> = &LAPIS_BLOCK;
    pub const SANDSTONE: &'static LazyLock<BlockItemId> = &SANDSTONE;
    pub const CHISELED_SANDSTONE: &'static LazyLock<BlockItemId> = &CHISELED_SANDSTONE;
    pub const CUT_SANDSTONE: &'static LazyLock<BlockItemId> = &CUT_SANDSTONE;
    pub const COBWEB: &'static LazyLock<BlockItemId> = &COBWEB;
    pub const SHORT_GRASS: &'static LazyLock<BlockItemId> = &SHORT_GRASS;
    pub const FERN: &'static LazyLock<BlockItemId> = &FERN;
    pub const BUSH: &'static LazyLock<BlockItemId> = &BUSH;
    pub const RED_SHRUB: &'static LazyLock<BlockItemId> = &RED_SHRUB;
    pub const AZALEA: &'static LazyLock<BlockItemId> = &AZALEA;
    pub const FLOWERING_AZALEA: &'static LazyLock<BlockItemId> = &FLOWERING_AZALEA;
    pub const DEAD_BUSH: &'static LazyLock<BlockItemId> = &DEAD_BUSH;
    pub const FIREFLY_BUSH: &'static LazyLock<BlockItemId> = &FIREFLY_BUSH;
    pub const SHORT_DRY_GRASS: &'static LazyLock<BlockItemId> = &SHORT_DRY_GRASS;
    pub const TALL_DRY_GRASS: &'static LazyLock<BlockItemId> = &TALL_DRY_GRASS;
    pub const SEAGRASS: &'static LazyLock<BlockItemId> = &SEAGRASS;
    pub const SEA_PICKLE: &'static LazyLock<BlockItemId> = &SEA_PICKLE;
    pub const DANDELION: &'static LazyLock<BlockItemId> = &DANDELION;
    pub const GOLDEN_DANDELION: &'static LazyLock<BlockItemId> = &GOLDEN_DANDELION;
    pub const OPEN_EYEBLOSSOM: &'static LazyLock<BlockItemId> = &OPEN_EYEBLOSSOM;
    pub const CLOSED_EYEBLOSSOM: &'static LazyLock<BlockItemId> = &CLOSED_EYEBLOSSOM;
    pub const POPPY: &'static LazyLock<BlockItemId> = &POPPY;
    pub const BLUE_ORCHID: &'static LazyLock<BlockItemId> = &BLUE_ORCHID;
    pub const ALLIUM: &'static LazyLock<BlockItemId> = &ALLIUM;
    pub const AZURE_BLUET: &'static LazyLock<BlockItemId> = &AZURE_BLUET;
    pub const RED_TULIP: &'static LazyLock<BlockItemId> = &RED_TULIP;
    pub const ORANGE_TULIP: &'static LazyLock<BlockItemId> = &ORANGE_TULIP;
    pub const WHITE_TULIP: &'static LazyLock<BlockItemId> = &WHITE_TULIP;
    pub const PINK_TULIP: &'static LazyLock<BlockItemId> = &PINK_TULIP;
    pub const OXEYE_DAISY: &'static LazyLock<BlockItemId> = &OXEYE_DAISY;
    pub const CORNFLOWER: &'static LazyLock<BlockItemId> = &CORNFLOWER;
    pub const LILY_OF_THE_VALLEY: &'static LazyLock<BlockItemId> = &LILY_OF_THE_VALLEY;
    pub const WITHER_ROSE: &'static LazyLock<BlockItemId> = &WITHER_ROSE;
    pub const TORCHFLOWER: &'static LazyLock<BlockItemId> = &TORCHFLOWER;
    pub const PITCHER_PLANT: &'static LazyLock<BlockItemId> = &PITCHER_PLANT;
    pub const SPORE_BLOSSOM: &'static LazyLock<BlockItemId> = &SPORE_BLOSSOM;
    pub const BROWN_MUSHROOM: &'static LazyLock<BlockItemId> = &BROWN_MUSHROOM;
    pub const RED_MUSHROOM: &'static LazyLock<BlockItemId> = &RED_MUSHROOM;
    pub const SHELF_MUSHROOM: &'static LazyLock<BlockItemId> = &SHELF_MUSHROOM;
    pub const CRIMSON_FUNGUS: &'static LazyLock<BlockItemId> = &CRIMSON_FUNGUS;
    pub const WARPED_FUNGUS: &'static LazyLock<BlockItemId> = &WARPED_FUNGUS;
    pub const CRIMSON_ROOTS: &'static LazyLock<BlockItemId> = &CRIMSON_ROOTS;
    pub const WARPED_ROOTS: &'static LazyLock<BlockItemId> = &WARPED_ROOTS;
    pub const NETHER_SPROUTS: &'static LazyLock<BlockItemId> = &NETHER_SPROUTS;
    pub const WEEPING_VINES: &'static LazyLock<BlockItemId> = &WEEPING_VINES;
    pub const TWISTING_VINES: &'static LazyLock<BlockItemId> = &TWISTING_VINES;
    pub const SUGAR_CANE: &'static LazyLock<BlockItemId> = &SUGAR_CANE;
    pub const KELP: &'static LazyLock<BlockItemId> = &KELP;
    pub const PINK_PETALS: &'static LazyLock<BlockItemId> = &PINK_PETALS;
    pub const WILDFLOWERS: &'static LazyLock<BlockItemId> = &WILDFLOWERS;
    pub const LEAF_LITTER: &'static LazyLock<BlockItemId> = &LEAF_LITTER;
    pub const MOSS_CARPET: &'static LazyLock<BlockItemId> = &MOSS_CARPET;
    pub const MOSS_BLOCK: &'static LazyLock<BlockItemId> = &MOSS_BLOCK;
    pub const PALE_MOSS_CARPET: &'static LazyLock<BlockItemId> = &PALE_MOSS_CARPET;
    pub const PALE_HANGING_MOSS: &'static LazyLock<BlockItemId> = &PALE_HANGING_MOSS;
    pub const PALE_MOSS_BLOCK: &'static LazyLock<BlockItemId> = &PALE_MOSS_BLOCK;
    pub const HANGING_ROOTS: &'static LazyLock<BlockItemId> = &HANGING_ROOTS;
    pub const BIG_DRIPLEAF: &'static LazyLock<BlockItemId> = &BIG_DRIPLEAF;
    pub const SMALL_DRIPLEAF: &'static LazyLock<BlockItemId> = &SMALL_DRIPLEAF;
    pub const BAMBOO: &'static LazyLock<BlockItemId> = &BAMBOO;
    pub const OAK_SLAB: &'static LazyLock<BlockItemId> = &OAK_SLAB;
    pub const SPRUCE_SLAB: &'static LazyLock<BlockItemId> = &SPRUCE_SLAB;
    pub const BIRCH_SLAB: &'static LazyLock<BlockItemId> = &BIRCH_SLAB;
    pub const JUNGLE_SLAB: &'static LazyLock<BlockItemId> = &JUNGLE_SLAB;
    pub const ACACIA_SLAB: &'static LazyLock<BlockItemId> = &ACACIA_SLAB;
    pub const CHERRY_SLAB: &'static LazyLock<BlockItemId> = &CHERRY_SLAB;
    pub const DARK_OAK_SLAB: &'static LazyLock<BlockItemId> = &DARK_OAK_SLAB;
    pub const PALE_OAK_SLAB: &'static LazyLock<BlockItemId> = &PALE_OAK_SLAB;
    pub const MANGROVE_SLAB: &'static LazyLock<BlockItemId> = &MANGROVE_SLAB;
    pub const POPLAR_SLAB: &'static LazyLock<BlockItemId> = &POPLAR_SLAB;
    pub const BAMBOO_SLAB: &'static LazyLock<BlockItemId> = &BAMBOO_SLAB;
    pub const BAMBOO_MOSAIC_SLAB: &'static LazyLock<BlockItemId> = &BAMBOO_MOSAIC_SLAB;
    pub const CRIMSON_SLAB: &'static LazyLock<BlockItemId> = &CRIMSON_SLAB;
    pub const WARPED_SLAB: &'static LazyLock<BlockItemId> = &WARPED_SLAB;
    pub const STONE_SLAB: &'static LazyLock<BlockItemId> = &STONE_SLAB;
    pub const SMOOTH_STONE_SLAB: &'static LazyLock<BlockItemId> = &SMOOTH_STONE_SLAB;
    pub const SANDSTONE_SLAB: &'static LazyLock<BlockItemId> = &SANDSTONE_SLAB;
    pub const CUT_SANDSTONE_SLAB: &'static LazyLock<BlockItemId> = &CUT_SANDSTONE_SLAB;
    pub const PETRIFIED_OAK_SLAB: &'static LazyLock<BlockItemId> = &PETRIFIED_OAK_SLAB;
    pub const COBBLESTONE_SLAB: &'static LazyLock<BlockItemId> = &COBBLESTONE_SLAB;
    pub const BRICK_SLAB: &'static LazyLock<BlockItemId> = &BRICK_SLAB;
    pub const STONE_BRICK_SLAB: &'static LazyLock<BlockItemId> = &STONE_BRICK_SLAB;
    pub const MUD_BRICK_SLAB: &'static LazyLock<BlockItemId> = &MUD_BRICK_SLAB;
    pub const NETHER_BRICK_SLAB: &'static LazyLock<BlockItemId> = &NETHER_BRICK_SLAB;
    pub const QUARTZ_SLAB: &'static LazyLock<BlockItemId> = &QUARTZ_SLAB;
    pub const RED_SANDSTONE_SLAB: &'static LazyLock<BlockItemId> = &RED_SANDSTONE_SLAB;
    pub const CUT_RED_SANDSTONE_SLAB: &'static LazyLock<BlockItemId> = &CUT_RED_SANDSTONE_SLAB;
    pub const PURPUR_SLAB: &'static LazyLock<BlockItemId> = &PURPUR_SLAB;
    pub const PRISMARINE_SLAB: &'static LazyLock<BlockItemId> = &PRISMARINE_SLAB;
    pub const PRISMARINE_BRICK_SLAB: &'static LazyLock<BlockItemId> = &PRISMARINE_BRICK_SLAB;
    pub const DARK_PRISMARINE_SLAB: &'static LazyLock<BlockItemId> = &DARK_PRISMARINE_SLAB;
    pub const SMOOTH_QUARTZ: &'static LazyLock<BlockItemId> = &SMOOTH_QUARTZ;
    pub const SMOOTH_RED_SANDSTONE: &'static LazyLock<BlockItemId> = &SMOOTH_RED_SANDSTONE;
    pub const SMOOTH_SANDSTONE: &'static LazyLock<BlockItemId> = &SMOOTH_SANDSTONE;
    pub const SMOOTH_STONE: &'static LazyLock<BlockItemId> = &SMOOTH_STONE;
    pub const BRICKS: &'static LazyLock<BlockItemId> = &BRICKS;
    pub const ACACIA_SHELF: &'static LazyLock<BlockItemId> = &ACACIA_SHELF;
    pub const BAMBOO_SHELF: &'static LazyLock<BlockItemId> = &BAMBOO_SHELF;
    pub const BIRCH_SHELF: &'static LazyLock<BlockItemId> = &BIRCH_SHELF;
    pub const CHERRY_SHELF: &'static LazyLock<BlockItemId> = &CHERRY_SHELF;
    pub const CRIMSON_SHELF: &'static LazyLock<BlockItemId> = &CRIMSON_SHELF;
    pub const DARK_OAK_SHELF: &'static LazyLock<BlockItemId> = &DARK_OAK_SHELF;
    pub const JUNGLE_SHELF: &'static LazyLock<BlockItemId> = &JUNGLE_SHELF;
    pub const MANGROVE_SHELF: &'static LazyLock<BlockItemId> = &MANGROVE_SHELF;
    pub const POPLAR_SHELF: &'static LazyLock<BlockItemId> = &POPLAR_SHELF;
    pub const OAK_SHELF: &'static LazyLock<BlockItemId> = &OAK_SHELF;
    pub const PALE_OAK_SHELF: &'static LazyLock<BlockItemId> = &PALE_OAK_SHELF;
    pub const SPRUCE_SHELF: &'static LazyLock<BlockItemId> = &SPRUCE_SHELF;
    pub const WARPED_SHELF: &'static LazyLock<BlockItemId> = &WARPED_SHELF;
    pub const BOOKSHELF: &'static LazyLock<BlockItemId> = &BOOKSHELF;
    pub const CHISELED_BOOKSHELF: &'static LazyLock<BlockItemId> = &CHISELED_BOOKSHELF;
    pub const DECORATED_POT: &'static LazyLock<BlockItemId> = &DECORATED_POT;
    pub const MOSSY_COBBLESTONE: &'static LazyLock<BlockItemId> = &MOSSY_COBBLESTONE;
    pub const OBSIDIAN: &'static LazyLock<BlockItemId> = &OBSIDIAN;
    pub const TORCH: &'static LazyLock<BlockItemId> = &TORCH;
    pub const END_ROD: &'static LazyLock<BlockItemId> = &END_ROD;
    pub const CHORUS_PLANT: &'static LazyLock<BlockItemId> = &CHORUS_PLANT;
    pub const CHORUS_FLOWER: &'static LazyLock<BlockItemId> = &CHORUS_FLOWER;
    pub const PURPUR_BLOCK: &'static LazyLock<BlockItemId> = &PURPUR_BLOCK;
    pub const PURPUR_PILLAR: &'static LazyLock<BlockItemId> = &PURPUR_PILLAR;
    pub const PURPUR_STAIRS: &'static LazyLock<BlockItemId> = &PURPUR_STAIRS;
    pub const SPAWNER: &'static LazyLock<BlockItemId> = &SPAWNER;
    pub const CREAKING_HEART: &'static LazyLock<BlockItemId> = &CREAKING_HEART;
    pub const CHEST: &'static LazyLock<BlockItemId> = &CHEST;
    pub const CRAFTING_TABLE: &'static LazyLock<BlockItemId> = &CRAFTING_TABLE;
    pub const FARMLAND: &'static LazyLock<BlockItemId> = &FARMLAND;
    pub const FURNACE: &'static LazyLock<BlockItemId> = &FURNACE;
    pub const LADDER: &'static LazyLock<BlockItemId> = &LADDER;
    pub const COBBLESTONE_STAIRS: &'static LazyLock<BlockItemId> = &COBBLESTONE_STAIRS;
    pub const SNOW: &'static LazyLock<BlockItemId> = &SNOW;
    pub const ICE: &'static LazyLock<BlockItemId> = &ICE;
    pub const SNOW_BLOCK: &'static LazyLock<BlockItemId> = &SNOW_BLOCK;
    pub const CACTUS: &'static LazyLock<BlockItemId> = &CACTUS;
    pub const CACTUS_FLOWER: &'static LazyLock<BlockItemId> = &CACTUS_FLOWER;
    pub const CLAY: &'static LazyLock<BlockItemId> = &CLAY;
    pub const JUKEBOX: &'static LazyLock<BlockItemId> = &JUKEBOX;
    pub const OAK_FENCE: &'static LazyLock<BlockItemId> = &OAK_FENCE;
    pub const SPRUCE_FENCE: &'static LazyLock<BlockItemId> = &SPRUCE_FENCE;
    pub const BIRCH_FENCE: &'static LazyLock<BlockItemId> = &BIRCH_FENCE;
    pub const JUNGLE_FENCE: &'static LazyLock<BlockItemId> = &JUNGLE_FENCE;
    pub const ACACIA_FENCE: &'static LazyLock<BlockItemId> = &ACACIA_FENCE;
    pub const CHERRY_FENCE: &'static LazyLock<BlockItemId> = &CHERRY_FENCE;
    pub const DARK_OAK_FENCE: &'static LazyLock<BlockItemId> = &DARK_OAK_FENCE;
    pub const PALE_OAK_FENCE: &'static LazyLock<BlockItemId> = &PALE_OAK_FENCE;
    pub const MANGROVE_FENCE: &'static LazyLock<BlockItemId> = &MANGROVE_FENCE;
    pub const POPLAR_FENCE: &'static LazyLock<BlockItemId> = &POPLAR_FENCE;
    pub const BAMBOO_FENCE: &'static LazyLock<BlockItemId> = &BAMBOO_FENCE;
    pub const CRIMSON_FENCE: &'static LazyLock<BlockItemId> = &CRIMSON_FENCE;
    pub const WARPED_FENCE: &'static LazyLock<BlockItemId> = &WARPED_FENCE;
    pub const CARVED_PUMPKIN: &'static LazyLock<BlockItemId> = &CARVED_PUMPKIN;
    pub const JACK_O_LANTERN: &'static LazyLock<BlockItemId> = &JACK_O_LANTERN;
    pub const NETHERRACK: &'static LazyLock<BlockItemId> = &NETHERRACK;
    pub const SOUL_SAND: &'static LazyLock<BlockItemId> = &SOUL_SAND;
    pub const SOUL_SOIL: &'static LazyLock<BlockItemId> = &SOUL_SOIL;
    pub const BASALT: &'static LazyLock<BlockItemId> = &BASALT;
    pub const POLISHED_BASALT: &'static LazyLock<BlockItemId> = &POLISHED_BASALT;
    pub const SMOOTH_BASALT: &'static LazyLock<BlockItemId> = &SMOOTH_BASALT;
    pub const SOUL_TORCH: &'static LazyLock<BlockItemId> = &SOUL_TORCH;
    pub const COPPER_TORCH: &'static LazyLock<BlockItemId> = &COPPER_TORCH;
    pub const GLOWSTONE: &'static LazyLock<BlockItemId> = &GLOWSTONE;
    pub const INFESTED_STONE: &'static LazyLock<BlockItemId> = &INFESTED_STONE;
    pub const INFESTED_COBBLESTONE: &'static LazyLock<BlockItemId> = &INFESTED_COBBLESTONE;
    pub const INFESTED_STONE_BRICKS: &'static LazyLock<BlockItemId> = &INFESTED_STONE_BRICKS;
    pub const INFESTED_MOSSY_STONE_BRICKS: &'static LazyLock<BlockItemId> =
        &INFESTED_MOSSY_STONE_BRICKS;
    pub const INFESTED_CRACKED_STONE_BRICKS: &'static LazyLock<BlockItemId> =
        &INFESTED_CRACKED_STONE_BRICKS;
    pub const INFESTED_CHISELED_STONE_BRICKS: &'static LazyLock<BlockItemId> =
        &INFESTED_CHISELED_STONE_BRICKS;
    pub const INFESTED_DEEPSLATE: &'static LazyLock<BlockItemId> = &INFESTED_DEEPSLATE;
    pub const STONE_BRICKS: &'static LazyLock<BlockItemId> = &STONE_BRICKS;
    pub const MOSSY_STONE_BRICKS: &'static LazyLock<BlockItemId> = &MOSSY_STONE_BRICKS;
    pub const CRACKED_STONE_BRICKS: &'static LazyLock<BlockItemId> = &CRACKED_STONE_BRICKS;
    pub const CHISELED_STONE_BRICKS: &'static LazyLock<BlockItemId> = &CHISELED_STONE_BRICKS;
    pub const PACKED_MUD: &'static LazyLock<BlockItemId> = &PACKED_MUD;
    pub const MUD_BRICKS: &'static LazyLock<BlockItemId> = &MUD_BRICKS;
    pub const DEEPSLATE_BRICKS: &'static LazyLock<BlockItemId> = &DEEPSLATE_BRICKS;
    pub const CRACKED_DEEPSLATE_BRICKS: &'static LazyLock<BlockItemId> = &CRACKED_DEEPSLATE_BRICKS;
    pub const DEEPSLATE_TILES: &'static LazyLock<BlockItemId> = &DEEPSLATE_TILES;
    pub const CRACKED_DEEPSLATE_TILES: &'static LazyLock<BlockItemId> = &CRACKED_DEEPSLATE_TILES;
    pub const CHISELED_DEEPSLATE: &'static LazyLock<BlockItemId> = &CHISELED_DEEPSLATE;
    pub const REINFORCED_DEEPSLATE: &'static LazyLock<BlockItemId> = &REINFORCED_DEEPSLATE;
    pub const BROWN_MUSHROOM_BLOCK: &'static LazyLock<BlockItemId> = &BROWN_MUSHROOM_BLOCK;
    pub const RED_MUSHROOM_BLOCK: &'static LazyLock<BlockItemId> = &RED_MUSHROOM_BLOCK;
    pub const MUSHROOM_STEM: &'static LazyLock<BlockItemId> = &MUSHROOM_STEM;
    pub const IRON_BARS: &'static LazyLock<BlockItemId> = &IRON_BARS;
    pub const IRON_CHAIN: &'static LazyLock<BlockItemId> = &IRON_CHAIN;
    pub const GLASS_PANE: &'static LazyLock<BlockItemId> = &GLASS_PANE;
    pub const VINE: &'static LazyLock<BlockItemId> = &VINE;
    pub const GLOW_LICHEN: &'static LazyLock<BlockItemId> = &GLOW_LICHEN;
    pub const RESIN_CLUMP: &'static LazyLock<BlockItemId> = &RESIN_CLUMP;
    pub const RESIN_BLOCK: &'static LazyLock<BlockItemId> = &RESIN_BLOCK;
    pub const RESIN_BRICKS: &'static LazyLock<BlockItemId> = &RESIN_BRICKS;
    pub const RESIN_BRICK_STAIRS: &'static LazyLock<BlockItemId> = &RESIN_BRICK_STAIRS;
    pub const RESIN_BRICK_SLAB: &'static LazyLock<BlockItemId> = &RESIN_BRICK_SLAB;
    pub const RESIN_BRICK_WALL: &'static LazyLock<BlockItemId> = &RESIN_BRICK_WALL;
    pub const CHISELED_RESIN_BRICKS: &'static LazyLock<BlockItemId> = &CHISELED_RESIN_BRICKS;
    pub const BRICK_STAIRS: &'static LazyLock<BlockItemId> = &BRICK_STAIRS;
    pub const STONE_BRICK_STAIRS: &'static LazyLock<BlockItemId> = &STONE_BRICK_STAIRS;
    pub const MUD_BRICK_STAIRS: &'static LazyLock<BlockItemId> = &MUD_BRICK_STAIRS;
    pub const MYCELIUM: &'static LazyLock<BlockItemId> = &MYCELIUM;
    pub const LILY_PAD: &'static LazyLock<BlockItemId> = &LILY_PAD;
    pub const NETHER_BRICKS: &'static LazyLock<BlockItemId> = &NETHER_BRICKS;
    pub const CRACKED_NETHER_BRICKS: &'static LazyLock<BlockItemId> = &CRACKED_NETHER_BRICKS;
    pub const CHISELED_NETHER_BRICKS: &'static LazyLock<BlockItemId> = &CHISELED_NETHER_BRICKS;
    pub const NETHER_BRICK_FENCE: &'static LazyLock<BlockItemId> = &NETHER_BRICK_FENCE;
    pub const NETHER_BRICK_STAIRS: &'static LazyLock<BlockItemId> = &NETHER_BRICK_STAIRS;
    pub const SCULK: &'static LazyLock<BlockItemId> = &SCULK;
    pub const SCULK_VEIN: &'static LazyLock<BlockItemId> = &SCULK_VEIN;
    pub const SCULK_CATALYST: &'static LazyLock<BlockItemId> = &SCULK_CATALYST;
    pub const SCULK_SHRIEKER: &'static LazyLock<BlockItemId> = &SCULK_SHRIEKER;
    pub const ENCHANTING_TABLE: &'static LazyLock<BlockItemId> = &ENCHANTING_TABLE;
    pub const END_PORTAL_FRAME: &'static LazyLock<BlockItemId> = &END_PORTAL_FRAME;
    pub const END_STONE: &'static LazyLock<BlockItemId> = &END_STONE;
    pub const END_STONE_BRICKS: &'static LazyLock<BlockItemId> = &END_STONE_BRICKS;
    pub const DRAGON_EGG: &'static LazyLock<BlockItemId> = &DRAGON_EGG;
    pub const SANDSTONE_STAIRS: &'static LazyLock<BlockItemId> = &SANDSTONE_STAIRS;
    pub const ENDER_CHEST: &'static LazyLock<BlockItemId> = &ENDER_CHEST;
    pub const EMERALD_BLOCK: &'static LazyLock<BlockItemId> = &EMERALD_BLOCK;
    pub const OAK_STAIRS: &'static LazyLock<BlockItemId> = &OAK_STAIRS;
    pub const SPRUCE_STAIRS: &'static LazyLock<BlockItemId> = &SPRUCE_STAIRS;
    pub const BIRCH_STAIRS: &'static LazyLock<BlockItemId> = &BIRCH_STAIRS;
    pub const JUNGLE_STAIRS: &'static LazyLock<BlockItemId> = &JUNGLE_STAIRS;
    pub const ACACIA_STAIRS: &'static LazyLock<BlockItemId> = &ACACIA_STAIRS;
    pub const CHERRY_STAIRS: &'static LazyLock<BlockItemId> = &CHERRY_STAIRS;
    pub const DARK_OAK_STAIRS: &'static LazyLock<BlockItemId> = &DARK_OAK_STAIRS;
    pub const PALE_OAK_STAIRS: &'static LazyLock<BlockItemId> = &PALE_OAK_STAIRS;
    pub const MANGROVE_STAIRS: &'static LazyLock<BlockItemId> = &MANGROVE_STAIRS;
    pub const POPLAR_STAIRS: &'static LazyLock<BlockItemId> = &POPLAR_STAIRS;
    pub const BAMBOO_STAIRS: &'static LazyLock<BlockItemId> = &BAMBOO_STAIRS;
    pub const BAMBOO_MOSAIC_STAIRS: &'static LazyLock<BlockItemId> = &BAMBOO_MOSAIC_STAIRS;
    pub const CRIMSON_STAIRS: &'static LazyLock<BlockItemId> = &CRIMSON_STAIRS;
    pub const WARPED_STAIRS: &'static LazyLock<BlockItemId> = &WARPED_STAIRS;
    pub const COMMAND_BLOCK: &'static LazyLock<BlockItemId> = &COMMAND_BLOCK;
    pub const BEACON: &'static LazyLock<BlockItemId> = &BEACON;
    pub const COBBLESTONE_WALL: &'static LazyLock<BlockItemId> = &COBBLESTONE_WALL;
    pub const MOSSY_COBBLESTONE_WALL: &'static LazyLock<BlockItemId> = &MOSSY_COBBLESTONE_WALL;
    pub const BRICK_WALL: &'static LazyLock<BlockItemId> = &BRICK_WALL;
    pub const PRISMARINE_WALL: &'static LazyLock<BlockItemId> = &PRISMARINE_WALL;
    pub const RED_SANDSTONE_WALL: &'static LazyLock<BlockItemId> = &RED_SANDSTONE_WALL;
    pub const MOSSY_STONE_BRICK_WALL: &'static LazyLock<BlockItemId> = &MOSSY_STONE_BRICK_WALL;
    pub const GRANITE_WALL: &'static LazyLock<BlockItemId> = &GRANITE_WALL;
    pub const STONE_BRICK_WALL: &'static LazyLock<BlockItemId> = &STONE_BRICK_WALL;
    pub const MUD_BRICK_WALL: &'static LazyLock<BlockItemId> = &MUD_BRICK_WALL;
    pub const NETHER_BRICK_WALL: &'static LazyLock<BlockItemId> = &NETHER_BRICK_WALL;
    pub const ANDESITE_WALL: &'static LazyLock<BlockItemId> = &ANDESITE_WALL;
    pub const RED_NETHER_BRICK_WALL: &'static LazyLock<BlockItemId> = &RED_NETHER_BRICK_WALL;
    pub const SANDSTONE_WALL: &'static LazyLock<BlockItemId> = &SANDSTONE_WALL;
    pub const END_STONE_BRICK_WALL: &'static LazyLock<BlockItemId> = &END_STONE_BRICK_WALL;
    pub const DIORITE_WALL: &'static LazyLock<BlockItemId> = &DIORITE_WALL;
    pub const BLACKSTONE_WALL: &'static LazyLock<BlockItemId> = &BLACKSTONE_WALL;
    pub const POLISHED_BLACKSTONE_WALL: &'static LazyLock<BlockItemId> = &POLISHED_BLACKSTONE_WALL;
    pub const POLISHED_BLACKSTONE_BRICK_WALL: &'static LazyLock<BlockItemId> =
        &POLISHED_BLACKSTONE_BRICK_WALL;
    pub const COBBLED_DEEPSLATE_WALL: &'static LazyLock<BlockItemId> = &COBBLED_DEEPSLATE_WALL;
    pub const POLISHED_DEEPSLATE_WALL: &'static LazyLock<BlockItemId> = &POLISHED_DEEPSLATE_WALL;
    pub const DEEPSLATE_BRICK_WALL: &'static LazyLock<BlockItemId> = &DEEPSLATE_BRICK_WALL;
    pub const DEEPSLATE_TILE_WALL: &'static LazyLock<BlockItemId> = &DEEPSLATE_TILE_WALL;
    pub const ANVIL: &'static LazyLock<BlockItemId> = &ANVIL;
    pub const CHIPPED_ANVIL: &'static LazyLock<BlockItemId> = &CHIPPED_ANVIL;
    pub const DAMAGED_ANVIL: &'static LazyLock<BlockItemId> = &DAMAGED_ANVIL;
    pub const CHISELED_QUARTZ_BLOCK: &'static LazyLock<BlockItemId> = &CHISELED_QUARTZ_BLOCK;
    pub const QUARTZ_BLOCK: &'static LazyLock<BlockItemId> = &QUARTZ_BLOCK;
    pub const QUARTZ_BRICKS: &'static LazyLock<BlockItemId> = &QUARTZ_BRICKS;
    pub const QUARTZ_PILLAR: &'static LazyLock<BlockItemId> = &QUARTZ_PILLAR;
    pub const QUARTZ_STAIRS: &'static LazyLock<BlockItemId> = &QUARTZ_STAIRS;
    pub const BARRIER: &'static LazyLock<BlockItemId> = &BARRIER;
    pub const LIGHT: &'static LazyLock<BlockItemId> = &LIGHT;
    pub const HAY_BLOCK: &'static LazyLock<BlockItemId> = &HAY_BLOCK;
    pub const TERRACOTTA: &'static LazyLock<BlockItemId> = &TERRACOTTA;
    pub const PACKED_ICE: &'static LazyLock<BlockItemId> = &PACKED_ICE;
    pub const DIRT_PATH: &'static LazyLock<BlockItemId> = &DIRT_PATH;
    pub const SUNFLOWER: &'static LazyLock<BlockItemId> = &SUNFLOWER;
    pub const LILAC: &'static LazyLock<BlockItemId> = &LILAC;
    pub const ROSE_BUSH: &'static LazyLock<BlockItemId> = &ROSE_BUSH;
    pub const PEONY: &'static LazyLock<BlockItemId> = &PEONY;
    pub const TALL_GRASS: &'static LazyLock<BlockItemId> = &TALL_GRASS;
    pub const LARGE_FERN: &'static LazyLock<BlockItemId> = &LARGE_FERN;
    pub const PRISMARINE: &'static LazyLock<BlockItemId> = &PRISMARINE;
    pub const PRISMARINE_BRICKS: &'static LazyLock<BlockItemId> = &PRISMARINE_BRICKS;
    pub const DARK_PRISMARINE: &'static LazyLock<BlockItemId> = &DARK_PRISMARINE;
    pub const PRISMARINE_STAIRS: &'static LazyLock<BlockItemId> = &PRISMARINE_STAIRS;
    pub const PRISMARINE_BRICK_STAIRS: &'static LazyLock<BlockItemId> = &PRISMARINE_BRICK_STAIRS;
    pub const DARK_PRISMARINE_STAIRS: &'static LazyLock<BlockItemId> = &DARK_PRISMARINE_STAIRS;
    pub const SEA_LANTERN: &'static LazyLock<BlockItemId> = &SEA_LANTERN;
    pub const RED_SANDSTONE: &'static LazyLock<BlockItemId> = &RED_SANDSTONE;
    pub const CHISELED_RED_SANDSTONE: &'static LazyLock<BlockItemId> = &CHISELED_RED_SANDSTONE;
    pub const CUT_RED_SANDSTONE: &'static LazyLock<BlockItemId> = &CUT_RED_SANDSTONE;
    pub const RED_SANDSTONE_STAIRS: &'static LazyLock<BlockItemId> = &RED_SANDSTONE_STAIRS;
    pub const REPEATING_COMMAND_BLOCK: &'static LazyLock<BlockItemId> = &REPEATING_COMMAND_BLOCK;
    pub const CHAIN_COMMAND_BLOCK: &'static LazyLock<BlockItemId> = &CHAIN_COMMAND_BLOCK;
    pub const MAGMA_BLOCK: &'static LazyLock<BlockItemId> = &MAGMA_BLOCK;
    pub const NETHER_WART_BLOCK: &'static LazyLock<BlockItemId> = &NETHER_WART_BLOCK;
    pub const WARPED_WART_BLOCK: &'static LazyLock<BlockItemId> = &WARPED_WART_BLOCK;
    pub const RED_NETHER_BRICKS: &'static LazyLock<BlockItemId> = &RED_NETHER_BRICKS;
    pub const BONE_BLOCK: &'static LazyLock<BlockItemId> = &BONE_BLOCK;
    pub const STRUCTURE_VOID: &'static LazyLock<BlockItemId> = &STRUCTURE_VOID;
    pub const SHULKER_BOX: &'static LazyLock<BlockItemId> = &SHULKER_BOX;
    pub const TURTLE_EGG: &'static LazyLock<BlockItemId> = &TURTLE_EGG;
    pub const SNIFFER_EGG: &'static LazyLock<BlockItemId> = &SNIFFER_EGG;
    pub const DRIED_GHAST: &'static LazyLock<BlockItemId> = &DRIED_GHAST;
    pub const DEAD_TUBE_CORAL_BLOCK: &'static LazyLock<BlockItemId> = &DEAD_TUBE_CORAL_BLOCK;
    pub const DEAD_BRAIN_CORAL_BLOCK: &'static LazyLock<BlockItemId> = &DEAD_BRAIN_CORAL_BLOCK;
    pub const DEAD_BUBBLE_CORAL_BLOCK: &'static LazyLock<BlockItemId> = &DEAD_BUBBLE_CORAL_BLOCK;
    pub const DEAD_FIRE_CORAL_BLOCK: &'static LazyLock<BlockItemId> = &DEAD_FIRE_CORAL_BLOCK;
    pub const DEAD_HORN_CORAL_BLOCK: &'static LazyLock<BlockItemId> = &DEAD_HORN_CORAL_BLOCK;
    pub const TUBE_CORAL_BLOCK: &'static LazyLock<BlockItemId> = &TUBE_CORAL_BLOCK;
    pub const BRAIN_CORAL_BLOCK: &'static LazyLock<BlockItemId> = &BRAIN_CORAL_BLOCK;
    pub const BUBBLE_CORAL_BLOCK: &'static LazyLock<BlockItemId> = &BUBBLE_CORAL_BLOCK;
    pub const FIRE_CORAL_BLOCK: &'static LazyLock<BlockItemId> = &FIRE_CORAL_BLOCK;
    pub const HORN_CORAL_BLOCK: &'static LazyLock<BlockItemId> = &HORN_CORAL_BLOCK;
    pub const TUBE_CORAL: &'static LazyLock<BlockItemId> = &TUBE_CORAL;
    pub const BRAIN_CORAL: &'static LazyLock<BlockItemId> = &BRAIN_CORAL;
    pub const BUBBLE_CORAL: &'static LazyLock<BlockItemId> = &BUBBLE_CORAL;
    pub const FIRE_CORAL: &'static LazyLock<BlockItemId> = &FIRE_CORAL;
    pub const HORN_CORAL: &'static LazyLock<BlockItemId> = &HORN_CORAL;
    pub const DEAD_BRAIN_CORAL: &'static LazyLock<BlockItemId> = &DEAD_BRAIN_CORAL;
    pub const DEAD_BUBBLE_CORAL: &'static LazyLock<BlockItemId> = &DEAD_BUBBLE_CORAL;
    pub const DEAD_FIRE_CORAL: &'static LazyLock<BlockItemId> = &DEAD_FIRE_CORAL;
    pub const DEAD_HORN_CORAL: &'static LazyLock<BlockItemId> = &DEAD_HORN_CORAL;
    pub const DEAD_TUBE_CORAL: &'static LazyLock<BlockItemId> = &DEAD_TUBE_CORAL;
    pub const TUBE_CORAL_FAN: &'static LazyLock<BlockItemId> = &TUBE_CORAL_FAN;
    pub const BRAIN_CORAL_FAN: &'static LazyLock<BlockItemId> = &BRAIN_CORAL_FAN;
    pub const BUBBLE_CORAL_FAN: &'static LazyLock<BlockItemId> = &BUBBLE_CORAL_FAN;
    pub const FIRE_CORAL_FAN: &'static LazyLock<BlockItemId> = &FIRE_CORAL_FAN;
    pub const HORN_CORAL_FAN: &'static LazyLock<BlockItemId> = &HORN_CORAL_FAN;
    pub const DEAD_TUBE_CORAL_FAN: &'static LazyLock<BlockItemId> = &DEAD_TUBE_CORAL_FAN;
    pub const DEAD_BRAIN_CORAL_FAN: &'static LazyLock<BlockItemId> = &DEAD_BRAIN_CORAL_FAN;
    pub const DEAD_BUBBLE_CORAL_FAN: &'static LazyLock<BlockItemId> = &DEAD_BUBBLE_CORAL_FAN;
    pub const DEAD_FIRE_CORAL_FAN: &'static LazyLock<BlockItemId> = &DEAD_FIRE_CORAL_FAN;
    pub const DEAD_HORN_CORAL_FAN: &'static LazyLock<BlockItemId> = &DEAD_HORN_CORAL_FAN;
    pub const BLUE_ICE: &'static LazyLock<BlockItemId> = &BLUE_ICE;
    pub const CONDUIT: &'static LazyLock<BlockItemId> = &CONDUIT;
    pub const POLISHED_GRANITE_STAIRS: &'static LazyLock<BlockItemId> = &POLISHED_GRANITE_STAIRS;
    pub const SMOOTH_RED_SANDSTONE_STAIRS: &'static LazyLock<BlockItemId> =
        &SMOOTH_RED_SANDSTONE_STAIRS;
    pub const MOSSY_STONE_BRICK_STAIRS: &'static LazyLock<BlockItemId> = &MOSSY_STONE_BRICK_STAIRS;
    pub const POLISHED_DIORITE_STAIRS: &'static LazyLock<BlockItemId> = &POLISHED_DIORITE_STAIRS;
    pub const MOSSY_COBBLESTONE_STAIRS: &'static LazyLock<BlockItemId> = &MOSSY_COBBLESTONE_STAIRS;
    pub const END_STONE_BRICK_STAIRS: &'static LazyLock<BlockItemId> = &END_STONE_BRICK_STAIRS;
    pub const STONE_STAIRS: &'static LazyLock<BlockItemId> = &STONE_STAIRS;
    pub const SMOOTH_SANDSTONE_STAIRS: &'static LazyLock<BlockItemId> = &SMOOTH_SANDSTONE_STAIRS;
    pub const SMOOTH_QUARTZ_STAIRS: &'static LazyLock<BlockItemId> = &SMOOTH_QUARTZ_STAIRS;
    pub const GRANITE_STAIRS: &'static LazyLock<BlockItemId> = &GRANITE_STAIRS;
    pub const ANDESITE_STAIRS: &'static LazyLock<BlockItemId> = &ANDESITE_STAIRS;
    pub const RED_NETHER_BRICK_STAIRS: &'static LazyLock<BlockItemId> = &RED_NETHER_BRICK_STAIRS;
    pub const POLISHED_ANDESITE_STAIRS: &'static LazyLock<BlockItemId> = &POLISHED_ANDESITE_STAIRS;
    pub const DIORITE_STAIRS: &'static LazyLock<BlockItemId> = &DIORITE_STAIRS;
    pub const COBBLED_DEEPSLATE_STAIRS: &'static LazyLock<BlockItemId> = &COBBLED_DEEPSLATE_STAIRS;
    pub const POLISHED_DEEPSLATE_STAIRS: &'static LazyLock<BlockItemId> =
        &POLISHED_DEEPSLATE_STAIRS;
    pub const DEEPSLATE_BRICK_STAIRS: &'static LazyLock<BlockItemId> = &DEEPSLATE_BRICK_STAIRS;
    pub const DEEPSLATE_TILE_STAIRS: &'static LazyLock<BlockItemId> = &DEEPSLATE_TILE_STAIRS;
    pub const POLISHED_GRANITE_SLAB: &'static LazyLock<BlockItemId> = &POLISHED_GRANITE_SLAB;
    pub const SMOOTH_RED_SANDSTONE_SLAB: &'static LazyLock<BlockItemId> =
        &SMOOTH_RED_SANDSTONE_SLAB;
    pub const MOSSY_STONE_BRICK_SLAB: &'static LazyLock<BlockItemId> = &MOSSY_STONE_BRICK_SLAB;
    pub const POLISHED_DIORITE_SLAB: &'static LazyLock<BlockItemId> = &POLISHED_DIORITE_SLAB;
    pub const MOSSY_COBBLESTONE_SLAB: &'static LazyLock<BlockItemId> = &MOSSY_COBBLESTONE_SLAB;
    pub const END_STONE_BRICK_SLAB: &'static LazyLock<BlockItemId> = &END_STONE_BRICK_SLAB;
    pub const SMOOTH_SANDSTONE_SLAB: &'static LazyLock<BlockItemId> = &SMOOTH_SANDSTONE_SLAB;
    pub const SMOOTH_QUARTZ_SLAB: &'static LazyLock<BlockItemId> = &SMOOTH_QUARTZ_SLAB;
    pub const GRANITE_SLAB: &'static LazyLock<BlockItemId> = &GRANITE_SLAB;
    pub const ANDESITE_SLAB: &'static LazyLock<BlockItemId> = &ANDESITE_SLAB;
    pub const RED_NETHER_BRICK_SLAB: &'static LazyLock<BlockItemId> = &RED_NETHER_BRICK_SLAB;
    pub const POLISHED_ANDESITE_SLAB: &'static LazyLock<BlockItemId> = &POLISHED_ANDESITE_SLAB;
    pub const DIORITE_SLAB: &'static LazyLock<BlockItemId> = &DIORITE_SLAB;
    pub const COBBLED_DEEPSLATE_SLAB: &'static LazyLock<BlockItemId> = &COBBLED_DEEPSLATE_SLAB;
    pub const POLISHED_DEEPSLATE_SLAB: &'static LazyLock<BlockItemId> = &POLISHED_DEEPSLATE_SLAB;
    pub const DEEPSLATE_BRICK_SLAB: &'static LazyLock<BlockItemId> = &DEEPSLATE_BRICK_SLAB;
    pub const DEEPSLATE_TILE_SLAB: &'static LazyLock<BlockItemId> = &DEEPSLATE_TILE_SLAB;
    pub const SCAFFOLDING: &'static LazyLock<BlockItemId> = &SCAFFOLDING;
    pub const REDSTONE_TORCH: &'static LazyLock<BlockItemId> = &REDSTONE_TORCH;
    pub const REDSTONE_BLOCK: &'static LazyLock<BlockItemId> = &REDSTONE_BLOCK;
    pub const REPEATER: &'static LazyLock<BlockItemId> = &REPEATER;
    pub const COMPARATOR: &'static LazyLock<BlockItemId> = &COMPARATOR;
    pub const PISTON: &'static LazyLock<BlockItemId> = &PISTON;
    pub const STICKY_PISTON: &'static LazyLock<BlockItemId> = &STICKY_PISTON;
    pub const SLIME_BLOCK: &'static LazyLock<BlockItemId> = &SLIME_BLOCK;
    pub const HONEY_BLOCK: &'static LazyLock<BlockItemId> = &HONEY_BLOCK;
    pub const OBSERVER: &'static LazyLock<BlockItemId> = &OBSERVER;
    pub const HOPPER: &'static LazyLock<BlockItemId> = &HOPPER;
    pub const DISPENSER: &'static LazyLock<BlockItemId> = &DISPENSER;
    pub const DROPPER: &'static LazyLock<BlockItemId> = &DROPPER;
    pub const LECTERN: &'static LazyLock<BlockItemId> = &LECTERN;
    pub const TARGET: &'static LazyLock<BlockItemId> = &TARGET;
    pub const LEVER: &'static LazyLock<BlockItemId> = &LEVER;
    pub const DAYLIGHT_DETECTOR: &'static LazyLock<BlockItemId> = &DAYLIGHT_DETECTOR;
    pub const SCULK_SENSOR: &'static LazyLock<BlockItemId> = &SCULK_SENSOR;
    pub const CALIBRATED_SCULK_SENSOR: &'static LazyLock<BlockItemId> = &CALIBRATED_SCULK_SENSOR;
    pub const TRIPWIRE_HOOK: &'static LazyLock<BlockItemId> = &TRIPWIRE_HOOK;
    pub const TRAPPED_CHEST: &'static LazyLock<BlockItemId> = &TRAPPED_CHEST;
    pub const TNT: &'static LazyLock<BlockItemId> = &TNT;
    pub const REDSTONE_LAMP: &'static LazyLock<BlockItemId> = &REDSTONE_LAMP;
    pub const NOTE_BLOCK: &'static LazyLock<BlockItemId> = &NOTE_BLOCK;
    pub const STONE_BUTTON: &'static LazyLock<BlockItemId> = &STONE_BUTTON;
    pub const POLISHED_BLACKSTONE_BUTTON: &'static LazyLock<BlockItemId> =
        &POLISHED_BLACKSTONE_BUTTON;
    pub const OAK_BUTTON: &'static LazyLock<BlockItemId> = &OAK_BUTTON;
    pub const SPRUCE_BUTTON: &'static LazyLock<BlockItemId> = &SPRUCE_BUTTON;
    pub const BIRCH_BUTTON: &'static LazyLock<BlockItemId> = &BIRCH_BUTTON;
    pub const JUNGLE_BUTTON: &'static LazyLock<BlockItemId> = &JUNGLE_BUTTON;
    pub const ACACIA_BUTTON: &'static LazyLock<BlockItemId> = &ACACIA_BUTTON;
    pub const CHERRY_BUTTON: &'static LazyLock<BlockItemId> = &CHERRY_BUTTON;
    pub const DARK_OAK_BUTTON: &'static LazyLock<BlockItemId> = &DARK_OAK_BUTTON;
    pub const PALE_OAK_BUTTON: &'static LazyLock<BlockItemId> = &PALE_OAK_BUTTON;
    pub const MANGROVE_BUTTON: &'static LazyLock<BlockItemId> = &MANGROVE_BUTTON;
    pub const POPLAR_BUTTON: &'static LazyLock<BlockItemId> = &POPLAR_BUTTON;
    pub const BAMBOO_BUTTON: &'static LazyLock<BlockItemId> = &BAMBOO_BUTTON;
    pub const CRIMSON_BUTTON: &'static LazyLock<BlockItemId> = &CRIMSON_BUTTON;
    pub const WARPED_BUTTON: &'static LazyLock<BlockItemId> = &WARPED_BUTTON;
    pub const STONE_PRESSURE_PLATE: &'static LazyLock<BlockItemId> = &STONE_PRESSURE_PLATE;
    pub const POLISHED_BLACKSTONE_PRESSURE_PLATE: &'static LazyLock<BlockItemId> =
        &POLISHED_BLACKSTONE_PRESSURE_PLATE;
    pub const LIGHT_WEIGHTED_PRESSURE_PLATE: &'static LazyLock<BlockItemId> =
        &LIGHT_WEIGHTED_PRESSURE_PLATE;
    pub const HEAVY_WEIGHTED_PRESSURE_PLATE: &'static LazyLock<BlockItemId> =
        &HEAVY_WEIGHTED_PRESSURE_PLATE;
    pub const OAK_PRESSURE_PLATE: &'static LazyLock<BlockItemId> = &OAK_PRESSURE_PLATE;
    pub const SPRUCE_PRESSURE_PLATE: &'static LazyLock<BlockItemId> = &SPRUCE_PRESSURE_PLATE;
    pub const BIRCH_PRESSURE_PLATE: &'static LazyLock<BlockItemId> = &BIRCH_PRESSURE_PLATE;
    pub const JUNGLE_PRESSURE_PLATE: &'static LazyLock<BlockItemId> = &JUNGLE_PRESSURE_PLATE;
    pub const ACACIA_PRESSURE_PLATE: &'static LazyLock<BlockItemId> = &ACACIA_PRESSURE_PLATE;
    pub const CHERRY_PRESSURE_PLATE: &'static LazyLock<BlockItemId> = &CHERRY_PRESSURE_PLATE;
    pub const DARK_OAK_PRESSURE_PLATE: &'static LazyLock<BlockItemId> = &DARK_OAK_PRESSURE_PLATE;
    pub const PALE_OAK_PRESSURE_PLATE: &'static LazyLock<BlockItemId> = &PALE_OAK_PRESSURE_PLATE;
    pub const MANGROVE_PRESSURE_PLATE: &'static LazyLock<BlockItemId> = &MANGROVE_PRESSURE_PLATE;
    pub const POPLAR_PRESSURE_PLATE: &'static LazyLock<BlockItemId> = &POPLAR_PRESSURE_PLATE;
    pub const BAMBOO_PRESSURE_PLATE: &'static LazyLock<BlockItemId> = &BAMBOO_PRESSURE_PLATE;
    pub const CRIMSON_PRESSURE_PLATE: &'static LazyLock<BlockItemId> = &CRIMSON_PRESSURE_PLATE;
    pub const WARPED_PRESSURE_PLATE: &'static LazyLock<BlockItemId> = &WARPED_PRESSURE_PLATE;
    pub const IRON_DOOR: &'static LazyLock<BlockItemId> = &IRON_DOOR;
    pub const OAK_DOOR: &'static LazyLock<BlockItemId> = &OAK_DOOR;
    pub const SPRUCE_DOOR: &'static LazyLock<BlockItemId> = &SPRUCE_DOOR;
    pub const BIRCH_DOOR: &'static LazyLock<BlockItemId> = &BIRCH_DOOR;
    pub const JUNGLE_DOOR: &'static LazyLock<BlockItemId> = &JUNGLE_DOOR;
    pub const ACACIA_DOOR: &'static LazyLock<BlockItemId> = &ACACIA_DOOR;
    pub const CHERRY_DOOR: &'static LazyLock<BlockItemId> = &CHERRY_DOOR;
    pub const DARK_OAK_DOOR: &'static LazyLock<BlockItemId> = &DARK_OAK_DOOR;
    pub const PALE_OAK_DOOR: &'static LazyLock<BlockItemId> = &PALE_OAK_DOOR;
    pub const MANGROVE_DOOR: &'static LazyLock<BlockItemId> = &MANGROVE_DOOR;
    pub const POPLAR_DOOR: &'static LazyLock<BlockItemId> = &POPLAR_DOOR;
    pub const BAMBOO_DOOR: &'static LazyLock<BlockItemId> = &BAMBOO_DOOR;
    pub const CRIMSON_DOOR: &'static LazyLock<BlockItemId> = &CRIMSON_DOOR;
    pub const WARPED_DOOR: &'static LazyLock<BlockItemId> = &WARPED_DOOR;
    pub const IRON_TRAPDOOR: &'static LazyLock<BlockItemId> = &IRON_TRAPDOOR;
    pub const OAK_TRAPDOOR: &'static LazyLock<BlockItemId> = &OAK_TRAPDOOR;
    pub const SPRUCE_TRAPDOOR: &'static LazyLock<BlockItemId> = &SPRUCE_TRAPDOOR;
    pub const BIRCH_TRAPDOOR: &'static LazyLock<BlockItemId> = &BIRCH_TRAPDOOR;
    pub const JUNGLE_TRAPDOOR: &'static LazyLock<BlockItemId> = &JUNGLE_TRAPDOOR;
    pub const ACACIA_TRAPDOOR: &'static LazyLock<BlockItemId> = &ACACIA_TRAPDOOR;
    pub const CHERRY_TRAPDOOR: &'static LazyLock<BlockItemId> = &CHERRY_TRAPDOOR;
    pub const DARK_OAK_TRAPDOOR: &'static LazyLock<BlockItemId> = &DARK_OAK_TRAPDOOR;
    pub const PALE_OAK_TRAPDOOR: &'static LazyLock<BlockItemId> = &PALE_OAK_TRAPDOOR;
    pub const MANGROVE_TRAPDOOR: &'static LazyLock<BlockItemId> = &MANGROVE_TRAPDOOR;
    pub const POPLAR_TRAPDOOR: &'static LazyLock<BlockItemId> = &POPLAR_TRAPDOOR;
    pub const BAMBOO_TRAPDOOR: &'static LazyLock<BlockItemId> = &BAMBOO_TRAPDOOR;
    pub const CRIMSON_TRAPDOOR: &'static LazyLock<BlockItemId> = &CRIMSON_TRAPDOOR;
    pub const WARPED_TRAPDOOR: &'static LazyLock<BlockItemId> = &WARPED_TRAPDOOR;
    pub const OAK_FENCE_GATE: &'static LazyLock<BlockItemId> = &OAK_FENCE_GATE;
    pub const SPRUCE_FENCE_GATE: &'static LazyLock<BlockItemId> = &SPRUCE_FENCE_GATE;
    pub const BIRCH_FENCE_GATE: &'static LazyLock<BlockItemId> = &BIRCH_FENCE_GATE;
    pub const JUNGLE_FENCE_GATE: &'static LazyLock<BlockItemId> = &JUNGLE_FENCE_GATE;
    pub const ACACIA_FENCE_GATE: &'static LazyLock<BlockItemId> = &ACACIA_FENCE_GATE;
    pub const CHERRY_FENCE_GATE: &'static LazyLock<BlockItemId> = &CHERRY_FENCE_GATE;
    pub const DARK_OAK_FENCE_GATE: &'static LazyLock<BlockItemId> = &DARK_OAK_FENCE_GATE;
    pub const PALE_OAK_FENCE_GATE: &'static LazyLock<BlockItemId> = &PALE_OAK_FENCE_GATE;
    pub const MANGROVE_FENCE_GATE: &'static LazyLock<BlockItemId> = &MANGROVE_FENCE_GATE;
    pub const POPLAR_FENCE_GATE: &'static LazyLock<BlockItemId> = &POPLAR_FENCE_GATE;
    pub const BAMBOO_FENCE_GATE: &'static LazyLock<BlockItemId> = &BAMBOO_FENCE_GATE;
    pub const CRIMSON_FENCE_GATE: &'static LazyLock<BlockItemId> = &CRIMSON_FENCE_GATE;
    pub const WARPED_FENCE_GATE: &'static LazyLock<BlockItemId> = &WARPED_FENCE_GATE;
    pub const POWERED_RAIL: &'static LazyLock<BlockItemId> = &POWERED_RAIL;
    pub const DETECTOR_RAIL: &'static LazyLock<BlockItemId> = &DETECTOR_RAIL;
    pub const RAIL: &'static LazyLock<BlockItemId> = &RAIL;
    pub const ACTIVATOR_RAIL: &'static LazyLock<BlockItemId> = &ACTIVATOR_RAIL;
    pub const STRUCTURE_BLOCK: &'static LazyLock<BlockItemId> = &STRUCTURE_BLOCK;
    pub const JIGSAW: &'static LazyLock<BlockItemId> = &JIGSAW;
    pub const TEST_BLOCK: &'static LazyLock<BlockItemId> = &TEST_BLOCK;
    pub const TEST_INSTANCE_BLOCK: &'static LazyLock<BlockItemId> = &TEST_INSTANCE_BLOCK;
    pub const OAK_SIGN: &'static LazyLock<BlockItemId> = &OAK_SIGN;
    pub const SPRUCE_SIGN: &'static LazyLock<BlockItemId> = &SPRUCE_SIGN;
    pub const BIRCH_SIGN: &'static LazyLock<BlockItemId> = &BIRCH_SIGN;
    pub const JUNGLE_SIGN: &'static LazyLock<BlockItemId> = &JUNGLE_SIGN;
    pub const ACACIA_SIGN: &'static LazyLock<BlockItemId> = &ACACIA_SIGN;
    pub const CHERRY_SIGN: &'static LazyLock<BlockItemId> = &CHERRY_SIGN;
    pub const DARK_OAK_SIGN: &'static LazyLock<BlockItemId> = &DARK_OAK_SIGN;
    pub const PALE_OAK_SIGN: &'static LazyLock<BlockItemId> = &PALE_OAK_SIGN;
    pub const MANGROVE_SIGN: &'static LazyLock<BlockItemId> = &MANGROVE_SIGN;
    pub const POPLAR_SIGN: &'static LazyLock<BlockItemId> = &POPLAR_SIGN;
    pub const BAMBOO_SIGN: &'static LazyLock<BlockItemId> = &BAMBOO_SIGN;
    pub const CRIMSON_SIGN: &'static LazyLock<BlockItemId> = &CRIMSON_SIGN;
    pub const WARPED_SIGN: &'static LazyLock<BlockItemId> = &WARPED_SIGN;
    pub const OAK_HANGING_SIGN: &'static LazyLock<BlockItemId> = &OAK_HANGING_SIGN;
    pub const SPRUCE_HANGING_SIGN: &'static LazyLock<BlockItemId> = &SPRUCE_HANGING_SIGN;
    pub const BIRCH_HANGING_SIGN: &'static LazyLock<BlockItemId> = &BIRCH_HANGING_SIGN;
    pub const JUNGLE_HANGING_SIGN: &'static LazyLock<BlockItemId> = &JUNGLE_HANGING_SIGN;
    pub const ACACIA_HANGING_SIGN: &'static LazyLock<BlockItemId> = &ACACIA_HANGING_SIGN;
    pub const CHERRY_HANGING_SIGN: &'static LazyLock<BlockItemId> = &CHERRY_HANGING_SIGN;
    pub const DARK_OAK_HANGING_SIGN: &'static LazyLock<BlockItemId> = &DARK_OAK_HANGING_SIGN;
    pub const PALE_OAK_HANGING_SIGN: &'static LazyLock<BlockItemId> = &PALE_OAK_HANGING_SIGN;
    pub const MANGROVE_HANGING_SIGN: &'static LazyLock<BlockItemId> = &MANGROVE_HANGING_SIGN;
    pub const POPLAR_HANGING_SIGN: &'static LazyLock<BlockItemId> = &POPLAR_HANGING_SIGN;
    pub const BAMBOO_HANGING_SIGN: &'static LazyLock<BlockItemId> = &BAMBOO_HANGING_SIGN;
    pub const CRIMSON_HANGING_SIGN: &'static LazyLock<BlockItemId> = &CRIMSON_HANGING_SIGN;
    pub const WARPED_HANGING_SIGN: &'static LazyLock<BlockItemId> = &WARPED_HANGING_SIGN;
    pub const DRIED_KELP_BLOCK: &'static LazyLock<BlockItemId> = &DRIED_KELP_BLOCK;
    pub const CAKE: &'static LazyLock<BlockItemId> = &CAKE;
    pub const CRAFTER: &'static LazyLock<BlockItemId> = &CRAFTER;
    pub const NETHER_WART: &'static LazyLock<BlockItemId> = &NETHER_WART;
    pub const BREWING_STAND: &'static LazyLock<BlockItemId> = &BREWING_STAND;
    pub const CAULDRON: &'static LazyLock<BlockItemId> = &CAULDRON;
    pub const FLOWER_POT: &'static LazyLock<BlockItemId> = &FLOWER_POT;
    pub const SKELETON_SKULL: &'static LazyLock<BlockItemId> = &SKELETON_SKULL;
    pub const WITHER_SKELETON_SKULL: &'static LazyLock<BlockItemId> = &WITHER_SKELETON_SKULL;
    pub const PLAYER_HEAD: &'static LazyLock<BlockItemId> = &PLAYER_HEAD;
    pub const ZOMBIE_HEAD: &'static LazyLock<BlockItemId> = &ZOMBIE_HEAD;
    pub const CREEPER_HEAD: &'static LazyLock<BlockItemId> = &CREEPER_HEAD;
    pub const DRAGON_HEAD: &'static LazyLock<BlockItemId> = &DRAGON_HEAD;
    pub const PIGLIN_HEAD: &'static LazyLock<BlockItemId> = &PIGLIN_HEAD;
    pub const LOOM: &'static LazyLock<BlockItemId> = &LOOM;
    pub const COMPOSTER: &'static LazyLock<BlockItemId> = &COMPOSTER;
    pub const BARREL: &'static LazyLock<BlockItemId> = &BARREL;
    pub const SMOKER: &'static LazyLock<BlockItemId> = &SMOKER;
    pub const BLAST_FURNACE: &'static LazyLock<BlockItemId> = &BLAST_FURNACE;
    pub const CARTOGRAPHY_TABLE: &'static LazyLock<BlockItemId> = &CARTOGRAPHY_TABLE;
    pub const FLETCHING_TABLE: &'static LazyLock<BlockItemId> = &FLETCHING_TABLE;
    pub const GRINDSTONE: &'static LazyLock<BlockItemId> = &GRINDSTONE;
    pub const SMITHING_TABLE: &'static LazyLock<BlockItemId> = &SMITHING_TABLE;
    pub const STONECUTTER: &'static LazyLock<BlockItemId> = &STONECUTTER;
    pub const BELL: &'static LazyLock<BlockItemId> = &BELL;
    pub const LANTERN: &'static LazyLock<BlockItemId> = &LANTERN;
    pub const SOUL_LANTERN: &'static LazyLock<BlockItemId> = &SOUL_LANTERN;
    pub const CAMPFIRE: &'static LazyLock<BlockItemId> = &CAMPFIRE;
    pub const SOUL_CAMPFIRE: &'static LazyLock<BlockItemId> = &SOUL_CAMPFIRE;
    pub const SHROOMLIGHT: &'static LazyLock<BlockItemId> = &SHROOMLIGHT;
    pub const BEE_NEST: &'static LazyLock<BlockItemId> = &BEE_NEST;
    pub const BEEHIVE: &'static LazyLock<BlockItemId> = &BEEHIVE;
    pub const HONEYCOMB_BLOCK: &'static LazyLock<BlockItemId> = &HONEYCOMB_BLOCK;
    pub const LODESTONE: &'static LazyLock<BlockItemId> = &LODESTONE;
    pub const CRYING_OBSIDIAN: &'static LazyLock<BlockItemId> = &CRYING_OBSIDIAN;
    pub const BLACKSTONE: &'static LazyLock<BlockItemId> = &BLACKSTONE;
    pub const BLACKSTONE_SLAB: &'static LazyLock<BlockItemId> = &BLACKSTONE_SLAB;
    pub const BLACKSTONE_STAIRS: &'static LazyLock<BlockItemId> = &BLACKSTONE_STAIRS;
    pub const GILDED_BLACKSTONE: &'static LazyLock<BlockItemId> = &GILDED_BLACKSTONE;
    pub const POLISHED_BLACKSTONE: &'static LazyLock<BlockItemId> = &POLISHED_BLACKSTONE;
    pub const POLISHED_BLACKSTONE_SLAB: &'static LazyLock<BlockItemId> = &POLISHED_BLACKSTONE_SLAB;
    pub const POLISHED_BLACKSTONE_STAIRS: &'static LazyLock<BlockItemId> =
        &POLISHED_BLACKSTONE_STAIRS;
    pub const CHISELED_POLISHED_BLACKSTONE: &'static LazyLock<BlockItemId> =
        &CHISELED_POLISHED_BLACKSTONE;
    pub const POLISHED_BLACKSTONE_BRICKS: &'static LazyLock<BlockItemId> =
        &POLISHED_BLACKSTONE_BRICKS;
    pub const POLISHED_BLACKSTONE_BRICK_SLAB: &'static LazyLock<BlockItemId> =
        &POLISHED_BLACKSTONE_BRICK_SLAB;
    pub const POLISHED_BLACKSTONE_BRICK_STAIRS: &'static LazyLock<BlockItemId> =
        &POLISHED_BLACKSTONE_BRICK_STAIRS;
    pub const CRACKED_POLISHED_BLACKSTONE_BRICKS: &'static LazyLock<BlockItemId> =
        &CRACKED_POLISHED_BLACKSTONE_BRICKS;
    pub const RESPAWN_ANCHOR: &'static LazyLock<BlockItemId> = &RESPAWN_ANCHOR;
    pub const CANDLE: &'static LazyLock<BlockItemId> = &CANDLE;
    pub const SMALL_AMETHYST_BUD: &'static LazyLock<BlockItemId> = &SMALL_AMETHYST_BUD;
    pub const MEDIUM_AMETHYST_BUD: &'static LazyLock<BlockItemId> = &MEDIUM_AMETHYST_BUD;
    pub const LARGE_AMETHYST_BUD: &'static LazyLock<BlockItemId> = &LARGE_AMETHYST_BUD;
    pub const AMETHYST_CLUSTER: &'static LazyLock<BlockItemId> = &AMETHYST_CLUSTER;
    pub const POINTED_DRIPSTONE: &'static LazyLock<BlockItemId> = &POINTED_DRIPSTONE;
    pub const SULFUR_SPIKE: &'static LazyLock<BlockItemId> = &SULFUR_SPIKE;
    pub const OCHRE_FROGLIGHT: &'static LazyLock<BlockItemId> = &OCHRE_FROGLIGHT;
    pub const VERDANT_FROGLIGHT: &'static LazyLock<BlockItemId> = &VERDANT_FROGLIGHT;
    pub const PEARLESCENT_FROGLIGHT: &'static LazyLock<BlockItemId> = &PEARLESCENT_FROGLIGHT;
    pub const FROGSPAWN: &'static LazyLock<BlockItemId> = &FROGSPAWN;
    pub const TRIAL_SPAWNER: &'static LazyLock<BlockItemId> = &TRIAL_SPAWNER;
    pub const VAULT: &'static LazyLock<BlockItemId> = &VAULT;
    pub const STRAW_BED: &'static LazyLock<BlockItemId> = &STRAW_BED;
    pub const REDSTONE_DUST: &'static LazyLock<BlockItemId> = &REDSTONE_DUST;
    pub const TRIPWIRE: &'static LazyLock<BlockItemId> = &TRIPWIRE;
    pub const POWDER_SNOW: &'static LazyLock<BlockItemId> = &POWDER_SNOW;
    pub const WHEAT_CROP: &'static LazyLock<BlockItemId> = &WHEAT_CROP;
    pub const COCOA_CROP: &'static LazyLock<BlockItemId> = &COCOA_CROP;
    pub const CARROT_CROP: &'static LazyLock<BlockItemId> = &CARROT_CROP;
    pub const POTATO_CROP: &'static LazyLock<BlockItemId> = &POTATO_CROP;
    pub const TORCHFLOWER_CROP: &'static LazyLock<BlockItemId> = &TORCHFLOWER_CROP;
    pub const PITCHER_CROP: &'static LazyLock<BlockItemId> = &PITCHER_CROP;
    pub const BEETROOT_CROP: &'static LazyLock<BlockItemId> = &BEETROOT_CROP;
    pub const SWEET_BERRY_CROP: &'static LazyLock<BlockItemId> = &SWEET_BERRY_CROP;
    pub const GLOW_BERRY_CROP: &'static LazyLock<BlockItemId> = &GLOW_BERRY_CROP;
    pub const PUMPKIN_CROP: &'static LazyLock<BlockItemId> = &PUMPKIN_CROP;
    pub const MELON_CROP: &'static LazyLock<BlockItemId> = &MELON_CROP;
    pub const BED: &'static LazyLock<ColorCollection<BlockItemId>> = &BED;
    pub const WOOL: &'static LazyLock<ColorCollection<BlockItemId>> = &WOOL;
    pub const WOOL_STAIRS: &'static LazyLock<ColorCollection<BlockItemId>> = &WOOL_STAIRS;
    pub const WOOL_SLAB: &'static LazyLock<ColorCollection<BlockItemId>> = &WOOL_SLAB;
    pub const STAINED_GLASS: &'static LazyLock<ColorCollection<BlockItemId>> = &STAINED_GLASS;
    pub const DYED_TERRACOTTA: &'static LazyLock<ColorCollection<BlockItemId>> = &DYED_TERRACOTTA;
    pub const STAINED_GLASS_PANE: &'static LazyLock<ColorCollection<BlockItemId>> =
        &STAINED_GLASS_PANE;
    pub const CARPET: &'static LazyLock<ColorCollection<BlockItemId>> = &CARPET;
    pub const BANNER: &'static LazyLock<ColorCollection<BlockItemId>> = &BANNER;
    pub const DYED_SHULKER_BOX: &'static LazyLock<ColorCollection<BlockItemId>> = &DYED_SHULKER_BOX;
    pub const GLAZED_TERRACOTTA: &'static LazyLock<ColorCollection<BlockItemId>> =
        &GLAZED_TERRACOTTA;
    pub const CONCRETE: &'static LazyLock<ColorCollection<BlockItemId>> = &CONCRETE;
    pub const CONCRETE_STAIRS: &'static LazyLock<ColorCollection<BlockItemId>> = &CONCRETE_STAIRS;
    pub const CONCRETE_SLAB: &'static LazyLock<ColorCollection<BlockItemId>> = &CONCRETE_SLAB;
    pub const CONCRETE_POWDER: &'static LazyLock<ColorCollection<BlockItemId>> = &CONCRETE_POWDER;
    pub const DYED_CANDLE: &'static LazyLock<ColorCollection<BlockItemId>> = &DYED_CANDLE;
    pub const COPPER_BLOCK: &'static LazyLock<WeatheringCopperCollection<BlockItemId>> =
        &COPPER_BLOCK;
    pub const COPPER_BARS: &'static LazyLock<WeatheringCopperCollection<BlockItemId>> =
        &COPPER_BARS;
    pub const COPPER_CHAIN: &'static LazyLock<WeatheringCopperCollection<BlockItemId>> =
        &COPPER_CHAIN;
    pub const COPPER_LANTERN: &'static LazyLock<WeatheringCopperCollection<BlockItemId>> =
        &COPPER_LANTERN;
    pub const CUT_COPPER: &'static LazyLock<WeatheringCopperCollection<BlockItemId>> = &CUT_COPPER;
    pub const CHISELED_COPPER: &'static LazyLock<WeatheringCopperCollection<BlockItemId>> =
        &CHISELED_COPPER;
    pub const CUT_COPPER_STAIRS: &'static LazyLock<WeatheringCopperCollection<BlockItemId>> =
        &CUT_COPPER_STAIRS;
    pub const CUT_COPPER_SLAB: &'static LazyLock<WeatheringCopperCollection<BlockItemId>> =
        &CUT_COPPER_SLAB;
    pub const COPPER_DOOR: &'static LazyLock<WeatheringCopperCollection<BlockItemId>> =
        &COPPER_DOOR;
    pub const COPPER_TRAPDOOR: &'static LazyLock<WeatheringCopperCollection<BlockItemId>> =
        &COPPER_TRAPDOOR;
    pub const COPPER_GRATE: &'static LazyLock<WeatheringCopperCollection<BlockItemId>> =
        &COPPER_GRATE;
    pub const COPPER_BULB: &'static LazyLock<WeatheringCopperCollection<BlockItemId>> =
        &COPPER_BULB;
    pub const COPPER_CHEST: &'static LazyLock<WeatheringCopperCollection<BlockItemId>> =
        &COPPER_CHEST;
    pub const COPPER_GOLEM_STATUE: &'static LazyLock<WeatheringCopperCollection<BlockItemId>> =
        &COPPER_GOLEM_STATUE;
    pub const LIGHTNING_ROD: &'static LazyLock<WeatheringCopperCollection<BlockItemId>> =
        &LIGHTNING_ROD;
}

fn create_simple_colored(base_name: &str) -> ColorCollection<BlockItemId> {
    ColorCollection::prefix_with_color(&ColorCollection::create(base_name.to_owned()))
        .map(|name| BlockItemId::create(name))
}

fn create_simple_copper(base_name: &str) -> WeatheringCopperCollection<BlockItemId> {
    WeatheringCopperCollection::prefix_with_state(&WeatheringCopperCollection::create(base_name))
        .map(|name| BlockItemId::create(name))
}
