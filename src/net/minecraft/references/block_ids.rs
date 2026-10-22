//! net/minecraft/references/BlockIds.java
//!
//! The ids of the blocks that have no item. Each is made on first use, like `Registries`' keys.

use std::sync::LazyLock;

use crate::net::minecraft::core::registries::registries::Registries;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::resources::resource_key::ResourceKey;
use crate::net::minecraft::world::level::block::color_collection::ColorCollection;

static ATTACHED_PUMPKIN_STEM: LazyLock<ResourceKey> =
    LazyLock::new(|| create("attached_pumpkin_stem"));
static ATTACHED_MELON_STEM: LazyLock<ResourceKey> = LazyLock::new(|| create("attached_melon_stem"));
static WATER: LazyLock<ResourceKey> = LazyLock::new(|| create("water"));
static LAVA: LazyLock<ResourceKey> = LazyLock::new(|| create("lava"));
static TALL_SEAGRASS: LazyLock<ResourceKey> = LazyLock::new(|| create("tall_seagrass"));
static PISTON_HEAD: LazyLock<ResourceKey> = LazyLock::new(|| create("piston_head"));
static MOVING_PISTON: LazyLock<ResourceKey> = LazyLock::new(|| create("moving_piston"));
static WALL_TORCH: LazyLock<ResourceKey> = LazyLock::new(|| create("wall_torch"));
static FIRE: LazyLock<ResourceKey> = LazyLock::new(|| create("fire"));
static SOUL_FIRE: LazyLock<ResourceKey> = LazyLock::new(|| create("soul_fire"));
static OAK_WALL_SIGN: LazyLock<ResourceKey> = LazyLock::new(|| create("oak_wall_sign"));
static SPRUCE_WALL_SIGN: LazyLock<ResourceKey> = LazyLock::new(|| create("spruce_wall_sign"));
static BIRCH_WALL_SIGN: LazyLock<ResourceKey> = LazyLock::new(|| create("birch_wall_sign"));
static ACACIA_WALL_SIGN: LazyLock<ResourceKey> = LazyLock::new(|| create("acacia_wall_sign"));
static CHERRY_WALL_SIGN: LazyLock<ResourceKey> = LazyLock::new(|| create("cherry_wall_sign"));
static JUNGLE_WALL_SIGN: LazyLock<ResourceKey> = LazyLock::new(|| create("jungle_wall_sign"));
static DARK_OAK_WALL_SIGN: LazyLock<ResourceKey> = LazyLock::new(|| create("dark_oak_wall_sign"));
static PALE_OAK_WALL_SIGN: LazyLock<ResourceKey> = LazyLock::new(|| create("pale_oak_wall_sign"));
static MANGROVE_WALL_SIGN: LazyLock<ResourceKey> = LazyLock::new(|| create("mangrove_wall_sign"));
static POPLAR_WALL_SIGN: LazyLock<ResourceKey> = LazyLock::new(|| create("poplar_wall_sign"));
static BAMBOO_WALL_SIGN: LazyLock<ResourceKey> = LazyLock::new(|| create("bamboo_wall_sign"));
static OAK_WALL_HANGING_SIGN: LazyLock<ResourceKey> =
    LazyLock::new(|| create("oak_wall_hanging_sign"));
static SPRUCE_WALL_HANGING_SIGN: LazyLock<ResourceKey> =
    LazyLock::new(|| create("spruce_wall_hanging_sign"));
static BIRCH_WALL_HANGING_SIGN: LazyLock<ResourceKey> =
    LazyLock::new(|| create("birch_wall_hanging_sign"));
static ACACIA_WALL_HANGING_SIGN: LazyLock<ResourceKey> =
    LazyLock::new(|| create("acacia_wall_hanging_sign"));
static CHERRY_WALL_HANGING_SIGN: LazyLock<ResourceKey> =
    LazyLock::new(|| create("cherry_wall_hanging_sign"));
static JUNGLE_WALL_HANGING_SIGN: LazyLock<ResourceKey> =
    LazyLock::new(|| create("jungle_wall_hanging_sign"));
static DARK_OAK_WALL_HANGING_SIGN: LazyLock<ResourceKey> =
    LazyLock::new(|| create("dark_oak_wall_hanging_sign"));
static PALE_OAK_WALL_HANGING_SIGN: LazyLock<ResourceKey> =
    LazyLock::new(|| create("pale_oak_wall_hanging_sign"));
static MANGROVE_WALL_HANGING_SIGN: LazyLock<ResourceKey> =
    LazyLock::new(|| create("mangrove_wall_hanging_sign"));
static POPLAR_WALL_HANGING_SIGN: LazyLock<ResourceKey> =
    LazyLock::new(|| create("poplar_wall_hanging_sign"));
static CRIMSON_WALL_HANGING_SIGN: LazyLock<ResourceKey> =
    LazyLock::new(|| create("crimson_wall_hanging_sign"));
static WARPED_WALL_HANGING_SIGN: LazyLock<ResourceKey> =
    LazyLock::new(|| create("warped_wall_hanging_sign"));
static BAMBOO_WALL_HANGING_SIGN: LazyLock<ResourceKey> =
    LazyLock::new(|| create("bamboo_wall_hanging_sign"));
static REDSTONE_WALL_TORCH: LazyLock<ResourceKey> = LazyLock::new(|| create("redstone_wall_torch"));
static SOUL_WALL_TORCH: LazyLock<ResourceKey> = LazyLock::new(|| create("soul_wall_torch"));
static COPPER_WALL_TORCH: LazyLock<ResourceKey> = LazyLock::new(|| create("copper_wall_torch"));
static NETHER_PORTAL: LazyLock<ResourceKey> = LazyLock::new(|| create("nether_portal"));
static WATER_CAULDRON: LazyLock<ResourceKey> = LazyLock::new(|| create("water_cauldron"));
static LAVA_CAULDRON: LazyLock<ResourceKey> = LazyLock::new(|| create("lava_cauldron"));
static POWDER_SNOW_CAULDRON: LazyLock<ResourceKey> =
    LazyLock::new(|| create("powder_snow_cauldron"));
static END_PORTAL: LazyLock<ResourceKey> = LazyLock::new(|| create("end_portal"));
static POTTED_TORCHFLOWER: LazyLock<ResourceKey> = LazyLock::new(|| create("potted_torchflower"));
static POTTED_OAK_SAPLING: LazyLock<ResourceKey> = LazyLock::new(|| create("potted_oak_sapling"));
static POTTED_SPRUCE_SAPLING: LazyLock<ResourceKey> =
    LazyLock::new(|| create("potted_spruce_sapling"));
static POTTED_BIRCH_SAPLING: LazyLock<ResourceKey> =
    LazyLock::new(|| create("potted_birch_sapling"));
static POTTED_JUNGLE_SAPLING: LazyLock<ResourceKey> =
    LazyLock::new(|| create("potted_jungle_sapling"));
static POTTED_ACACIA_SAPLING: LazyLock<ResourceKey> =
    LazyLock::new(|| create("potted_acacia_sapling"));
static POTTED_CHERRY_SAPLING: LazyLock<ResourceKey> =
    LazyLock::new(|| create("potted_cherry_sapling"));
static POTTED_DARK_OAK_SAPLING: LazyLock<ResourceKey> =
    LazyLock::new(|| create("potted_dark_oak_sapling"));
static POTTED_PALE_OAK_SAPLING: LazyLock<ResourceKey> =
    LazyLock::new(|| create("potted_pale_oak_sapling"));
static POTTED_MANGROVE_PROPAGULE: LazyLock<ResourceKey> =
    LazyLock::new(|| create("potted_mangrove_propagule"));
static POTTED_POPLAR_SAPLING: LazyLock<ResourceKey> =
    LazyLock::new(|| create("potted_poplar_sapling"));
static POTTED_FERN: LazyLock<ResourceKey> = LazyLock::new(|| create("potted_fern"));
static POTTED_DANDELION: LazyLock<ResourceKey> = LazyLock::new(|| create("potted_dandelion"));
static POTTED_GOLDEN_DANDELION: LazyLock<ResourceKey> =
    LazyLock::new(|| create("potted_golden_dandelion"));
static POTTED_POPPY: LazyLock<ResourceKey> = LazyLock::new(|| create("potted_poppy"));
static POTTED_BLUE_ORCHID: LazyLock<ResourceKey> = LazyLock::new(|| create("potted_blue_orchid"));
static POTTED_ALLIUM: LazyLock<ResourceKey> = LazyLock::new(|| create("potted_allium"));
static POTTED_AZURE_BLUET: LazyLock<ResourceKey> = LazyLock::new(|| create("potted_azure_bluet"));
static POTTED_RED_TULIP: LazyLock<ResourceKey> = LazyLock::new(|| create("potted_red_tulip"));
static POTTED_ORANGE_TULIP: LazyLock<ResourceKey> = LazyLock::new(|| create("potted_orange_tulip"));
static POTTED_WHITE_TULIP: LazyLock<ResourceKey> = LazyLock::new(|| create("potted_white_tulip"));
static POTTED_PINK_TULIP: LazyLock<ResourceKey> = LazyLock::new(|| create("potted_pink_tulip"));
static POTTED_OXEYE_DAISY: LazyLock<ResourceKey> = LazyLock::new(|| create("potted_oxeye_daisy"));
static POTTED_CORNFLOWER: LazyLock<ResourceKey> = LazyLock::new(|| create("potted_cornflower"));
static POTTED_LILY_OF_THE_VALLEY: LazyLock<ResourceKey> =
    LazyLock::new(|| create("potted_lily_of_the_valley"));
static POTTED_WITHER_ROSE: LazyLock<ResourceKey> = LazyLock::new(|| create("potted_wither_rose"));
static POTTED_RED_MUSHROOM: LazyLock<ResourceKey> = LazyLock::new(|| create("potted_red_mushroom"));
static POTTED_BROWN_MUSHROOM: LazyLock<ResourceKey> =
    LazyLock::new(|| create("potted_brown_mushroom"));
static POTTED_DEAD_BUSH: LazyLock<ResourceKey> = LazyLock::new(|| create("potted_dead_bush"));
static POTTED_CACTUS: LazyLock<ResourceKey> = LazyLock::new(|| create("potted_cactus"));
static SKELETON_WALL_SKULL: LazyLock<ResourceKey> = LazyLock::new(|| create("skeleton_wall_skull"));
static WITHER_SKELETON_WALL_SKULL: LazyLock<ResourceKey> =
    LazyLock::new(|| create("wither_skeleton_wall_skull"));
static ZOMBIE_WALL_HEAD: LazyLock<ResourceKey> = LazyLock::new(|| create("zombie_wall_head"));
static PLAYER_WALL_HEAD: LazyLock<ResourceKey> = LazyLock::new(|| create("player_wall_head"));
static CREEPER_WALL_HEAD: LazyLock<ResourceKey> = LazyLock::new(|| create("creeper_wall_head"));
static DRAGON_WALL_HEAD: LazyLock<ResourceKey> = LazyLock::new(|| create("dragon_wall_head"));
static PIGLIN_WALL_HEAD: LazyLock<ResourceKey> = LazyLock::new(|| create("piglin_wall_head"));
static END_GATEWAY: LazyLock<ResourceKey> = LazyLock::new(|| create("end_gateway"));
static FROSTED_ICE: LazyLock<ResourceKey> = LazyLock::new(|| create("frosted_ice"));
static KELP_PLANT: LazyLock<ResourceKey> = LazyLock::new(|| create("kelp_plant"));
static DEAD_TUBE_CORAL_WALL_FAN: LazyLock<ResourceKey> =
    LazyLock::new(|| create("dead_tube_coral_wall_fan"));
static DEAD_BRAIN_CORAL_WALL_FAN: LazyLock<ResourceKey> =
    LazyLock::new(|| create("dead_brain_coral_wall_fan"));
static DEAD_BUBBLE_CORAL_WALL_FAN: LazyLock<ResourceKey> =
    LazyLock::new(|| create("dead_bubble_coral_wall_fan"));
static DEAD_FIRE_CORAL_WALL_FAN: LazyLock<ResourceKey> =
    LazyLock::new(|| create("dead_fire_coral_wall_fan"));
static DEAD_HORN_CORAL_WALL_FAN: LazyLock<ResourceKey> =
    LazyLock::new(|| create("dead_horn_coral_wall_fan"));
static TUBE_CORAL_WALL_FAN: LazyLock<ResourceKey> = LazyLock::new(|| create("tube_coral_wall_fan"));
static BRAIN_CORAL_WALL_FAN: LazyLock<ResourceKey> =
    LazyLock::new(|| create("brain_coral_wall_fan"));
static BUBBLE_CORAL_WALL_FAN: LazyLock<ResourceKey> =
    LazyLock::new(|| create("bubble_coral_wall_fan"));
static FIRE_CORAL_WALL_FAN: LazyLock<ResourceKey> = LazyLock::new(|| create("fire_coral_wall_fan"));
static HORN_CORAL_WALL_FAN: LazyLock<ResourceKey> = LazyLock::new(|| create("horn_coral_wall_fan"));
static BAMBOO_SAPLING: LazyLock<ResourceKey> = LazyLock::new(|| create("bamboo_sapling"));
static POTTED_BAMBOO: LazyLock<ResourceKey> = LazyLock::new(|| create("potted_bamboo"));
static VOID_AIR: LazyLock<ResourceKey> = LazyLock::new(|| create("void_air"));
static CAVE_AIR: LazyLock<ResourceKey> = LazyLock::new(|| create("cave_air"));
static BUBBLE_COLUMN: LazyLock<ResourceKey> = LazyLock::new(|| create("bubble_column"));
static WEEPING_VINES_PLANT: LazyLock<ResourceKey> = LazyLock::new(|| create("weeping_vines_plant"));
static TWISTING_VINES_PLANT: LazyLock<ResourceKey> =
    LazyLock::new(|| create("twisting_vines_plant"));
static CRIMSON_WALL_SIGN: LazyLock<ResourceKey> = LazyLock::new(|| create("crimson_wall_sign"));
static WARPED_WALL_SIGN: LazyLock<ResourceKey> = LazyLock::new(|| create("warped_wall_sign"));
static POTTED_CRIMSON_FUNGUS: LazyLock<ResourceKey> =
    LazyLock::new(|| create("potted_crimson_fungus"));
static POTTED_WARPED_FUNGUS: LazyLock<ResourceKey> =
    LazyLock::new(|| create("potted_warped_fungus"));
static POTTED_CRIMSON_ROOTS: LazyLock<ResourceKey> =
    LazyLock::new(|| create("potted_crimson_roots"));
static POTTED_WARPED_ROOTS: LazyLock<ResourceKey> = LazyLock::new(|| create("potted_warped_roots"));
static CANDLE_CAKE: LazyLock<ResourceKey> = LazyLock::new(|| create("candle_cake"));
static CAVE_VINES_PLANT: LazyLock<ResourceKey> = LazyLock::new(|| create("cave_vines_plant"));
static BIG_DRIPLEAF_STEM: LazyLock<ResourceKey> = LazyLock::new(|| create("big_dripleaf_stem"));
static POTTED_AZALEA_BUSH: LazyLock<ResourceKey> = LazyLock::new(|| create("potted_azalea_bush"));
static POTTED_FLOWERING_AZALEA_BUSH: LazyLock<ResourceKey> =
    LazyLock::new(|| create("potted_flowering_azalea_bush"));
static POTTED_OPEN_EYEBLOSSOM: LazyLock<ResourceKey> =
    LazyLock::new(|| create("potted_open_eyeblossom"));
static POTTED_CLOSED_EYEBLOSSOM: LazyLock<ResourceKey> =
    LazyLock::new(|| create("potted_closed_eyeblossom"));
static WALL_BANNER: LazyLock<ColorCollection<ResourceKey>> =
    LazyLock::new(|| create_simple_colored("wall_banner"));
static DYED_CANDLE_CAKE: LazyLock<ColorCollection<ResourceKey>> =
    LazyLock::new(|| create_simple_colored("candle_cake"));

pub struct BlockIds;

impl BlockIds {
    pub const ATTACHED_PUMPKIN_STEM: &'static LazyLock<ResourceKey> = &ATTACHED_PUMPKIN_STEM;
    pub const ATTACHED_MELON_STEM: &'static LazyLock<ResourceKey> = &ATTACHED_MELON_STEM;
    pub const WATER: &'static LazyLock<ResourceKey> = &WATER;
    pub const LAVA: &'static LazyLock<ResourceKey> = &LAVA;
    pub const TALL_SEAGRASS: &'static LazyLock<ResourceKey> = &TALL_SEAGRASS;
    pub const PISTON_HEAD: &'static LazyLock<ResourceKey> = &PISTON_HEAD;
    pub const MOVING_PISTON: &'static LazyLock<ResourceKey> = &MOVING_PISTON;
    pub const WALL_TORCH: &'static LazyLock<ResourceKey> = &WALL_TORCH;
    pub const FIRE: &'static LazyLock<ResourceKey> = &FIRE;
    pub const SOUL_FIRE: &'static LazyLock<ResourceKey> = &SOUL_FIRE;
    pub const OAK_WALL_SIGN: &'static LazyLock<ResourceKey> = &OAK_WALL_SIGN;
    pub const SPRUCE_WALL_SIGN: &'static LazyLock<ResourceKey> = &SPRUCE_WALL_SIGN;
    pub const BIRCH_WALL_SIGN: &'static LazyLock<ResourceKey> = &BIRCH_WALL_SIGN;
    pub const ACACIA_WALL_SIGN: &'static LazyLock<ResourceKey> = &ACACIA_WALL_SIGN;
    pub const CHERRY_WALL_SIGN: &'static LazyLock<ResourceKey> = &CHERRY_WALL_SIGN;
    pub const JUNGLE_WALL_SIGN: &'static LazyLock<ResourceKey> = &JUNGLE_WALL_SIGN;
    pub const DARK_OAK_WALL_SIGN: &'static LazyLock<ResourceKey> = &DARK_OAK_WALL_SIGN;
    pub const PALE_OAK_WALL_SIGN: &'static LazyLock<ResourceKey> = &PALE_OAK_WALL_SIGN;
    pub const MANGROVE_WALL_SIGN: &'static LazyLock<ResourceKey> = &MANGROVE_WALL_SIGN;
    pub const POPLAR_WALL_SIGN: &'static LazyLock<ResourceKey> = &POPLAR_WALL_SIGN;
    pub const BAMBOO_WALL_SIGN: &'static LazyLock<ResourceKey> = &BAMBOO_WALL_SIGN;
    pub const OAK_WALL_HANGING_SIGN: &'static LazyLock<ResourceKey> = &OAK_WALL_HANGING_SIGN;
    pub const SPRUCE_WALL_HANGING_SIGN: &'static LazyLock<ResourceKey> = &SPRUCE_WALL_HANGING_SIGN;
    pub const BIRCH_WALL_HANGING_SIGN: &'static LazyLock<ResourceKey> = &BIRCH_WALL_HANGING_SIGN;
    pub const ACACIA_WALL_HANGING_SIGN: &'static LazyLock<ResourceKey> = &ACACIA_WALL_HANGING_SIGN;
    pub const CHERRY_WALL_HANGING_SIGN: &'static LazyLock<ResourceKey> = &CHERRY_WALL_HANGING_SIGN;
    pub const JUNGLE_WALL_HANGING_SIGN: &'static LazyLock<ResourceKey> = &JUNGLE_WALL_HANGING_SIGN;
    pub const DARK_OAK_WALL_HANGING_SIGN: &'static LazyLock<ResourceKey> =
        &DARK_OAK_WALL_HANGING_SIGN;
    pub const PALE_OAK_WALL_HANGING_SIGN: &'static LazyLock<ResourceKey> =
        &PALE_OAK_WALL_HANGING_SIGN;
    pub const MANGROVE_WALL_HANGING_SIGN: &'static LazyLock<ResourceKey> =
        &MANGROVE_WALL_HANGING_SIGN;
    pub const POPLAR_WALL_HANGING_SIGN: &'static LazyLock<ResourceKey> = &POPLAR_WALL_HANGING_SIGN;
    pub const CRIMSON_WALL_HANGING_SIGN: &'static LazyLock<ResourceKey> =
        &CRIMSON_WALL_HANGING_SIGN;
    pub const WARPED_WALL_HANGING_SIGN: &'static LazyLock<ResourceKey> = &WARPED_WALL_HANGING_SIGN;
    pub const BAMBOO_WALL_HANGING_SIGN: &'static LazyLock<ResourceKey> = &BAMBOO_WALL_HANGING_SIGN;
    pub const REDSTONE_WALL_TORCH: &'static LazyLock<ResourceKey> = &REDSTONE_WALL_TORCH;
    pub const SOUL_WALL_TORCH: &'static LazyLock<ResourceKey> = &SOUL_WALL_TORCH;
    pub const COPPER_WALL_TORCH: &'static LazyLock<ResourceKey> = &COPPER_WALL_TORCH;
    pub const NETHER_PORTAL: &'static LazyLock<ResourceKey> = &NETHER_PORTAL;
    pub const WATER_CAULDRON: &'static LazyLock<ResourceKey> = &WATER_CAULDRON;
    pub const LAVA_CAULDRON: &'static LazyLock<ResourceKey> = &LAVA_CAULDRON;
    pub const POWDER_SNOW_CAULDRON: &'static LazyLock<ResourceKey> = &POWDER_SNOW_CAULDRON;
    pub const END_PORTAL: &'static LazyLock<ResourceKey> = &END_PORTAL;
    pub const POTTED_TORCHFLOWER: &'static LazyLock<ResourceKey> = &POTTED_TORCHFLOWER;
    pub const POTTED_OAK_SAPLING: &'static LazyLock<ResourceKey> = &POTTED_OAK_SAPLING;
    pub const POTTED_SPRUCE_SAPLING: &'static LazyLock<ResourceKey> = &POTTED_SPRUCE_SAPLING;
    pub const POTTED_BIRCH_SAPLING: &'static LazyLock<ResourceKey> = &POTTED_BIRCH_SAPLING;
    pub const POTTED_JUNGLE_SAPLING: &'static LazyLock<ResourceKey> = &POTTED_JUNGLE_SAPLING;
    pub const POTTED_ACACIA_SAPLING: &'static LazyLock<ResourceKey> = &POTTED_ACACIA_SAPLING;
    pub const POTTED_CHERRY_SAPLING: &'static LazyLock<ResourceKey> = &POTTED_CHERRY_SAPLING;
    pub const POTTED_DARK_OAK_SAPLING: &'static LazyLock<ResourceKey> = &POTTED_DARK_OAK_SAPLING;
    pub const POTTED_PALE_OAK_SAPLING: &'static LazyLock<ResourceKey> = &POTTED_PALE_OAK_SAPLING;
    pub const POTTED_MANGROVE_PROPAGULE: &'static LazyLock<ResourceKey> =
        &POTTED_MANGROVE_PROPAGULE;
    pub const POTTED_POPLAR_SAPLING: &'static LazyLock<ResourceKey> = &POTTED_POPLAR_SAPLING;
    pub const POTTED_FERN: &'static LazyLock<ResourceKey> = &POTTED_FERN;
    pub const POTTED_DANDELION: &'static LazyLock<ResourceKey> = &POTTED_DANDELION;
    pub const POTTED_GOLDEN_DANDELION: &'static LazyLock<ResourceKey> = &POTTED_GOLDEN_DANDELION;
    pub const POTTED_POPPY: &'static LazyLock<ResourceKey> = &POTTED_POPPY;
    pub const POTTED_BLUE_ORCHID: &'static LazyLock<ResourceKey> = &POTTED_BLUE_ORCHID;
    pub const POTTED_ALLIUM: &'static LazyLock<ResourceKey> = &POTTED_ALLIUM;
    pub const POTTED_AZURE_BLUET: &'static LazyLock<ResourceKey> = &POTTED_AZURE_BLUET;
    pub const POTTED_RED_TULIP: &'static LazyLock<ResourceKey> = &POTTED_RED_TULIP;
    pub const POTTED_ORANGE_TULIP: &'static LazyLock<ResourceKey> = &POTTED_ORANGE_TULIP;
    pub const POTTED_WHITE_TULIP: &'static LazyLock<ResourceKey> = &POTTED_WHITE_TULIP;
    pub const POTTED_PINK_TULIP: &'static LazyLock<ResourceKey> = &POTTED_PINK_TULIP;
    pub const POTTED_OXEYE_DAISY: &'static LazyLock<ResourceKey> = &POTTED_OXEYE_DAISY;
    pub const POTTED_CORNFLOWER: &'static LazyLock<ResourceKey> = &POTTED_CORNFLOWER;
    pub const POTTED_LILY_OF_THE_VALLEY: &'static LazyLock<ResourceKey> =
        &POTTED_LILY_OF_THE_VALLEY;
    pub const POTTED_WITHER_ROSE: &'static LazyLock<ResourceKey> = &POTTED_WITHER_ROSE;
    pub const POTTED_RED_MUSHROOM: &'static LazyLock<ResourceKey> = &POTTED_RED_MUSHROOM;
    pub const POTTED_BROWN_MUSHROOM: &'static LazyLock<ResourceKey> = &POTTED_BROWN_MUSHROOM;
    pub const POTTED_DEAD_BUSH: &'static LazyLock<ResourceKey> = &POTTED_DEAD_BUSH;
    pub const POTTED_CACTUS: &'static LazyLock<ResourceKey> = &POTTED_CACTUS;
    pub const SKELETON_WALL_SKULL: &'static LazyLock<ResourceKey> = &SKELETON_WALL_SKULL;
    pub const WITHER_SKELETON_WALL_SKULL: &'static LazyLock<ResourceKey> =
        &WITHER_SKELETON_WALL_SKULL;
    pub const ZOMBIE_WALL_HEAD: &'static LazyLock<ResourceKey> = &ZOMBIE_WALL_HEAD;
    pub const PLAYER_WALL_HEAD: &'static LazyLock<ResourceKey> = &PLAYER_WALL_HEAD;
    pub const CREEPER_WALL_HEAD: &'static LazyLock<ResourceKey> = &CREEPER_WALL_HEAD;
    pub const DRAGON_WALL_HEAD: &'static LazyLock<ResourceKey> = &DRAGON_WALL_HEAD;
    pub const PIGLIN_WALL_HEAD: &'static LazyLock<ResourceKey> = &PIGLIN_WALL_HEAD;
    pub const END_GATEWAY: &'static LazyLock<ResourceKey> = &END_GATEWAY;
    pub const FROSTED_ICE: &'static LazyLock<ResourceKey> = &FROSTED_ICE;
    pub const KELP_PLANT: &'static LazyLock<ResourceKey> = &KELP_PLANT;
    pub const DEAD_TUBE_CORAL_WALL_FAN: &'static LazyLock<ResourceKey> = &DEAD_TUBE_CORAL_WALL_FAN;
    pub const DEAD_BRAIN_CORAL_WALL_FAN: &'static LazyLock<ResourceKey> =
        &DEAD_BRAIN_CORAL_WALL_FAN;
    pub const DEAD_BUBBLE_CORAL_WALL_FAN: &'static LazyLock<ResourceKey> =
        &DEAD_BUBBLE_CORAL_WALL_FAN;
    pub const DEAD_FIRE_CORAL_WALL_FAN: &'static LazyLock<ResourceKey> = &DEAD_FIRE_CORAL_WALL_FAN;
    pub const DEAD_HORN_CORAL_WALL_FAN: &'static LazyLock<ResourceKey> = &DEAD_HORN_CORAL_WALL_FAN;
    pub const TUBE_CORAL_WALL_FAN: &'static LazyLock<ResourceKey> = &TUBE_CORAL_WALL_FAN;
    pub const BRAIN_CORAL_WALL_FAN: &'static LazyLock<ResourceKey> = &BRAIN_CORAL_WALL_FAN;
    pub const BUBBLE_CORAL_WALL_FAN: &'static LazyLock<ResourceKey> = &BUBBLE_CORAL_WALL_FAN;
    pub const FIRE_CORAL_WALL_FAN: &'static LazyLock<ResourceKey> = &FIRE_CORAL_WALL_FAN;
    pub const HORN_CORAL_WALL_FAN: &'static LazyLock<ResourceKey> = &HORN_CORAL_WALL_FAN;
    pub const BAMBOO_SAPLING: &'static LazyLock<ResourceKey> = &BAMBOO_SAPLING;
    pub const POTTED_BAMBOO: &'static LazyLock<ResourceKey> = &POTTED_BAMBOO;
    pub const VOID_AIR: &'static LazyLock<ResourceKey> = &VOID_AIR;
    pub const CAVE_AIR: &'static LazyLock<ResourceKey> = &CAVE_AIR;
    pub const BUBBLE_COLUMN: &'static LazyLock<ResourceKey> = &BUBBLE_COLUMN;
    pub const WEEPING_VINES_PLANT: &'static LazyLock<ResourceKey> = &WEEPING_VINES_PLANT;
    pub const TWISTING_VINES_PLANT: &'static LazyLock<ResourceKey> = &TWISTING_VINES_PLANT;
    pub const CRIMSON_WALL_SIGN: &'static LazyLock<ResourceKey> = &CRIMSON_WALL_SIGN;
    pub const WARPED_WALL_SIGN: &'static LazyLock<ResourceKey> = &WARPED_WALL_SIGN;
    pub const POTTED_CRIMSON_FUNGUS: &'static LazyLock<ResourceKey> = &POTTED_CRIMSON_FUNGUS;
    pub const POTTED_WARPED_FUNGUS: &'static LazyLock<ResourceKey> = &POTTED_WARPED_FUNGUS;
    pub const POTTED_CRIMSON_ROOTS: &'static LazyLock<ResourceKey> = &POTTED_CRIMSON_ROOTS;
    pub const POTTED_WARPED_ROOTS: &'static LazyLock<ResourceKey> = &POTTED_WARPED_ROOTS;
    pub const CANDLE_CAKE: &'static LazyLock<ResourceKey> = &CANDLE_CAKE;
    pub const CAVE_VINES_PLANT: &'static LazyLock<ResourceKey> = &CAVE_VINES_PLANT;
    pub const BIG_DRIPLEAF_STEM: &'static LazyLock<ResourceKey> = &BIG_DRIPLEAF_STEM;
    pub const POTTED_AZALEA_BUSH: &'static LazyLock<ResourceKey> = &POTTED_AZALEA_BUSH;
    pub const POTTED_FLOWERING_AZALEA_BUSH: &'static LazyLock<ResourceKey> =
        &POTTED_FLOWERING_AZALEA_BUSH;
    pub const POTTED_OPEN_EYEBLOSSOM: &'static LazyLock<ResourceKey> = &POTTED_OPEN_EYEBLOSSOM;
    pub const POTTED_CLOSED_EYEBLOSSOM: &'static LazyLock<ResourceKey> = &POTTED_CLOSED_EYEBLOSSOM;
    pub const WALL_BANNER: &'static LazyLock<ColorCollection<ResourceKey>> = &WALL_BANNER;
    pub const DYED_CANDLE_CAKE: &'static LazyLock<ColorCollection<ResourceKey>> = &DYED_CANDLE_CAKE;
}

fn create(name: &str) -> ResourceKey {
    ResourceKey::create(Registries::BLOCK, Identifier::with_default_namespace(name))
}

fn create_simple_colored(base_name: &str) -> ColorCollection<ResourceKey> {
    ColorCollection::prefix_with_color(&ColorCollection::create(base_name.to_owned()))
        .map(|name| create(name))
}
