//! net/minecraft/world/level/block/Blocks.java
//!
//! Every block the game has, registered in Java's order, which numbers them and their states.
//! Each registration is the Java one with what's ported: not yet `Properties.sound` (`SoundType`),
//! `isValidSpawn` (`EntityType`) or `requiredFeatures` (`FeatureFlag`), and only the constructor
//! arguments that decide a block's states (a liquid's fluid, an infested block's host).
//!
//! Java's fields are `Block` handles here, consts numbered by the order the fields register in:
//! each field's first id is how many blocks the fields before it hold. `blocks!` makes the consts
//! and the function registering them, which checks each registration lands where its field says.
//! Registering needs what earlier blocks were made with (`ofLegacyCopy(base)`, a stair's base
//! state), so it reads them from the registry being built, `registry`, which is Java's global
//! `BuiltInRegistries.BLOCK` passed as a parameter. Once every block is registered, the static
//! block numbers their states (`Block.BLOCK_STATE_REGISTRY`) and works out what each keeps
//! (`initCache`), and the registry is what `Block` and `BlockState` read.

use std::cell::{Cell, RefCell};
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::LazyLock;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::core::direction::Axis;
use crate::net::minecraft::core::id_mapper::IdMapper;
use crate::net::minecraft::references::block_ids::BlockIds;
use crate::net::minecraft::references::block_item_id::BlockItemId;
use crate::net::minecraft::references::block_item_ids::BlockItemIds;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::resources::resource_key::ResourceKey;
use crate::net::minecraft::world::item::dye_color::DyeColor;
use crate::net::minecraft::world::level::block::abstract_bed_block::AbstractBedBlock;
use crate::net::minecraft::world::level::block::air_block::AirBlock;
use crate::net::minecraft::world::level::block::amethyst_block::AmethystBlock;
use crate::net::minecraft::world::level::block::amethyst_cluster_block::AmethystClusterBlock;
use crate::net::minecraft::world::level::block::anvil_block::AnvilBlock;
use crate::net::minecraft::world::level::block::attached_stem_block::AttachedStemBlock;
use crate::net::minecraft::world::level::block::azalea_block::AzaleaBlock;
use crate::net::minecraft::world::level::block::bamboo_sapling_block::BambooSaplingBlock;
use crate::net::minecraft::world::level::block::bamboo_stalk_block::BambooStalkBlock;
use crate::net::minecraft::world::level::block::banner_block::BannerBlock;
use crate::net::minecraft::world::level::block::barrel_block::BarrelBlock;
use crate::net::minecraft::world::level::block::barrier_block::BarrierBlock;
use crate::net::minecraft::world::level::block::base_coral_fan_block::BaseCoralFanBlock;
use crate::net::minecraft::world::level::block::base_coral_plant_block::BaseCoralPlantBlock;
use crate::net::minecraft::world::level::block::base_coral_wall_fan_block::BaseCoralWallFanBlock;
use crate::net::minecraft::world::level::block::beacon_block::BeaconBlock;
use crate::net::minecraft::world::level::block::bed_block::BedBlock;
use crate::net::minecraft::world::level::block::beehive_block::BeehiveBlock;
use crate::net::minecraft::world::level::block::beetroot_block::BeetrootBlock;
use crate::net::minecraft::world::level::block::bell_block::BellBlock;
use crate::net::minecraft::world::level::block::big_dripleaf_block::BigDripleafBlock;
use crate::net::minecraft::world::level::block::big_dripleaf_stem_block::BigDripleafStemBlock;
use crate::net::minecraft::world::level::block::blast_furnace_block::BlastFurnaceBlock;
use crate::net::minecraft::world::level::block::block::{Block, BlockData};
use crate::net::minecraft::world::level::block::bonemealable_feature_placer_block::BonemealableFeaturePlacerBlock;
use crate::net::minecraft::world::level::block::brewing_stand_block::BrewingStandBlock;
use crate::net::minecraft::world::level::block::brushable_block::BrushableBlock;
use crate::net::minecraft::world::level::block::bubble_column_block::BubbleColumnBlock;
use crate::net::minecraft::world::level::block::budding_amethyst_block::BuddingAmethystBlock;
use crate::net::minecraft::world::level::block::bush_block::BushBlock;
use crate::net::minecraft::world::level::block::button_block::ButtonBlock;
use crate::net::minecraft::world::level::block::cactus_block::CactusBlock;
use crate::net::minecraft::world::level::block::cactus_flower_block::CactusFlowerBlock;
use crate::net::minecraft::world::level::block::cake_block::CakeBlock;
use crate::net::minecraft::world::level::block::calibrated_sculk_sensor_block::CalibratedSculkSensorBlock;
use crate::net::minecraft::world::level::block::campfire_block::CampfireBlock;
use crate::net::minecraft::world::level::block::candle_block::CandleBlock;
use crate::net::minecraft::world::level::block::candle_cake_block::CandleCakeBlock;
use crate::net::minecraft::world::level::block::carpet_block::CarpetBlock;
use crate::net::minecraft::world::level::block::carrot_block::CarrotBlock;
use crate::net::minecraft::world::level::block::cartography_table_block::CartographyTableBlock;
use crate::net::minecraft::world::level::block::carved_pumpkin_block::CarvedPumpkinBlock;
use crate::net::minecraft::world::level::block::cauldron_block::CauldronBlock;
use crate::net::minecraft::world::level::block::cave_vines::CaveVines;
use crate::net::minecraft::world::level::block::cave_vines_block::CaveVinesBlock;
use crate::net::minecraft::world::level::block::cave_vines_plant_block::CaveVinesPlantBlock;
use crate::net::minecraft::world::level::block::ceiling_hanging_sign_block::CeilingHangingSignBlock;
use crate::net::minecraft::world::level::block::chain_block::ChainBlock;
use crate::net::minecraft::world::level::block::chest_block::ChestBlock;
use crate::net::minecraft::world::level::block::chiseled_book_shelf_block::ChiseledBookShelfBlock;
use crate::net::minecraft::world::level::block::chorus_flower_block::ChorusFlowerBlock;
use crate::net::minecraft::world::level::block::chorus_plant_block::ChorusPlantBlock;
use crate::net::minecraft::world::level::block::cocoa_block::CocoaBlock;
use crate::net::minecraft::world::level::block::color_collection::ColorCollection;
use crate::net::minecraft::world::level::block::colored_falling_block::ColoredFallingBlock;
use crate::net::minecraft::world::level::block::command_block::CommandBlock;
use crate::net::minecraft::world::level::block::comparator_block::ComparatorBlock;
use crate::net::minecraft::world::level::block::composter_block::ComposterBlock;
use crate::net::minecraft::world::level::block::concrete_powder_block::ConcretePowderBlock;
use crate::net::minecraft::world::level::block::conduit_block::ConduitBlock;
use crate::net::minecraft::world::level::block::copper_bulb_block::CopperBulbBlock;
use crate::net::minecraft::world::level::block::copper_chest_block::CopperChestBlock;
use crate::net::minecraft::world::level::block::copper_golem_statue_block::CopperGolemStatueBlock;
use crate::net::minecraft::world::level::block::coral_block::CoralBlock;
use crate::net::minecraft::world::level::block::coral_fan_block::CoralFanBlock;
use crate::net::minecraft::world::level::block::coral_plant_block::CoralPlantBlock;
use crate::net::minecraft::world::level::block::coral_wall_fan_block::CoralWallFanBlock;
use crate::net::minecraft::world::level::block::crafter_block::CrafterBlock;
use crate::net::minecraft::world::level::block::crafting_table_block::CraftingTableBlock;
use crate::net::minecraft::world::level::block::creaking_heart_block::CreakingHeartBlock;
use crate::net::minecraft::world::level::block::crop_block::CropBlock;
use crate::net::minecraft::world::level::block::crying_obsidian_block::CryingObsidianBlock;
use crate::net::minecraft::world::level::block::daylight_detector_block::DaylightDetectorBlock;
use crate::net::minecraft::world::level::block::decorated_pot_block::DecoratedPotBlock;
use crate::net::minecraft::world::level::block::detector_rail_block::DetectorRailBlock;
use crate::net::minecraft::world::level::block::dispenser_block::DispenserBlock;
use crate::net::minecraft::world::level::block::door_block::DoorBlock;
use crate::net::minecraft::world::level::block::double_plant_block::DoublePlantBlock;
use crate::net::minecraft::world::level::block::dragon_egg_block::DragonEggBlock;
use crate::net::minecraft::world::level::block::dried_ghast_block::DriedGhastBlock;
use crate::net::minecraft::world::level::block::drop_experience_block::DropExperienceBlock;
use crate::net::minecraft::world::level::block::dropper_block::DropperBlock;
use crate::net::minecraft::world::level::block::dry_vegetation_block::DryVegetationBlock;
use crate::net::minecraft::world::level::block::enchanting_table_block::EnchantingTableBlock;
use crate::net::minecraft::world::level::block::end_gateway_block::EndGatewayBlock;
use crate::net::minecraft::world::level::block::end_portal_block::EndPortalBlock;
use crate::net::minecraft::world::level::block::end_portal_frame_block::EndPortalFrameBlock;
use crate::net::minecraft::world::level::block::end_rod_block::EndRodBlock;
use crate::net::minecraft::world::level::block::ender_chest_block::EnderChestBlock;
use crate::net::minecraft::world::level::block::eyeblossom_block::EyeblossomBlock;
use crate::net::minecraft::world::level::block::farmland_block::FarmlandBlock;
use crate::net::minecraft::world::level::block::fence_block::FenceBlock;
use crate::net::minecraft::world::level::block::fence_gate_block::FenceGateBlock;
use crate::net::minecraft::world::level::block::fire_block::FireBlock;
use crate::net::minecraft::world::level::block::firefly_bush_block::FireflyBushBlock;
use crate::net::minecraft::world::level::block::flower_bed_block::FlowerBedBlock;
use crate::net::minecraft::world::level::block::flower_block::FlowerBlock;
use crate::net::minecraft::world::level::block::flower_pot_block::FlowerPotBlock;
use crate::net::minecraft::world::level::block::frogspawn_block::FrogspawnBlock;
use crate::net::minecraft::world::level::block::frosted_ice_block::FrostedIceBlock;
use crate::net::minecraft::world::level::block::furnace_block::FurnaceBlock;
use crate::net::minecraft::world::level::block::glazed_terracotta_block::GlazedTerracottaBlock;
use crate::net::minecraft::world::level::block::glow_lichen_block::GlowLichenBlock;
use crate::net::minecraft::world::level::block::grass_block::GrassBlock;
use crate::net::minecraft::world::level::block::grindstone_block::GrindstoneBlock;
use crate::net::minecraft::world::level::block::half_transparent_block::HalfTransparentBlock;
use crate::net::minecraft::world::level::block::hanging_moss_block::HangingMossBlock;
use crate::net::minecraft::world::level::block::hanging_roots_block::HangingRootsBlock;
use crate::net::minecraft::world::level::block::hay_block::HayBlock;
use crate::net::minecraft::world::level::block::heavy_core_block::HeavyCoreBlock;
use crate::net::minecraft::world::level::block::honey_block::HoneyBlock;
use crate::net::minecraft::world::level::block::hopper_block::HopperBlock;
use crate::net::minecraft::world::level::block::huge_mushroom_block::HugeMushroomBlock;
use crate::net::minecraft::world::level::block::ice_block::IceBlock;
use crate::net::minecraft::world::level::block::infested_block::InfestedBlock;
use crate::net::minecraft::world::level::block::infested_rotated_pillar_block::InfestedRotatedPillarBlock;
use crate::net::minecraft::world::level::block::iron_bars_block::IronBarsBlock;
use crate::net::minecraft::world::level::block::jigsaw_block::JigsawBlock;
use crate::net::minecraft::world::level::block::jukebox_block::JukeboxBlock;
use crate::net::minecraft::world::level::block::kelp_block::KelpBlock;
use crate::net::minecraft::world::level::block::kelp_plant_block::KelpPlantBlock;
use crate::net::minecraft::world::level::block::ladder_block::LadderBlock;
use crate::net::minecraft::world::level::block::lantern_block::LanternBlock;
use crate::net::minecraft::world::level::block::lava_cauldron_block::LavaCauldronBlock;
use crate::net::minecraft::world::level::block::layered_cauldron_block::LayeredCauldronBlock;
use crate::net::minecraft::world::level::block::leaf_litter_block::LeafLitterBlock;
use crate::net::minecraft::world::level::block::leaves_block::LeavesBlock;
use crate::net::minecraft::world::level::block::lectern_block::LecternBlock;
use crate::net::minecraft::world::level::block::lever_block::LeverBlock;
use crate::net::minecraft::world::level::block::light_block::LightBlock;
use crate::net::minecraft::world::level::block::lightning_rod_block::LightningRodBlock;
use crate::net::minecraft::world::level::block::lily_pad_block::LilyPadBlock;
use crate::net::minecraft::world::level::block::liquid_block::LiquidBlock;
use crate::net::minecraft::world::level::block::loom_block::LoomBlock;
use crate::net::minecraft::world::level::block::magma_block::MagmaBlock;
use crate::net::minecraft::world::level::block::mangrove_leaves_block::MangroveLeavesBlock;
use crate::net::minecraft::world::level::block::mangrove_propagule_block::MangrovePropaguleBlock;
use crate::net::minecraft::world::level::block::mangrove_roots_block::MangroveRootsBlock;
use crate::net::minecraft::world::level::block::mossy_carpet_block::MossyCarpetBlock;
use crate::net::minecraft::world::level::block::mud_block::MudBlock;
use crate::net::minecraft::world::level::block::multiface_block::MultifaceBlock;
use crate::net::minecraft::world::level::block::mushroom_block::MushroomBlock;
use crate::net::minecraft::world::level::block::mycelium_block::MyceliumBlock;
use crate::net::minecraft::world::level::block::nether_fungus_block::NetherFungusBlock;
use crate::net::minecraft::world::level::block::nether_portal_block::NetherPortalBlock;
use crate::net::minecraft::world::level::block::nether_roots_block::NetherRootsBlock;
use crate::net::minecraft::world::level::block::nether_sprouts_block::NetherSproutsBlock;
use crate::net::minecraft::world::level::block::nether_wart_block::NetherWartBlock;
use crate::net::minecraft::world::level::block::netherrack_block::NetherrackBlock;
use crate::net::minecraft::world::level::block::note_block::NoteBlock;
use crate::net::minecraft::world::level::block::nylium_block::NyliumBlock;
use crate::net::minecraft::world::level::block::observer_block::ObserverBlock;
use crate::net::minecraft::world::level::block::path_block::PathBlock;
use crate::net::minecraft::world::level::block::piglin_wall_skull_block::PiglinWallSkullBlock;
use crate::net::minecraft::world::level::block::piston::moving_piston_block::MovingPistonBlock;
use crate::net::minecraft::world::level::block::piston::piston_base_block::PistonBaseBlock;
use crate::net::minecraft::world::level::block::piston::piston_head_block::PistonHeadBlock;
use crate::net::minecraft::world::level::block::pitcher_crop_block::PitcherCropBlock;
use crate::net::minecraft::world::level::block::player_head_block::PlayerHeadBlock;
use crate::net::minecraft::world::level::block::player_wall_head_block::PlayerWallHeadBlock;
use crate::net::minecraft::world::level::block::pointed_dripstone_block::PointedDripstoneBlock;
use crate::net::minecraft::world::level::block::potato_block::PotatoBlock;
use crate::net::minecraft::world::level::block::potent_sulfur_block::PotentSulfurBlock;
use crate::net::minecraft::world::level::block::powder_snow_block::PowderSnowBlock;
use crate::net::minecraft::world::level::block::powered_block::PoweredBlock;
use crate::net::minecraft::world::level::block::powered_rail_block::PoweredRailBlock;
use crate::net::minecraft::world::level::block::pressure_plate_block::PressurePlateBlock;
use crate::net::minecraft::world::level::block::pumpkin_block::PumpkinBlock;
use crate::net::minecraft::world::level::block::rail_block::RailBlock;
use crate::net::minecraft::world::level::block::red_stone_ore_block::RedStoneOreBlock;
use crate::net::minecraft::world::level::block::redstone_lamp_block::RedstoneLampBlock;
use crate::net::minecraft::world::level::block::redstone_torch_block::RedstoneTorchBlock;
use crate::net::minecraft::world::level::block::redstone_wall_torch_block::RedstoneWallTorchBlock;
use crate::net::minecraft::world::level::block::redstone_wire_block::RedstoneWireBlock;
use crate::net::minecraft::world::level::block::repeater_block::RepeaterBlock;
use crate::net::minecraft::world::level::block::respawn_anchor_block::RespawnAnchorBlock;
use crate::net::minecraft::world::level::block::rooted_dirt_block::RootedDirtBlock;
use crate::net::minecraft::world::level::block::rotated_pillar_block::RotatedPillarBlock;
use crate::net::minecraft::world::level::block::sand_block::SandBlock;
use crate::net::minecraft::world::level::block::sapling_block::SaplingBlock;
use crate::net::minecraft::world::level::block::scaffolding_block::ScaffoldingBlock;
use crate::net::minecraft::world::level::block::sculk_block::SculkBlock;
use crate::net::minecraft::world::level::block::sculk_catalyst_block::SculkCatalystBlock;
use crate::net::minecraft::world::level::block::sculk_sensor_block::SculkSensorBlock;
use crate::net::minecraft::world::level::block::sculk_shrieker_block::SculkShriekerBlock;
use crate::net::minecraft::world::level::block::sculk_vein_block::SculkVeinBlock;
use crate::net::minecraft::world::level::block::sea_pickle_block::SeaPickleBlock;
use crate::net::minecraft::world::level::block::seagrass_block::SeagrassBlock;
use crate::net::minecraft::world::level::block::shelf_block::ShelfBlock;
use crate::net::minecraft::world::level::block::shelf_mushroom_block::ShelfMushroomBlock;
use crate::net::minecraft::world::level::block::short_dry_grass_block::ShortDryGrassBlock;
use crate::net::minecraft::world::level::block::shulker_box_block::ShulkerBoxBlock;
use crate::net::minecraft::world::level::block::skull_block::{SkullBlock, Types as SkullTypes};
use crate::net::minecraft::world::level::block::slab_block::SlabBlock;
use crate::net::minecraft::world::level::block::slime_block::SlimeBlock;
use crate::net::minecraft::world::level::block::small_dripleaf_block::SmallDripleafBlock;
use crate::net::minecraft::world::level::block::smithing_table_block::SmithingTableBlock;
use crate::net::minecraft::world::level::block::smoker_block::SmokerBlock;
use crate::net::minecraft::world::level::block::sniffer_egg_block::SnifferEggBlock;
use crate::net::minecraft::world::level::block::snow_layer_block::SnowLayerBlock;
use crate::net::minecraft::world::level::block::snowy_block::SnowyBlock;
use crate::net::minecraft::world::level::block::soul_fire_block::SoulFireBlock;
use crate::net::minecraft::world::level::block::soul_sand_block::SoulSandBlock;
use crate::net::minecraft::world::level::block::spawner_block::SpawnerBlock;
use crate::net::minecraft::world::level::block::sponge_block::SpongeBlock;
use crate::net::minecraft::world::level::block::spore_blossom_block::SporeBlossomBlock;
use crate::net::minecraft::world::level::block::stained_glass_block::StainedGlassBlock;
use crate::net::minecraft::world::level::block::stained_glass_pane_block::StainedGlassPaneBlock;
use crate::net::minecraft::world::level::block::stair_block::StairBlock;
use crate::net::minecraft::world::level::block::standing_sign_block::StandingSignBlock;
use crate::net::minecraft::world::level::block::state::block_behaviour::{
    BlockStateBase, OffsetType, Properties, StatePredicate, ViewBlocking,
};
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::block::state::properties::bed_part::BedPart;
use crate::net::minecraft::world::level::block::state::properties::block_state_properties::BlockStateProperties;
use crate::net::minecraft::world::level::block::state::properties::note_block_instrument::NoteBlockInstrument;
use crate::net::minecraft::world::level::block::state::properties::sculk_sensor_phase::SculkSensorPhase;
use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;
use crate::net::minecraft::world::level::block::stem_block::StemBlock;
use crate::net::minecraft::world::level::block::stonecutter_block::StonecutterBlock;
use crate::net::minecraft::world::level::block::straw_bed_block::StrawBedBlock;
use crate::net::minecraft::world::level::block::structure_block::StructureBlock;
use crate::net::minecraft::world::level::block::structure_void_block::StructureVoidBlock;
use crate::net::minecraft::world::level::block::sugar_cane_block::SugarCaneBlock;
use crate::net::minecraft::world::level::block::sulfur_spike_block::SulfurSpikeBlock;
use crate::net::minecraft::world::level::block::sweet_berry_bush_block::SweetBerryBushBlock;
use crate::net::minecraft::world::level::block::tall_dry_grass_block::TallDryGrassBlock;
use crate::net::minecraft::world::level::block::tall_flower_block::TallFlowerBlock;
use crate::net::minecraft::world::level::block::tall_grass_block::TallGrassBlock;
use crate::net::minecraft::world::level::block::tall_seagrass_block::TallSeagrassBlock;
use crate::net::minecraft::world::level::block::target_block::TargetBlock;
use crate::net::minecraft::world::level::block::test_block::TestBlock;
use crate::net::minecraft::world::level::block::test_instance_block::TestInstanceBlock;
use crate::net::minecraft::world::level::block::tinted_glass_block::TintedGlassBlock;
use crate::net::minecraft::world::level::block::tinted_particle_leaves_block::TintedParticleLeavesBlock;
use crate::net::minecraft::world::level::block::tnt_block::TntBlock;
use crate::net::minecraft::world::level::block::torch_block::TorchBlock;
use crate::net::minecraft::world::level::block::torchflower_crop_block::TorchflowerCropBlock;
use crate::net::minecraft::world::level::block::transparent_block::TransparentBlock;
use crate::net::minecraft::world::level::block::trap_door_block::TrapDoorBlock;
use crate::net::minecraft::world::level::block::trapped_chest_block::TrappedChestBlock;
use crate::net::minecraft::world::level::block::trial_spawner_block::TrialSpawnerBlock;
use crate::net::minecraft::world::level::block::trip_wire_block::TripWireBlock;
use crate::net::minecraft::world::level::block::trip_wire_hook_block::TripWireHookBlock;
use crate::net::minecraft::world::level::block::turtle_egg_block::TurtleEggBlock;
use crate::net::minecraft::world::level::block::twisting_vines_block::TwistingVinesBlock;
use crate::net::minecraft::world::level::block::twisting_vines_plant_block::TwistingVinesPlantBlock;
use crate::net::minecraft::world::level::block::untinted_particle_leaves_block::UntintedParticleLeavesBlock;
use crate::net::minecraft::world::level::block::vault_block::VaultBlock;
use crate::net::minecraft::world::level::block::vine_block::VineBlock;
use crate::net::minecraft::world::level::block::wall_banner_block::WallBannerBlock;
use crate::net::minecraft::world::level::block::wall_block::WallBlock;
use crate::net::minecraft::world::level::block::wall_hanging_sign_block::WallHangingSignBlock;
use crate::net::minecraft::world::level::block::wall_sign_block::WallSignBlock;
use crate::net::minecraft::world::level::block::wall_skull_block::WallSkullBlock;
use crate::net::minecraft::world::level::block::wall_torch_block::WallTorchBlock;
use crate::net::minecraft::world::level::block::waterlogged_transparent_block::WaterloggedTransparentBlock;
use crate::net::minecraft::world::level::block::weathering_copper::WeatherState;
use crate::net::minecraft::world::level::block::weathering_copper_bars_block::WeatheringCopperBarsBlock;
use crate::net::minecraft::world::level::block::weathering_copper_bulb_block::WeatheringCopperBulbBlock;
use crate::net::minecraft::world::level::block::weathering_copper_chain_block::WeatheringCopperChainBlock;
use crate::net::minecraft::world::level::block::weathering_copper_chest_block::WeatheringCopperChestBlock;
use crate::net::minecraft::world::level::block::weathering_copper_collection::WeatheringCopperCollection;
use crate::net::minecraft::world::level::block::weathering_copper_door_block::WeatheringCopperDoorBlock;
use crate::net::minecraft::world::level::block::weathering_copper_full_block::WeatheringCopperFullBlock;
use crate::net::minecraft::world::level::block::weathering_copper_golem_statue_block::WeatheringCopperGolemStatueBlock;
use crate::net::minecraft::world::level::block::weathering_copper_grate_block::WeatheringCopperGrateBlock;
use crate::net::minecraft::world::level::block::weathering_copper_slab_block::WeatheringCopperSlabBlock;
use crate::net::minecraft::world::level::block::weathering_copper_stair_block::WeatheringCopperStairBlock;
use crate::net::minecraft::world::level::block::weathering_copper_trap_door_block::WeatheringCopperTrapDoorBlock;
use crate::net::minecraft::world::level::block::weathering_lantern_block::WeatheringLanternBlock;
use crate::net::minecraft::world::level::block::weathering_lightning_rod_block::WeatheringLightningRodBlock;
use crate::net::minecraft::world::level::block::web_block::WebBlock;
use crate::net::minecraft::world::level::block::weeping_vines_block::WeepingVinesBlock;
use crate::net::minecraft::world::level::block::weeping_vines_plant_block::WeepingVinesPlantBlock;
use crate::net::minecraft::world::level::block::weighted_pressure_plate_block::WeightedPressurePlateBlock;
use crate::net::minecraft::world::level::block::wet_sponge_block::WetSpongeBlock;
use crate::net::minecraft::world::level::block::wither_rose_block::WitherRoseBlock;
use crate::net::minecraft::world::level::block::wither_skull_block::WitherSkullBlock;
use crate::net::minecraft::world::level::block::wither_wall_skull_block::WitherWallSkullBlock;
use crate::net::minecraft::world::level::block::wool_carpet_block::WoolCarpetBlock;
use crate::net::minecraft::world::level::material::fluids::Fluids;
use crate::net::minecraft::world::level::material::map_color::MapColor;
use crate::net::minecraft::world::level::material::push_reaction::PushReaction;

/// `BuiltInRegistries.BLOCK`, with what `Block.BLOCK_STATE_REGISTRY` and each state keep: every
/// block in the order `Blocks` registers them, and every state by id.
pub struct BlockRegistry {
    pub(crate) blocks: Vec<BlockData>,
    keys: Vec<ResourceKey>,
    by_key: HashMap<Identifier, Block>,
    pub(crate) states: Vec<BlockStateBase>,
    pub(crate) block_state_registry: IdMapper<BlockState>,
}

/// The registry, which `Blocks` fills on first use, like Java's static initializers.
pub(crate) static REGISTRY: LazyLock<BlockRegistry> = LazyLock::new(Blocks::bootstrap);

// Whether `REGISTRY` is made, so a block printed while it's being made doesn't wait for it.
static BOOTSTRAPPED: AtomicBool = AtomicBool::new(false);

impl BlockRegistry {
    /// `getKey`: the block's registered name.
    pub fn get_key(&self, block: Block) -> &Identifier {
        self.keys[block.0 as usize].identifier()
    }

    /// `getValue(Identifier)`.
    pub fn get_value(&self, key: &Identifier) -> Option<Block> {
        self.by_key.get(key).copied()
    }

    /// `byId(int)`.
    pub fn by_id(&self, id: i32) -> Option<Block> {
        usize::try_from(id)
            .ok()
            .filter(|&id| id < self.blocks.len())
            .map(|id| Block(id as u16))
    }

    /// Every block's key, by id.
    pub fn keys(&self) -> &[ResourceKey] {
        &self.keys
    }

    /// Every block, by id.
    pub fn iter(&self) -> impl Iterator<Item = Block> + use<> {
        (0..self.blocks.len() as u16).map(Block)
    }

    pub fn size(&self) -> usize {
        self.blocks.len()
    }
}

/// `BuiltInRegistries.BLOCK`.
pub fn registry() -> &'static BlockRegistry {
    &REGISTRY
}

/// `minecraft:stone`, or the block's id while the registry is being made.
pub(crate) fn registered_name(block: Block) -> String {
    if BOOTSTRAPPED.load(Ordering::Acquire) {
        REGISTRY.get_key(block).to_string()
    } else {
        format!("#{}", block.0)
    }
}

/// The registry while `Blocks` registers into it. It lends out the blocks registered so far as
/// shared references, so a registration can read earlier blocks while it registers.
#[derive(Default)]
pub struct Registrar {
    blocks: RefCell<Vec<Rc<BlockData>>>,
    keys: RefCell<Vec<ResourceKey>>,
    next_state_id: Cell<u32>,
}

impl Registrar {
    /// A block registered already.
    pub fn get(&self, block: Block) -> Rc<BlockData> {
        Rc::clone(&self.blocks.borrow()[block.0 as usize])
    }

    /// `Registry.register(BuiltInRegistries.BLOCK, id, block)`: the next id, and its states the
    /// ids after the last block's.
    fn register(&self, id: ResourceKey, block: BlockData) -> Block {
        let handle = Block(self.blocks.borrow().len() as u16);
        let definition = block.get_state_definition();
        definition.register(handle, self.next_state_id.get());
        self.next_state_id
            .set(self.next_state_id.get() + definition.state_count());
        self.blocks.borrow_mut().push(Rc::new(block));
        self.keys.borrow_mut().push(id);
        handle
    }
}

/// What a block is registered under: a `BlockItemId`'s block, or a block's `ResourceKey` (Java's
/// overloads of `register`).
pub trait BlockKey {
    fn block_key(&self) -> ResourceKey;
}

impl BlockKey for BlockItemId {
    fn block_key(&self) -> ResourceKey {
        self.block().clone()
    }
}

impl BlockKey for ResourceKey {
    fn block_key(&self) -> ResourceKey {
        self.clone()
    }
}

impl<T: BlockKey> BlockKey for LazyLock<T> {
    fn block_key(&self) -> ResourceKey {
        (**self).block_key()
    }
}

/// Each `pub const NAME: Type = registration;` becomes a const holding the blocks the
/// registration makes, numbered after the fields before it, and a step of `register_all`, which
/// checks the registration makes them.
macro_rules! blocks {
    ($registry:ident; $(pub const $name:ident: $type:ty = $register:expr;)*) => {
        #[allow(non_camel_case_types, clippy::upper_case_acronyms)]
        #[derive(Clone, Copy)]
        enum Field {
            $($name,)*
        }

        // How many blocks each field holds, in order.
        const COUNTS: &[u16] = &[$(<$type>::COUNT,)*];

        /// The id of the field's first block: how many blocks the fields before it hold.
        const fn first_id(field: Field) -> u16 {
            let mut id = 0;
            let mut i = 0;
            while i < field as usize {
                id += COUNTS[i];
                i += 1;
            }
            id
        }

        impl Blocks {
            $(pub const $name: $type = <$type>::starting_at(first_id(Field::$name));)*
        }

        fn register_all($registry: &Registrar) {
            $(
                let registered: $type = $register;
                assert!(
                    registered == Blocks::$name,
                    concat!(stringify!($name), " isn't registered where its field says")
                );
            )*
        }
    };
}

pub struct Blocks;

impl Blocks {
    /// Registers every block, then numbers their states and works out what each keeps, like
    /// Java's static initializer.
    fn bootstrap() -> BlockRegistry {
        let registrar = Registrar::default();
        register_all(&registrar);
        let blocks: Vec<BlockData> = registrar
            .blocks
            .into_inner()
            .into_iter()
            .map(|block| {
                Rc::into_inner(block).expect("nothing holds a block once they're all registered")
            })
            .collect();
        let keys = registrar.keys.into_inner();
        let mut block_state_registry = IdMapper::default();
        let mut states = Vec::with_capacity(registrar.next_state_id.get() as usize);
        for block in &blocks {
            for state in block.get_state_definition().get_possible_states() {
                block_state_registry.add(state);
                let mut base = BlockStateBase::new(block.properties(), state);
                base.init_cache(block, state);
                states.push(base);
            }
        }
        let by_key = keys
            .iter()
            .enumerate()
            .map(|(id, key)| (key.identifier().clone(), Block(id as u16)))
            .collect();
        let registry = BlockRegistry {
            blocks,
            keys,
            by_key,
            states,
            block_state_registry,
        };
        BOOTSTRAPPED.store(true, Ordering::Release);
        registry
    }
}

// `NOT_CLOSED_SHULKER`: whether a shulker box's block entity is closed, or true without one.
const NOT_CLOSED_SHULKER: StatePredicate = StatePredicate::NotClosedShulker;
const NOT_EXTENDED_PISTON: StatePredicate =
    StatePredicate::State(Some("NOT_EXTENDED_PISTON"), |state| {
        !state.get_value(PistonBaseBlock::EXTENDED)
    });
// `NEAR_PLANE_INTERSECTS_OUTLINE`: whether a box of the occlusion shape meets the camera's near
// plane.
const NEAR_PLANE_INTERSECTS_OUTLINE: ViewBlocking = ViewBlocking::NearPlaneIntersectsOutline;

fn lit_block_emission(light_emission: i32) -> impl Fn(BlockState) -> i32 + Send + Sync + 'static {
    move |state| {
        if state.get_value(BlockStateProperties::LIT) {
            light_emission
        } else {
            0
        }
    }
}

fn waterlogged_map_color(
    map_color: MapColor,
) -> impl Fn(BlockState) -> MapColor + Send + Sync + 'static {
    move |block_state| {
        if block_state.get_value(BlockStateProperties::WATERLOGGED) {
            MapColor::WATER
        } else {
            map_color
        }
    }
}

/// The `StatePredicate` overload.
fn never(_state: BlockState) -> bool {
    false
}

/// The `StatePredicate` overload.
fn always(_state: BlockState) -> bool {
    true
}

fn log_properties(top_color: MapColor, side_color: MapColor) -> Properties {
    Properties::of()
        .map_color_fn(move |state| {
            if state.get_value(RotatedPillarBlock::AXIS) == Axis::Y {
                top_color
            } else {
                side_color
            }
        })
        .instrument(NoteBlockInstrument::BASS)
        .strength(2.0)
        .ignited_by_lava()
}

fn nether_stem_properties(map_color: MapColor) -> Properties {
    Properties::of()
        .map_color_fn(move |_state| map_color)
        .instrument(NoteBlockInstrument::BASS)
        .strength(2.0)
}

fn post_process_self(_state: BlockState, block_pos: BlockPos) -> Option<BlockPos> {
    Some(block_pos)
}

fn post_process_above(_state: BlockState, block_pos: BlockPos) -> Option<BlockPos> {
    Some(block_pos.above())
}

fn leaves_properties() -> Properties {
    Properties::of()
        .map_color(MapColor::PLANT)
        .strength(0.2)
        .random_ticks()
        .no_occlusion()
        .is_suffocating(StatePredicate::State(None, never))
        .ignited_by_lava()
        .push_reaction(PushReaction::POPPED)
        .is_redstone_conductor(StatePredicate::State(None, never))
}

fn shulker_box_properties(map_color: MapColor) -> Properties {
    Properties::of()
        .map_color(map_color)
        .force_solid_on()
        .strength(2.0)
        .dynamic_shape()
        .no_occlusion()
        .is_suffocating(NOT_CLOSED_SHULKER)
        .push_reaction(PushReaction::POPPED)
}

fn piston_properties() -> Properties {
    Properties::of()
        .map_color(MapColor::STONE)
        .strength(1.5)
        .is_redstone_conductor(StatePredicate::State(None, never))
        .is_suffocating(NOT_EXTENDED_PISTON)
        .push_reaction(PushReaction::IMMOVEABLE)
}

fn button_properties() -> Properties {
    Properties::of()
        .no_collision()
        .strength(0.5)
        .push_reaction(PushReaction::POPPED)
}

fn flower_pot_properties() -> Properties {
    Properties::of()
        .instabreak()
        .no_occlusion()
        .push_reaction(PushReaction::POPPED)
}

fn candle_properties(color: MapColor) -> Properties {
    Properties::of()
        .map_color(color)
        .no_occlusion()
        .strength(0.1)
        .light_level(CandleBlock::light_emission)
        .push_reaction(PushReaction::POPPED)
}

fn register_legacy_stair(registry: &Registrar, id: &impl BlockKey, base: Block) -> Block {
    let properties = Properties::of_legacy_copy(&registry.get(base))
        .is_view_blocking(NEAR_PLANE_INTERSECTS_OUTLINE);
    register_with(registry, id, StairBlock::new, properties)
}

fn register_stair(registry: &Registrar, id: &impl BlockKey, base: Block) -> Block {
    let properties = Properties::of_full_copy(&registry.get(base))
        .is_view_blocking(NEAR_PLANE_INTERSECTS_OUTLINE);
    register_with(registry, id, StairBlock::new, properties)
}

fn register_slab(registry: &Registrar, id: &impl BlockKey, base: Block) -> Block {
    let properties = Properties::of_legacy_copy(&registry.get(base))
        .is_view_blocking(NEAR_PLANE_INTERSECTS_OUTLINE);
    register_with(registry, id, SlabBlock::new, properties)
}

/// `registerSlab(id, base, destroyTime)`.
fn register_slab_destroy_time(
    registry: &Registrar,
    id: &impl BlockKey,
    base: Block,
    destroy_time: f32,
) -> Block {
    let properties = Properties::of_legacy_copy(&registry.get(base))
        .is_view_blocking(NEAR_PLANE_INTERSECTS_OUTLINE)
        .destroy_time(destroy_time);
    register_with(registry, id, SlabBlock::new, properties)
}

/// `registerSlab(id, base, destroyTime, explosionResistance)`.
fn register_slab_strength(
    registry: &Registrar,
    id: &impl BlockKey,
    base: Block,
    destroy_time: f32,
    explosion_resistance: f32,
) -> Block {
    let properties = Properties::of_legacy_copy(&registry.get(base))
        .is_view_blocking(NEAR_PLANE_INTERSECTS_OUTLINE)
        .strength_and_resistance(destroy_time, explosion_resistance);
    register_with(registry, id, SlabBlock::new, properties)
}

fn register_wall(registry: &Registrar, id: &impl BlockKey, base: Block) -> Block {
    let properties = Properties::of_legacy_copy(&registry.get(base)).force_solid_on();
    register_with(registry, id, WallBlock::new, properties)
}

/// A wall variant (a wall torch, a wall sign) drops what its standing block does, and may share
/// its name.
fn wall_variant(registry: &Registrar, standing_block: Block, copy_name: bool) -> Properties {
    let standing_block = registry.get(standing_block);
    let mut wall_properties =
        Properties::of().override_loot_table(standing_block.get_loot_table().cloned());
    if copy_name {
        wall_properties =
            wall_properties.override_description(standing_block.get_description_id().to_owned());
    }
    wall_properties
}

/// `register(id, factory, properties)`: the block the factory makes with the properties and
/// the id.
pub(crate) fn register_with(
    registry: &Registrar,
    id: &impl BlockKey,
    factory: impl FnOnce(Properties) -> BlockData,
    properties: Properties,
) -> Block {
    let id = id.block_key();
    let block = factory(properties.set_id(id.clone()));
    registry.register(id, block)
}

/// `register(id, properties)`: a plain `Block`.
fn register(registry: &Registrar, id: &impl BlockKey, properties: Properties) -> Block {
    register_with(registry, id, Block::new, properties)
}

blocks! {
    registry;
    pub const AIR: Block = register_with(registry, BlockItemIds::AIR, AirBlock::new, Properties::of().replaceable().no_collision().no_loot_table().air());
    pub const STONE: Block = register(registry, BlockItemIds::STONE, Properties::of().map_color(MapColor::STONE).instrument(NoteBlockInstrument::BASEDRUM).requires_correct_tool_for_drops().strength_and_resistance(1.5, 6.0));
    pub const GRANITE: Block = register(registry, BlockItemIds::GRANITE, Properties::of().map_color(MapColor::DIRT).instrument(NoteBlockInstrument::BASEDRUM).requires_correct_tool_for_drops().strength_and_resistance(1.5, 6.0));
    pub const POLISHED_GRANITE: Block = register(registry, BlockItemIds::POLISHED_GRANITE, Properties::of().map_color(MapColor::DIRT).instrument(NoteBlockInstrument::BASEDRUM).requires_correct_tool_for_drops().strength_and_resistance(1.5, 6.0));
    pub const DIORITE: Block = register(registry, BlockItemIds::DIORITE, Properties::of().map_color(MapColor::QUARTZ).instrument(NoteBlockInstrument::BASEDRUM).requires_correct_tool_for_drops().strength_and_resistance(1.5, 6.0));
    pub const POLISHED_DIORITE: Block = register(registry, BlockItemIds::POLISHED_DIORITE, Properties::of().map_color(MapColor::QUARTZ).instrument(NoteBlockInstrument::BASEDRUM).requires_correct_tool_for_drops().strength_and_resistance(1.5, 6.0));
    pub const ANDESITE: Block = register(registry, BlockItemIds::ANDESITE, Properties::of().map_color(MapColor::STONE).instrument(NoteBlockInstrument::BASEDRUM).requires_correct_tool_for_drops().strength_and_resistance(1.5, 6.0));
    pub const POLISHED_ANDESITE: Block = register(registry, BlockItemIds::POLISHED_ANDESITE, Properties::of().map_color(MapColor::STONE).instrument(NoteBlockInstrument::BASEDRUM).requires_correct_tool_for_drops().strength_and_resistance(1.5, 6.0));
    pub const GRASS_BLOCK: Block = register_with(registry, BlockItemIds::GRASS_BLOCK, GrassBlock::new, Properties::of().map_color(MapColor::GRASS).random_ticks().strength(0.6));
    pub const DIRT: Block = register(registry, BlockItemIds::DIRT, Properties::of().map_color(MapColor::DIRT).strength(0.5));
    pub const COARSE_DIRT: Block = register(registry, BlockItemIds::COARSE_DIRT, Properties::of().map_color(MapColor::DIRT).strength(0.5));
    pub const PODZOL: Block = register_with(registry, BlockItemIds::PODZOL, SnowyBlock::new, Properties::of().map_color(MapColor::PODZOL).strength(0.5));
    pub const COBBLESTONE: Block = register(registry, BlockItemIds::COBBLESTONE, Properties::of().map_color(MapColor::STONE).instrument(NoteBlockInstrument::BASEDRUM).requires_correct_tool_for_drops().strength_and_resistance(2.0, 6.0));
    pub const OAK_PLANKS: Block = register(registry, BlockItemIds::OAK_PLANKS, Properties::of().map_color(MapColor::WOOD).instrument(NoteBlockInstrument::BASS).strength_and_resistance(2.0, 3.0).ignited_by_lava());
    pub const SPRUCE_PLANKS: Block = register(registry, BlockItemIds::SPRUCE_PLANKS, Properties::of() .map_color(MapColor::PODZOL) .instrument(NoteBlockInstrument::BASS) .strength_and_resistance(2.0, 3.0)  .ignited_by_lava());
    pub const BIRCH_PLANKS: Block = register(registry, BlockItemIds::BIRCH_PLANKS, Properties::of().map_color(MapColor::SAND).instrument(NoteBlockInstrument::BASS).strength_and_resistance(2.0, 3.0).ignited_by_lava());
    pub const JUNGLE_PLANKS: Block = register(registry, BlockItemIds::JUNGLE_PLANKS, Properties::of().map_color(MapColor::DIRT).instrument(NoteBlockInstrument::BASS).strength_and_resistance(2.0, 3.0).ignited_by_lava());
    pub const ACACIA_PLANKS: Block = register(registry, BlockItemIds::ACACIA_PLANKS, Properties::of() .map_color(MapColor::COLOR_ORANGE) .instrument(NoteBlockInstrument::BASS) .strength_and_resistance(2.0, 3.0)  .ignited_by_lava());
    pub const CHERRY_PLANKS: Block = register(registry, BlockItemIds::CHERRY_PLANKS, Properties::of() .map_color(MapColor::TERRACOTTA_WHITE) .instrument(NoteBlockInstrument::BASS) .strength_and_resistance(2.0, 3.0)  .ignited_by_lava());
    pub const DARK_OAK_PLANKS: Block = register(registry, BlockItemIds::DARK_OAK_PLANKS, Properties::of() .map_color(MapColor::COLOR_BROWN) .instrument(NoteBlockInstrument::BASS) .strength_and_resistance(2.0, 3.0)  .ignited_by_lava());
    pub const PALE_OAK_WOOD: Block = register_with(registry, BlockItemIds::PALE_OAK_WOOD, RotatedPillarBlock::new, Properties::of().map_color(MapColor::STONE).instrument(NoteBlockInstrument::BASS).strength(2.0).ignited_by_lava());
    pub const PALE_OAK_PLANKS: Block = register(registry, BlockItemIds::PALE_OAK_PLANKS, Properties::of() .map_color(MapColor::QUARTZ) .instrument(NoteBlockInstrument::BASS) .strength_and_resistance(2.0, 3.0)  .ignited_by_lava());
    pub const MANGROVE_PLANKS: Block = register(registry, BlockItemIds::MANGROVE_PLANKS, Properties::of() .map_color(MapColor::COLOR_RED) .instrument(NoteBlockInstrument::BASS) .strength_and_resistance(2.0, 3.0)  .ignited_by_lava());
    pub const POPLAR_PLANKS: Block = register(registry, BlockItemIds::POPLAR_PLANKS, Properties::of() .map_color(MapColor::COLOR_LIGHT_GRAY) .instrument(NoteBlockInstrument::BASS) .strength_and_resistance(2.0, 3.0)  .ignited_by_lava());
    pub const BAMBOO_PLANKS: Block = register(registry, BlockItemIds::BAMBOO_PLANKS, Properties::of() .map_color(MapColor::COLOR_YELLOW) .instrument(NoteBlockInstrument::BASS) .strength_and_resistance(2.0, 3.0)  .ignited_by_lava());
    pub const BAMBOO_MOSAIC: Block = register(registry, BlockItemIds::BAMBOO_MOSAIC, Properties::of() .map_color(MapColor::COLOR_YELLOW) .instrument(NoteBlockInstrument::BASS) .strength_and_resistance(2.0, 3.0)  .ignited_by_lava());
    pub const OAK_SAPLING: Block = register_with(registry, BlockItemIds::OAK_SAPLING, SaplingBlock::new, Properties::of() .map_color(MapColor::PLANT) .no_collision() .random_ticks() .instabreak()  .push_reaction(PushReaction::POPPED));
    pub const SPRUCE_SAPLING: Block = register_with(registry, BlockItemIds::SPRUCE_SAPLING, SaplingBlock::new, Properties::of() .map_color(MapColor::PLANT) .no_collision() .random_ticks() .instabreak()  .push_reaction(PushReaction::POPPED));
    pub const BIRCH_SAPLING: Block = register_with(registry, BlockItemIds::BIRCH_SAPLING, SaplingBlock::new, Properties::of() .map_color(MapColor::PLANT) .no_collision() .random_ticks() .instabreak()  .push_reaction(PushReaction::POPPED));
    pub const JUNGLE_SAPLING: Block = register_with(registry, BlockItemIds::JUNGLE_SAPLING, SaplingBlock::new, Properties::of() .map_color(MapColor::PLANT) .no_collision() .random_ticks() .instabreak()  .push_reaction(PushReaction::POPPED));
    pub const ACACIA_SAPLING: Block = register_with(registry, BlockItemIds::ACACIA_SAPLING, SaplingBlock::new, Properties::of() .map_color(MapColor::PLANT) .no_collision() .random_ticks() .instabreak()  .push_reaction(PushReaction::POPPED));
    pub const CHERRY_SAPLING: Block = register_with(registry, BlockItemIds::CHERRY_SAPLING, SaplingBlock::new, Properties::of() .map_color(MapColor::COLOR_PINK) .no_collision() .random_ticks() .instabreak()  .push_reaction(PushReaction::POPPED));
    pub const DARK_OAK_SAPLING: Block = register_with(registry, BlockItemIds::DARK_OAK_SAPLING, SaplingBlock::new, Properties::of() .map_color(MapColor::PLANT) .no_collision() .random_ticks() .instabreak()  .push_reaction(PushReaction::POPPED));
    pub const PALE_OAK_SAPLING: Block = register_with(registry, BlockItemIds::PALE_OAK_SAPLING, SaplingBlock::new, Properties::of() .map_color(MapColor::METAL) .no_collision() .random_ticks() .instabreak()  .push_reaction(PushReaction::POPPED));
    pub const MANGROVE_PROPAGULE: Block = register_with(registry, BlockItemIds::MANGROVE_PROPAGULE, MangrovePropaguleBlock::new, Properties::of() .map_color(MapColor::PLANT) .no_collision() .random_ticks() .instabreak()  .offset_type(OffsetType::XZ) .push_reaction(PushReaction::POPPED));
    pub const POPLAR_SAPLING: Block = register_with(registry, BlockItemIds::POPLAR_SAPLING, SaplingBlock::new, Properties::of() .map_color(MapColor::METAL) .no_collision() .random_ticks() .instabreak()  .push_reaction(PushReaction::POPPED));
    pub const BEDROCK: Block = register(registry, BlockItemIds::BEDROCK, Properties::of() .map_color(MapColor::STONE) .instrument(NoteBlockInstrument::BASEDRUM) .strength_and_resistance(-1.0, 3600000.0) .no_loot_table() );
    pub const WATER: Block = register_with(registry, BlockIds::WATER, |p| LiquidBlock::new(Fluids::WATER, p), Properties::of().map_color(MapColor::WATER).replaceable().no_collision().strength(100.0).push_reaction(PushReaction::POPPED).no_loot_table().liquid());
    pub const LAVA: Block = register_with(registry, BlockIds::LAVA, |p| LiquidBlock::new(Fluids::LAVA, p), Properties::of().map_color(MapColor::FIRE).replaceable().no_collision().random_ticks().strength(100.0).light_level(|_| 15).push_reaction(PushReaction::POPPED).no_loot_table().liquid());
    pub const SAND: Block = register_with(registry, BlockItemIds::SAND, SandBlock::new, Properties::of().map_color(MapColor::SAND).instrument(NoteBlockInstrument::SNARE).strength(0.5));
    pub const SUSPICIOUS_SAND: Block = register_with(registry, BlockItemIds::SUSPICIOUS_SAND, BrushableBlock::new, Properties::of() .map_color(MapColor::SAND) .instrument(NoteBlockInstrument::SNARE) .strength(0.25)  .push_reaction(PushReaction::POPPED));
    pub const RED_SAND: Block = register_with(registry, BlockItemIds::RED_SAND, SandBlock::new, Properties::of().map_color(MapColor::COLOR_ORANGE).instrument(NoteBlockInstrument::SNARE).strength(0.5));
    pub const GRAVEL: Block = register_with(registry, BlockItemIds::GRAVEL, ColoredFallingBlock::new, Properties::of().map_color(MapColor::STONE).instrument(NoteBlockInstrument::SNARE).strength(0.6));
    pub const SUSPICIOUS_GRAVEL: Block = register_with(registry, BlockItemIds::SUSPICIOUS_GRAVEL, BrushableBlock::new, Properties::of() .map_color(MapColor::STONE) .instrument(NoteBlockInstrument::SNARE) .strength(0.25)  .push_reaction(PushReaction::POPPED));
    pub const GOLD_ORE: Block = register_with(registry, BlockItemIds::GOLD_ORE, DropExperienceBlock::new, Properties::of().map_color(MapColor::STONE).instrument(NoteBlockInstrument::BASEDRUM).requires_correct_tool_for_drops().strength_and_resistance(3.0, 3.0));
    pub const DEEPSLATE_GOLD_ORE: Block = register_with(registry, BlockItemIds::DEEPSLATE_GOLD_ORE, DropExperienceBlock::new, Properties::of_legacy_copy(&registry.get(Blocks::GOLD_ORE)).map_color(MapColor::DEEPSLATE).strength_and_resistance(4.5, 3.0));
    pub const IRON_ORE: Block = register_with(registry, BlockItemIds::IRON_ORE, DropExperienceBlock::new, Properties::of().map_color(MapColor::STONE).instrument(NoteBlockInstrument::BASEDRUM).requires_correct_tool_for_drops().strength_and_resistance(3.0, 3.0));
    pub const DEEPSLATE_IRON_ORE: Block = register_with(registry, BlockItemIds::DEEPSLATE_IRON_ORE, DropExperienceBlock::new, Properties::of_legacy_copy(&registry.get(Blocks::IRON_ORE)).map_color(MapColor::DEEPSLATE).strength_and_resistance(4.5, 3.0));
    pub const COAL_ORE: Block = register_with(registry, BlockItemIds::COAL_ORE, DropExperienceBlock::new, Properties::of().map_color(MapColor::STONE).instrument(NoteBlockInstrument::BASEDRUM).requires_correct_tool_for_drops().strength_and_resistance(3.0, 3.0));
    pub const DEEPSLATE_COAL_ORE: Block = register_with(registry, BlockItemIds::DEEPSLATE_COAL_ORE, DropExperienceBlock::new, Properties::of_legacy_copy(&registry.get(Blocks::COAL_ORE)).map_color(MapColor::DEEPSLATE).strength_and_resistance(4.5, 3.0));
    pub const NETHER_GOLD_ORE: Block = register_with(registry, BlockItemIds::NETHER_GOLD_ORE, DropExperienceBlock::new, Properties::of() .map_color(MapColor::NETHER) .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .strength_and_resistance(3.0, 3.0) );
    pub const OAK_LOG: Block = register_with(registry, BlockItemIds::OAK_LOG, RotatedPillarBlock::new, log_properties(MapColor::WOOD, MapColor::PODZOL));
    pub const SPRUCE_LOG: Block = register_with(registry, BlockItemIds::SPRUCE_LOG, RotatedPillarBlock::new, log_properties(MapColor::PODZOL, MapColor::COLOR_BROWN));
    pub const BIRCH_LOG: Block = register_with(registry, BlockItemIds::BIRCH_LOG, RotatedPillarBlock::new, log_properties(MapColor::SAND, MapColor::QUARTZ));
    pub const JUNGLE_LOG: Block = register_with(registry, BlockItemIds::JUNGLE_LOG, RotatedPillarBlock::new, log_properties(MapColor::DIRT, MapColor::PODZOL));
    pub const ACACIA_LOG: Block = register_with(registry, BlockItemIds::ACACIA_LOG, RotatedPillarBlock::new, log_properties(MapColor::COLOR_ORANGE, MapColor::STONE));
    pub const CHERRY_LOG: Block = register_with(registry, BlockItemIds::CHERRY_LOG, RotatedPillarBlock::new, log_properties(MapColor::TERRACOTTA_WHITE, MapColor::TERRACOTTA_GRAY));
    pub const DARK_OAK_LOG: Block = register_with(registry, BlockItemIds::DARK_OAK_LOG, RotatedPillarBlock::new, log_properties(MapColor::COLOR_BROWN, MapColor::COLOR_BROWN));
    pub const PALE_OAK_LOG: Block = register_with(registry, BlockItemIds::PALE_OAK_LOG, RotatedPillarBlock::new, log_properties(registry.get(Blocks::PALE_OAK_PLANKS).default_map_color(), registry.get(Blocks::PALE_OAK_WOOD).default_map_color()));
    pub const MANGROVE_LOG: Block = register_with(registry, BlockItemIds::MANGROVE_LOG, RotatedPillarBlock::new, log_properties(MapColor::COLOR_RED, MapColor::PODZOL));
    pub const POPLAR_LOG: Block = register_with(registry, BlockItemIds::POPLAR_LOG, RotatedPillarBlock::new, log_properties(MapColor::COLOR_LIGHT_GRAY, MapColor::PODZOL));
    pub const MANGROVE_ROOTS: Block = register_with(registry, BlockItemIds::MANGROVE_ROOTS, MangroveRootsBlock::new, Properties::of() .map_color(MapColor::PODZOL) .instrument(NoteBlockInstrument::BASS) .strength(0.7)  .no_occlusion() .is_suffocating(StatePredicate::State(None, never)) .no_occlusion() .ignited_by_lava());
    pub const MUDDY_MANGROVE_ROOTS: Block = register_with(registry, BlockItemIds::MUDDY_MANGROVE_ROOTS, RotatedPillarBlock::new, Properties::of().map_color(MapColor::PODZOL).strength(0.7));
    pub const BAMBOO_BLOCK: Block = register_with(registry, BlockItemIds::BAMBOO_BLOCK, RotatedPillarBlock::new, log_properties(MapColor::COLOR_YELLOW, MapColor::PLANT));
    pub const STRIPPED_SPRUCE_LOG: Block = register_with(registry, BlockItemIds::STRIPPED_SPRUCE_LOG, RotatedPillarBlock::new, log_properties(MapColor::PODZOL, MapColor::PODZOL));
    pub const STRIPPED_BIRCH_LOG: Block = register_with(registry, BlockItemIds::STRIPPED_BIRCH_LOG, RotatedPillarBlock::new, log_properties(MapColor::SAND, MapColor::SAND));
    pub const STRIPPED_JUNGLE_LOG: Block = register_with(registry, BlockItemIds::STRIPPED_JUNGLE_LOG, RotatedPillarBlock::new, log_properties(MapColor::DIRT, MapColor::DIRT));
    pub const STRIPPED_ACACIA_LOG: Block = register_with(registry, BlockItemIds::STRIPPED_ACACIA_LOG, RotatedPillarBlock::new, log_properties(MapColor::COLOR_ORANGE, MapColor::COLOR_ORANGE));
    pub const STRIPPED_CHERRY_LOG: Block = register_with(registry, BlockItemIds::STRIPPED_CHERRY_LOG, RotatedPillarBlock::new, log_properties(MapColor::TERRACOTTA_WHITE, MapColor::TERRACOTTA_PINK));
    pub const STRIPPED_DARK_OAK_LOG: Block = register_with(registry, BlockItemIds::STRIPPED_DARK_OAK_LOG, RotatedPillarBlock::new, log_properties(MapColor::COLOR_BROWN, MapColor::COLOR_BROWN));
    pub const STRIPPED_PALE_OAK_LOG: Block = register_with(registry, BlockItemIds::STRIPPED_PALE_OAK_LOG, RotatedPillarBlock::new, log_properties(registry.get(Blocks::PALE_OAK_PLANKS).default_map_color(), registry.get(Blocks::PALE_OAK_PLANKS).default_map_color()));
    pub const STRIPPED_OAK_LOG: Block = register_with(registry, BlockItemIds::STRIPPED_OAK_LOG, RotatedPillarBlock::new, log_properties(MapColor::WOOD, MapColor::WOOD));
    pub const STRIPPED_MANGROVE_LOG: Block = register_with(registry, BlockItemIds::STRIPPED_MANGROVE_LOG, RotatedPillarBlock::new, log_properties(MapColor::COLOR_RED, MapColor::COLOR_RED));
    pub const STRIPPED_POPLAR_LOG: Block = register_with(registry, BlockItemIds::STRIPPED_POPLAR_LOG, RotatedPillarBlock::new, log_properties(registry.get(Blocks::POPLAR_PLANKS).default_map_color(), registry.get(Blocks::POPLAR_PLANKS).default_map_color()));
    pub const STRIPPED_BAMBOO_BLOCK: Block = register_with(registry, BlockItemIds::STRIPPED_BAMBOO_BLOCK, RotatedPillarBlock::new, log_properties(MapColor::COLOR_YELLOW, MapColor::COLOR_YELLOW));
    pub const OAK_WOOD: Block = register_with(registry, BlockItemIds::OAK_WOOD, RotatedPillarBlock::new, Properties::of().map_color(MapColor::WOOD).instrument(NoteBlockInstrument::BASS).strength(2.0).ignited_by_lava());
    pub const SPRUCE_WOOD: Block = register_with(registry, BlockItemIds::SPRUCE_WOOD, RotatedPillarBlock::new, Properties::of().map_color(MapColor::PODZOL).instrument(NoteBlockInstrument::BASS).strength(2.0).ignited_by_lava());
    pub const BIRCH_WOOD: Block = register_with(registry, BlockItemIds::BIRCH_WOOD, RotatedPillarBlock::new, Properties::of().map_color(MapColor::SAND).instrument(NoteBlockInstrument::BASS).strength(2.0).ignited_by_lava());
    pub const JUNGLE_WOOD: Block = register_with(registry, BlockItemIds::JUNGLE_WOOD, RotatedPillarBlock::new, Properties::of().map_color(MapColor::DIRT).instrument(NoteBlockInstrument::BASS).strength(2.0).ignited_by_lava());
    pub const ACACIA_WOOD: Block = register_with(registry, BlockItemIds::ACACIA_WOOD, RotatedPillarBlock::new, Properties::of().map_color(MapColor::COLOR_GRAY).instrument(NoteBlockInstrument::BASS).strength(2.0).ignited_by_lava());
    pub const CHERRY_WOOD: Block = register_with(registry, BlockItemIds::CHERRY_WOOD, RotatedPillarBlock::new, Properties::of() .map_color(MapColor::TERRACOTTA_GRAY) .instrument(NoteBlockInstrument::BASS) .strength(2.0)  .ignited_by_lava());
    pub const DARK_OAK_WOOD: Block = register_with(registry, BlockItemIds::DARK_OAK_WOOD, RotatedPillarBlock::new, Properties::of().map_color(MapColor::COLOR_BROWN).instrument(NoteBlockInstrument::BASS).strength(2.0).ignited_by_lava());
    pub const MANGROVE_WOOD: Block = register_with(registry, BlockItemIds::MANGROVE_WOOD, RotatedPillarBlock::new, Properties::of().map_color(MapColor::COLOR_RED).instrument(NoteBlockInstrument::BASS).strength(2.0).ignited_by_lava());
    pub const POPLAR_WOOD: Block = register_with(registry, BlockItemIds::POPLAR_WOOD, RotatedPillarBlock::new, Properties::of().map_color(MapColor::PODZOL).instrument(NoteBlockInstrument::BASS).strength(2.0).ignited_by_lava());
    pub const STRIPPED_OAK_WOOD: Block = register_with(registry, BlockItemIds::STRIPPED_OAK_WOOD, RotatedPillarBlock::new, Properties::of().map_color(MapColor::WOOD).instrument(NoteBlockInstrument::BASS).strength(2.0).ignited_by_lava());
    pub const STRIPPED_SPRUCE_WOOD: Block = register_with(registry, BlockItemIds::STRIPPED_SPRUCE_WOOD, RotatedPillarBlock::new, Properties::of().map_color(MapColor::PODZOL).instrument(NoteBlockInstrument::BASS).strength(2.0).ignited_by_lava());
    pub const STRIPPED_BIRCH_WOOD: Block = register_with(registry, BlockItemIds::STRIPPED_BIRCH_WOOD, RotatedPillarBlock::new, Properties::of().map_color(MapColor::SAND).instrument(NoteBlockInstrument::BASS).strength(2.0).ignited_by_lava());
    pub const STRIPPED_JUNGLE_WOOD: Block = register_with(registry, BlockItemIds::STRIPPED_JUNGLE_WOOD, RotatedPillarBlock::new, Properties::of().map_color(MapColor::DIRT).instrument(NoteBlockInstrument::BASS).strength(2.0).ignited_by_lava());
    pub const STRIPPED_ACACIA_WOOD: Block = register_with(registry, BlockItemIds::STRIPPED_ACACIA_WOOD, RotatedPillarBlock::new, Properties::of() .map_color(MapColor::COLOR_ORANGE) .instrument(NoteBlockInstrument::BASS) .strength(2.0)  .ignited_by_lava());
    pub const STRIPPED_CHERRY_WOOD: Block = register_with(registry, BlockItemIds::STRIPPED_CHERRY_WOOD, RotatedPillarBlock::new, Properties::of() .map_color(MapColor::TERRACOTTA_PINK) .instrument(NoteBlockInstrument::BASS) .strength(2.0)  .ignited_by_lava());
    pub const STRIPPED_DARK_OAK_WOOD: Block = register_with(registry, BlockItemIds::STRIPPED_DARK_OAK_WOOD, RotatedPillarBlock::new, Properties::of().map_color(MapColor::COLOR_BROWN).instrument(NoteBlockInstrument::BASS).strength(2.0).ignited_by_lava());
    pub const STRIPPED_PALE_OAK_WOOD: Block = register_with(registry, BlockItemIds::STRIPPED_PALE_OAK_WOOD, RotatedPillarBlock::new, Properties::of() .map_color(registry.get(Blocks::PALE_OAK_PLANKS).default_map_color()) .instrument(NoteBlockInstrument::BASS) .strength(2.0)  .ignited_by_lava());
    pub const STRIPPED_MANGROVE_WOOD: Block = register_with(registry, BlockItemIds::STRIPPED_MANGROVE_WOOD, RotatedPillarBlock::new, log_properties(MapColor::COLOR_RED, MapColor::COLOR_RED));
    pub const STRIPPED_POPLAR_WOOD: Block = register_with(registry, BlockItemIds::STRIPPED_POPLAR_WOOD, RotatedPillarBlock::new, Properties::of() .map_color(registry.get(Blocks::POPLAR_PLANKS).default_map_color()) .instrument(NoteBlockInstrument::BASS) .strength(2.0)  .ignited_by_lava());
    pub const OAK_LEAVES: Block = register_with(registry, BlockItemIds::OAK_LEAVES, TintedParticleLeavesBlock::new, leaves_properties());
    pub const SPRUCE_LEAVES: Block = register_with(registry, BlockItemIds::SPRUCE_LEAVES, LeavesBlock::new, leaves_properties());
    pub const BIRCH_LEAVES: Block = register_with(registry, BlockItemIds::BIRCH_LEAVES, TintedParticleLeavesBlock::new, leaves_properties());
    pub const JUNGLE_LEAVES: Block = register_with(registry, BlockItemIds::JUNGLE_LEAVES, TintedParticleLeavesBlock::new, leaves_properties());
    pub const ACACIA_LEAVES: Block = register_with(registry, BlockItemIds::ACACIA_LEAVES, TintedParticleLeavesBlock::new, leaves_properties());
    pub const CHERRY_LEAVES: Block = register_with(registry, BlockItemIds::CHERRY_LEAVES, UntintedParticleLeavesBlock::new, Properties::of() .map_color(MapColor::COLOR_PINK) .strength(0.2) .random_ticks()  .no_occlusion()  .is_suffocating(StatePredicate::State(None, never)) .ignited_by_lava() .push_reaction(PushReaction::POPPED) .is_redstone_conductor(StatePredicate::State(None, never)));
    pub const DARK_OAK_LEAVES: Block = register_with(registry, BlockItemIds::DARK_OAK_LEAVES, TintedParticleLeavesBlock::new, leaves_properties());
    pub const PALE_OAK_LEAVES: Block = register_with(registry, BlockItemIds::PALE_OAK_LEAVES, UntintedParticleLeavesBlock::new, Properties::of() .map_color(MapColor::METAL) .strength(0.2) .random_ticks()  .no_occlusion()  .is_suffocating(StatePredicate::State(None, never)) .ignited_by_lava() .push_reaction(PushReaction::POPPED) .is_redstone_conductor(StatePredicate::State(None, never)));
    pub const MANGROVE_LEAVES: Block = register_with(registry, BlockItemIds::MANGROVE_LEAVES, MangroveLeavesBlock::new, leaves_properties());
    pub const RED_POPLAR_LEAVES: Block = register_with(registry, BlockItemIds::RED_POPLAR_LEAVES, UntintedParticleLeavesBlock::new, leaves_properties().map_color(MapColor::COLOR_RED));
    pub const ORANGE_POPLAR_LEAVES: Block = register_with(registry, BlockItemIds::ORANGE_POPLAR_LEAVES, UntintedParticleLeavesBlock::new, leaves_properties().map_color(MapColor::COLOR_ORANGE));
    pub const YELLOW_POPLAR_LEAVES: Block = register_with(registry, BlockItemIds::YELLOW_POPLAR_LEAVES, UntintedParticleLeavesBlock::new, leaves_properties().map_color(MapColor::COLOR_YELLOW));
    pub const AZALEA_LEAVES: Block = register_with(registry, BlockItemIds::AZALEA_LEAVES, UntintedParticleLeavesBlock::new, leaves_properties());
    pub const FLOWERING_AZALEA_LEAVES: Block = register_with(registry, BlockItemIds::FLOWERING_AZALEA_LEAVES, UntintedParticleLeavesBlock::new, leaves_properties());
    pub const SPONGE: Block = register_with(registry, BlockItemIds::SPONGE, SpongeBlock::new, Properties::of().map_color(MapColor::COLOR_YELLOW).strength(0.6));
    pub const WET_SPONGE: Block = register_with(registry, BlockItemIds::WET_SPONGE, WetSpongeBlock::new, Properties::of().map_color(MapColor::COLOR_YELLOW).strength(0.6));
    pub const GLASS: Block = register_with(registry, BlockItemIds::GLASS, TransparentBlock::new, Properties::of() .instrument(NoteBlockInstrument::HAT) .strength(0.3)  .no_occlusion()  .is_redstone_conductor(StatePredicate::State(None, never)) .is_suffocating(StatePredicate::State(None, never)));
    pub const LAPIS_ORE: Block = register_with(registry, BlockItemIds::LAPIS_ORE, DropExperienceBlock::new, Properties::of().map_color(MapColor::STONE).instrument(NoteBlockInstrument::BASEDRUM).requires_correct_tool_for_drops().strength_and_resistance(3.0, 3.0));
    pub const DEEPSLATE_LAPIS_ORE: Block = register_with(registry, BlockItemIds::DEEPSLATE_LAPIS_ORE, DropExperienceBlock::new, Properties::of_legacy_copy(&registry.get(Blocks::LAPIS_ORE)).map_color(MapColor::DEEPSLATE).strength_and_resistance(4.5, 3.0));
    pub const LAPIS_BLOCK: Block = register(registry, BlockItemIds::LAPIS_BLOCK, Properties::of().map_color(MapColor::LAPIS).requires_correct_tool_for_drops().strength_and_resistance(3.0, 3.0));
    pub const DISPENSER: Block = register_with(registry, BlockItemIds::DISPENSER, DispenserBlock::new, Properties::of().map_color(MapColor::STONE).instrument(NoteBlockInstrument::BASEDRUM).requires_correct_tool_for_drops().strength(3.5));
    pub const SANDSTONE: Block = register(registry, BlockItemIds::SANDSTONE, Properties::of().map_color(MapColor::SAND).instrument(NoteBlockInstrument::BASEDRUM).requires_correct_tool_for_drops().strength(0.8));
    pub const CHISELED_SANDSTONE: Block = register(registry, BlockItemIds::CHISELED_SANDSTONE, Properties::of().map_color(MapColor::SAND).instrument(NoteBlockInstrument::BASEDRUM).requires_correct_tool_for_drops().strength(0.8));
    pub const CUT_SANDSTONE: Block = register(registry, BlockItemIds::CUT_SANDSTONE, Properties::of().map_color(MapColor::SAND).instrument(NoteBlockInstrument::BASEDRUM).requires_correct_tool_for_drops().strength(0.8));
    pub const NOTE_BLOCK: Block = register_with(registry, BlockItemIds::NOTE_BLOCK, NoteBlock::new, Properties::of().map_color(MapColor::WOOD).instrument(NoteBlockInstrument::BASS).strength(0.8).ignited_by_lava());
    pub const BED: ColorCollection<Block> = ColorCollection::register_blocks(registry, BlockItemIds::BED, |_, p| BedBlock::new(p), |color| Properties::of().map_color_fn(move |state| if state.get_value(AbstractBedBlock::PART) == BedPart::FOOT { color.get_map_color() } else { MapColor::WOOL }).strength(0.2).bounce_restitution(0.75).fall_distance_reduction(0.5).no_occlusion().ignited_by_lava().push_reaction(PushReaction::POPPED));
    pub const STRAW_BED: Block = register_with(registry, BlockItemIds::STRAW_BED, StrawBedBlock::new, Properties::of() .map_color(MapColor::COLOR_YELLOW)  .strength(0.2) .no_occlusion() .ignited_by_lava() .push_reaction(PushReaction::POPPED));
    pub const POWERED_RAIL: Block = register_with(registry, BlockItemIds::POWERED_RAIL, PoweredRailBlock::new, Properties::of().no_collision().strength(0.7));
    pub const DETECTOR_RAIL: Block = register_with(registry, BlockItemIds::DETECTOR_RAIL, DetectorRailBlock::new, Properties::of().no_collision().strength(0.7));
    pub const STICKY_PISTON: Block = register_with(registry, BlockItemIds::STICKY_PISTON, PistonBaseBlock::new, piston_properties());
    pub const COBWEB: Block = register_with(registry, BlockItemIds::COBWEB, WebBlock::new, Properties::of() .map_color(MapColor::WOOL)  .force_solid_on() .no_collision() .requires_correct_tool_for_drops() .strength(4.0) .push_reaction(PushReaction::POPPED));
    pub const SHORT_GRASS: Block = register_with(registry, BlockItemIds::SHORT_GRASS, TallGrassBlock::new, Properties::of() .map_color(MapColor::PLANT) .replaceable() .no_collision() .instabreak()  .offset_type(OffsetType::XYZ) .ignited_by_lava() .push_reaction(PushReaction::POPPED));
    pub const FERN: Block = register_with(registry, BlockItemIds::FERN, TallGrassBlock::new, Properties::of() .map_color(MapColor::PLANT) .replaceable() .no_collision() .instabreak()  .offset_type(OffsetType::XYZ) .ignited_by_lava() .push_reaction(PushReaction::POPPED));
    pub const DEAD_BUSH: Block = register_with(registry, BlockItemIds::DEAD_BUSH, DryVegetationBlock::new, Properties::of() .map_color(MapColor::WOOD) .replaceable() .no_collision() .instabreak()  .ignited_by_lava() .push_reaction(PushReaction::POPPED));
    pub const BUSH: Block = register_with(registry, BlockItemIds::BUSH, BushBlock::new, Properties::of() .map_color(MapColor::PLANT) .replaceable() .no_collision() .instabreak()  .ignited_by_lava() .push_reaction(PushReaction::POPPED));
    pub const RED_SHRUB: Block = register_with(registry, BlockItemIds::RED_SHRUB, BushBlock::new, Properties::of() .map_color(MapColor::CRIMSON_NYLIUM) .replaceable() .no_collision() .instabreak()  .ignited_by_lava() .push_reaction(PushReaction::POPPED));
    pub const SHORT_DRY_GRASS: Block = register_with(registry, BlockItemIds::SHORT_DRY_GRASS, ShortDryGrassBlock::new, Properties::of() .map_color(MapColor::COLOR_YELLOW) .replaceable() .no_collision() .instabreak()  .ignited_by_lava() .offset_type(OffsetType::XYZ) .push_reaction(PushReaction::POPPED));
    pub const TALL_DRY_GRASS: Block = register_with(registry, BlockItemIds::TALL_DRY_GRASS, TallDryGrassBlock::new, Properties::of() .map_color(MapColor::COLOR_YELLOW) .replaceable() .no_collision() .instabreak()  .ignited_by_lava() .offset_type(OffsetType::XYZ) .push_reaction(PushReaction::POPPED));
    pub const SEAGRASS: Block = register_with(registry, BlockItemIds::SEAGRASS, SeagrassBlock::new, Properties::of() .map_color(MapColor::WATER) .replaceable() .no_collision() .instabreak()  .push_reaction(PushReaction::POPPED));
    pub const TALL_SEAGRASS: Block = register_with(registry, BlockIds::TALL_SEAGRASS, TallSeagrassBlock::new, Properties::of() .map_color(MapColor::WATER) .replaceable() .no_collision() .instabreak()  .offset_type(OffsetType::XZ) .push_reaction(PushReaction::POPPED));
    pub const PISTON: Block = register_with(registry, BlockItemIds::PISTON, PistonBaseBlock::new, piston_properties());
    pub const PISTON_HEAD: Block = register_with(registry, BlockIds::PISTON_HEAD, PistonHeadBlock::new, Properties::of().map_color(MapColor::STONE).strength(1.5).no_loot_table().push_reaction(PushReaction::IMMOVEABLE));
    pub const WOOL: ColorCollection<Block> = ColorCollection::register_blocks(registry, BlockItemIds::WOOL, |_, p| Block::new(p), |color| Properties::of() .map_color(color.get_map_color()) .instrument(NoteBlockInstrument::GUITAR) .strength(0.8)  .ignited_by_lava());
    pub const WOOL_STAIRS: ColorCollection<Block> = ColorCollection::zip_map(&ColorCollection::VALUES, BlockItemIds::WOOL_STAIRS, |&color, id| register_stair(registry, id, *Blocks::WOOL.pick(color)));
    pub const WOOL_SLAB: ColorCollection<Block> = ColorCollection::zip_map(&ColorCollection::VALUES, BlockItemIds::WOOL_SLAB, |&color, id| register_slab(registry, id, *Blocks::WOOL.pick(color)));
    pub const MOVING_PISTON: Block = register_with(registry, BlockIds::MOVING_PISTON, MovingPistonBlock::new, Properties::of() .map_color(MapColor::STONE) .force_solid_on() .strength(-1.0) .dynamic_shape() .no_loot_table() .no_occlusion()  .is_redstone_conductor(StatePredicate::State(None, never)) .is_suffocating(StatePredicate::State(None, never)) .push_reaction(PushReaction::IMMOVEABLE));
    pub const DANDELION: Block = register_with(registry, BlockItemIds::DANDELION, FlowerBlock::new, Properties::of() .map_color(MapColor::PLANT) .no_collision() .instabreak()  .offset_type(OffsetType::XZ) .push_reaction(PushReaction::POPPED));
    pub const GOLDEN_DANDELION: Block = register_with(registry, BlockItemIds::GOLDEN_DANDELION, FlowerBlock::new, Properties::of() .map_color(MapColor::PLANT) .no_collision() .instabreak()  .offset_type(OffsetType::XZ) .push_reaction(PushReaction::POPPED));
    pub const TORCHFLOWER: Block = register_with(registry, BlockItemIds::TORCHFLOWER, FlowerBlock::new, Properties::of() .map_color(MapColor::PLANT) .no_collision() .instabreak()  .offset_type(OffsetType::XZ) .push_reaction(PushReaction::POPPED));
    pub const POPPY: Block = register_with(registry, BlockItemIds::POPPY, FlowerBlock::new, Properties::of() .map_color(MapColor::PLANT) .no_collision() .instabreak()  .offset_type(OffsetType::XZ) .push_reaction(PushReaction::POPPED));
    pub const BLUE_ORCHID: Block = register_with(registry, BlockItemIds::BLUE_ORCHID, FlowerBlock::new, Properties::of() .map_color(MapColor::PLANT) .no_collision() .instabreak()  .offset_type(OffsetType::XZ) .push_reaction(PushReaction::POPPED));
    pub const ALLIUM: Block = register_with(registry, BlockItemIds::ALLIUM, FlowerBlock::new, Properties::of() .map_color(MapColor::PLANT) .no_collision() .instabreak()  .offset_type(OffsetType::XZ) .push_reaction(PushReaction::POPPED));
    pub const AZURE_BLUET: Block = register_with(registry, BlockItemIds::AZURE_BLUET, FlowerBlock::new, Properties::of() .map_color(MapColor::PLANT) .no_collision() .instabreak()  .offset_type(OffsetType::XZ) .push_reaction(PushReaction::POPPED));
    pub const RED_TULIP: Block = register_with(registry, BlockItemIds::RED_TULIP, FlowerBlock::new, Properties::of() .map_color(MapColor::PLANT) .no_collision() .instabreak()  .offset_type(OffsetType::XZ) .push_reaction(PushReaction::POPPED));
    pub const ORANGE_TULIP: Block = register_with(registry, BlockItemIds::ORANGE_TULIP, FlowerBlock::new, Properties::of() .map_color(MapColor::PLANT) .no_collision() .instabreak()  .offset_type(OffsetType::XZ) .push_reaction(PushReaction::POPPED));
    pub const WHITE_TULIP: Block = register_with(registry, BlockItemIds::WHITE_TULIP, FlowerBlock::new, Properties::of() .map_color(MapColor::PLANT) .no_collision() .instabreak()  .offset_type(OffsetType::XZ) .push_reaction(PushReaction::POPPED));
    pub const PINK_TULIP: Block = register_with(registry, BlockItemIds::PINK_TULIP, FlowerBlock::new, Properties::of() .map_color(MapColor::PLANT) .no_collision() .instabreak()  .offset_type(OffsetType::XZ) .push_reaction(PushReaction::POPPED));
    pub const OXEYE_DAISY: Block = register_with(registry, BlockItemIds::OXEYE_DAISY, FlowerBlock::new, Properties::of() .map_color(MapColor::PLANT) .no_collision() .instabreak()  .offset_type(OffsetType::XZ) .push_reaction(PushReaction::POPPED));
    pub const CORNFLOWER: Block = register_with(registry, BlockItemIds::CORNFLOWER, FlowerBlock::new, Properties::of() .map_color(MapColor::PLANT) .no_collision() .instabreak()  .offset_type(OffsetType::XZ) .push_reaction(PushReaction::POPPED));
    pub const WITHER_ROSE: Block = register_with(registry, BlockItemIds::WITHER_ROSE, WitherRoseBlock::new, Properties::of() .map_color(MapColor::PLANT) .no_collision() .instabreak()  .offset_type(OffsetType::XZ) .push_reaction(PushReaction::POPPED));
    pub const LILY_OF_THE_VALLEY: Block = register_with(registry, BlockItemIds::LILY_OF_THE_VALLEY, FlowerBlock::new, Properties::of() .map_color(MapColor::PLANT) .no_collision() .instabreak()  .offset_type(OffsetType::XZ) .push_reaction(PushReaction::POPPED));
    pub const BROWN_MUSHROOM: Block = register_with(registry, BlockItemIds::BROWN_MUSHROOM, MushroomBlock::new, Properties::of() .map_color(MapColor::COLOR_BROWN) .no_collision() .random_ticks() .instabreak()  .light_level(|_| 1) .post_process(post_process_self) .push_reaction(PushReaction::POPPED));
    pub const RED_MUSHROOM: Block = register_with(registry, BlockItemIds::RED_MUSHROOM, MushroomBlock::new, Properties::of() .map_color(MapColor::COLOR_RED) .no_collision() .random_ticks() .instabreak()  .post_process(post_process_self) .push_reaction(PushReaction::POPPED));
    pub const GOLD_BLOCK: Block = register(registry, BlockItemIds::GOLD_BLOCK, Properties::of() .map_color(MapColor::GOLD) .instrument(NoteBlockInstrument::BELL) .requires_correct_tool_for_drops() .strength_and_resistance(3.0, 6.0) );
    pub const IRON_BLOCK: Block = register(registry, BlockItemIds::IRON_BLOCK, Properties::of() .map_color(MapColor::METAL) .instrument(NoteBlockInstrument::IRON_XYLOPHONE) .requires_correct_tool_for_drops() .strength_and_resistance(5.0, 6.0) );
    pub const BRICKS: Block = register(registry, BlockItemIds::BRICKS, Properties::of().map_color(MapColor::COLOR_RED).instrument(NoteBlockInstrument::BASEDRUM).requires_correct_tool_for_drops().strength_and_resistance(2.0, 6.0));
    pub const TNT: Block = register_with(registry, BlockItemIds::TNT, TntBlock::new, Properties::of().map_color(MapColor::FIRE).instabreak().ignited_by_lava().is_redstone_conductor(StatePredicate::State(None, never)));
    pub const BOOKSHELF: Block = register(registry, BlockItemIds::BOOKSHELF, Properties::of().map_color(MapColor::WOOD).instrument(NoteBlockInstrument::BASS).strength(1.5).ignited_by_lava());
    pub const CHISELED_BOOKSHELF: Block = register_with(registry, BlockItemIds::CHISELED_BOOKSHELF, ChiseledBookShelfBlock::new, Properties::of() .map_color(MapColor::WOOD) .instrument(NoteBlockInstrument::BASS) .strength(1.5)  .ignited_by_lava());
    pub const ACACIA_SHELF: Block = register_with(registry, BlockItemIds::ACACIA_SHELF, ShelfBlock::new, Properties::of() .map_color(registry.get(Blocks::ACACIA_PLANKS).default_map_color()) .instrument(NoteBlockInstrument::BASS)  .ignited_by_lava() .strength_and_resistance(2.0, 3.0));
    pub const BAMBOO_SHELF: Block = register_with(registry, BlockItemIds::BAMBOO_SHELF, ShelfBlock::new, Properties::of() .map_color(registry.get(Blocks::BAMBOO_PLANKS).default_map_color()) .instrument(NoteBlockInstrument::BASS)  .ignited_by_lava() .strength_and_resistance(2.0, 3.0));
    pub const BIRCH_SHELF: Block = register_with(registry, BlockItemIds::BIRCH_SHELF, ShelfBlock::new, Properties::of() .map_color(registry.get(Blocks::BIRCH_PLANKS).default_map_color()) .instrument(NoteBlockInstrument::BASS)  .ignited_by_lava() .strength_and_resistance(2.0, 3.0));
    pub const CHERRY_SHELF: Block = register_with(registry, BlockItemIds::CHERRY_SHELF, ShelfBlock::new, Properties::of() .map_color(registry.get(Blocks::CHERRY_PLANKS).default_map_color()) .instrument(NoteBlockInstrument::BASS)  .ignited_by_lava() .strength_and_resistance(2.0, 3.0));
    pub const CRIMSON_SHELF: Block = register_with(registry, BlockItemIds::CRIMSON_SHELF, ShelfBlock::new, Properties::of() .map_color(MapColor::CRIMSON_STEM) .instrument(NoteBlockInstrument::BASS)  .ignited_by_lava() .strength_and_resistance(2.0, 3.0));
    pub const DARK_OAK_SHELF: Block = register_with(registry, BlockItemIds::DARK_OAK_SHELF, ShelfBlock::new, Properties::of() .map_color(registry.get(Blocks::DARK_OAK_LOG).default_map_color()) .instrument(NoteBlockInstrument::BASS)  .ignited_by_lava() .strength_and_resistance(2.0, 3.0));
    pub const JUNGLE_SHELF: Block = register_with(registry, BlockItemIds::JUNGLE_SHELF, ShelfBlock::new, Properties::of() .map_color(registry.get(Blocks::JUNGLE_LOG).default_map_color()) .instrument(NoteBlockInstrument::BASS)  .ignited_by_lava() .strength_and_resistance(2.0, 3.0));
    pub const MANGROVE_SHELF: Block = register_with(registry, BlockItemIds::MANGROVE_SHELF, ShelfBlock::new, Properties::of() .map_color(registry.get(Blocks::MANGROVE_LOG).default_map_color()) .instrument(NoteBlockInstrument::BASS)  .ignited_by_lava() .strength_and_resistance(2.0, 3.0));
    pub const OAK_SHELF: Block = register_with(registry, BlockItemIds::OAK_SHELF, ShelfBlock::new, Properties::of() .map_color(registry.get(Blocks::OAK_PLANKS).default_map_color()) .instrument(NoteBlockInstrument::BASS)  .ignited_by_lava() .strength_and_resistance(2.0, 3.0));
    pub const PALE_OAK_SHELF: Block = register_with(registry, BlockItemIds::PALE_OAK_SHELF, ShelfBlock::new, Properties::of() .map_color(registry.get(Blocks::PALE_OAK_PLANKS).default_map_color()) .instrument(NoteBlockInstrument::BASS)  .ignited_by_lava() .strength_and_resistance(2.0, 3.0));
    pub const POPLAR_SHELF: Block = register_with(registry, BlockItemIds::POPLAR_SHELF, ShelfBlock::new, Properties::of() .map_color(registry.get(Blocks::POPLAR_PLANKS).default_map_color()) .instrument(NoteBlockInstrument::BASS)  .ignited_by_lava() .strength_and_resistance(2.0, 3.0));
    pub const SPRUCE_SHELF: Block = register_with(registry, BlockItemIds::SPRUCE_SHELF, ShelfBlock::new, Properties::of() .map_color(registry.get(Blocks::SPRUCE_LOG).default_map_color()) .instrument(NoteBlockInstrument::BASS)  .ignited_by_lava() .strength_and_resistance(2.0, 3.0));
    pub const WARPED_SHELF: Block = register_with(registry, BlockItemIds::WARPED_SHELF, ShelfBlock::new, Properties::of() .map_color(MapColor::WARPED_STEM) .instrument(NoteBlockInstrument::BASS)  .ignited_by_lava() .strength_and_resistance(2.0, 3.0));
    pub const MOSSY_COBBLESTONE: Block = register(registry, BlockItemIds::MOSSY_COBBLESTONE, Properties::of().map_color(MapColor::STONE).instrument(NoteBlockInstrument::BASEDRUM).requires_correct_tool_for_drops().strength_and_resistance(2.0, 6.0));
    pub const OBSIDIAN: Block = register(registry, BlockItemIds::OBSIDIAN, Properties::of() .map_color(MapColor::COLOR_BLACK) .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .strength_and_resistance(50.0, 1200.0) .push_reaction(PushReaction::IMMOVEABLE));
    pub const TORCH: Block = register_with(registry, BlockItemIds::TORCH, TorchBlock::new, Properties::of().no_collision().instabreak().light_level(|_| 14).push_reaction(PushReaction::POPPED));
    pub const WALL_TORCH: Block = register_with(registry, BlockIds::WALL_TORCH, WallTorchBlock::new, wall_variant(registry, Blocks::TORCH, true).no_collision().instabreak().light_level(|_| 14).push_reaction(PushReaction::POPPED));
    pub const FIRE: Block = register_with(registry, BlockIds::FIRE, FireBlock::new, Properties::of() .map_color(MapColor::FIRE) .replaceable() .no_collision() .instabreak() .light_level(|_| 15)  .push_reaction(PushReaction::POPPED));
    pub const SOUL_FIRE: Block = register_with(registry, BlockIds::SOUL_FIRE, SoulFireBlock::new, Properties::of() .map_color(MapColor::COLOR_LIGHT_BLUE) .replaceable() .no_collision() .instabreak() .light_level(|_| 10)  .push_reaction(PushReaction::POPPED));
    pub const SPAWNER: Block = register_with(registry, BlockItemIds::SPAWNER, SpawnerBlock::new, Properties::of() .map_color(MapColor::STONE) .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .strength(5.0)  .no_occlusion());
    pub const CREAKING_HEART: Block = register_with(registry, BlockItemIds::CREAKING_HEART, CreakingHeartBlock::new, Properties::of().map_color(MapColor::COLOR_ORANGE).instrument(NoteBlockInstrument::BASEDRUM).strength(10.0));
    pub const OAK_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::OAK_STAIRS, Blocks::OAK_PLANKS);
    pub const CHEST: Block = register_with(registry, BlockItemIds::CHEST, ChestBlock::new, Properties::of().map_color(MapColor::WOOD).instrument(NoteBlockInstrument::BASS).strength(2.5).ignited_by_lava());
    pub const REDSTONE_WIRE: Block = register_with(registry, BlockItemIds::REDSTONE_DUST, RedstoneWireBlock::new, Properties::of().no_collision().instabreak().push_reaction(PushReaction::POPPED));
    pub const DIAMOND_ORE: Block = register_with(registry, BlockItemIds::DIAMOND_ORE, DropExperienceBlock::new, Properties::of().map_color(MapColor::STONE).instrument(NoteBlockInstrument::BASEDRUM).requires_correct_tool_for_drops().strength_and_resistance(3.0, 3.0));
    pub const DEEPSLATE_DIAMOND_ORE: Block = register_with(registry, BlockItemIds::DEEPSLATE_DIAMOND_ORE, DropExperienceBlock::new, Properties::of_legacy_copy(&registry.get(Blocks::DIAMOND_ORE)).map_color(MapColor::DEEPSLATE).strength_and_resistance(4.5, 3.0));
    pub const DIAMOND_BLOCK: Block = register(registry, BlockItemIds::DIAMOND_BLOCK, Properties::of().map_color(MapColor::DIAMOND).requires_correct_tool_for_drops().strength_and_resistance(5.0, 6.0));
    pub const CRAFTING_TABLE: Block = register_with(registry, BlockItemIds::CRAFTING_TABLE, CraftingTableBlock::new, Properties::of().map_color(MapColor::WOOD).instrument(NoteBlockInstrument::BASS).strength(2.5).ignited_by_lava());
    pub const WHEAT: Block = register_with(registry, BlockItemIds::WHEAT_CROP, CropBlock::new, Properties::of().map_color_fn(|state| if state.get_value(CropBlock::AGE) >= 6 { MapColor::COLOR_YELLOW } else { MapColor::PLANT }).no_collision().random_ticks().instabreak().push_reaction(PushReaction::POPPED));
    pub const FARMLAND: Block = register_with(registry, BlockItemIds::FARMLAND, FarmlandBlock::new, Properties::of().map_color(MapColor::DIRT).random_ticks().strength(0.6).is_suffocating(StatePredicate::State(None, always)));
    pub const FURNACE: Block = register_with(registry, BlockItemIds::FURNACE, FurnaceBlock::new, Properties::of() .map_color(MapColor::STONE) .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .strength(3.5) .light_level(lit_block_emission(13)));
    pub const OAK_SIGN: Block = register_with(registry, BlockItemIds::OAK_SIGN, StandingSignBlock::new, Properties::of().map_color(MapColor::WOOD).force_solid_on().instrument(NoteBlockInstrument::BASS).no_collision().strength(1.0).ignited_by_lava());
    pub const SPRUCE_SIGN: Block = register_with(registry, BlockItemIds::SPRUCE_SIGN, StandingSignBlock::new, Properties::of() .map_color(registry.get(Blocks::SPRUCE_LOG).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(1.0) .ignited_by_lava());
    pub const BIRCH_SIGN: Block = register_with(registry, BlockItemIds::BIRCH_SIGN, StandingSignBlock::new, Properties::of().map_color(MapColor::SAND).force_solid_on().instrument(NoteBlockInstrument::BASS).no_collision().strength(1.0).ignited_by_lava());
    pub const ACACIA_SIGN: Block = register_with(registry, BlockItemIds::ACACIA_SIGN, StandingSignBlock::new, Properties::of() .map_color(MapColor::COLOR_ORANGE) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(1.0) .ignited_by_lava());
    pub const CHERRY_SIGN: Block = register_with(registry, BlockItemIds::CHERRY_SIGN, StandingSignBlock::new, Properties::of() .map_color(registry.get(Blocks::CHERRY_PLANKS).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(1.0) .ignited_by_lava());
    pub const JUNGLE_SIGN: Block = register_with(registry, BlockItemIds::JUNGLE_SIGN, StandingSignBlock::new, Properties::of() .map_color(registry.get(Blocks::JUNGLE_LOG).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(1.0) .ignited_by_lava());
    pub const DARK_OAK_SIGN: Block = register_with(registry, BlockItemIds::DARK_OAK_SIGN, StandingSignBlock::new, Properties::of() .map_color(registry.get(Blocks::DARK_OAK_LOG).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(1.0) .ignited_by_lava());
    pub const PALE_OAK_SIGN: Block = register_with(registry, BlockItemIds::PALE_OAK_SIGN, StandingSignBlock::new, Properties::of() .map_color(registry.get(Blocks::PALE_OAK_PLANKS).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(1.0) .ignited_by_lava());
    pub const MANGROVE_SIGN: Block = register_with(registry, BlockItemIds::MANGROVE_SIGN, StandingSignBlock::new, Properties::of() .map_color(registry.get(Blocks::MANGROVE_LOG).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(1.0) .ignited_by_lava());
    pub const POPLAR_SIGN: Block = register_with(registry, BlockItemIds::POPLAR_SIGN, StandingSignBlock::new, Properties::of() .map_color(registry.get(Blocks::POPLAR_PLANKS).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(1.0) .ignited_by_lava());
    pub const BAMBOO_SIGN: Block = register_with(registry, BlockItemIds::BAMBOO_SIGN, StandingSignBlock::new, Properties::of() .map_color(registry.get(Blocks::BAMBOO_PLANKS).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(1.0) .ignited_by_lava());
    pub const OAK_DOOR: Block = register_with(registry, BlockItemIds::OAK_DOOR, DoorBlock::new, Properties::of() .map_color(registry.get(Blocks::OAK_PLANKS).default_map_color()) .instrument(NoteBlockInstrument::BASS) .strength(3.0) .no_occlusion() .ignited_by_lava() .push_reaction(PushReaction::POPPED));
    pub const LADDER: Block = register_with(registry, BlockItemIds::LADDER, LadderBlock::new, Properties::of().force_solid_off().strength(0.4).no_occlusion().push_reaction(PushReaction::POPPED));
    pub const RAIL: Block = register_with(registry, BlockItemIds::RAIL, RailBlock::new, Properties::of().no_collision().strength(0.7));
    pub const COBBLESTONE_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::COBBLESTONE_STAIRS, Blocks::COBBLESTONE);
    pub const OAK_WALL_SIGN: Block = register_with(registry, BlockIds::OAK_WALL_SIGN, WallSignBlock::new, wall_variant(registry, Blocks::OAK_SIGN, true).map_color(MapColor::WOOD).force_solid_on().instrument(NoteBlockInstrument::BASS).no_collision().strength(1.0).ignited_by_lava());
    pub const SPRUCE_WALL_SIGN: Block = register_with(registry, BlockIds::SPRUCE_WALL_SIGN, WallSignBlock::new, wall_variant(registry, Blocks::SPRUCE_SIGN, true) .map_color(registry.get(Blocks::SPRUCE_LOG).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(1.0) .ignited_by_lava());
    pub const BIRCH_WALL_SIGN: Block = register_with(registry, BlockIds::BIRCH_WALL_SIGN, WallSignBlock::new, wall_variant(registry, Blocks::BIRCH_SIGN, true).map_color(MapColor::SAND).force_solid_on().instrument(NoteBlockInstrument::BASS).no_collision().strength(1.0).ignited_by_lava());
    pub const ACACIA_WALL_SIGN: Block = register_with(registry, BlockIds::ACACIA_WALL_SIGN, WallSignBlock::new, wall_variant(registry, Blocks::ACACIA_SIGN, true) .map_color(MapColor::COLOR_ORANGE) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(1.0) .ignited_by_lava());
    pub const CHERRY_WALL_SIGN: Block = register_with(registry, BlockIds::CHERRY_WALL_SIGN, WallSignBlock::new, wall_variant(registry, Blocks::CHERRY_SIGN, true) .map_color(registry.get(Blocks::CHERRY_LOG).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(1.0) .ignited_by_lava());
    pub const JUNGLE_WALL_SIGN: Block = register_with(registry, BlockIds::JUNGLE_WALL_SIGN, WallSignBlock::new, wall_variant(registry, Blocks::JUNGLE_SIGN, true) .map_color(registry.get(Blocks::JUNGLE_LOG).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(1.0) .ignited_by_lava());
    pub const DARK_OAK_WALL_SIGN: Block = register_with(registry, BlockIds::DARK_OAK_WALL_SIGN, WallSignBlock::new, wall_variant(registry, Blocks::DARK_OAK_SIGN, true) .map_color(registry.get(Blocks::DARK_OAK_LOG).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(1.0) .ignited_by_lava());
    pub const PALE_OAK_WALL_SIGN: Block = register_with(registry, BlockIds::PALE_OAK_WALL_SIGN, WallSignBlock::new, wall_variant(registry, Blocks::PALE_OAK_SIGN, true) .map_color(registry.get(Blocks::PALE_OAK_PLANKS).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(1.0) .ignited_by_lava());
    pub const MANGROVE_WALL_SIGN: Block = register_with(registry, BlockIds::MANGROVE_WALL_SIGN, WallSignBlock::new, wall_variant(registry, Blocks::MANGROVE_SIGN, true) .map_color(registry.get(Blocks::MANGROVE_LOG).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(1.0) .ignited_by_lava());
    pub const POPLAR_WALL_SIGN: Block = register_with(registry, BlockIds::POPLAR_WALL_SIGN, WallSignBlock::new, wall_variant(registry, Blocks::POPLAR_SIGN, true) .map_color(registry.get(Blocks::POPLAR_PLANKS).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(1.0) .ignited_by_lava());
    pub const BAMBOO_WALL_SIGN: Block = register_with(registry, BlockIds::BAMBOO_WALL_SIGN, WallSignBlock::new, wall_variant(registry, Blocks::BAMBOO_SIGN, true) .map_color(registry.get(Blocks::BAMBOO_PLANKS).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(1.0) .ignited_by_lava());
    pub const OAK_HANGING_SIGN: Block = register_with(registry, BlockItemIds::OAK_HANGING_SIGN, CeilingHangingSignBlock::new, Properties::of() .map_color(registry.get(Blocks::OAK_LOG).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(1.0) .ignited_by_lava());
    pub const SPRUCE_HANGING_SIGN: Block = register_with(registry, BlockItemIds::SPRUCE_HANGING_SIGN, CeilingHangingSignBlock::new, Properties::of() .map_color(registry.get(Blocks::SPRUCE_LOG).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(1.0) .ignited_by_lava());
    pub const BIRCH_HANGING_SIGN: Block = register_with(registry, BlockItemIds::BIRCH_HANGING_SIGN, CeilingHangingSignBlock::new, Properties::of().map_color(MapColor::SAND).force_solid_on().instrument(NoteBlockInstrument::BASS).no_collision().strength(1.0).ignited_by_lava());
    pub const ACACIA_HANGING_SIGN: Block = register_with(registry, BlockItemIds::ACACIA_HANGING_SIGN, CeilingHangingSignBlock::new, Properties::of() .map_color(MapColor::COLOR_ORANGE) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(1.0) .ignited_by_lava());
    pub const CHERRY_HANGING_SIGN: Block = register_with(registry, BlockItemIds::CHERRY_HANGING_SIGN, CeilingHangingSignBlock::new, Properties::of() .map_color(MapColor::TERRACOTTA_PINK) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(1.0) .ignited_by_lava());
    pub const JUNGLE_HANGING_SIGN: Block = register_with(registry, BlockItemIds::JUNGLE_HANGING_SIGN, CeilingHangingSignBlock::new, Properties::of() .map_color(registry.get(Blocks::JUNGLE_LOG).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(1.0) .ignited_by_lava());
    pub const DARK_OAK_HANGING_SIGN: Block = register_with(registry, BlockItemIds::DARK_OAK_HANGING_SIGN, CeilingHangingSignBlock::new, Properties::of() .map_color(registry.get(Blocks::DARK_OAK_LOG).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(1.0) .ignited_by_lava());
    pub const PALE_OAK_HANGING_SIGN: Block = register_with(registry, BlockItemIds::PALE_OAK_HANGING_SIGN, CeilingHangingSignBlock::new, Properties::of() .map_color(registry.get(Blocks::PALE_OAK_PLANKS).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(1.0) .ignited_by_lava());
    pub const CRIMSON_HANGING_SIGN: Block = register_with(registry, BlockItemIds::CRIMSON_HANGING_SIGN, CeilingHangingSignBlock::new, Properties::of().map_color(MapColor::CRIMSON_STEM).force_solid_on().instrument(NoteBlockInstrument::BASS).no_collision().strength(1.0));
    pub const WARPED_HANGING_SIGN: Block = register_with(registry, BlockItemIds::WARPED_HANGING_SIGN, CeilingHangingSignBlock::new, Properties::of().map_color(MapColor::WARPED_STEM).force_solid_on().instrument(NoteBlockInstrument::BASS).no_collision().strength(1.0));
    pub const MANGROVE_HANGING_SIGN: Block = register_with(registry, BlockItemIds::MANGROVE_HANGING_SIGN, CeilingHangingSignBlock::new, Properties::of() .map_color(registry.get(Blocks::MANGROVE_LOG).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(1.0) .ignited_by_lava());
    pub const POPLAR_HANGING_SIGN: Block = register_with(registry, BlockItemIds::POPLAR_HANGING_SIGN, CeilingHangingSignBlock::new, Properties::of() .map_color(registry.get(Blocks::POPLAR_PLANKS).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(1.0) .ignited_by_lava());
    pub const BAMBOO_HANGING_SIGN: Block = register_with(registry, BlockItemIds::BAMBOO_HANGING_SIGN, CeilingHangingSignBlock::new, Properties::of() .map_color(MapColor::COLOR_YELLOW) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(1.0) .ignited_by_lava());
    pub const OAK_WALL_HANGING_SIGN: Block = register_with(registry, BlockIds::OAK_WALL_HANGING_SIGN, WallHangingSignBlock::new, wall_variant(registry, Blocks::OAK_HANGING_SIGN, true) .map_color(registry.get(Blocks::OAK_LOG).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(1.0) .ignited_by_lava());
    pub const SPRUCE_WALL_HANGING_SIGN: Block = register_with(registry, BlockIds::SPRUCE_WALL_HANGING_SIGN, WallHangingSignBlock::new, wall_variant(registry, Blocks::SPRUCE_HANGING_SIGN, true) .map_color(MapColor::WOOD) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(1.0) .ignited_by_lava());
    pub const BIRCH_WALL_HANGING_SIGN: Block = register_with(registry, BlockIds::BIRCH_WALL_HANGING_SIGN, WallHangingSignBlock::new, wall_variant(registry, Blocks::BIRCH_HANGING_SIGN, true) .map_color(MapColor::SAND) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(1.0) .ignited_by_lava());
    pub const ACACIA_WALL_HANGING_SIGN: Block = register_with(registry, BlockIds::ACACIA_WALL_HANGING_SIGN, WallHangingSignBlock::new, wall_variant(registry, Blocks::ACACIA_HANGING_SIGN, true) .map_color(MapColor::COLOR_ORANGE) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(1.0) .ignited_by_lava());
    pub const CHERRY_WALL_HANGING_SIGN: Block = register_with(registry, BlockIds::CHERRY_WALL_HANGING_SIGN, WallHangingSignBlock::new, wall_variant(registry, Blocks::CHERRY_HANGING_SIGN, true) .map_color(MapColor::TERRACOTTA_PINK) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(1.0) .ignited_by_lava());
    pub const JUNGLE_WALL_HANGING_SIGN: Block = register_with(registry, BlockIds::JUNGLE_WALL_HANGING_SIGN, WallHangingSignBlock::new, wall_variant(registry, Blocks::JUNGLE_HANGING_SIGN, true) .map_color(registry.get(Blocks::JUNGLE_LOG).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(1.0) .ignited_by_lava());
    pub const DARK_OAK_WALL_HANGING_SIGN: Block = register_with(registry, BlockIds::DARK_OAK_WALL_HANGING_SIGN, WallHangingSignBlock::new, wall_variant(registry, Blocks::DARK_OAK_HANGING_SIGN, true) .map_color(registry.get(Blocks::DARK_OAK_LOG).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(1.0) .ignited_by_lava());
    pub const PALE_OAK_WALL_HANGING_SIGN: Block = register_with(registry, BlockIds::PALE_OAK_WALL_HANGING_SIGN, WallHangingSignBlock::new, wall_variant(registry, Blocks::PALE_OAK_HANGING_SIGN, true) .map_color(registry.get(Blocks::PALE_OAK_PLANKS).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(1.0) .ignited_by_lava());
    pub const MANGROVE_WALL_HANGING_SIGN: Block = register_with(registry, BlockIds::MANGROVE_WALL_HANGING_SIGN, WallHangingSignBlock::new, wall_variant(registry, Blocks::MANGROVE_HANGING_SIGN, true) .map_color(registry.get(Blocks::MANGROVE_LOG).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(1.0) .ignited_by_lava());
    pub const POPLAR_WALL_HANGING_SIGN: Block = register_with(registry, BlockIds::POPLAR_WALL_HANGING_SIGN, WallHangingSignBlock::new, wall_variant(registry, Blocks::POPLAR_HANGING_SIGN, true) .map_color(registry.get(Blocks::POPLAR_PLANKS).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(1.0) .ignited_by_lava());
    pub const CRIMSON_WALL_HANGING_SIGN: Block = register_with(registry, BlockIds::CRIMSON_WALL_HANGING_SIGN, WallHangingSignBlock::new, wall_variant(registry, Blocks::CRIMSON_HANGING_SIGN, true) .map_color(MapColor::CRIMSON_STEM) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(1.0));
    pub const WARPED_WALL_HANGING_SIGN: Block = register_with(registry, BlockIds::WARPED_WALL_HANGING_SIGN, WallHangingSignBlock::new, wall_variant(registry, Blocks::WARPED_HANGING_SIGN, true).map_color(MapColor::WARPED_STEM).force_solid_on().instrument(NoteBlockInstrument::BASS).no_collision().strength(1.0));
    pub const BAMBOO_WALL_HANGING_SIGN: Block = register_with(registry, BlockIds::BAMBOO_WALL_HANGING_SIGN, WallHangingSignBlock::new, wall_variant(registry, Blocks::BAMBOO_HANGING_SIGN, true) .map_color(MapColor::COLOR_YELLOW) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(1.0) .ignited_by_lava());
    pub const LEVER: Block = register_with(registry, BlockItemIds::LEVER, LeverBlock::new, Properties::of().no_collision().strength(0.5).push_reaction(PushReaction::POPPED));
    pub const STONE_PRESSURE_PLATE: Block = register_with(registry, BlockItemIds::STONE_PRESSURE_PLATE, PressurePlateBlock::new, Properties::of() .map_color(MapColor::STONE) .force_solid_on() .instrument(NoteBlockInstrument::BASEDRUM) .no_collision() .strength(0.5) .push_reaction(PushReaction::POPPED));
    pub const IRON_DOOR: Block = register_with(registry, BlockItemIds::IRON_DOOR, DoorBlock::new, Properties::of().map_color(MapColor::METAL).strength(5.0).no_occlusion().push_reaction(PushReaction::POPPED));
    pub const OAK_PRESSURE_PLATE: Block = register_with(registry, BlockItemIds::OAK_PRESSURE_PLATE, PressurePlateBlock::new, Properties::of() .map_color(registry.get(Blocks::OAK_PLANKS).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(0.5) .ignited_by_lava() .push_reaction(PushReaction::POPPED));
    pub const SPRUCE_PRESSURE_PLATE: Block = register_with(registry, BlockItemIds::SPRUCE_PRESSURE_PLATE, PressurePlateBlock::new, Properties::of() .map_color(registry.get(Blocks::SPRUCE_PLANKS).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(0.5) .ignited_by_lava() .push_reaction(PushReaction::POPPED));
    pub const BIRCH_PRESSURE_PLATE: Block = register_with(registry, BlockItemIds::BIRCH_PRESSURE_PLATE, PressurePlateBlock::new, Properties::of() .map_color(registry.get(Blocks::BIRCH_PLANKS).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(0.5) .ignited_by_lava() .push_reaction(PushReaction::POPPED));
    pub const JUNGLE_PRESSURE_PLATE: Block = register_with(registry, BlockItemIds::JUNGLE_PRESSURE_PLATE, PressurePlateBlock::new, Properties::of() .map_color(registry.get(Blocks::JUNGLE_PLANKS).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(0.5) .ignited_by_lava() .push_reaction(PushReaction::POPPED));
    pub const ACACIA_PRESSURE_PLATE: Block = register_with(registry, BlockItemIds::ACACIA_PRESSURE_PLATE, PressurePlateBlock::new, Properties::of() .map_color(registry.get(Blocks::ACACIA_PLANKS).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(0.5) .ignited_by_lava() .push_reaction(PushReaction::POPPED));
    pub const CHERRY_PRESSURE_PLATE: Block = register_with(registry, BlockItemIds::CHERRY_PRESSURE_PLATE, PressurePlateBlock::new, Properties::of() .map_color(registry.get(Blocks::CHERRY_PLANKS).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(0.5) .ignited_by_lava() .push_reaction(PushReaction::POPPED));
    pub const DARK_OAK_PRESSURE_PLATE: Block = register_with(registry, BlockItemIds::DARK_OAK_PRESSURE_PLATE, PressurePlateBlock::new, Properties::of() .map_color(registry.get(Blocks::DARK_OAK_PLANKS).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(0.5) .ignited_by_lava() .push_reaction(PushReaction::POPPED));
    pub const PALE_OAK_PRESSURE_PLATE: Block = register_with(registry, BlockItemIds::PALE_OAK_PRESSURE_PLATE, PressurePlateBlock::new, Properties::of() .map_color(registry.get(Blocks::PALE_OAK_PLANKS).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(0.5) .ignited_by_lava() .push_reaction(PushReaction::POPPED));
    pub const MANGROVE_PRESSURE_PLATE: Block = register_with(registry, BlockItemIds::MANGROVE_PRESSURE_PLATE, PressurePlateBlock::new, Properties::of() .map_color(registry.get(Blocks::MANGROVE_PLANKS).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(0.5) .ignited_by_lava() .push_reaction(PushReaction::POPPED));
    pub const POPLAR_PRESSURE_PLATE: Block = register_with(registry, BlockItemIds::POPLAR_PRESSURE_PLATE, PressurePlateBlock::new, Properties::of() .map_color(registry.get(Blocks::POPLAR_PLANKS).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(0.5) .ignited_by_lava() .push_reaction(PushReaction::POPPED));
    pub const BAMBOO_PRESSURE_PLATE: Block = register_with(registry, BlockItemIds::BAMBOO_PRESSURE_PLATE, PressurePlateBlock::new, Properties::of() .map_color(registry.get(Blocks::BAMBOO_PLANKS).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(0.5) .ignited_by_lava() .push_reaction(PushReaction::POPPED));
    pub const REDSTONE_ORE: Block = register_with(registry, BlockItemIds::REDSTONE_ORE, RedStoneOreBlock::new, Properties::of() .map_color(MapColor::STONE) .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .random_ticks() .light_level(lit_block_emission(9)) .strength_and_resistance(3.0, 3.0));
    pub const DEEPSLATE_REDSTONE_ORE: Block = register_with(registry, BlockItemIds::DEEPSLATE_REDSTONE_ORE, RedStoneOreBlock::new, Properties::of_legacy_copy(&registry.get(Blocks::REDSTONE_ORE)).map_color(MapColor::DEEPSLATE).strength_and_resistance(4.5, 3.0));
    pub const REDSTONE_TORCH: Block = register_with(registry, BlockItemIds::REDSTONE_TORCH, RedstoneTorchBlock::new, Properties::of().no_collision().instabreak().light_level(lit_block_emission(7)).push_reaction(PushReaction::POPPED));
    pub const REDSTONE_WALL_TORCH: Block = register_with(registry, BlockIds::REDSTONE_WALL_TORCH, RedstoneWallTorchBlock::new, wall_variant(registry, Blocks::REDSTONE_TORCH, true).no_collision().instabreak().light_level(lit_block_emission(7)).push_reaction(PushReaction::POPPED));
    pub const STONE_BUTTON: Block = register_with(registry, BlockItemIds::STONE_BUTTON, ButtonBlock::new, button_properties());
    pub const SNOW: Block = register_with(registry, BlockItemIds::SNOW, SnowLayerBlock::new, Properties::of().map_color(MapColor::SNOW).replaceable().force_solid_off().random_ticks().strength(0.1).requires_correct_tool_for_drops().is_view_blocking(ViewBlocking::State(|state| state.get_value(SnowLayerBlock::LAYERS) >= 8)).push_reaction(PushReaction::POPPED));
    pub const ICE: Block = register_with(registry, BlockItemIds::ICE, IceBlock::new, Properties::of() .map_color(MapColor::ICE) .friction(0.98) .random_ticks() .strength(0.5)  .no_occlusion()  .is_redstone_conductor(StatePredicate::State(None, never)));
    pub const SNOW_BLOCK: Block = register(registry, BlockItemIds::SNOW_BLOCK, Properties::of().map_color(MapColor::SNOW).requires_correct_tool_for_drops().strength(0.2));
    pub const CACTUS: Block = register_with(registry, BlockItemIds::CACTUS, CactusBlock::new, Properties::of().map_color(MapColor::PLANT).random_ticks().strength(0.4).push_reaction(PushReaction::POPPED));
    pub const CACTUS_FLOWER: Block = register_with(registry, BlockItemIds::CACTUS_FLOWER, CactusFlowerBlock::new, Properties::of() .map_color(MapColor::COLOR_PINK) .no_collision() .instabreak() .ignited_by_lava()  .push_reaction(PushReaction::POPPED));
    pub const CLAY: Block = register(registry, BlockItemIds::CLAY, Properties::of().map_color(MapColor::CLAY).instrument(NoteBlockInstrument::FLUTE).strength(0.6));
    pub const SUGAR_CANE: Block = register_with(registry, BlockItemIds::SUGAR_CANE, SugarCaneBlock::new, Properties::of() .map_color(MapColor::PLANT) .no_collision() .random_ticks() .instabreak()  .push_reaction(PushReaction::POPPED));
    pub const JUKEBOX: Block = register_with(registry, BlockItemIds::JUKEBOX, JukeboxBlock::new, Properties::of().map_color(MapColor::DIRT).instrument(NoteBlockInstrument::BASS).strength_and_resistance(2.0, 6.0).ignited_by_lava());
    pub const OAK_FENCE: Block = register_with(registry, BlockItemIds::OAK_FENCE, FenceBlock::new, Properties::of() .map_color(registry.get(Blocks::OAK_PLANKS).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .strength_and_resistance(2.0, 3.0)  .ignited_by_lava());
    pub const NETHERRACK: Block = register_with(registry, BlockItemIds::NETHERRACK, NetherrackBlock::new, Properties::of() .map_color(MapColor::NETHER) .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .strength(0.4) );
    pub const SOUL_SAND: Block = register_with(registry, BlockItemIds::SOUL_SAND, SoulSandBlock::new, Properties::of() .map_color(MapColor::COLOR_BROWN) .instrument(NoteBlockInstrument::COW_BELL) .strength(0.5) .speed_factor(0.4)   .is_redstone_conductor(StatePredicate::State(None, always)) .is_suffocating(StatePredicate::State(None, always)) .post_process(post_process_above));
    pub const SOUL_SOIL: Block = register(registry, BlockItemIds::SOUL_SOIL, Properties::of().map_color(MapColor::COLOR_BROWN).strength(0.5));
    pub const BASALT: Block = register_with(registry, BlockItemIds::BASALT, RotatedPillarBlock::new, Properties::of() .map_color(MapColor::COLOR_BLACK) .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .strength_and_resistance(1.25, 4.2) );
    pub const POLISHED_BASALT: Block = register_with(registry, BlockItemIds::POLISHED_BASALT, RotatedPillarBlock::new, Properties::of() .map_color(MapColor::COLOR_BLACK) .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .strength_and_resistance(1.25, 4.2) );
    pub const SOUL_TORCH: Block = register_with(registry, BlockItemIds::SOUL_TORCH, TorchBlock::new, Properties::of().no_collision().instabreak().light_level(|_| 10).push_reaction(PushReaction::POPPED));
    pub const SOUL_WALL_TORCH: Block = register_with(registry, BlockIds::SOUL_WALL_TORCH, WallTorchBlock::new, wall_variant(registry, Blocks::SOUL_TORCH, true).no_collision().instabreak().light_level(|_| 10).push_reaction(PushReaction::POPPED));
    pub const COPPER_TORCH: Block = register_with(registry, BlockItemIds::COPPER_TORCH, TorchBlock::new, Properties::of().no_collision().instabreak().light_level(|_| 14).push_reaction(PushReaction::POPPED));
    pub const COPPER_WALL_TORCH: Block = register_with(registry, BlockIds::COPPER_WALL_TORCH, WallTorchBlock::new, wall_variant(registry, Blocks::COPPER_TORCH, true).no_collision().instabreak().light_level(|_| 14).push_reaction(PushReaction::POPPED));
    pub const GLOWSTONE: Block = register(registry, BlockItemIds::GLOWSTONE, Properties::of() .map_color(MapColor::SAND) .instrument(NoteBlockInstrument::PLING) .strength(0.3)  .light_level(|_| 15) .is_redstone_conductor(StatePredicate::State(None, never)));
    pub const NETHER_PORTAL: Block = register_with(registry, BlockIds::NETHER_PORTAL, NetherPortalBlock::new, Properties::of() .no_collision() .random_ticks() .strength(-1.0)  .light_level(|_| 11) .push_reaction(PushReaction::IMMOVEABLE));
    pub const CARVED_PUMPKIN: Block = register_with(registry, BlockItemIds::CARVED_PUMPKIN, CarvedPumpkinBlock::new, Properties::of() .map_color(MapColor::COLOR_ORANGE) .strength(1.0)   .push_reaction(PushReaction::POPPED));
    pub const JACK_O_LANTERN: Block = register_with(registry, BlockItemIds::JACK_O_LANTERN, CarvedPumpkinBlock::new, Properties::of() .map_color(MapColor::COLOR_ORANGE) .strength(1.0)  .light_level(|_| 15)  .push_reaction(PushReaction::POPPED));
    pub const CAKE: Block = register_with(registry, BlockItemIds::CAKE, CakeBlock::new, Properties::of().force_solid_on().strength(0.5).push_reaction(PushReaction::POPPED));
    pub const REPEATER: Block = register_with(registry, BlockItemIds::REPEATER, RepeaterBlock::new, Properties::of().instabreak().push_reaction(PushReaction::POPPED));
    pub const STAINED_GLASS: ColorCollection<Block> = ColorCollection::register_blocks(registry, BlockItemIds::STAINED_GLASS, |_, p| StainedGlassBlock::new(p), |color| Properties::of() .map_color(color.get_map_color()) .instrument(NoteBlockInstrument::HAT) .strength(0.3)  .no_occlusion()  .is_redstone_conductor(StatePredicate::State(None, never)) .is_suffocating(StatePredicate::State(None, never)));
    pub const OAK_TRAPDOOR: Block = register_with(registry, BlockItemIds::OAK_TRAPDOOR, TrapDoorBlock::new, Properties::of() .map_color(MapColor::WOOD) .instrument(NoteBlockInstrument::BASS) .strength(3.0) .no_occlusion()  .ignited_by_lava());
    pub const SPRUCE_TRAPDOOR: Block = register_with(registry, BlockItemIds::SPRUCE_TRAPDOOR, TrapDoorBlock::new, Properties::of() .map_color(MapColor::PODZOL) .instrument(NoteBlockInstrument::BASS) .strength(3.0) .no_occlusion()  .ignited_by_lava());
    pub const BIRCH_TRAPDOOR: Block = register_with(registry, BlockItemIds::BIRCH_TRAPDOOR, TrapDoorBlock::new, Properties::of() .map_color(MapColor::SAND) .instrument(NoteBlockInstrument::BASS) .strength(3.0) .no_occlusion()  .ignited_by_lava());
    pub const JUNGLE_TRAPDOOR: Block = register_with(registry, BlockItemIds::JUNGLE_TRAPDOOR, TrapDoorBlock::new, Properties::of() .map_color(MapColor::DIRT) .instrument(NoteBlockInstrument::BASS) .strength(3.0) .no_occlusion()  .ignited_by_lava());
    pub const ACACIA_TRAPDOOR: Block = register_with(registry, BlockItemIds::ACACIA_TRAPDOOR, TrapDoorBlock::new, Properties::of() .map_color(MapColor::COLOR_ORANGE) .instrument(NoteBlockInstrument::BASS) .strength(3.0) .no_occlusion()  .ignited_by_lava());
    pub const CHERRY_TRAPDOOR: Block = register_with(registry, BlockItemIds::CHERRY_TRAPDOOR, TrapDoorBlock::new, Properties::of() .map_color(MapColor::TERRACOTTA_WHITE) .instrument(NoteBlockInstrument::BASS) .strength(3.0) .no_occlusion()  .ignited_by_lava());
    pub const DARK_OAK_TRAPDOOR: Block = register_with(registry, BlockItemIds::DARK_OAK_TRAPDOOR, TrapDoorBlock::new, Properties::of() .map_color(MapColor::COLOR_BROWN) .instrument(NoteBlockInstrument::BASS) .strength(3.0) .no_occlusion()  .ignited_by_lava());
    pub const PALE_OAK_TRAPDOOR: Block = register_with(registry, BlockItemIds::PALE_OAK_TRAPDOOR, TrapDoorBlock::new, Properties::of() .map_color(registry.get(Blocks::PALE_OAK_PLANKS).default_map_color()) .instrument(NoteBlockInstrument::BASS) .strength(3.0) .no_occlusion()  .ignited_by_lava());
    pub const MANGROVE_TRAPDOOR: Block = register_with(registry, BlockItemIds::MANGROVE_TRAPDOOR, TrapDoorBlock::new, Properties::of() .map_color(MapColor::COLOR_RED) .instrument(NoteBlockInstrument::BASS) .strength(3.0) .no_occlusion()  .ignited_by_lava());
    pub const POPLAR_TRAPDOOR: Block = register_with(registry, BlockItemIds::POPLAR_TRAPDOOR, TrapDoorBlock::new, Properties::of() .map_color(registry.get(Blocks::POPLAR_PLANKS).default_map_color()) .instrument(NoteBlockInstrument::BASS) .strength(3.0) .no_occlusion()  .ignited_by_lava());
    pub const BAMBOO_TRAPDOOR: Block = register_with(registry, BlockItemIds::BAMBOO_TRAPDOOR, TrapDoorBlock::new, Properties::of() .map_color(MapColor::COLOR_YELLOW) .instrument(NoteBlockInstrument::BASS) .strength(3.0) .no_occlusion()  .ignited_by_lava());
    pub const STONE_BRICKS: Block = register(registry, BlockItemIds::STONE_BRICKS, Properties::of().map_color(MapColor::STONE).instrument(NoteBlockInstrument::BASEDRUM).requires_correct_tool_for_drops().strength_and_resistance(1.5, 6.0));
    pub const MOSSY_STONE_BRICKS: Block = register(registry, BlockItemIds::MOSSY_STONE_BRICKS, Properties::of().map_color(MapColor::STONE).instrument(NoteBlockInstrument::BASEDRUM).requires_correct_tool_for_drops().strength_and_resistance(1.5, 6.0));
    pub const CRACKED_STONE_BRICKS: Block = register(registry, BlockItemIds::CRACKED_STONE_BRICKS, Properties::of().map_color(MapColor::STONE).instrument(NoteBlockInstrument::BASEDRUM).requires_correct_tool_for_drops().strength_and_resistance(1.5, 6.0));
    pub const CHISELED_STONE_BRICKS: Block = register(registry, BlockItemIds::CHISELED_STONE_BRICKS, Properties::of().map_color(MapColor::STONE).instrument(NoteBlockInstrument::BASEDRUM).requires_correct_tool_for_drops().strength_and_resistance(1.5, 6.0));
    pub const PACKED_MUD: Block = register(registry, BlockItemIds::PACKED_MUD, Properties::of_legacy_copy(&registry.get(Blocks::DIRT)).strength_and_resistance(1.0, 3.0));
    pub const MUD_BRICKS: Block = register(registry, BlockItemIds::MUD_BRICKS, Properties::of() .map_color(MapColor::TERRACOTTA_LIGHT_GRAY) .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .strength_and_resistance(1.5, 3.0) );
    pub const INFESTED_STONE: Block = register_with(registry, BlockItemIds::INFESTED_STONE, |p| InfestedBlock::new(&registry.get(Blocks::STONE), p), Properties::of().map_color(MapColor::CLAY));
    pub const INFESTED_COBBLESTONE: Block = register_with(registry, BlockItemIds::INFESTED_COBBLESTONE, |p| InfestedBlock::new(&registry.get(Blocks::COBBLESTONE), p), Properties::of().map_color(MapColor::CLAY));
    pub const INFESTED_STONE_BRICKS: Block = register_with(registry, BlockItemIds::INFESTED_STONE_BRICKS, |p| InfestedBlock::new(&registry.get(Blocks::STONE_BRICKS), p), Properties::of().map_color(MapColor::CLAY));
    pub const INFESTED_MOSSY_STONE_BRICKS: Block = register_with(registry, BlockItemIds::INFESTED_MOSSY_STONE_BRICKS, |p| InfestedBlock::new(&registry.get(Blocks::MOSSY_STONE_BRICKS), p), Properties::of().map_color(MapColor::CLAY));
    pub const INFESTED_CRACKED_STONE_BRICKS: Block = register_with(registry, BlockItemIds::INFESTED_CRACKED_STONE_BRICKS, |p| InfestedBlock::new(&registry.get(Blocks::CRACKED_STONE_BRICKS), p), Properties::of().map_color(MapColor::CLAY));
    pub const INFESTED_CHISELED_STONE_BRICKS: Block = register_with(registry, BlockItemIds::INFESTED_CHISELED_STONE_BRICKS, |p| InfestedBlock::new(&registry.get(Blocks::CHISELED_STONE_BRICKS), p), Properties::of().map_color(MapColor::CLAY));
    pub const BROWN_MUSHROOM_BLOCK: Block = register_with(registry, BlockItemIds::BROWN_MUSHROOM_BLOCK, HugeMushroomBlock::new, Properties::of().map_color(MapColor::DIRT).instrument(NoteBlockInstrument::BASS).strength(0.2).ignited_by_lava());
    pub const RED_MUSHROOM_BLOCK: Block = register_with(registry, BlockItemIds::RED_MUSHROOM_BLOCK, HugeMushroomBlock::new, Properties::of().map_color(MapColor::COLOR_RED).instrument(NoteBlockInstrument::BASS).strength(0.2).ignited_by_lava());
    pub const MUSHROOM_STEM: Block = register_with(registry, BlockItemIds::MUSHROOM_STEM, HugeMushroomBlock::new, Properties::of().map_color(MapColor::WOOL).instrument(NoteBlockInstrument::BASS).strength(0.2).ignited_by_lava());
    pub const IRON_BARS: Block = register_with(registry, BlockItemIds::IRON_BARS, IronBarsBlock::new, Properties::of().requires_correct_tool_for_drops().strength_and_resistance(5.0, 6.0).no_occlusion());
    pub const COPPER_BARS: WeatheringCopperCollection<Block> = WeatheringCopperCollection::register_blocks(registry, BlockItemIds::COPPER_BARS, |_, p| IronBarsBlock::new(p), |_, p| WeatheringCopperBarsBlock::new(p), |_| Properties::of().requires_correct_tool_for_drops().strength_and_resistance(5.0, 6.0).no_occlusion());
    pub const IRON_CHAIN: Block = register_with(registry, BlockItemIds::IRON_CHAIN, ChainBlock::new, Properties::of().force_solid_on().requires_correct_tool_for_drops().strength_and_resistance(5.0, 6.0).no_occlusion());
    pub const COPPER_CHAIN: WeatheringCopperCollection<Block> = WeatheringCopperCollection::register_blocks(registry, BlockItemIds::COPPER_CHAIN, |_, p| ChainBlock::new(p), |_, p| WeatheringCopperChainBlock::new(p), |_| Properties::of().force_solid_on().requires_correct_tool_for_drops().strength_and_resistance(5.0, 6.0).no_occlusion());
    pub const GLASS_PANE: Block = register_with(registry, BlockItemIds::GLASS_PANE, IronBarsBlock::new, Properties::of().instrument(NoteBlockInstrument::HAT).strength(0.3).no_occlusion());
    pub const PUMPKIN: Block = register_with(registry, BlockItemIds::PUMPKIN, PumpkinBlock::new, Properties::of() .map_color(MapColor::COLOR_ORANGE) .instrument(NoteBlockInstrument::DIDGERIDOO) .strength(1.0)  .push_reaction(PushReaction::POPPED));
    pub const MELON: Block = register(registry, BlockItemIds::MELON, Properties::of().map_color(MapColor::COLOR_LIGHT_GREEN).strength(1.0).push_reaction(PushReaction::POPPED));
    pub const ATTACHED_PUMPKIN_STEM: Block = register_with(registry, BlockIds::ATTACHED_PUMPKIN_STEM, AttachedStemBlock::new, Properties::of().map_color(MapColor::PLANT).no_collision().instabreak().push_reaction(PushReaction::POPPED));
    pub const ATTACHED_MELON_STEM: Block = register_with(registry, BlockIds::ATTACHED_MELON_STEM, AttachedStemBlock::new, Properties::of().map_color(MapColor::PLANT).no_collision().instabreak().push_reaction(PushReaction::POPPED));
    pub const PUMPKIN_STEM: Block = register_with(registry, BlockItemIds::PUMPKIN_CROP, StemBlock::new, Properties::of() .map_color(MapColor::PLANT) .no_collision() .random_ticks() .instabreak()  .push_reaction(PushReaction::POPPED));
    pub const MELON_STEM: Block = register_with(registry, BlockItemIds::MELON_CROP, StemBlock::new, Properties::of() .map_color(MapColor::PLANT) .no_collision() .random_ticks() .instabreak()  .push_reaction(PushReaction::POPPED));
    pub const VINE: Block = register_with(registry, BlockItemIds::VINE, VineBlock::new, Properties::of() .map_color(MapColor::PLANT) .replaceable() .no_collision() .random_ticks() .strength(0.2)  .ignited_by_lava() .push_reaction(PushReaction::POPPED));
    pub const GLOW_LICHEN: Block = register_with(registry, BlockItemIds::GLOW_LICHEN, GlowLichenBlock::new, Properties::of() .map_color(MapColor::GLOW_LICHEN) .replaceable() .no_collision() .strength(0.2)  .light_level(GlowLichenBlock::emission(7)) .ignited_by_lava() .push_reaction(PushReaction::POPPED));
    pub const RESIN_CLUMP: Block = register_with(registry, BlockItemIds::RESIN_CLUMP, MultifaceBlock::new, Properties::of() .map_color(MapColor::TERRACOTTA_ORANGE) .replaceable() .no_collision()  .ignited_by_lava() .push_reaction(PushReaction::POPPED));
    pub const OAK_FENCE_GATE: Block = register_with(registry, BlockItemIds::OAK_FENCE_GATE, FenceGateBlock::new, Properties::of() .map_color(registry.get(Blocks::OAK_PLANKS).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .strength_and_resistance(2.0, 3.0) .ignited_by_lava());
    pub const BRICK_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::BRICK_STAIRS, Blocks::BRICKS);
    pub const STONE_BRICK_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::STONE_BRICK_STAIRS, Blocks::STONE_BRICKS);
    pub const MUD_BRICK_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::MUD_BRICK_STAIRS, Blocks::MUD_BRICKS);
    pub const MYCELIUM: Block = register_with(registry, BlockItemIds::MYCELIUM, MyceliumBlock::new, Properties::of().map_color(MapColor::COLOR_PURPLE).random_ticks().strength(0.6));
    pub const LILY_PAD: Block = register_with(registry, BlockItemIds::LILY_PAD, LilyPadBlock::new, Properties::of().map_color(MapColor::PLANT).instabreak().no_occlusion().push_reaction(PushReaction::POPPED));
    pub const RESIN_BLOCK: Block = register(registry, BlockItemIds::RESIN_BLOCK, Properties::of().map_color(MapColor::TERRACOTTA_ORANGE).instrument(NoteBlockInstrument::BASEDRUM));
    pub const RESIN_BRICKS: Block = register(registry, BlockItemIds::RESIN_BRICKS, Properties::of() .map_color(MapColor::TERRACOTTA_ORANGE) .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops()  .strength_and_resistance(1.5, 6.0));
    pub const RESIN_BRICK_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::RESIN_BRICK_STAIRS, Blocks::RESIN_BRICKS);
    pub const RESIN_BRICK_SLAB: Block = register_slab(registry, BlockItemIds::RESIN_BRICK_SLAB, Blocks::RESIN_BRICKS);
    pub const RESIN_BRICK_WALL: Block = register_wall(registry, BlockItemIds::RESIN_BRICK_WALL, Blocks::RESIN_BRICKS);
    pub const CHISELED_RESIN_BRICKS: Block = register(registry, BlockItemIds::CHISELED_RESIN_BRICKS, Properties::of() .map_color(MapColor::TERRACOTTA_ORANGE) .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops()  .strength_and_resistance(1.5, 6.0));
    pub const NETHER_BRICKS: Block = register(registry, BlockItemIds::NETHER_BRICKS, Properties::of() .map_color(MapColor::NETHER) .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .strength_and_resistance(2.0, 6.0) );
    pub const NETHER_BRICK_FENCE: Block = register_with(registry, BlockItemIds::NETHER_BRICK_FENCE, FenceBlock::new, Properties::of() .map_color(MapColor::NETHER) .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .strength_and_resistance(2.0, 6.0) );
    pub const NETHER_BRICK_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::NETHER_BRICK_STAIRS, Blocks::NETHER_BRICKS);
    pub const NETHER_WART: Block = register_with(registry, BlockItemIds::NETHER_WART, NetherWartBlock::new, Properties::of().map_color(MapColor::COLOR_RED).no_collision().random_ticks().push_reaction(PushReaction::POPPED));
    pub const ENCHANTING_TABLE: Block = register_with(registry, BlockItemIds::ENCHANTING_TABLE, EnchantingTableBlock::new, Properties::of() .map_color(MapColor::COLOR_RED) .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .light_level(|_| 7) .strength_and_resistance(5.0, 1200.0));
    pub const BREWING_STAND: Block = register_with(registry, BlockItemIds::BREWING_STAND, BrewingStandBlock::new, Properties::of().map_color(MapColor::METAL).strength(0.5).light_level(|_| 1).no_occlusion());
    pub const CAULDRON: Block = register_with(registry, BlockItemIds::CAULDRON, CauldronBlock::new, Properties::of().map_color(MapColor::STONE).requires_correct_tool_for_drops().strength(2.0).no_occlusion());
    pub const WATER_CAULDRON: Block = register_with(registry, BlockIds::WATER_CAULDRON, LayeredCauldronBlock::new, Properties::of_legacy_copy(&registry.get(Blocks::CAULDRON)));
    pub const LAVA_CAULDRON: Block = register_with(registry, BlockIds::LAVA_CAULDRON, LavaCauldronBlock::new, Properties::of_legacy_copy(&registry.get(Blocks::CAULDRON)).light_level(|_| 15));
    pub const POWDER_SNOW_CAULDRON: Block = register_with(registry, BlockIds::POWDER_SNOW_CAULDRON, LayeredCauldronBlock::new, Properties::of_legacy_copy(&registry.get(Blocks::CAULDRON)));
    pub const END_PORTAL: Block = register_with(registry, BlockIds::END_PORTAL, EndPortalBlock::new, Properties::of() .map_color(MapColor::COLOR_BLACK) .no_collision() .light_level(|_| 15) .strength_and_resistance(-1.0, 3600000.0) .no_loot_table() .push_reaction(PushReaction::IMMOVEABLE));
    pub const END_PORTAL_FRAME: Block = register_with(registry, BlockItemIds::END_PORTAL_FRAME, EndPortalFrameBlock::new, Properties::of() .map_color(MapColor::COLOR_GREEN) .instrument(NoteBlockInstrument::BASEDRUM)  .light_level(|_| 1) .strength_and_resistance(-1.0, 3600000.0) .no_loot_table());
    pub const END_STONE: Block = register(registry, BlockItemIds::END_STONE, Properties::of().map_color(MapColor::SAND).instrument(NoteBlockInstrument::BASEDRUM).requires_correct_tool_for_drops().strength_and_resistance(3.0, 9.0));
    pub const DRAGON_EGG: Block = register_with(registry, BlockItemIds::DRAGON_EGG, DragonEggBlock::new, Properties::of() .map_color(MapColor::COLOR_BLACK) .strength_and_resistance(3.0, 9.0) .light_level(|_| 1) .no_occlusion() .push_reaction(PushReaction::POPPED));
    pub const REDSTONE_LAMP: Block = register_with(registry, BlockItemIds::REDSTONE_LAMP, RedstoneLampBlock::new, Properties::of() .map_color(MapColor::TERRACOTTA_ORANGE) .light_level(lit_block_emission(15)) .strength(0.3)  );
    pub const COCOA: Block = register_with(registry, BlockItemIds::COCOA_CROP, CocoaBlock::new, Properties::of() .map_color(MapColor::PLANT) .random_ticks() .strength_and_resistance(0.2, 3.0)  .no_occlusion() .push_reaction(PushReaction::POPPED));
    pub const SHELF_MUSHROOM: Block = register_with(registry, BlockItemIds::SHELF_MUSHROOM, ShelfMushroomBlock::new, Properties::of() .map_color(MapColor::TERRACOTTA_YELLOW) .bounce_restitution(0.75) .fall_distance_reduction(0.5)  .no_occlusion() .push_reaction(PushReaction::POPPED));
    pub const SANDSTONE_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::SANDSTONE_STAIRS, Blocks::SANDSTONE);
    pub const EMERALD_ORE: Block = register_with(registry, BlockItemIds::EMERALD_ORE, DropExperienceBlock::new, Properties::of().map_color(MapColor::STONE).instrument(NoteBlockInstrument::BASEDRUM).requires_correct_tool_for_drops().strength_and_resistance(3.0, 3.0));
    pub const DEEPSLATE_EMERALD_ORE: Block = register_with(registry, BlockItemIds::DEEPSLATE_EMERALD_ORE, DropExperienceBlock::new, Properties::of_legacy_copy(&registry.get(Blocks::EMERALD_ORE)).map_color(MapColor::DEEPSLATE).strength_and_resistance(4.5, 3.0));
    pub const ENDER_CHEST: Block = register_with(registry, BlockItemIds::ENDER_CHEST, EnderChestBlock::new, Properties::of().map_color(MapColor::STONE).instrument(NoteBlockInstrument::BASEDRUM).strength_and_resistance(22.5, 600.0).light_level(|_| 7));
    pub const TRIPWIRE_HOOK: Block = register_with(registry, BlockItemIds::TRIPWIRE_HOOK, TripWireHookBlock::new, Properties::of().no_collision().push_reaction(PushReaction::POPPED));
    pub const TRIPWIRE: Block = register_with(registry, BlockItemIds::TRIPWIRE, TripWireBlock::new, Properties::of().no_collision().push_reaction(PushReaction::POPPED));
    pub const EMERALD_BLOCK: Block = register(registry, BlockItemIds::EMERALD_BLOCK, Properties::of() .map_color(MapColor::EMERALD) .instrument(NoteBlockInstrument::BIT) .requires_correct_tool_for_drops() .strength_and_resistance(5.0, 6.0) );
    pub const SPRUCE_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::SPRUCE_STAIRS, Blocks::SPRUCE_PLANKS);
    pub const BIRCH_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::BIRCH_STAIRS, Blocks::BIRCH_PLANKS);
    pub const JUNGLE_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::JUNGLE_STAIRS, Blocks::JUNGLE_PLANKS);
    pub const COMMAND_BLOCK: Block = register_with(registry, BlockItemIds::COMMAND_BLOCK, CommandBlock::new, Properties::of().map_color(MapColor::COLOR_BROWN).requires_correct_tool_for_drops().strength_and_resistance(-1.0, 3600000.0).no_loot_table());
    pub const BEACON: Block = register_with(registry, BlockItemIds::BEACON, BeaconBlock::new, Properties::of() .map_color(MapColor::DIAMOND) .instrument(NoteBlockInstrument::HAT) .strength(3.0) .light_level(|_| 15) .no_occlusion() .is_redstone_conductor(StatePredicate::State(None, never)));
    pub const COBBLESTONE_WALL: Block = register_wall(registry, BlockItemIds::COBBLESTONE_WALL, Blocks::COBBLESTONE);
    pub const MOSSY_COBBLESTONE_WALL: Block = register_wall(registry, BlockItemIds::MOSSY_COBBLESTONE_WALL, Blocks::COBBLESTONE);
    pub const FLOWER_POT: Block = register_with(registry, BlockItemIds::FLOWER_POT, FlowerPotBlock::new, flower_pot_properties());
    pub const POTTED_TORCHFLOWER: Block = register_with(registry, BlockIds::POTTED_TORCHFLOWER, FlowerPotBlock::new, flower_pot_properties());
    pub const POTTED_OAK_SAPLING: Block = register_with(registry, BlockIds::POTTED_OAK_SAPLING, FlowerPotBlock::new, flower_pot_properties());
    pub const POTTED_SPRUCE_SAPLING: Block = register_with(registry, BlockIds::POTTED_SPRUCE_SAPLING, FlowerPotBlock::new, flower_pot_properties());
    pub const POTTED_BIRCH_SAPLING: Block = register_with(registry, BlockIds::POTTED_BIRCH_SAPLING, FlowerPotBlock::new, flower_pot_properties());
    pub const POTTED_JUNGLE_SAPLING: Block = register_with(registry, BlockIds::POTTED_JUNGLE_SAPLING, FlowerPotBlock::new, flower_pot_properties());
    pub const POTTED_ACACIA_SAPLING: Block = register_with(registry, BlockIds::POTTED_ACACIA_SAPLING, FlowerPotBlock::new, flower_pot_properties());
    pub const POTTED_CHERRY_SAPLING: Block = register_with(registry, BlockIds::POTTED_CHERRY_SAPLING, FlowerPotBlock::new, flower_pot_properties());
    pub const POTTED_DARK_OAK_SAPLING: Block = register_with(registry, BlockIds::POTTED_DARK_OAK_SAPLING, FlowerPotBlock::new, flower_pot_properties());
    pub const POTTED_PALE_OAK_SAPLING: Block = register_with(registry, BlockIds::POTTED_PALE_OAK_SAPLING, FlowerPotBlock::new, flower_pot_properties());
    pub const POTTED_POPLAR_SAPLING: Block = register_with(registry, BlockIds::POTTED_POPLAR_SAPLING, FlowerPotBlock::new, flower_pot_properties());
    pub const POTTED_MANGROVE_PROPAGULE: Block = register_with(registry, BlockIds::POTTED_MANGROVE_PROPAGULE, FlowerPotBlock::new, flower_pot_properties());
    pub const POTTED_FERN: Block = register_with(registry, BlockIds::POTTED_FERN, FlowerPotBlock::new, flower_pot_properties());
    pub const POTTED_DANDELION: Block = register_with(registry, BlockIds::POTTED_DANDELION, FlowerPotBlock::new, flower_pot_properties());
    pub const POTTED_GOLDEN_DANDELION: Block = register_with(registry, BlockIds::POTTED_GOLDEN_DANDELION, FlowerPotBlock::new, flower_pot_properties());
    pub const POTTED_POPPY: Block = register_with(registry, BlockIds::POTTED_POPPY, FlowerPotBlock::new, flower_pot_properties());
    pub const POTTED_BLUE_ORCHID: Block = register_with(registry, BlockIds::POTTED_BLUE_ORCHID, FlowerPotBlock::new, flower_pot_properties());
    pub const POTTED_ALLIUM: Block = register_with(registry, BlockIds::POTTED_ALLIUM, FlowerPotBlock::new, flower_pot_properties());
    pub const POTTED_AZURE_BLUET: Block = register_with(registry, BlockIds::POTTED_AZURE_BLUET, FlowerPotBlock::new, flower_pot_properties());
    pub const POTTED_RED_TULIP: Block = register_with(registry, BlockIds::POTTED_RED_TULIP, FlowerPotBlock::new, flower_pot_properties());
    pub const POTTED_ORANGE_TULIP: Block = register_with(registry, BlockIds::POTTED_ORANGE_TULIP, FlowerPotBlock::new, flower_pot_properties());
    pub const POTTED_WHITE_TULIP: Block = register_with(registry, BlockIds::POTTED_WHITE_TULIP, FlowerPotBlock::new, flower_pot_properties());
    pub const POTTED_PINK_TULIP: Block = register_with(registry, BlockIds::POTTED_PINK_TULIP, FlowerPotBlock::new, flower_pot_properties());
    pub const POTTED_OXEYE_DAISY: Block = register_with(registry, BlockIds::POTTED_OXEYE_DAISY, FlowerPotBlock::new, flower_pot_properties());
    pub const POTTED_CORNFLOWER: Block = register_with(registry, BlockIds::POTTED_CORNFLOWER, FlowerPotBlock::new, flower_pot_properties());
    pub const POTTED_LILY_OF_THE_VALLEY: Block = register_with(registry, BlockIds::POTTED_LILY_OF_THE_VALLEY, FlowerPotBlock::new, flower_pot_properties());
    pub const POTTED_WITHER_ROSE: Block = register_with(registry, BlockIds::POTTED_WITHER_ROSE, FlowerPotBlock::new, flower_pot_properties());
    pub const POTTED_RED_MUSHROOM: Block = register_with(registry, BlockIds::POTTED_RED_MUSHROOM, FlowerPotBlock::new, flower_pot_properties());
    pub const POTTED_BROWN_MUSHROOM: Block = register_with(registry, BlockIds::POTTED_BROWN_MUSHROOM, FlowerPotBlock::new, flower_pot_properties());
    pub const POTTED_DEAD_BUSH: Block = register_with(registry, BlockIds::POTTED_DEAD_BUSH, FlowerPotBlock::new, flower_pot_properties());
    pub const POTTED_CACTUS: Block = register_with(registry, BlockIds::POTTED_CACTUS, FlowerPotBlock::new, flower_pot_properties());
    pub const CARROTS: Block = register_with(registry, BlockItemIds::CARROT_CROP, CarrotBlock::new, Properties::of() .map_color(MapColor::PLANT) .no_collision() .random_ticks() .instabreak()  .push_reaction(PushReaction::POPPED));
    pub const POTATOES: Block = register_with(registry, BlockItemIds::POTATO_CROP, PotatoBlock::new, Properties::of() .map_color(MapColor::PLANT) .no_collision() .random_ticks() .instabreak()  .push_reaction(PushReaction::POPPED));
    pub const OAK_BUTTON: Block = register_with(registry, BlockItemIds::OAK_BUTTON, ButtonBlock::new, button_properties());
    pub const SPRUCE_BUTTON: Block = register_with(registry, BlockItemIds::SPRUCE_BUTTON, ButtonBlock::new, button_properties());
    pub const BIRCH_BUTTON: Block = register_with(registry, BlockItemIds::BIRCH_BUTTON, ButtonBlock::new, button_properties());
    pub const JUNGLE_BUTTON: Block = register_with(registry, BlockItemIds::JUNGLE_BUTTON, ButtonBlock::new, button_properties());
    pub const ACACIA_BUTTON: Block = register_with(registry, BlockItemIds::ACACIA_BUTTON, ButtonBlock::new, button_properties());
    pub const CHERRY_BUTTON: Block = register_with(registry, BlockItemIds::CHERRY_BUTTON, ButtonBlock::new, button_properties());
    pub const DARK_OAK_BUTTON: Block = register_with(registry, BlockItemIds::DARK_OAK_BUTTON, ButtonBlock::new, button_properties());
    pub const PALE_OAK_BUTTON: Block = register_with(registry, BlockItemIds::PALE_OAK_BUTTON, ButtonBlock::new, button_properties());
    pub const MANGROVE_BUTTON: Block = register_with(registry, BlockItemIds::MANGROVE_BUTTON, ButtonBlock::new, button_properties());
    pub const POPLAR_BUTTON: Block = register_with(registry, BlockItemIds::POPLAR_BUTTON, ButtonBlock::new, button_properties());
    pub const BAMBOO_BUTTON: Block = register_with(registry, BlockItemIds::BAMBOO_BUTTON, ButtonBlock::new, button_properties());
    pub const SKELETON_SKULL: Block = register_with(registry, BlockItemIds::SKELETON_SKULL, |p| SkullBlock::new(SkullTypes::SKELETON, p), Properties::of().instrument(NoteBlockInstrument::SKELETON).strength(1.0).push_reaction(PushReaction::POPPED).no_occlusion());
    pub const SKELETON_WALL_SKULL: Block = register_with(registry, BlockIds::SKELETON_WALL_SKULL, |p| WallSkullBlock::new(SkullTypes::SKELETON, p), wall_variant(registry, Blocks::SKELETON_SKULL, true).strength(1.0).push_reaction(PushReaction::POPPED));
    pub const WITHER_SKELETON_SKULL: Block = register_with(registry, BlockItemIds::WITHER_SKELETON_SKULL, WitherSkullBlock::new, Properties::of().instrument(NoteBlockInstrument::WITHER_SKELETON).strength(1.0).push_reaction(PushReaction::POPPED).no_occlusion());
    pub const WITHER_SKELETON_WALL_SKULL: Block = register_with(registry, BlockIds::WITHER_SKELETON_WALL_SKULL, WitherWallSkullBlock::new, wall_variant(registry, Blocks::WITHER_SKELETON_SKULL, true).strength(1.0).push_reaction(PushReaction::POPPED));
    pub const ZOMBIE_HEAD: Block = register_with(registry, BlockItemIds::ZOMBIE_HEAD, |p| SkullBlock::new(SkullTypes::ZOMBIE, p), Properties::of().instrument(NoteBlockInstrument::ZOMBIE).strength(1.0).push_reaction(PushReaction::POPPED).no_occlusion());
    pub const ZOMBIE_WALL_HEAD: Block = register_with(registry, BlockIds::ZOMBIE_WALL_HEAD, |p| WallSkullBlock::new(SkullTypes::ZOMBIE, p), wall_variant(registry, Blocks::ZOMBIE_HEAD, true).strength(1.0).push_reaction(PushReaction::POPPED));
    pub const PLAYER_HEAD: Block = register_with(registry, BlockItemIds::PLAYER_HEAD, PlayerHeadBlock::new, Properties::of().instrument(NoteBlockInstrument::CUSTOM_HEAD).strength(1.0).push_reaction(PushReaction::POPPED).no_occlusion());
    pub const PLAYER_WALL_HEAD: Block = register_with(registry, BlockIds::PLAYER_WALL_HEAD, PlayerWallHeadBlock::new, wall_variant(registry, Blocks::PLAYER_HEAD, true).strength(1.0).push_reaction(PushReaction::POPPED));
    pub const CREEPER_HEAD: Block = register_with(registry, BlockItemIds::CREEPER_HEAD, |p| SkullBlock::new(SkullTypes::CREEPER, p), Properties::of().instrument(NoteBlockInstrument::CREEPER).strength(1.0).push_reaction(PushReaction::POPPED).no_occlusion());
    pub const CREEPER_WALL_HEAD: Block = register_with(registry, BlockIds::CREEPER_WALL_HEAD, |p| WallSkullBlock::new(SkullTypes::CREEPER, p), wall_variant(registry, Blocks::CREEPER_HEAD, true).strength(1.0).push_reaction(PushReaction::POPPED));
    pub const DRAGON_HEAD: Block = register_with(registry, BlockItemIds::DRAGON_HEAD, |p| SkullBlock::new(SkullTypes::DRAGON, p), Properties::of().instrument(NoteBlockInstrument::DRAGON).strength(1.0).push_reaction(PushReaction::POPPED).no_occlusion());
    pub const DRAGON_WALL_HEAD: Block = register_with(registry, BlockIds::DRAGON_WALL_HEAD, |p| WallSkullBlock::new(SkullTypes::DRAGON, p), wall_variant(registry, Blocks::DRAGON_HEAD, true).strength(1.0).push_reaction(PushReaction::POPPED));
    pub const PIGLIN_HEAD: Block = register_with(registry, BlockItemIds::PIGLIN_HEAD, |p| SkullBlock::new(SkullTypes::PIGLIN, p), Properties::of().instrument(NoteBlockInstrument::PIGLIN).strength(1.0).push_reaction(PushReaction::POPPED).no_occlusion());
    pub const PIGLIN_WALL_HEAD: Block = register_with(registry, BlockIds::PIGLIN_WALL_HEAD, PiglinWallSkullBlock::new, wall_variant(registry, Blocks::PIGLIN_HEAD, true).strength(1.0).push_reaction(PushReaction::POPPED));
    pub const ANVIL: Block = register_with(registry, BlockItemIds::ANVIL, AnvilBlock::new, Properties::of() .map_color(MapColor::METAL) .requires_correct_tool_for_drops() .strength_and_resistance(5.0, 1200.0)  .push_reaction(PushReaction::IMMOVEABLE));
    pub const CHIPPED_ANVIL: Block = register_with(registry, BlockItemIds::CHIPPED_ANVIL, AnvilBlock::new, Properties::of() .map_color(MapColor::METAL) .requires_correct_tool_for_drops() .strength_and_resistance(5.0, 1200.0)  .push_reaction(PushReaction::IMMOVEABLE));
    pub const DAMAGED_ANVIL: Block = register_with(registry, BlockItemIds::DAMAGED_ANVIL, AnvilBlock::new, Properties::of() .map_color(MapColor::METAL) .requires_correct_tool_for_drops() .strength_and_resistance(5.0, 1200.0)  .push_reaction(PushReaction::IMMOVEABLE));
    pub const TRAPPED_CHEST: Block = register_with(registry, BlockItemIds::TRAPPED_CHEST, TrappedChestBlock::new, Properties::of().map_color(MapColor::WOOD).instrument(NoteBlockInstrument::BASS).strength(2.5).ignited_by_lava());
    pub const LIGHT_WEIGHTED_PRESSURE_PLATE: Block = register_with(registry, BlockItemIds::LIGHT_WEIGHTED_PRESSURE_PLATE, WeightedPressurePlateBlock::new, Properties::of().map_color(MapColor::GOLD).force_solid_on().no_collision().strength(0.5).push_reaction(PushReaction::POPPED));
    pub const HEAVY_WEIGHTED_PRESSURE_PLATE: Block = register_with(registry, BlockItemIds::HEAVY_WEIGHTED_PRESSURE_PLATE, WeightedPressurePlateBlock::new, Properties::of().map_color(MapColor::METAL).force_solid_on().no_collision().strength(0.5).push_reaction(PushReaction::POPPED));
    pub const COMPARATOR: Block = register_with(registry, BlockItemIds::COMPARATOR, ComparatorBlock::new, Properties::of().instabreak().push_reaction(PushReaction::POPPED));
    pub const DAYLIGHT_DETECTOR: Block = register_with(registry, BlockItemIds::DAYLIGHT_DETECTOR, DaylightDetectorBlock::new, Properties::of().map_color(MapColor::WOOD).instrument(NoteBlockInstrument::BASS).strength(0.2).ignited_by_lava());
    pub const REDSTONE_BLOCK: Block = register_with(registry, BlockItemIds::REDSTONE_BLOCK, PoweredBlock::new, Properties::of() .map_color(MapColor::FIRE) .requires_correct_tool_for_drops() .strength_and_resistance(5.0, 6.0)  .is_redstone_conductor(StatePredicate::State(None, never)));
    pub const NETHER_QUARTZ_ORE: Block = register_with(registry, BlockItemIds::NETHER_QUARTZ_ORE, DropExperienceBlock::new, Properties::of() .map_color(MapColor::NETHER) .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .strength_and_resistance(3.0, 3.0) );
    pub const HOPPER: Block = register_with(registry, BlockItemIds::HOPPER, HopperBlock::new, Properties::of().map_color(MapColor::STONE).requires_correct_tool_for_drops().strength_and_resistance(3.0, 4.8).no_occlusion());
    pub const QUARTZ_BLOCK: Block = register(registry, BlockItemIds::QUARTZ_BLOCK, Properties::of().map_color(MapColor::QUARTZ).instrument(NoteBlockInstrument::BASEDRUM).requires_correct_tool_for_drops().strength(0.8));
    pub const CHISELED_QUARTZ_BLOCK: Block = register(registry, BlockItemIds::CHISELED_QUARTZ_BLOCK, Properties::of().map_color(MapColor::QUARTZ).instrument(NoteBlockInstrument::BASEDRUM).requires_correct_tool_for_drops().strength(0.8));
    pub const QUARTZ_PILLAR: Block = register_with(registry, BlockItemIds::QUARTZ_PILLAR, RotatedPillarBlock::new, Properties::of().map_color(MapColor::QUARTZ).instrument(NoteBlockInstrument::BASEDRUM).requires_correct_tool_for_drops().strength(0.8));
    pub const QUARTZ_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::QUARTZ_STAIRS, Blocks::QUARTZ_BLOCK);
    pub const ACTIVATOR_RAIL: Block = register_with(registry, BlockItemIds::ACTIVATOR_RAIL, PoweredRailBlock::new, Properties::of().no_collision().strength(0.7));
    pub const DROPPER: Block = register_with(registry, BlockItemIds::DROPPER, DropperBlock::new, Properties::of().map_color(MapColor::STONE).instrument(NoteBlockInstrument::BASEDRUM).requires_correct_tool_for_drops().strength(3.5));
    pub const DYED_TERRACOTTA: ColorCollection<Block> = ColorCollection::register_blocks(registry, BlockItemIds::DYED_TERRACOTTA, |_, p| Block::new(p), |color| Properties::of().map_color(color.get_terracotta_color()).instrument(NoteBlockInstrument::BASEDRUM).requires_correct_tool_for_drops().strength_and_resistance(1.25, 4.2));
    pub const STAINED_GLASS_PANE: ColorCollection<Block> = ColorCollection::register_blocks(registry, BlockItemIds::STAINED_GLASS_PANE, |_, p| StainedGlassPaneBlock::new(p), |_| Properties::of().instrument(NoteBlockInstrument::HAT).strength(0.3).no_occlusion());
    pub const ACACIA_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::ACACIA_STAIRS, Blocks::ACACIA_PLANKS);
    pub const CHERRY_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::CHERRY_STAIRS, Blocks::CHERRY_PLANKS);
    pub const DARK_OAK_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::DARK_OAK_STAIRS, Blocks::DARK_OAK_PLANKS);
    pub const PALE_OAK_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::PALE_OAK_STAIRS, Blocks::PALE_OAK_PLANKS);
    pub const MANGROVE_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::MANGROVE_STAIRS, Blocks::MANGROVE_PLANKS);
    pub const POPLAR_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::POPLAR_STAIRS, Blocks::POPLAR_PLANKS);
    pub const BAMBOO_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::BAMBOO_STAIRS, Blocks::BAMBOO_PLANKS);
    pub const BAMBOO_MOSAIC_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::BAMBOO_MOSAIC_STAIRS, Blocks::BAMBOO_MOSAIC);
    pub const SLIME_BLOCK: Block = register_with(registry, BlockItemIds::SLIME_BLOCK, SlimeBlock::new, Properties::of().map_color(MapColor::GRASS).friction(0.8).bounce_restitution(1.0).no_occlusion());
    pub const BARRIER: Block = register_with(registry, BlockItemIds::BARRIER, BarrierBlock::new, Properties::of() .strength_and_resistance(-1.0, 3600000.8) .map_color_fn(waterlogged_map_color(MapColor::NONE)) .no_loot_table() .no_occlusion()  .no_terrain_particles() .push_reaction(PushReaction::IMMOVEABLE));
    pub const LIGHT: Block = register_with(registry, BlockItemIds::LIGHT, LightBlock::new, Properties::of() .replaceable() .strength_and_resistance(-1.0, 3600000.8) .map_color_fn(waterlogged_map_color(MapColor::NONE)) .no_loot_table() .no_occlusion() .light_level(LightBlock::light_emission));
    pub const IRON_TRAPDOOR: Block = register_with(registry, BlockItemIds::IRON_TRAPDOOR, TrapDoorBlock::new, Properties::of().map_color(MapColor::METAL).requires_correct_tool_for_drops().strength(5.0).no_occlusion());
    pub const PRISMARINE: Block = register(registry, BlockItemIds::PRISMARINE, Properties::of() .map_color(MapColor::COLOR_CYAN) .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .strength_and_resistance(1.5, 6.0));
    pub const PRISMARINE_BRICKS: Block = register(registry, BlockItemIds::PRISMARINE_BRICKS, Properties::of().map_color(MapColor::DIAMOND).instrument(NoteBlockInstrument::BASEDRUM).requires_correct_tool_for_drops().strength_and_resistance(1.5, 6.0));
    pub const DARK_PRISMARINE: Block = register(registry, BlockItemIds::DARK_PRISMARINE, Properties::of().map_color(MapColor::DIAMOND).instrument(NoteBlockInstrument::BASEDRUM).requires_correct_tool_for_drops().strength_and_resistance(1.5, 6.0));
    pub const PRISMARINE_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::PRISMARINE_STAIRS, Blocks::PRISMARINE);
    pub const PRISMARINE_BRICK_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::PRISMARINE_BRICK_STAIRS, Blocks::PRISMARINE_BRICKS);
    pub const DARK_PRISMARINE_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::DARK_PRISMARINE_STAIRS, Blocks::DARK_PRISMARINE);
    pub const PRISMARINE_SLAB: Block = register_slab(registry, BlockItemIds::PRISMARINE_SLAB, Blocks::PRISMARINE);
    pub const PRISMARINE_BRICK_SLAB: Block = register_slab(registry, BlockItemIds::PRISMARINE_BRICK_SLAB, Blocks::PRISMARINE_BRICKS);
    pub const DARK_PRISMARINE_SLAB: Block = register_slab(registry, BlockItemIds::DARK_PRISMARINE_SLAB, Blocks::DARK_PRISMARINE);
    pub const SEA_LANTERN: Block = register(registry, BlockItemIds::SEA_LANTERN, Properties::of() .map_color(MapColor::QUARTZ) .instrument(NoteBlockInstrument::HAT) .strength(0.3)  .light_level(|_| 15) .is_redstone_conductor(StatePredicate::State(None, never)));
    pub const HAY_BLOCK: Block = register_with(registry, BlockItemIds::HAY_BLOCK, HayBlock::new, Properties::of().map_color(MapColor::COLOR_YELLOW).instrument(NoteBlockInstrument::BANJO).strength(0.5));
    pub const CARPET: ColorCollection<Block> = ColorCollection::register_blocks(registry, BlockItemIds::CARPET, |_, p| WoolCarpetBlock::new(p), |color| Properties::of().map_color(color.get_map_color()).strength(0.1).ignited_by_lava());
    pub const TERRACOTTA: Block = register(registry, BlockItemIds::TERRACOTTA, Properties::of() .map_color(MapColor::COLOR_ORANGE) .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .strength_and_resistance(1.25, 4.2));
    pub const COAL_BLOCK: Block = register(registry, BlockItemIds::COAL_BLOCK, Properties::of() .map_color(MapColor::COLOR_BLACK) .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .strength_and_resistance(5.0, 6.0));
    pub const PACKED_ICE: Block = register(registry, BlockItemIds::PACKED_ICE, Properties::of().map_color(MapColor::ICE).instrument(NoteBlockInstrument::CHIME).friction(0.98).strength(0.5));
    pub const SUNFLOWER: Block = register_with(registry, BlockItemIds::SUNFLOWER, TallFlowerBlock::new, Properties::of() .map_color(MapColor::PLANT) .no_collision() .instabreak()  .offset_type(OffsetType::XZ) .ignited_by_lava() .push_reaction(PushReaction::POPPED));
    pub const LILAC: Block = register_with(registry, BlockItemIds::LILAC, TallFlowerBlock::new, Properties::of() .map_color(MapColor::PLANT) .no_collision() .instabreak()  .offset_type(OffsetType::XZ) .ignited_by_lava() .push_reaction(PushReaction::POPPED));
    pub const ROSE_BUSH: Block = register_with(registry, BlockItemIds::ROSE_BUSH, TallFlowerBlock::new, Properties::of() .map_color(MapColor::PLANT) .no_collision() .instabreak()  .offset_type(OffsetType::XZ) .ignited_by_lava() .push_reaction(PushReaction::POPPED));
    pub const PEONY: Block = register_with(registry, BlockItemIds::PEONY, TallFlowerBlock::new, Properties::of() .map_color(MapColor::PLANT) .no_collision() .instabreak()  .offset_type(OffsetType::XZ) .ignited_by_lava() .push_reaction(PushReaction::POPPED));
    pub const TALL_GRASS: Block = register_with(registry, BlockItemIds::TALL_GRASS, DoublePlantBlock::new, Properties::of() .map_color(MapColor::PLANT) .replaceable() .no_collision() .instabreak()  .offset_type(OffsetType::XZ) .ignited_by_lava() .push_reaction(PushReaction::POPPED));
    pub const LARGE_FERN: Block = register_with(registry, BlockItemIds::LARGE_FERN, DoublePlantBlock::new, Properties::of() .map_color(MapColor::PLANT) .replaceable() .no_collision() .instabreak()  .offset_type(OffsetType::XZ) .ignited_by_lava() .push_reaction(PushReaction::POPPED));
    pub const BANNER: ColorCollection<Block> = ColorCollection::register_blocks(registry, BlockItemIds::BANNER, |_, p| BannerBlock::new(p), |_| Properties::of().map_color(MapColor::WOOD).force_solid_on().instrument(NoteBlockInstrument::BASS).no_collision().strength(1.0).ignited_by_lava());
    pub const WALL_BANNER: ColorCollection<Block> = ColorCollection::register_blocks(registry, BlockIds::WALL_BANNER, |_, p| WallBannerBlock::new(p), |color| wall_variant(registry, *Blocks::BANNER.pick(color), true).map_color(MapColor::WOOD).force_solid_on().instrument(NoteBlockInstrument::BASS).no_collision().strength(1.0).ignited_by_lava());
    pub const RED_SANDSTONE: Block = register(registry, BlockItemIds::RED_SANDSTONE, Properties::of().map_color(MapColor::COLOR_ORANGE).instrument(NoteBlockInstrument::BASEDRUM).requires_correct_tool_for_drops().strength(0.8));
    pub const CHISELED_RED_SANDSTONE: Block = register(registry, BlockItemIds::CHISELED_RED_SANDSTONE, Properties::of().map_color(MapColor::COLOR_ORANGE).instrument(NoteBlockInstrument::BASEDRUM).requires_correct_tool_for_drops().strength(0.8));
    pub const CUT_RED_SANDSTONE: Block = register(registry, BlockItemIds::CUT_RED_SANDSTONE, Properties::of().map_color(MapColor::COLOR_ORANGE).instrument(NoteBlockInstrument::BASEDRUM).requires_correct_tool_for_drops().strength(0.8));
    pub const RED_SANDSTONE_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::RED_SANDSTONE_STAIRS, Blocks::RED_SANDSTONE);
    pub const OAK_SLAB: Block = register_slab(registry, BlockItemIds::OAK_SLAB, Blocks::OAK_PLANKS);
    pub const SPRUCE_SLAB: Block = register_slab(registry, BlockItemIds::SPRUCE_SLAB, Blocks::SPRUCE_PLANKS);
    pub const BIRCH_SLAB: Block = register_slab(registry, BlockItemIds::BIRCH_SLAB, Blocks::BIRCH_PLANKS);
    pub const JUNGLE_SLAB: Block = register_slab(registry, BlockItemIds::JUNGLE_SLAB, Blocks::JUNGLE_PLANKS);
    pub const ACACIA_SLAB: Block = register_slab(registry, BlockItemIds::ACACIA_SLAB, Blocks::ACACIA_PLANKS);
    pub const CHERRY_SLAB: Block = register_slab(registry, BlockItemIds::CHERRY_SLAB, Blocks::CHERRY_PLANKS);
    pub const DARK_OAK_SLAB: Block = register_slab(registry, BlockItemIds::DARK_OAK_SLAB, Blocks::DARK_OAK_PLANKS);
    pub const PALE_OAK_SLAB: Block = register_slab(registry, BlockItemIds::PALE_OAK_SLAB, Blocks::PALE_OAK_PLANKS);
    pub const MANGROVE_SLAB: Block = register_slab(registry, BlockItemIds::MANGROVE_SLAB, Blocks::MANGROVE_PLANKS);
    pub const POPLAR_SLAB: Block = register_slab(registry, BlockItemIds::POPLAR_SLAB, Blocks::POPLAR_PLANKS);
    pub const BAMBOO_SLAB: Block = register_slab(registry, BlockItemIds::BAMBOO_SLAB, Blocks::BAMBOO_PLANKS);
    pub const BAMBOO_MOSAIC_SLAB: Block = register_slab(registry, BlockItemIds::BAMBOO_MOSAIC_SLAB, Blocks::BAMBOO_MOSAIC);
    pub const STONE_SLAB: Block = register_slab_destroy_time(registry, BlockItemIds::STONE_SLAB, Blocks::STONE, 2.0);
    pub const SANDSTONE_SLAB: Block = register_slab_strength(registry, BlockItemIds::SANDSTONE_SLAB, Blocks::SANDSTONE, 2.0, 6.0);
    pub const CUT_SANDSTONE_SLAB: Block = register_slab_strength(registry, BlockItemIds::CUT_SANDSTONE_SLAB, Blocks::CUT_SANDSTONE, 2.0, 6.0);
    pub const PETRIFIED_OAK_SLAB: Block = register_with(registry, BlockItemIds::PETRIFIED_OAK_SLAB, SlabBlock::new, Properties::of().map_color(MapColor::WOOD).instrument(NoteBlockInstrument::BASEDRUM).requires_correct_tool_for_drops().strength_and_resistance(2.0, 6.0));
    pub const COBBLESTONE_SLAB: Block = register_slab(registry, BlockItemIds::COBBLESTONE_SLAB, Blocks::COBBLESTONE);
    pub const BRICK_SLAB: Block = register_slab(registry, BlockItemIds::BRICK_SLAB, Blocks::BRICKS);
    pub const STONE_BRICK_SLAB: Block = register_slab_destroy_time(registry, BlockItemIds::STONE_BRICK_SLAB, Blocks::STONE_BRICKS, 2.0);
    pub const MUD_BRICK_SLAB: Block = register_slab(registry, BlockItemIds::MUD_BRICK_SLAB, Blocks::MUD_BRICKS);
    pub const NETHER_BRICK_SLAB: Block = register_slab(registry, BlockItemIds::NETHER_BRICK_SLAB, Blocks::NETHER_BRICKS);
    pub const QUARTZ_SLAB: Block = register_slab_destroy_time(registry, BlockItemIds::QUARTZ_SLAB, Blocks::QUARTZ_BLOCK, 2.0);
    pub const RED_SANDSTONE_SLAB: Block = register_slab_destroy_time(registry, BlockItemIds::RED_SANDSTONE_SLAB, Blocks::RED_SANDSTONE, 2.0);
    pub const CUT_RED_SANDSTONE_SLAB: Block = register_slab_destroy_time(registry, BlockItemIds::CUT_RED_SANDSTONE_SLAB, Blocks::CUT_RED_SANDSTONE, 2.0);
    pub const SMOOTH_STONE: Block = register(registry, BlockItemIds::SMOOTH_STONE, Properties::of().map_color(MapColor::STONE).instrument(NoteBlockInstrument::BASEDRUM).requires_correct_tool_for_drops().strength_and_resistance(2.0, 6.0));
    pub const SMOOTH_STONE_SLAB: Block = register_slab(registry, BlockItemIds::SMOOTH_STONE_SLAB, Blocks::SMOOTH_STONE);
    pub const SMOOTH_SANDSTONE: Block = register(registry, BlockItemIds::SMOOTH_SANDSTONE, Properties::of().map_color(MapColor::SAND).instrument(NoteBlockInstrument::BASEDRUM).requires_correct_tool_for_drops().strength_and_resistance(2.0, 6.0));
    pub const SMOOTH_QUARTZ: Block = register(registry, BlockItemIds::SMOOTH_QUARTZ, Properties::of().map_color(MapColor::QUARTZ).instrument(NoteBlockInstrument::BASEDRUM).requires_correct_tool_for_drops().strength_and_resistance(2.0, 6.0));
    pub const SMOOTH_RED_SANDSTONE: Block = register(registry, BlockItemIds::SMOOTH_RED_SANDSTONE, Properties::of() .map_color(MapColor::COLOR_ORANGE) .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .strength_and_resistance(2.0, 6.0));
    pub const SPRUCE_FENCE_GATE: Block = register_with(registry, BlockItemIds::SPRUCE_FENCE_GATE, FenceGateBlock::new, Properties::of() .map_color(registry.get(Blocks::SPRUCE_PLANKS).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .strength_and_resistance(2.0, 3.0) .ignited_by_lava());
    pub const BIRCH_FENCE_GATE: Block = register_with(registry, BlockItemIds::BIRCH_FENCE_GATE, FenceGateBlock::new, Properties::of() .map_color(registry.get(Blocks::BIRCH_PLANKS).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .strength_and_resistance(2.0, 3.0) .ignited_by_lava());
    pub const JUNGLE_FENCE_GATE: Block = register_with(registry, BlockItemIds::JUNGLE_FENCE_GATE, FenceGateBlock::new, Properties::of() .map_color(registry.get(Blocks::JUNGLE_PLANKS).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .strength_and_resistance(2.0, 3.0) .ignited_by_lava());
    pub const ACACIA_FENCE_GATE: Block = register_with(registry, BlockItemIds::ACACIA_FENCE_GATE, FenceGateBlock::new, Properties::of() .map_color(registry.get(Blocks::ACACIA_PLANKS).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .strength_and_resistance(2.0, 3.0) .ignited_by_lava());
    pub const CHERRY_FENCE_GATE: Block = register_with(registry, BlockItemIds::CHERRY_FENCE_GATE, FenceGateBlock::new, Properties::of() .map_color(registry.get(Blocks::CHERRY_PLANKS).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .strength_and_resistance(2.0, 3.0) .ignited_by_lava());
    pub const DARK_OAK_FENCE_GATE: Block = register_with(registry, BlockItemIds::DARK_OAK_FENCE_GATE, FenceGateBlock::new, Properties::of() .map_color(registry.get(Blocks::DARK_OAK_PLANKS).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .strength_and_resistance(2.0, 3.0) .ignited_by_lava());
    pub const PALE_OAK_FENCE_GATE: Block = register_with(registry, BlockItemIds::PALE_OAK_FENCE_GATE, FenceGateBlock::new, Properties::of() .map_color(registry.get(Blocks::PALE_OAK_PLANKS).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .strength_and_resistance(2.0, 3.0) .ignited_by_lava());
    pub const MANGROVE_FENCE_GATE: Block = register_with(registry, BlockItemIds::MANGROVE_FENCE_GATE, FenceGateBlock::new, Properties::of() .map_color(registry.get(Blocks::MANGROVE_PLANKS).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .strength_and_resistance(2.0, 3.0) .ignited_by_lava());
    pub const POPLAR_FENCE_GATE: Block = register_with(registry, BlockItemIds::POPLAR_FENCE_GATE, FenceGateBlock::new, Properties::of() .map_color(registry.get(Blocks::POPLAR_PLANKS).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .strength_and_resistance(2.0, 3.0) .ignited_by_lava());
    pub const BAMBOO_FENCE_GATE: Block = register_with(registry, BlockItemIds::BAMBOO_FENCE_GATE, FenceGateBlock::new, Properties::of() .map_color(registry.get(Blocks::BAMBOO_PLANKS).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .strength_and_resistance(2.0, 3.0) .ignited_by_lava());
    pub const SPRUCE_FENCE: Block = register_with(registry, BlockItemIds::SPRUCE_FENCE, FenceBlock::new, Properties::of() .map_color(registry.get(Blocks::SPRUCE_PLANKS).default_map_color()) .instrument(NoteBlockInstrument::BASS) .strength_and_resistance(2.0, 3.0) .ignited_by_lava() );
    pub const BIRCH_FENCE: Block = register_with(registry, BlockItemIds::BIRCH_FENCE, FenceBlock::new, Properties::of() .map_color(registry.get(Blocks::BIRCH_PLANKS).default_map_color()) .instrument(NoteBlockInstrument::BASS) .strength_and_resistance(2.0, 3.0) .ignited_by_lava() );
    pub const JUNGLE_FENCE: Block = register_with(registry, BlockItemIds::JUNGLE_FENCE, FenceBlock::new, Properties::of() .map_color(registry.get(Blocks::JUNGLE_PLANKS).default_map_color()) .instrument(NoteBlockInstrument::BASS) .strength_and_resistance(2.0, 3.0) .ignited_by_lava() );
    pub const ACACIA_FENCE: Block = register_with(registry, BlockItemIds::ACACIA_FENCE, FenceBlock::new, Properties::of() .map_color(registry.get(Blocks::ACACIA_PLANKS).default_map_color()) .instrument(NoteBlockInstrument::BASS) .strength_and_resistance(2.0, 3.0) .ignited_by_lava() );
    pub const CHERRY_FENCE: Block = register_with(registry, BlockItemIds::CHERRY_FENCE, FenceBlock::new, Properties::of() .map_color(registry.get(Blocks::CHERRY_PLANKS).default_map_color()) .instrument(NoteBlockInstrument::BASS) .strength_and_resistance(2.0, 3.0) .ignited_by_lava() );
    pub const DARK_OAK_FENCE: Block = register_with(registry, BlockItemIds::DARK_OAK_FENCE, FenceBlock::new, Properties::of() .map_color(registry.get(Blocks::DARK_OAK_PLANKS).default_map_color()) .instrument(NoteBlockInstrument::BASS) .strength_and_resistance(2.0, 3.0) .ignited_by_lava() );
    pub const PALE_OAK_FENCE: Block = register_with(registry, BlockItemIds::PALE_OAK_FENCE, FenceBlock::new, Properties::of() .map_color(registry.get(Blocks::PALE_OAK_PLANKS).default_map_color()) .instrument(NoteBlockInstrument::BASS) .strength_and_resistance(2.0, 3.0) .ignited_by_lava() );
    pub const MANGROVE_FENCE: Block = register_with(registry, BlockItemIds::MANGROVE_FENCE, FenceBlock::new, Properties::of() .map_color(registry.get(Blocks::MANGROVE_PLANKS).default_map_color()) .instrument(NoteBlockInstrument::BASS) .strength_and_resistance(2.0, 3.0) .ignited_by_lava() );
    pub const POPLAR_FENCE: Block = register_with(registry, BlockItemIds::POPLAR_FENCE, FenceBlock::new, Properties::of() .map_color(registry.get(Blocks::POPLAR_PLANKS).default_map_color()) .instrument(NoteBlockInstrument::BASS) .strength_and_resistance(2.0, 3.0) .ignited_by_lava() );
    pub const BAMBOO_FENCE: Block = register_with(registry, BlockItemIds::BAMBOO_FENCE, FenceBlock::new, Properties::of() .map_color(registry.get(Blocks::BAMBOO_PLANKS).default_map_color()) .instrument(NoteBlockInstrument::BASS) .strength_and_resistance(2.0, 3.0)  .ignited_by_lava());
    pub const SPRUCE_DOOR: Block = register_with(registry, BlockItemIds::SPRUCE_DOOR, DoorBlock::new, Properties::of() .map_color(registry.get(Blocks::SPRUCE_PLANKS).default_map_color()) .instrument(NoteBlockInstrument::BASS) .strength(3.0) .no_occlusion() .ignited_by_lava() .push_reaction(PushReaction::POPPED));
    pub const BIRCH_DOOR: Block = register_with(registry, BlockItemIds::BIRCH_DOOR, DoorBlock::new, Properties::of() .map_color(registry.get(Blocks::BIRCH_PLANKS).default_map_color()) .instrument(NoteBlockInstrument::BASS) .strength(3.0) .no_occlusion() .ignited_by_lava() .push_reaction(PushReaction::POPPED));
    pub const JUNGLE_DOOR: Block = register_with(registry, BlockItemIds::JUNGLE_DOOR, DoorBlock::new, Properties::of() .map_color(registry.get(Blocks::JUNGLE_PLANKS).default_map_color()) .instrument(NoteBlockInstrument::BASS) .strength(3.0) .no_occlusion() .ignited_by_lava() .push_reaction(PushReaction::POPPED));
    pub const ACACIA_DOOR: Block = register_with(registry, BlockItemIds::ACACIA_DOOR, DoorBlock::new, Properties::of() .map_color(registry.get(Blocks::ACACIA_PLANKS).default_map_color()) .instrument(NoteBlockInstrument::BASS) .strength(3.0) .no_occlusion() .ignited_by_lava() .push_reaction(PushReaction::POPPED));
    pub const CHERRY_DOOR: Block = register_with(registry, BlockItemIds::CHERRY_DOOR, DoorBlock::new, Properties::of() .map_color(registry.get(Blocks::CHERRY_PLANKS).default_map_color()) .instrument(NoteBlockInstrument::BASS) .strength(3.0) .no_occlusion() .ignited_by_lava() .push_reaction(PushReaction::POPPED));
    pub const DARK_OAK_DOOR: Block = register_with(registry, BlockItemIds::DARK_OAK_DOOR, DoorBlock::new, Properties::of() .map_color(registry.get(Blocks::DARK_OAK_PLANKS).default_map_color()) .instrument(NoteBlockInstrument::BASS) .strength(3.0) .no_occlusion() .ignited_by_lava() .push_reaction(PushReaction::POPPED));
    pub const PALE_OAK_DOOR: Block = register_with(registry, BlockItemIds::PALE_OAK_DOOR, DoorBlock::new, Properties::of() .map_color(registry.get(Blocks::PALE_OAK_PLANKS).default_map_color()) .instrument(NoteBlockInstrument::BASS) .strength(3.0) .no_occlusion() .ignited_by_lava() .push_reaction(PushReaction::POPPED));
    pub const MANGROVE_DOOR: Block = register_with(registry, BlockItemIds::MANGROVE_DOOR, DoorBlock::new, Properties::of() .map_color(registry.get(Blocks::MANGROVE_PLANKS).default_map_color()) .instrument(NoteBlockInstrument::BASS) .strength(3.0) .no_occlusion() .ignited_by_lava() .push_reaction(PushReaction::POPPED));
    pub const POPLAR_DOOR: Block = register_with(registry, BlockItemIds::POPLAR_DOOR, DoorBlock::new, Properties::of() .map_color(registry.get(Blocks::POPLAR_PLANKS).default_map_color()) .instrument(NoteBlockInstrument::BASS) .strength(3.0) .no_occlusion() .ignited_by_lava() .push_reaction(PushReaction::POPPED));
    pub const BAMBOO_DOOR: Block = register_with(registry, BlockItemIds::BAMBOO_DOOR, DoorBlock::new, Properties::of() .map_color(registry.get(Blocks::BAMBOO_PLANKS).default_map_color()) .instrument(NoteBlockInstrument::BASS) .strength(3.0) .no_occlusion() .ignited_by_lava() .push_reaction(PushReaction::POPPED));
    pub const END_ROD: Block = register_with(registry, BlockItemIds::END_ROD, EndRodBlock::new, Properties::of().force_solid_off().instabreak().light_level(|_| 14).no_occlusion());
    pub const CHORUS_PLANT: Block = register_with(registry, BlockItemIds::CHORUS_PLANT, ChorusPlantBlock::new, Properties::of() .map_color(MapColor::COLOR_PURPLE) .force_solid_off() .strength(0.4)  .no_occlusion() .push_reaction(PushReaction::POPPED));
    pub const CHORUS_FLOWER: Block = register_with(registry, BlockItemIds::CHORUS_FLOWER, ChorusFlowerBlock::new, Properties::of() .map_color(MapColor::COLOR_PURPLE) .force_solid_off() .random_ticks() .strength(0.4)  .no_occlusion()  .push_reaction(PushReaction::POPPED) .is_redstone_conductor(StatePredicate::State(None, never)));
    pub const PURPUR_BLOCK: Block = register(registry, BlockItemIds::PURPUR_BLOCK, Properties::of() .map_color(MapColor::COLOR_MAGENTA) .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .strength_and_resistance(1.5, 6.0));
    pub const PURPUR_SLAB: Block = register_slab_destroy_time(registry, BlockItemIds::PURPUR_SLAB, Blocks::PURPUR_BLOCK, 2.0);
    pub const PURPUR_PILLAR: Block = register_with(registry, BlockItemIds::PURPUR_PILLAR, RotatedPillarBlock::new, Properties::of() .map_color(MapColor::COLOR_MAGENTA) .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .strength_and_resistance(1.5, 6.0));
    pub const PURPUR_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::PURPUR_STAIRS, Blocks::PURPUR_BLOCK);
    pub const END_STONE_BRICKS: Block = register(registry, BlockItemIds::END_STONE_BRICKS, Properties::of().map_color(MapColor::SAND).instrument(NoteBlockInstrument::BASEDRUM).requires_correct_tool_for_drops().strength_and_resistance(3.0, 9.0));
    pub const TORCHFLOWER_CROP: Block = register_with(registry, BlockItemIds::TORCHFLOWER_CROP, TorchflowerCropBlock::new, Properties::of() .map_color(MapColor::PLANT) .no_collision() .random_ticks() .instabreak()  .push_reaction(PushReaction::POPPED));
    pub const PITCHER_CROP: Block = register_with(registry, BlockItemIds::PITCHER_CROP, PitcherCropBlock::new, Properties::of() .map_color(MapColor::PLANT) .no_collision() .random_ticks() .instabreak()  .push_reaction(PushReaction::POPPED));
    pub const PITCHER_PLANT: Block = register_with(registry, BlockItemIds::PITCHER_PLANT, DoublePlantBlock::new, Properties::of() .map_color(MapColor::PLANT) .no_collision() .instabreak()  .offset_type(OffsetType::XZ) .ignited_by_lava() .push_reaction(PushReaction::POPPED));
    pub const BEETROOTS: Block = register_with(registry, BlockItemIds::BEETROOT_CROP, BeetrootBlock::new, Properties::of() .map_color(MapColor::PLANT) .no_collision() .random_ticks() .instabreak()  .push_reaction(PushReaction::POPPED));
    pub const DIRT_PATH: Block = register_with(registry, BlockItemIds::DIRT_PATH, PathBlock::new, Properties::of().map_color(MapColor::DIRT).strength(0.65).is_suffocating(StatePredicate::State(None, always)));
    pub const END_GATEWAY: Block = register_with(registry, BlockIds::END_GATEWAY, EndGatewayBlock::new, Properties::of() .map_color(MapColor::COLOR_BLACK) .no_collision() .light_level(|_| 15) .strength_and_resistance(-1.0, 3600000.0) .no_loot_table() .push_reaction(PushReaction::IMMOVEABLE));
    pub const REPEATING_COMMAND_BLOCK: Block = register_with(registry, BlockItemIds::REPEATING_COMMAND_BLOCK, CommandBlock::new, Properties::of().map_color(MapColor::COLOR_PURPLE).requires_correct_tool_for_drops().strength_and_resistance(-1.0, 3600000.0).no_loot_table());
    pub const CHAIN_COMMAND_BLOCK: Block = register_with(registry, BlockItemIds::CHAIN_COMMAND_BLOCK, CommandBlock::new, Properties::of().map_color(MapColor::COLOR_GREEN).requires_correct_tool_for_drops().strength_and_resistance(-1.0, 3600000.0).no_loot_table());
    pub const FROSTED_ICE: Block = register_with(registry, BlockIds::FROSTED_ICE, FrostedIceBlock::new, Properties::of() .map_color(MapColor::ICE) .friction(0.98) .strength(0.5)  .no_occlusion()  .is_redstone_conductor(StatePredicate::State(None, never)));
    pub const MAGMA_BLOCK: Block = register_with(registry, BlockItemIds::MAGMA_BLOCK, MagmaBlock::new, Properties::of() .map_color(MapColor::NETHER) .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .light_level(|_| 3) .strength(0.5) .post_process(post_process_above) .emissive_rendering(|_| true));
    pub const NETHER_WART_BLOCK: Block = register(registry, BlockItemIds::NETHER_WART_BLOCK, Properties::of().map_color(MapColor::COLOR_RED).strength(1.0));
    pub const RED_NETHER_BRICKS: Block = register(registry, BlockItemIds::RED_NETHER_BRICKS, Properties::of() .map_color(MapColor::NETHER) .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .strength_and_resistance(2.0, 6.0) );
    pub const BONE_BLOCK: Block = register_with(registry, BlockItemIds::BONE_BLOCK, RotatedPillarBlock::new, Properties::of() .map_color(MapColor::SAND) .instrument(NoteBlockInstrument::XYLOPHONE) .requires_correct_tool_for_drops() .strength(2.0) );
    pub const STRUCTURE_VOID: Block = register_with(registry, BlockItemIds::STRUCTURE_VOID, StructureVoidBlock::new, Properties::of().replaceable().no_collision().no_loot_table().no_terrain_particles().push_reaction(PushReaction::POPPED));
    pub const OBSERVER: Block = register_with(registry, BlockItemIds::OBSERVER, ObserverBlock::new, Properties::of() .map_color(MapColor::STONE) .instrument(NoteBlockInstrument::BASEDRUM) .strength(3.0) .requires_correct_tool_for_drops() .is_redstone_conductor(StatePredicate::State(None, never)));
    pub const SHULKER_BOX: Block = register_with(registry, BlockItemIds::SHULKER_BOX, ShulkerBoxBlock::new, shulker_box_properties(MapColor::COLOR_PURPLE));
    pub const DYED_SHULKER_BOX: ColorCollection<Block> = ColorCollection::register_blocks(registry, BlockItemIds::DYED_SHULKER_BOX, |_, p| ShulkerBoxBlock::new(p), |color| shulker_box_properties(if color == DyeColor::PURPLE { MapColor::TERRACOTTA_PURPLE } else { color.get_map_color() }));
    pub const GLAZED_TERRACOTTA: ColorCollection<Block> = ColorCollection::register_blocks(registry, BlockItemIds::GLAZED_TERRACOTTA, |_, p| GlazedTerracottaBlock::new(p), |color| Properties::of() .map_color(color.get_map_color()) .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .strength(1.4) .push_reaction(PushReaction::PUSH));
    pub const CONCRETE: ColorCollection<Block> = ColorCollection::register_blocks(registry, BlockItemIds::CONCRETE, |_, p| Block::new(p), |color| Properties::of() .map_color(color.get_map_color()) .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .strength(1.8));
    pub const CONCRETE_STAIRS: ColorCollection<Block> = ColorCollection::zip_map(&ColorCollection::VALUES, BlockItemIds::CONCRETE_STAIRS, |&color, id| register_stair(registry, id, *Blocks::CONCRETE.pick(color)));
    pub const CONCRETE_SLAB: ColorCollection<Block> = ColorCollection::zip_map(&ColorCollection::VALUES, BlockItemIds::CONCRETE_SLAB, |&color, id| register_slab(registry, id, *Blocks::CONCRETE.pick(color)));
    pub const CONCRETE_POWDER: ColorCollection<Block> = ColorCollection::register_blocks(registry, BlockItemIds::CONCRETE_POWDER, |_, p| ConcretePowderBlock::new(p), |color| Properties::of().map_color(color.get_map_color()).instrument(NoteBlockInstrument::SNARE).strength(0.5));
    pub const KELP: Block = register_with(registry, BlockItemIds::KELP, KelpBlock::new, Properties::of() .map_color(MapColor::WATER) .no_collision() .random_ticks() .instabreak()  .push_reaction(PushReaction::POPPED));
    pub const KELP_PLANT: Block = register_with(registry, BlockIds::KELP_PLANT, KelpPlantBlock::new, Properties::of().map_color(MapColor::WATER).no_collision().instabreak().push_reaction(PushReaction::POPPED));
    pub const DRIED_KELP_BLOCK: Block = register(registry, BlockItemIds::DRIED_KELP_BLOCK, Properties::of().map_color(MapColor::COLOR_GREEN).strength_and_resistance(0.5, 2.5));
    pub const TURTLE_EGG: Block = register_with(registry, BlockItemIds::TURTLE_EGG, TurtleEggBlock::new, Properties::of() .map_color(MapColor::SAND) .force_solid_on() .strength(0.5)  .random_ticks() .no_occlusion() .push_reaction(PushReaction::POPPED));
    pub const SNIFFER_EGG: Block = register_with(registry, BlockItemIds::SNIFFER_EGG, SnifferEggBlock::new, Properties::of().map_color(MapColor::COLOR_RED).strength(0.5).no_occlusion());
    pub const DRIED_GHAST: Block = register_with(registry, BlockItemIds::DRIED_GHAST, DriedGhastBlock::new, Properties::of().map_color(MapColor::COLOR_GRAY).force_solid_on().instabreak().no_occlusion().random_ticks());
    pub const DEAD_TUBE_CORAL_BLOCK: Block = register(registry, BlockItemIds::DEAD_TUBE_CORAL_BLOCK, Properties::of() .map_color(MapColor::COLOR_GRAY) .force_solid_on() .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .strength_and_resistance(1.5, 6.0));
    pub const DEAD_BRAIN_CORAL_BLOCK: Block = register(registry, BlockItemIds::DEAD_BRAIN_CORAL_BLOCK, Properties::of() .map_color(MapColor::COLOR_GRAY) .force_solid_on() .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .strength_and_resistance(1.5, 6.0));
    pub const DEAD_BUBBLE_CORAL_BLOCK: Block = register(registry, BlockItemIds::DEAD_BUBBLE_CORAL_BLOCK, Properties::of() .map_color(MapColor::COLOR_GRAY) .force_solid_on() .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .strength_and_resistance(1.5, 6.0));
    pub const DEAD_FIRE_CORAL_BLOCK: Block = register(registry, BlockItemIds::DEAD_FIRE_CORAL_BLOCK, Properties::of() .map_color(MapColor::COLOR_GRAY) .force_solid_on() .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .strength_and_resistance(1.5, 6.0));
    pub const DEAD_HORN_CORAL_BLOCK: Block = register(registry, BlockItemIds::DEAD_HORN_CORAL_BLOCK, Properties::of() .map_color(MapColor::COLOR_GRAY) .force_solid_on() .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .strength_and_resistance(1.5, 6.0));
    pub const TUBE_CORAL_BLOCK: Block = register_with(registry, BlockItemIds::TUBE_CORAL_BLOCK, CoralBlock::new, Properties::of() .map_color(MapColor::COLOR_BLUE) .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .strength_and_resistance(1.5, 6.0) );
    pub const BRAIN_CORAL_BLOCK: Block = register_with(registry, BlockItemIds::BRAIN_CORAL_BLOCK, CoralBlock::new, Properties::of() .map_color(MapColor::COLOR_PINK) .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .strength_and_resistance(1.5, 6.0) );
    pub const BUBBLE_CORAL_BLOCK: Block = register_with(registry, BlockItemIds::BUBBLE_CORAL_BLOCK, CoralBlock::new, Properties::of() .map_color(MapColor::COLOR_PURPLE) .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .strength_and_resistance(1.5, 6.0) );
    pub const FIRE_CORAL_BLOCK: Block = register_with(registry, BlockItemIds::FIRE_CORAL_BLOCK, CoralBlock::new, Properties::of() .map_color(MapColor::COLOR_RED) .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .strength_and_resistance(1.5, 6.0) );
    pub const HORN_CORAL_BLOCK: Block = register_with(registry, BlockItemIds::HORN_CORAL_BLOCK, CoralBlock::new, Properties::of() .map_color(MapColor::COLOR_YELLOW) .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .strength_and_resistance(1.5, 6.0) );
    pub const DEAD_TUBE_CORAL: Block = register_with(registry, BlockItemIds::DEAD_TUBE_CORAL, BaseCoralPlantBlock::new, Properties::of() .map_color(MapColor::COLOR_GRAY) .force_solid_on() .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .no_collision() .instabreak());
    pub const DEAD_BRAIN_CORAL: Block = register_with(registry, BlockItemIds::DEAD_BRAIN_CORAL, BaseCoralPlantBlock::new, Properties::of() .map_color(MapColor::COLOR_GRAY) .force_solid_on() .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .no_collision() .instabreak());
    pub const DEAD_BUBBLE_CORAL: Block = register_with(registry, BlockItemIds::DEAD_BUBBLE_CORAL, BaseCoralPlantBlock::new, Properties::of() .map_color(MapColor::COLOR_GRAY) .force_solid_on() .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .no_collision() .instabreak());
    pub const DEAD_FIRE_CORAL: Block = register_with(registry, BlockItemIds::DEAD_FIRE_CORAL, BaseCoralPlantBlock::new, Properties::of() .map_color(MapColor::COLOR_GRAY) .force_solid_on() .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .no_collision() .instabreak());
    pub const DEAD_HORN_CORAL: Block = register_with(registry, BlockItemIds::DEAD_HORN_CORAL, BaseCoralPlantBlock::new, Properties::of() .map_color(MapColor::COLOR_GRAY) .force_solid_on() .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .no_collision() .instabreak());
    pub const TUBE_CORAL: Block = register_with(registry, BlockItemIds::TUBE_CORAL, CoralPlantBlock::new, Properties::of().map_color(MapColor::COLOR_BLUE).no_collision().instabreak().push_reaction(PushReaction::POPPED));
    pub const BRAIN_CORAL: Block = register_with(registry, BlockItemIds::BRAIN_CORAL, CoralPlantBlock::new, Properties::of().map_color(MapColor::COLOR_PINK).no_collision().instabreak().push_reaction(PushReaction::POPPED));
    pub const BUBBLE_CORAL: Block = register_with(registry, BlockItemIds::BUBBLE_CORAL, CoralPlantBlock::new, Properties::of().map_color(MapColor::COLOR_PURPLE).no_collision().instabreak().push_reaction(PushReaction::POPPED));
    pub const FIRE_CORAL: Block = register_with(registry, BlockItemIds::FIRE_CORAL, CoralPlantBlock::new, Properties::of().map_color(MapColor::COLOR_RED).no_collision().instabreak().push_reaction(PushReaction::POPPED));
    pub const HORN_CORAL: Block = register_with(registry, BlockItemIds::HORN_CORAL, CoralPlantBlock::new, Properties::of().map_color(MapColor::COLOR_YELLOW).no_collision().instabreak().push_reaction(PushReaction::POPPED));
    pub const DEAD_TUBE_CORAL_FAN: Block = register_with(registry, BlockItemIds::DEAD_TUBE_CORAL_FAN, BaseCoralFanBlock::new, Properties::of() .map_color(MapColor::COLOR_GRAY) .force_solid_on() .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .no_collision() .instabreak());
    pub const DEAD_BRAIN_CORAL_FAN: Block = register_with(registry, BlockItemIds::DEAD_BRAIN_CORAL_FAN, BaseCoralFanBlock::new, Properties::of() .map_color(MapColor::COLOR_GRAY) .force_solid_on() .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .no_collision() .instabreak());
    pub const DEAD_BUBBLE_CORAL_FAN: Block = register_with(registry, BlockItemIds::DEAD_BUBBLE_CORAL_FAN, BaseCoralFanBlock::new, Properties::of() .map_color(MapColor::COLOR_GRAY) .force_solid_on() .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .no_collision() .instabreak());
    pub const DEAD_FIRE_CORAL_FAN: Block = register_with(registry, BlockItemIds::DEAD_FIRE_CORAL_FAN, BaseCoralFanBlock::new, Properties::of() .map_color(MapColor::COLOR_GRAY) .force_solid_on() .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .no_collision() .instabreak());
    pub const DEAD_HORN_CORAL_FAN: Block = register_with(registry, BlockItemIds::DEAD_HORN_CORAL_FAN, BaseCoralFanBlock::new, Properties::of() .map_color(MapColor::COLOR_GRAY) .force_solid_on() .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .no_collision() .instabreak());
    pub const TUBE_CORAL_FAN: Block = register_with(registry, BlockItemIds::TUBE_CORAL_FAN, CoralFanBlock::new, Properties::of().map_color(MapColor::COLOR_BLUE).no_collision().instabreak().push_reaction(PushReaction::POPPED));
    pub const BRAIN_CORAL_FAN: Block = register_with(registry, BlockItemIds::BRAIN_CORAL_FAN, CoralFanBlock::new, Properties::of().map_color(MapColor::COLOR_PINK).no_collision().instabreak().push_reaction(PushReaction::POPPED));
    pub const BUBBLE_CORAL_FAN: Block = register_with(registry, BlockItemIds::BUBBLE_CORAL_FAN, CoralFanBlock::new, Properties::of().map_color(MapColor::COLOR_PURPLE).no_collision().instabreak().push_reaction(PushReaction::POPPED));
    pub const FIRE_CORAL_FAN: Block = register_with(registry, BlockItemIds::FIRE_CORAL_FAN, CoralFanBlock::new, Properties::of().map_color(MapColor::COLOR_RED).no_collision().instabreak().push_reaction(PushReaction::POPPED));
    pub const HORN_CORAL_FAN: Block = register_with(registry, BlockItemIds::HORN_CORAL_FAN, CoralFanBlock::new, Properties::of().map_color(MapColor::COLOR_YELLOW).no_collision().instabreak().push_reaction(PushReaction::POPPED));
    pub const DEAD_TUBE_CORAL_WALL_FAN: Block = register_with(registry, BlockIds::DEAD_TUBE_CORAL_WALL_FAN, BaseCoralWallFanBlock::new, wall_variant(registry, Blocks::DEAD_TUBE_CORAL_FAN, false) .map_color(MapColor::COLOR_GRAY) .force_solid_on() .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .no_collision() .instabreak());
    pub const DEAD_BRAIN_CORAL_WALL_FAN: Block = register_with(registry, BlockIds::DEAD_BRAIN_CORAL_WALL_FAN, BaseCoralWallFanBlock::new, wall_variant(registry, Blocks::DEAD_BRAIN_CORAL_FAN, false) .map_color(MapColor::COLOR_GRAY) .force_solid_on() .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .no_collision() .instabreak());
    pub const DEAD_BUBBLE_CORAL_WALL_FAN: Block = register_with(registry, BlockIds::DEAD_BUBBLE_CORAL_WALL_FAN, BaseCoralWallFanBlock::new, wall_variant(registry, Blocks::DEAD_BUBBLE_CORAL_FAN, false) .map_color(MapColor::COLOR_GRAY) .force_solid_on() .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .no_collision() .instabreak());
    pub const DEAD_FIRE_CORAL_WALL_FAN: Block = register_with(registry, BlockIds::DEAD_FIRE_CORAL_WALL_FAN, BaseCoralWallFanBlock::new, wall_variant(registry, Blocks::DEAD_FIRE_CORAL_FAN, false) .map_color(MapColor::COLOR_GRAY) .force_solid_on() .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .no_collision() .instabreak());
    pub const DEAD_HORN_CORAL_WALL_FAN: Block = register_with(registry, BlockIds::DEAD_HORN_CORAL_WALL_FAN, BaseCoralWallFanBlock::new, wall_variant(registry, Blocks::DEAD_HORN_CORAL_FAN, false) .map_color(MapColor::COLOR_GRAY) .force_solid_on() .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .no_collision() .instabreak());
    pub const TUBE_CORAL_WALL_FAN: Block = register_with(registry, BlockIds::TUBE_CORAL_WALL_FAN, CoralWallFanBlock::new, wall_variant(registry, Blocks::TUBE_CORAL_FAN, false) .map_color(MapColor::COLOR_BLUE) .no_collision() .instabreak()  .push_reaction(PushReaction::POPPED));
    pub const BRAIN_CORAL_WALL_FAN: Block = register_with(registry, BlockIds::BRAIN_CORAL_WALL_FAN, CoralWallFanBlock::new, wall_variant(registry, Blocks::BRAIN_CORAL_FAN, false) .map_color(MapColor::COLOR_PINK) .no_collision() .instabreak()  .push_reaction(PushReaction::POPPED));
    pub const BUBBLE_CORAL_WALL_FAN: Block = register_with(registry, BlockIds::BUBBLE_CORAL_WALL_FAN, CoralWallFanBlock::new, wall_variant(registry, Blocks::BUBBLE_CORAL_FAN, false) .map_color(MapColor::COLOR_PURPLE) .no_collision() .instabreak()  .push_reaction(PushReaction::POPPED));
    pub const FIRE_CORAL_WALL_FAN: Block = register_with(registry, BlockIds::FIRE_CORAL_WALL_FAN, CoralWallFanBlock::new, wall_variant(registry, Blocks::FIRE_CORAL_FAN, false).map_color(MapColor::COLOR_RED).no_collision().instabreak().push_reaction(PushReaction::POPPED));
    pub const HORN_CORAL_WALL_FAN: Block = register_with(registry, BlockIds::HORN_CORAL_WALL_FAN, CoralWallFanBlock::new, wall_variant(registry, Blocks::HORN_CORAL_FAN, false) .map_color(MapColor::COLOR_YELLOW) .no_collision() .instabreak()  .push_reaction(PushReaction::POPPED));
    pub const SEA_PICKLE: Block = register_with(registry, BlockItemIds::SEA_PICKLE, SeaPickleBlock::new, Properties::of().map_color(MapColor::COLOR_GREEN).light_level(|state| if SeaPickleBlock::is_dead(state) { 0 } else { 3 + 3 * state.get_value(SeaPickleBlock::PICKLES) }).no_occlusion().push_reaction(PushReaction::POPPED));
    pub const BLUE_ICE: Block = register_with(registry, BlockItemIds::BLUE_ICE, HalfTransparentBlock::new, Properties::of().map_color(MapColor::ICE).strength(2.8).friction(0.989));
    pub const CONDUIT: Block = register_with(registry, BlockItemIds::CONDUIT, ConduitBlock::new, Properties::of() .map_color(MapColor::DIAMOND) .force_solid_on() .instrument(NoteBlockInstrument::HAT) .strength(3.0) .light_level(|_| 15) .no_occlusion());
    pub const BAMBOO_SAPLING: Block = register_with(registry, BlockIds::BAMBOO_SAPLING, BambooSaplingBlock::new, Properties::of() .map_color(MapColor::WOOD) .force_solid_on() .random_ticks() .instabreak() .no_collision() .strength(1.0)  .offset_type(OffsetType::XZ) .ignited_by_lava() .push_reaction(PushReaction::POPPED));
    pub const BAMBOO: Block = register_with(registry, BlockItemIds::BAMBOO, BambooStalkBlock::new, Properties::of() .map_color(MapColor::PLANT) .force_solid_on() .random_ticks() .instabreak() .strength(1.0)  .no_occlusion() .dynamic_shape() .offset_type(OffsetType::XZ) .ignited_by_lava() .push_reaction(PushReaction::POPPED) .is_redstone_conductor(StatePredicate::State(None, never)));
    pub const POTTED_BAMBOO: Block = register_with(registry, BlockIds::POTTED_BAMBOO, FlowerPotBlock::new, flower_pot_properties());
    pub const VOID_AIR: Block = register_with(registry, BlockIds::VOID_AIR, AirBlock::new, Properties::of().replaceable().no_collision().no_loot_table().air());
    pub const CAVE_AIR: Block = register_with(registry, BlockIds::CAVE_AIR, AirBlock::new, Properties::of().replaceable().no_collision().no_loot_table().air());
    pub const BUBBLE_COLUMN: Block = register_with(registry, BlockIds::BUBBLE_COLUMN, BubbleColumnBlock::new, Properties::of() .map_color(MapColor::WATER) .replaceable() .no_collision() .no_loot_table() .push_reaction(PushReaction::POPPED) .liquid() );
    pub const POLISHED_GRANITE_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::POLISHED_GRANITE_STAIRS, Blocks::POLISHED_GRANITE);
    pub const SMOOTH_RED_SANDSTONE_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::SMOOTH_RED_SANDSTONE_STAIRS, Blocks::SMOOTH_RED_SANDSTONE);
    pub const MOSSY_STONE_BRICK_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::MOSSY_STONE_BRICK_STAIRS, Blocks::MOSSY_STONE_BRICKS);
    pub const POLISHED_DIORITE_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::POLISHED_DIORITE_STAIRS, Blocks::POLISHED_DIORITE);
    pub const MOSSY_COBBLESTONE_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::MOSSY_COBBLESTONE_STAIRS, Blocks::MOSSY_COBBLESTONE);
    pub const END_STONE_BRICK_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::END_STONE_BRICK_STAIRS, Blocks::END_STONE_BRICKS);
    pub const STONE_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::STONE_STAIRS, Blocks::STONE);
    pub const SMOOTH_SANDSTONE_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::SMOOTH_SANDSTONE_STAIRS, Blocks::SMOOTH_SANDSTONE);
    pub const SMOOTH_QUARTZ_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::SMOOTH_QUARTZ_STAIRS, Blocks::SMOOTH_QUARTZ);
    pub const GRANITE_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::GRANITE_STAIRS, Blocks::GRANITE);
    pub const ANDESITE_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::ANDESITE_STAIRS, Blocks::ANDESITE);
    pub const RED_NETHER_BRICK_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::RED_NETHER_BRICK_STAIRS, Blocks::RED_NETHER_BRICKS);
    pub const POLISHED_ANDESITE_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::POLISHED_ANDESITE_STAIRS, Blocks::POLISHED_ANDESITE);
    pub const DIORITE_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::DIORITE_STAIRS, Blocks::DIORITE);
    pub const POLISHED_GRANITE_SLAB: Block = register_slab(registry, BlockItemIds::POLISHED_GRANITE_SLAB, Blocks::POLISHED_GRANITE);
    pub const SMOOTH_RED_SANDSTONE_SLAB: Block = register_slab(registry, BlockItemIds::SMOOTH_RED_SANDSTONE_SLAB, Blocks::SMOOTH_RED_SANDSTONE);
    pub const MOSSY_STONE_BRICK_SLAB: Block = register_slab(registry, BlockItemIds::MOSSY_STONE_BRICK_SLAB, Blocks::MOSSY_STONE_BRICKS);
    pub const POLISHED_DIORITE_SLAB: Block = register_slab(registry, BlockItemIds::POLISHED_DIORITE_SLAB, Blocks::POLISHED_DIORITE);
    pub const MOSSY_COBBLESTONE_SLAB: Block = register_slab(registry, BlockItemIds::MOSSY_COBBLESTONE_SLAB, Blocks::MOSSY_COBBLESTONE);
    pub const END_STONE_BRICK_SLAB: Block = register_slab(registry, BlockItemIds::END_STONE_BRICK_SLAB, Blocks::END_STONE_BRICKS);
    pub const SMOOTH_SANDSTONE_SLAB: Block = register_slab(registry, BlockItemIds::SMOOTH_SANDSTONE_SLAB, Blocks::SMOOTH_SANDSTONE);
    pub const SMOOTH_QUARTZ_SLAB: Block = register_slab(registry, BlockItemIds::SMOOTH_QUARTZ_SLAB, Blocks::SMOOTH_QUARTZ);
    pub const GRANITE_SLAB: Block = register_slab(registry, BlockItemIds::GRANITE_SLAB, Blocks::GRANITE);
    pub const ANDESITE_SLAB: Block = register_slab(registry, BlockItemIds::ANDESITE_SLAB, Blocks::ANDESITE);
    pub const RED_NETHER_BRICK_SLAB: Block = register_slab(registry, BlockItemIds::RED_NETHER_BRICK_SLAB, Blocks::RED_NETHER_BRICKS);
    pub const POLISHED_ANDESITE_SLAB: Block = register_slab(registry, BlockItemIds::POLISHED_ANDESITE_SLAB, Blocks::POLISHED_ANDESITE);
    pub const DIORITE_SLAB: Block = register_slab(registry, BlockItemIds::DIORITE_SLAB, Blocks::DIORITE);
    pub const BRICK_WALL: Block = register_wall(registry, BlockItemIds::BRICK_WALL, Blocks::BRICKS);
    pub const PRISMARINE_WALL: Block = register_wall(registry, BlockItemIds::PRISMARINE_WALL, Blocks::PRISMARINE);
    pub const RED_SANDSTONE_WALL: Block = register_wall(registry, BlockItemIds::RED_SANDSTONE_WALL, Blocks::RED_SANDSTONE);
    pub const MOSSY_STONE_BRICK_WALL: Block = register_wall(registry, BlockItemIds::MOSSY_STONE_BRICK_WALL, Blocks::MOSSY_STONE_BRICKS);
    pub const GRANITE_WALL: Block = register_wall(registry, BlockItemIds::GRANITE_WALL, Blocks::GRANITE);
    pub const STONE_BRICK_WALL: Block = register_wall(registry, BlockItemIds::STONE_BRICK_WALL, Blocks::STONE_BRICKS);
    pub const MUD_BRICK_WALL: Block = register_wall(registry, BlockItemIds::MUD_BRICK_WALL, Blocks::MUD_BRICKS);
    pub const NETHER_BRICK_WALL: Block = register_wall(registry, BlockItemIds::NETHER_BRICK_WALL, Blocks::NETHER_BRICKS);
    pub const ANDESITE_WALL: Block = register_wall(registry, BlockItemIds::ANDESITE_WALL, Blocks::ANDESITE);
    pub const RED_NETHER_BRICK_WALL: Block = register_wall(registry, BlockItemIds::RED_NETHER_BRICK_WALL, Blocks::RED_NETHER_BRICKS);
    pub const SANDSTONE_WALL: Block = register_wall(registry, BlockItemIds::SANDSTONE_WALL, Blocks::SANDSTONE);
    pub const END_STONE_BRICK_WALL: Block = register_wall(registry, BlockItemIds::END_STONE_BRICK_WALL, Blocks::END_STONE_BRICKS);
    pub const DIORITE_WALL: Block = register_wall(registry, BlockItemIds::DIORITE_WALL, Blocks::DIORITE);
    pub const SCAFFOLDING: Block = register_with(registry, BlockItemIds::SCAFFOLDING, ScaffoldingBlock::new, Properties::of() .map_color(MapColor::SAND) .no_collision()  .dynamic_shape()  .push_reaction(PushReaction::POPPED) .is_redstone_conductor(StatePredicate::State(None, never)));
    pub const LOOM: Block = register_with(registry, BlockItemIds::LOOM, LoomBlock::new, Properties::of().map_color(MapColor::WOOD).instrument(NoteBlockInstrument::BASS).strength(2.5).ignited_by_lava());
    pub const BARREL: Block = register_with(registry, BlockItemIds::BARREL, BarrelBlock::new, Properties::of().map_color(MapColor::WOOD).instrument(NoteBlockInstrument::BASS).strength(2.5).ignited_by_lava());
    pub const SMOKER: Block = register_with(registry, BlockItemIds::SMOKER, SmokerBlock::new, Properties::of() .map_color(MapColor::STONE) .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .strength(3.5) .light_level(lit_block_emission(13)));
    pub const BLAST_FURNACE: Block = register_with(registry, BlockItemIds::BLAST_FURNACE, BlastFurnaceBlock::new, Properties::of() .map_color(MapColor::STONE) .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .strength(3.5) .light_level(lit_block_emission(13)));
    pub const CARTOGRAPHY_TABLE: Block = register_with(registry, BlockItemIds::CARTOGRAPHY_TABLE, CartographyTableBlock::new, Properties::of().map_color(MapColor::WOOD).instrument(NoteBlockInstrument::BASS).strength(2.5).ignited_by_lava());
    pub const FLETCHING_TABLE: Block = register(registry, BlockItemIds::FLETCHING_TABLE, Properties::of().map_color(MapColor::WOOD).instrument(NoteBlockInstrument::BASS).strength(2.5).ignited_by_lava());
    pub const GRINDSTONE: Block = register_with(registry, BlockItemIds::GRINDSTONE, GrindstoneBlock::new, Properties::of() .map_color(MapColor::METAL) .requires_correct_tool_for_drops() .strength_and_resistance(2.0, 6.0)  .push_reaction(PushReaction::IMMOVEABLE));
    pub const LECTERN: Block = register_with(registry, BlockItemIds::LECTERN, LecternBlock::new, Properties::of().map_color(MapColor::WOOD).instrument(NoteBlockInstrument::BASS).strength(2.5).ignited_by_lava());
    pub const SMITHING_TABLE: Block = register_with(registry, BlockItemIds::SMITHING_TABLE, SmithingTableBlock::new, Properties::of().map_color(MapColor::WOOD).instrument(NoteBlockInstrument::BASS).strength(2.5).ignited_by_lava());
    pub const STONECUTTER: Block = register_with(registry, BlockItemIds::STONECUTTER, StonecutterBlock::new, Properties::of().map_color(MapColor::STONE).instrument(NoteBlockInstrument::BASEDRUM).requires_correct_tool_for_drops().strength(3.5));
    pub const BELL: Block = register_with(registry, BlockItemIds::BELL, BellBlock::new, Properties::of().map_color(MapColor::GOLD).force_solid_on().strength(5.0).push_reaction(PushReaction::POPPED));
    pub const LANTERN: Block = register_with(registry, BlockItemIds::LANTERN, LanternBlock::new, Properties::of() .map_color(MapColor::METAL) .force_solid_on() .strength(3.5)  .light_level(|_| 15) .no_occlusion() .push_reaction(PushReaction::POPPED));
    pub const SOUL_LANTERN: Block = register_with(registry, BlockItemIds::SOUL_LANTERN, LanternBlock::new, Properties::of() .map_color(MapColor::METAL) .force_solid_on() .strength(3.5)  .light_level(|_| 10) .no_occlusion() .push_reaction(PushReaction::POPPED));
    pub const COPPER_LANTERN: WeatheringCopperCollection<Block> = WeatheringCopperCollection::register_blocks(registry, BlockItemIds::COPPER_LANTERN, |_, p| LanternBlock::new(p), |_, p| WeatheringLanternBlock::new(p), |_| Properties::of().map_color(MapColor::METAL).force_solid_on().strength(3.5).light_level(|_| 15).no_occlusion().push_reaction(PushReaction::POPPED));
    pub const CAMPFIRE: Block = register_with(registry, BlockItemIds::CAMPFIRE, CampfireBlock::new, Properties::of() .map_color(MapColor::PODZOL) .instrument(NoteBlockInstrument::BASS) .strength(2.0)  .light_level(lit_block_emission(15)) .no_occlusion() .ignited_by_lava());
    pub const SOUL_CAMPFIRE: Block = register_with(registry, BlockItemIds::SOUL_CAMPFIRE, CampfireBlock::new, Properties::of() .map_color(MapColor::PODZOL) .instrument(NoteBlockInstrument::BASS) .strength(2.0)  .light_level(lit_block_emission(10)) .no_occlusion() .ignited_by_lava());
    pub const SWEET_BERRY_BUSH: Block = register_with(registry, BlockItemIds::SWEET_BERRY_CROP, SweetBerryBushBlock::new, Properties::of().map_color(MapColor::PLANT).random_ticks().no_collision().push_reaction(PushReaction::POPPED));
    pub const WARPED_STEM: Block = register_with(registry, BlockItemIds::WARPED_STEM, RotatedPillarBlock::new, nether_stem_properties(MapColor::WARPED_STEM));
    pub const STRIPPED_WARPED_STEM: Block = register_with(registry, BlockItemIds::STRIPPED_WARPED_STEM, RotatedPillarBlock::new, nether_stem_properties(MapColor::WARPED_STEM));
    pub const WARPED_HYPHAE: Block = register_with(registry, BlockItemIds::WARPED_HYPHAE, RotatedPillarBlock::new, Properties::of().map_color(MapColor::WARPED_HYPHAE).instrument(NoteBlockInstrument::BASS).strength(2.0));
    pub const STRIPPED_WARPED_HYPHAE: Block = register_with(registry, BlockItemIds::STRIPPED_WARPED_HYPHAE, RotatedPillarBlock::new, Properties::of().map_color(MapColor::WARPED_HYPHAE).instrument(NoteBlockInstrument::BASS).strength(2.0));
    pub const WARPED_NYLIUM: Block = register_with(registry, BlockItemIds::WARPED_NYLIUM, NyliumBlock::new, Properties::of() .map_color(MapColor::WARPED_NYLIUM) .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .strength(0.4)  .random_ticks());
    pub const WARPED_FUNGUS: Block = register_with(registry, BlockItemIds::WARPED_FUNGUS, NetherFungusBlock::new, Properties::of().map_color(MapColor::COLOR_CYAN).instabreak().no_collision().push_reaction(PushReaction::POPPED));
    pub const WARPED_WART_BLOCK: Block = register(registry, BlockItemIds::WARPED_WART_BLOCK, Properties::of().map_color(MapColor::WARPED_WART_BLOCK).strength(1.0));
    pub const WARPED_ROOTS: Block = register_with(registry, BlockItemIds::WARPED_ROOTS, NetherRootsBlock::new, Properties::of() .map_color(MapColor::COLOR_CYAN) .replaceable() .no_collision() .instabreak()  .offset_type(OffsetType::XZ) .push_reaction(PushReaction::POPPED));
    pub const NETHER_SPROUTS: Block = register_with(registry, BlockItemIds::NETHER_SPROUTS, NetherSproutsBlock::new, Properties::of() .map_color(MapColor::COLOR_CYAN) .replaceable() .no_collision() .instabreak()  .offset_type(OffsetType::XZ) .push_reaction(PushReaction::POPPED));
    pub const CRIMSON_STEM: Block = register_with(registry, BlockItemIds::CRIMSON_STEM, RotatedPillarBlock::new, nether_stem_properties(MapColor::CRIMSON_STEM));
    pub const STRIPPED_CRIMSON_STEM: Block = register_with(registry, BlockItemIds::STRIPPED_CRIMSON_STEM, RotatedPillarBlock::new, nether_stem_properties(MapColor::CRIMSON_STEM));
    pub const CRIMSON_HYPHAE: Block = register_with(registry, BlockItemIds::CRIMSON_HYPHAE, RotatedPillarBlock::new, Properties::of().map_color(MapColor::CRIMSON_HYPHAE).instrument(NoteBlockInstrument::BASS).strength(2.0));
    pub const STRIPPED_CRIMSON_HYPHAE: Block = register_with(registry, BlockItemIds::STRIPPED_CRIMSON_HYPHAE, RotatedPillarBlock::new, Properties::of().map_color(MapColor::CRIMSON_HYPHAE).instrument(NoteBlockInstrument::BASS).strength(2.0));
    pub const CRIMSON_NYLIUM: Block = register_with(registry, BlockItemIds::CRIMSON_NYLIUM, NyliumBlock::new, Properties::of() .map_color(MapColor::CRIMSON_NYLIUM) .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .strength(0.4)  .random_ticks());
    pub const CRIMSON_FUNGUS: Block = register_with(registry, BlockItemIds::CRIMSON_FUNGUS, NetherFungusBlock::new, Properties::of().map_color(MapColor::NETHER).instabreak().no_collision().push_reaction(PushReaction::POPPED));
    pub const SHROOMLIGHT: Block = register(registry, BlockItemIds::SHROOMLIGHT, Properties::of().map_color(MapColor::COLOR_RED).strength(1.0).light_level(|_| 15));
    pub const WEEPING_VINES: Block = register_with(registry, BlockItemIds::WEEPING_VINES, WeepingVinesBlock::new, Properties::of() .map_color(MapColor::NETHER) .random_ticks() .no_collision() .instabreak()  .push_reaction(PushReaction::POPPED));
    pub const WEEPING_VINES_PLANT: Block = register_with(registry, BlockIds::WEEPING_VINES_PLANT, WeepingVinesPlantBlock::new, Properties::of().map_color(MapColor::NETHER).no_collision().instabreak().push_reaction(PushReaction::POPPED));
    pub const TWISTING_VINES: Block = register_with(registry, BlockItemIds::TWISTING_VINES, TwistingVinesBlock::new, Properties::of() .map_color(MapColor::COLOR_CYAN) .random_ticks() .no_collision() .instabreak()  .push_reaction(PushReaction::POPPED));
    pub const TWISTING_VINES_PLANT: Block = register_with(registry, BlockIds::TWISTING_VINES_PLANT, TwistingVinesPlantBlock::new, Properties::of() .map_color(MapColor::COLOR_CYAN) .no_collision() .instabreak()  .push_reaction(PushReaction::POPPED));
    pub const CRIMSON_ROOTS: Block = register_with(registry, BlockItemIds::CRIMSON_ROOTS, NetherRootsBlock::new, Properties::of() .map_color(MapColor::NETHER) .replaceable() .no_collision() .instabreak()  .offset_type(OffsetType::XZ) .push_reaction(PushReaction::POPPED));
    pub const CRIMSON_PLANKS: Block = register(registry, BlockItemIds::CRIMSON_PLANKS, Properties::of().map_color(MapColor::CRIMSON_STEM).instrument(NoteBlockInstrument::BASS).strength_and_resistance(2.0, 3.0));
    pub const WARPED_PLANKS: Block = register(registry, BlockItemIds::WARPED_PLANKS, Properties::of().map_color(MapColor::WARPED_STEM).instrument(NoteBlockInstrument::BASS).strength_and_resistance(2.0, 3.0));
    pub const CRIMSON_SLAB: Block = register_slab(registry, BlockItemIds::CRIMSON_SLAB, Blocks::CRIMSON_PLANKS);
    pub const WARPED_SLAB: Block = register_slab(registry, BlockItemIds::WARPED_SLAB, Blocks::WARPED_PLANKS);
    pub const CRIMSON_PRESSURE_PLATE: Block = register_with(registry, BlockItemIds::CRIMSON_PRESSURE_PLATE, PressurePlateBlock::new, Properties::of() .map_color(registry.get(Blocks::CRIMSON_PLANKS).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(0.5) .push_reaction(PushReaction::POPPED));
    pub const WARPED_PRESSURE_PLATE: Block = register_with(registry, BlockItemIds::WARPED_PRESSURE_PLATE, PressurePlateBlock::new, Properties::of() .map_color(registry.get(Blocks::WARPED_PLANKS).default_map_color()) .force_solid_on() .instrument(NoteBlockInstrument::BASS) .no_collision() .strength(0.5) .push_reaction(PushReaction::POPPED));
    pub const CRIMSON_FENCE: Block = register_with(registry, BlockItemIds::CRIMSON_FENCE, FenceBlock::new, Properties::of() .map_color(registry.get(Blocks::CRIMSON_PLANKS).default_map_color()) .instrument(NoteBlockInstrument::BASS) .strength_and_resistance(2.0, 3.0) );
    pub const WARPED_FENCE: Block = register_with(registry, BlockItemIds::WARPED_FENCE, FenceBlock::new, Properties::of() .map_color(registry.get(Blocks::WARPED_PLANKS).default_map_color()) .instrument(NoteBlockInstrument::BASS) .strength_and_resistance(2.0, 3.0) );
    pub const CRIMSON_TRAPDOOR: Block = register_with(registry, BlockItemIds::CRIMSON_TRAPDOOR, TrapDoorBlock::new, Properties::of() .map_color(registry.get(Blocks::CRIMSON_PLANKS).default_map_color()) .instrument(NoteBlockInstrument::BASS) .strength(3.0) .no_occlusion() );
    pub const WARPED_TRAPDOOR: Block = register_with(registry, BlockItemIds::WARPED_TRAPDOOR, TrapDoorBlock::new, Properties::of() .map_color(registry.get(Blocks::WARPED_PLANKS).default_map_color()) .instrument(NoteBlockInstrument::BASS) .strength(3.0) .no_occlusion() );
    pub const CRIMSON_FENCE_GATE: Block = register_with(registry, BlockItemIds::CRIMSON_FENCE_GATE, FenceGateBlock::new, Properties::of().map_color(registry.get(Blocks::CRIMSON_PLANKS).default_map_color()).force_solid_on().instrument(NoteBlockInstrument::BASS).strength_and_resistance(2.0, 3.0));
    pub const WARPED_FENCE_GATE: Block = register_with(registry, BlockItemIds::WARPED_FENCE_GATE, FenceGateBlock::new, Properties::of().map_color(registry.get(Blocks::WARPED_PLANKS).default_map_color()).force_solid_on().instrument(NoteBlockInstrument::BASS).strength_and_resistance(2.0, 3.0));
    pub const CRIMSON_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::CRIMSON_STAIRS, Blocks::CRIMSON_PLANKS);
    pub const WARPED_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::WARPED_STAIRS, Blocks::WARPED_PLANKS);
    pub const CRIMSON_BUTTON: Block = register_with(registry, BlockItemIds::CRIMSON_BUTTON, ButtonBlock::new, button_properties());
    pub const WARPED_BUTTON: Block = register_with(registry, BlockItemIds::WARPED_BUTTON, ButtonBlock::new, button_properties());
    pub const CRIMSON_DOOR: Block = register_with(registry, BlockItemIds::CRIMSON_DOOR, DoorBlock::new, Properties::of() .map_color(registry.get(Blocks::CRIMSON_PLANKS).default_map_color()) .instrument(NoteBlockInstrument::BASS) .strength(3.0) .no_occlusion() .push_reaction(PushReaction::POPPED));
    pub const WARPED_DOOR: Block = register_with(registry, BlockItemIds::WARPED_DOOR, DoorBlock::new, Properties::of() .map_color(registry.get(Blocks::WARPED_PLANKS).default_map_color()) .instrument(NoteBlockInstrument::BASS) .strength(3.0) .no_occlusion() .push_reaction(PushReaction::POPPED));
    pub const CRIMSON_SIGN: Block = register_with(registry, BlockItemIds::CRIMSON_SIGN, StandingSignBlock::new, Properties::of() .map_color(registry.get(Blocks::CRIMSON_PLANKS).default_map_color()) .instrument(NoteBlockInstrument::BASS) .force_solid_on() .no_collision() .strength(1.0));
    pub const WARPED_SIGN: Block = register_with(registry, BlockItemIds::WARPED_SIGN, StandingSignBlock::new, Properties::of() .map_color(registry.get(Blocks::WARPED_PLANKS).default_map_color()) .instrument(NoteBlockInstrument::BASS) .force_solid_on() .no_collision() .strength(1.0));
    pub const CRIMSON_WALL_SIGN: Block = register_with(registry, BlockIds::CRIMSON_WALL_SIGN, WallSignBlock::new, wall_variant(registry, Blocks::CRIMSON_SIGN, true) .map_color(registry.get(Blocks::CRIMSON_PLANKS).default_map_color()) .instrument(NoteBlockInstrument::BASS) .force_solid_on() .no_collision() .strength(1.0));
    pub const WARPED_WALL_SIGN: Block = register_with(registry, BlockIds::WARPED_WALL_SIGN, WallSignBlock::new, wall_variant(registry, Blocks::WARPED_SIGN, true) .map_color(registry.get(Blocks::WARPED_PLANKS).default_map_color()) .instrument(NoteBlockInstrument::BASS) .force_solid_on() .no_collision() .strength(1.0));
    pub const STRUCTURE_BLOCK: Block = register_with(registry, BlockItemIds::STRUCTURE_BLOCK, StructureBlock::new, Properties::of().map_color(MapColor::COLOR_LIGHT_GRAY).requires_correct_tool_for_drops().strength_and_resistance(-1.0, 3600000.0).no_loot_table());
    pub const JIGSAW: Block = register_with(registry, BlockItemIds::JIGSAW, JigsawBlock::new, Properties::of().map_color(MapColor::COLOR_LIGHT_GRAY).requires_correct_tool_for_drops().strength_and_resistance(-1.0, 3600000.0).no_loot_table());
    pub const TEST_BLOCK: Block = register_with(registry, BlockItemIds::TEST_BLOCK, TestBlock::new, Properties::of().map_color(MapColor::COLOR_LIGHT_GRAY).strength_and_resistance(-1.0, 3600000.0).no_loot_table());
    pub const TEST_INSTANCE_BLOCK: Block = register_with(registry, BlockItemIds::TEST_INSTANCE_BLOCK, TestInstanceBlock::new, Properties::of().no_occlusion().strength_and_resistance(-1.0, 3600000.0).no_loot_table());
    pub const COMPOSTER: Block = register_with(registry, BlockItemIds::COMPOSTER, ComposterBlock::new, Properties::of().map_color(MapColor::WOOD).instrument(NoteBlockInstrument::BASS).strength(0.6).ignited_by_lava());
    pub const TARGET: Block = register_with(registry, BlockItemIds::TARGET, TargetBlock::new, Properties::of().map_color(MapColor::QUARTZ).strength(0.5));
    pub const BEE_NEST: Block = register_with(registry, BlockItemIds::BEE_NEST, BeehiveBlock::new, Properties::of() .map_color(MapColor::COLOR_YELLOW) .instrument(NoteBlockInstrument::BASS) .strength(0.3)  .ignited_by_lava());
    pub const BEEHIVE: Block = register_with(registry, BlockItemIds::BEEHIVE, BeehiveBlock::new, Properties::of().map_color(MapColor::WOOD).instrument(NoteBlockInstrument::BASS).strength(0.6).ignited_by_lava());
    pub const HONEY_BLOCK: Block = register_with(registry, BlockItemIds::HONEY_BLOCK, HoneyBlock::new, Properties::of().map_color(MapColor::COLOR_ORANGE).speed_factor(0.4).jump_factor(0.5).no_occlusion());
    pub const HONEYCOMB_BLOCK: Block = register(registry, BlockItemIds::HONEYCOMB_BLOCK, Properties::of().map_color(MapColor::COLOR_ORANGE).strength(0.6));
    pub const NETHERITE_BLOCK: Block = register(registry, BlockItemIds::NETHERITE_BLOCK, Properties::of().map_color(MapColor::COLOR_BLACK).requires_correct_tool_for_drops().strength_and_resistance(50.0, 1200.0));
    pub const ANCIENT_DEBRIS: Block = register(registry, BlockItemIds::ANCIENT_DEBRIS, Properties::of().map_color(MapColor::COLOR_BLACK).requires_correct_tool_for_drops().strength_and_resistance(30.0, 1200.0));
    pub const CRYING_OBSIDIAN: Block = register_with(registry, BlockItemIds::CRYING_OBSIDIAN, CryingObsidianBlock::new, Properties::of() .map_color(MapColor::COLOR_BLACK) .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .strength_and_resistance(50.0, 1200.0) .light_level(|_| 10) .push_reaction(PushReaction::IMMOVEABLE));
    pub const RESPAWN_ANCHOR: Block = register_with(registry, BlockItemIds::RESPAWN_ANCHOR, RespawnAnchorBlock::new, Properties::of().map_color(MapColor::COLOR_BLACK).instrument(NoteBlockInstrument::BASEDRUM).requires_correct_tool_for_drops().strength_and_resistance(50.0, 1200.0).light_level(|state| RespawnAnchorBlock::get_scaled_charge_level(state, 15)).push_reaction(PushReaction::IMMOVEABLE));
    pub const POTTED_CRIMSON_FUNGUS: Block = register_with(registry, BlockIds::POTTED_CRIMSON_FUNGUS, FlowerPotBlock::new, flower_pot_properties());
    pub const POTTED_WARPED_FUNGUS: Block = register_with(registry, BlockIds::POTTED_WARPED_FUNGUS, FlowerPotBlock::new, flower_pot_properties());
    pub const POTTED_CRIMSON_ROOTS: Block = register_with(registry, BlockIds::POTTED_CRIMSON_ROOTS, FlowerPotBlock::new, flower_pot_properties());
    pub const POTTED_WARPED_ROOTS: Block = register_with(registry, BlockIds::POTTED_WARPED_ROOTS, FlowerPotBlock::new, flower_pot_properties());
    pub const LODESTONE: Block = register(registry, BlockItemIds::LODESTONE, Properties::of() .map_color(MapColor::METAL) .requires_correct_tool_for_drops() .strength(3.5)  .push_reaction(PushReaction::IMMOVEABLE));
    pub const BLACKSTONE: Block = register(registry, BlockItemIds::BLACKSTONE, Properties::of() .map_color(MapColor::COLOR_BLACK) .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .strength_and_resistance(1.5, 6.0));
    pub const BLACKSTONE_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::BLACKSTONE_STAIRS, Blocks::BLACKSTONE);
    pub const BLACKSTONE_WALL: Block = register_wall(registry, BlockItemIds::BLACKSTONE_WALL, Blocks::BLACKSTONE);
    pub const BLACKSTONE_SLAB: Block = register_slab_destroy_time(registry, BlockItemIds::BLACKSTONE_SLAB, Blocks::BLACKSTONE, 2.0);
    pub const POLISHED_BLACKSTONE: Block = register(registry, BlockItemIds::POLISHED_BLACKSTONE, Properties::of_legacy_copy(&registry.get(Blocks::BLACKSTONE)).strength_and_resistance(2.0, 6.0));
    pub const POLISHED_BLACKSTONE_BRICKS: Block = register(registry, BlockItemIds::POLISHED_BLACKSTONE_BRICKS, Properties::of_legacy_copy(&registry.get(Blocks::POLISHED_BLACKSTONE)).strength_and_resistance(1.5, 6.0));
    pub const CRACKED_POLISHED_BLACKSTONE_BRICKS: Block = register(registry, BlockItemIds::CRACKED_POLISHED_BLACKSTONE_BRICKS, Properties::of_legacy_copy(&registry.get(Blocks::POLISHED_BLACKSTONE_BRICKS)));
    pub const CHISELED_POLISHED_BLACKSTONE: Block = register(registry, BlockItemIds::CHISELED_POLISHED_BLACKSTONE, Properties::of_legacy_copy(&registry.get(Blocks::POLISHED_BLACKSTONE)).strength_and_resistance(1.5, 6.0));
    pub const POLISHED_BLACKSTONE_BRICK_SLAB: Block = register_slab_destroy_time(registry, BlockItemIds::POLISHED_BLACKSTONE_BRICK_SLAB, Blocks::POLISHED_BLACKSTONE_BRICKS, 2.0);
    pub const POLISHED_BLACKSTONE_BRICK_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::POLISHED_BLACKSTONE_BRICK_STAIRS, Blocks::POLISHED_BLACKSTONE_BRICKS);
    pub const POLISHED_BLACKSTONE_BRICK_WALL: Block = register_wall(registry, BlockItemIds::POLISHED_BLACKSTONE_BRICK_WALL, Blocks::POLISHED_BLACKSTONE_BRICKS);
    pub const GILDED_BLACKSTONE: Block = register(registry, BlockItemIds::GILDED_BLACKSTONE, Properties::of_legacy_copy(&registry.get(Blocks::BLACKSTONE)));
    pub const POLISHED_BLACKSTONE_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::POLISHED_BLACKSTONE_STAIRS, Blocks::POLISHED_BLACKSTONE);
    pub const POLISHED_BLACKSTONE_SLAB: Block = register_slab(registry, BlockItemIds::POLISHED_BLACKSTONE_SLAB, Blocks::POLISHED_BLACKSTONE);
    pub const POLISHED_BLACKSTONE_PRESSURE_PLATE: Block = register_with(registry, BlockItemIds::POLISHED_BLACKSTONE_PRESSURE_PLATE, PressurePlateBlock::new, Properties::of() .map_color(MapColor::COLOR_BLACK) .force_solid_on() .instrument(NoteBlockInstrument::BASEDRUM) .no_collision() .strength(0.5) .push_reaction(PushReaction::POPPED));
    pub const POLISHED_BLACKSTONE_BUTTON: Block = register_with(registry, BlockItemIds::POLISHED_BLACKSTONE_BUTTON, ButtonBlock::new, button_properties());
    pub const POLISHED_BLACKSTONE_WALL: Block = register_wall(registry, BlockItemIds::POLISHED_BLACKSTONE_WALL, Blocks::POLISHED_BLACKSTONE);
    pub const CHISELED_NETHER_BRICKS: Block = register(registry, BlockItemIds::CHISELED_NETHER_BRICKS, Properties::of() .map_color(MapColor::NETHER) .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .strength_and_resistance(2.0, 6.0) );
    pub const CRACKED_NETHER_BRICKS: Block = register(registry, BlockItemIds::CRACKED_NETHER_BRICKS, Properties::of() .map_color(MapColor::NETHER) .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .strength_and_resistance(2.0, 6.0) );
    pub const QUARTZ_BRICKS: Block = register(registry, BlockItemIds::QUARTZ_BRICKS, Properties::of_legacy_copy(&registry.get(Blocks::QUARTZ_BLOCK)));
    pub const CANDLE: Block = register_with(registry, BlockItemIds::CANDLE, CandleBlock::new, candle_properties(MapColor::SAND));
    pub const DYED_CANDLE: ColorCollection<Block> = ColorCollection::register_blocks(registry, BlockItemIds::DYED_CANDLE, |_, p| CandleBlock::new(p), |color| candle_properties(if color == DyeColor::WHITE { MapColor::WOOL } else { color.get_map_color() }));
    pub const CANDLE_CAKE: Block = register_with(registry, BlockIds::CANDLE_CAKE, CandleCakeBlock::new, Properties::of_legacy_copy(&registry.get(Blocks::CAKE)).light_level(lit_block_emission(3)));
    pub const DYED_CANDLE_CAKE: ColorCollection<Block> = ColorCollection::register_blocks(registry, BlockIds::DYED_CANDLE_CAKE, |_, p| CandleCakeBlock::new(p), |_| Properties::of_legacy_copy(&registry.get(Blocks::CANDLE_CAKE)));
    pub const AMETHYST_BLOCK: Block = register_with(registry, BlockItemIds::AMETHYST_BLOCK, AmethystBlock::new, Properties::of().map_color(MapColor::COLOR_PURPLE).strength(1.5).requires_correct_tool_for_drops());
    pub const BUDDING_AMETHYST: Block = register_with(registry, BlockItemIds::BUDDING_AMETHYST, BuddingAmethystBlock::new, Properties::of() .map_color(MapColor::COLOR_PURPLE) .random_ticks() .strength(1.5)  .requires_correct_tool_for_drops() .push_reaction(PushReaction::POPPED));
    pub const AMETHYST_CLUSTER: Block = register_with(registry, BlockItemIds::AMETHYST_CLUSTER, |p| AmethystClusterBlock::new(7.0, 10.0, p), Properties::of() .map_color(MapColor::COLOR_PURPLE) .force_solid_on() .no_occlusion()  .strength(1.5) .light_level(|_| 5) .push_reaction(PushReaction::POPPED));
    pub const LARGE_AMETHYST_BUD: Block = register_with(registry, BlockItemIds::LARGE_AMETHYST_BUD, |p| AmethystClusterBlock::new(5.0, 10.0, p), Properties::of_legacy_copy(&registry.get(Blocks::AMETHYST_CLUSTER)).light_level(|_| 4));
    pub const MEDIUM_AMETHYST_BUD: Block = register_with(registry, BlockItemIds::MEDIUM_AMETHYST_BUD, |p| AmethystClusterBlock::new(4.0, 10.0, p), Properties::of_legacy_copy(&registry.get(Blocks::AMETHYST_CLUSTER)).light_level(|_| 2));
    pub const SMALL_AMETHYST_BUD: Block = register_with(registry, BlockItemIds::SMALL_AMETHYST_BUD, |p| AmethystClusterBlock::new(3.0, 8.0, p), Properties::of_legacy_copy(&registry.get(Blocks::AMETHYST_CLUSTER)).light_level(|_| 1));
    pub const TUFF: Block = register(registry, BlockItemIds::TUFF, Properties::of() .map_color(MapColor::TERRACOTTA_GRAY) .instrument(NoteBlockInstrument::BASEDRUM)  .requires_correct_tool_for_drops() .strength_and_resistance(1.5, 6.0));
    pub const TUFF_SLAB: Block = register_slab(registry, BlockItemIds::TUFF_SLAB, Blocks::TUFF);
    pub const TUFF_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::TUFF_STAIRS, Blocks::TUFF);
    pub const TUFF_WALL: Block = register_wall(registry, BlockItemIds::TUFF_WALL, Blocks::TUFF);
    pub const POLISHED_TUFF: Block = register(registry, BlockItemIds::POLISHED_TUFF, Properties::of_legacy_copy(&registry.get(Blocks::TUFF)));
    pub const POLISHED_TUFF_SLAB: Block = register_slab(registry, BlockItemIds::POLISHED_TUFF_SLAB, Blocks::POLISHED_TUFF);
    pub const POLISHED_TUFF_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::POLISHED_TUFF_STAIRS, Blocks::POLISHED_TUFF);
    pub const POLISHED_TUFF_WALL: Block = register_wall(registry, BlockItemIds::POLISHED_TUFF_WALL, Blocks::POLISHED_TUFF);
    pub const CHISELED_TUFF: Block = register(registry, BlockItemIds::CHISELED_TUFF, Properties::of_legacy_copy(&registry.get(Blocks::TUFF)));
    pub const TUFF_BRICKS: Block = register(registry, BlockItemIds::TUFF_BRICKS, Properties::of_legacy_copy(&registry.get(Blocks::TUFF)));
    pub const TUFF_BRICK_SLAB: Block = register_slab(registry, BlockItemIds::TUFF_BRICK_SLAB, Blocks::TUFF_BRICKS);
    pub const TUFF_BRICK_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::TUFF_BRICK_STAIRS, Blocks::TUFF_BRICKS);
    pub const TUFF_BRICK_WALL: Block = register_wall(registry, BlockItemIds::TUFF_BRICK_WALL, Blocks::TUFF_BRICKS);
    pub const CHISELED_TUFF_BRICKS: Block = register(registry, BlockItemIds::CHISELED_TUFF_BRICKS, Properties::of_legacy_copy(&registry.get(Blocks::TUFF_BRICKS)));
    pub const SULFUR: Block = register(registry, BlockItemIds::SULFUR, Properties::of()  .map_color(MapColor::COLOR_YELLOW) .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .strength_and_resistance(1.5, 6.0));
    pub const POTENT_SULFUR: Block = register_with(registry, BlockItemIds::POTENT_SULFUR, PotentSulfurBlock::new, Properties::of_full_copy(&registry.get(Blocks::SULFUR)).map_color(MapColor::GOLD));
    pub const SULFUR_SLAB: Block = register_slab(registry, BlockItemIds::SULFUR_SLAB, Blocks::SULFUR);
    pub const SULFUR_STAIRS: Block = register_stair(registry, BlockItemIds::SULFUR_STAIRS, Blocks::SULFUR);
    pub const SULFUR_WALL: Block = register_wall(registry, BlockItemIds::SULFUR_WALL, Blocks::SULFUR);
    pub const POLISHED_SULFUR: Block = register(registry, BlockItemIds::POLISHED_SULFUR, Properties::of_legacy_copy(&registry.get(Blocks::SULFUR)));
    pub const POLISHED_SULFUR_SLAB: Block = register_slab(registry, BlockItemIds::POLISHED_SULFUR_SLAB, Blocks::POLISHED_SULFUR);
    pub const POLISHED_SULFUR_STAIRS: Block = register_stair(registry, BlockItemIds::POLISHED_SULFUR_STAIRS, Blocks::POLISHED_SULFUR);
    pub const POLISHED_SULFUR_WALL: Block = register_wall(registry, BlockItemIds::POLISHED_SULFUR_WALL, Blocks::POLISHED_SULFUR);
    pub const SULFUR_BRICKS: Block = register(registry, BlockItemIds::SULFUR_BRICKS, Properties::of_legacy_copy(&registry.get(Blocks::SULFUR)));
    pub const SULFUR_BRICK_SLAB: Block = register_slab(registry, BlockItemIds::SULFUR_BRICK_SLAB, Blocks::SULFUR_BRICKS);
    pub const SULFUR_BRICK_STAIRS: Block = register_stair(registry, BlockItemIds::SULFUR_BRICK_STAIRS, Blocks::SULFUR_BRICKS);
    pub const SULFUR_BRICK_WALL: Block = register_wall(registry, BlockItemIds::SULFUR_BRICK_WALL, Blocks::SULFUR_BRICKS);
    pub const CHISELED_SULFUR: Block = register(registry, BlockItemIds::CHISELED_SULFUR, Properties::of_legacy_copy(&registry.get(Blocks::SULFUR)));
    pub const CINNABAR: Block = register(registry, BlockItemIds::CINNABAR, Properties::of()  .map_color(MapColor::COLOR_RED) .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .strength_and_resistance(1.5, 6.0));
    pub const CINNABAR_SLAB: Block = register_slab(registry, BlockItemIds::CINNABAR_SLAB, Blocks::CINNABAR);
    pub const CINNABAR_STAIRS: Block = register_stair(registry, BlockItemIds::CINNABAR_STAIRS, Blocks::CINNABAR);
    pub const CINNABAR_WALL: Block = register_wall(registry, BlockItemIds::CINNABAR_WALL, Blocks::CINNABAR);
    pub const POLISHED_CINNABAR: Block = register(registry, BlockItemIds::POLISHED_CINNABAR, Properties::of_legacy_copy(&registry.get(Blocks::CINNABAR)));
    pub const POLISHED_CINNABAR_SLAB: Block = register_slab(registry, BlockItemIds::POLISHED_CINNABAR_SLAB, Blocks::POLISHED_CINNABAR);
    pub const POLISHED_CINNABAR_STAIRS: Block = register_stair(registry, BlockItemIds::POLISHED_CINNABAR_STAIRS, Blocks::POLISHED_CINNABAR);
    pub const POLISHED_CINNABAR_WALL: Block = register_wall(registry, BlockItemIds::POLISHED_CINNABAR_WALL, Blocks::POLISHED_CINNABAR);
    pub const CINNABAR_BRICKS: Block = register(registry, BlockItemIds::CINNABAR_BRICKS, Properties::of_legacy_copy(&registry.get(Blocks::CINNABAR)));
    pub const CINNABAR_BRICK_SLAB: Block = register_slab(registry, BlockItemIds::CINNABAR_BRICK_SLAB, Blocks::CINNABAR_BRICKS);
    pub const CINNABAR_BRICK_STAIRS: Block = register_stair(registry, BlockItemIds::CINNABAR_BRICK_STAIRS, Blocks::CINNABAR_BRICKS);
    pub const CINNABAR_BRICK_WALL: Block = register_wall(registry, BlockItemIds::CINNABAR_BRICK_WALL, Blocks::CINNABAR_BRICKS);
    pub const CHISELED_CINNABAR: Block = register(registry, BlockItemIds::CHISELED_CINNABAR, Properties::of_legacy_copy(&registry.get(Blocks::CINNABAR)));
    pub const CALCITE: Block = register(registry, BlockItemIds::CALCITE, Properties::of() .map_color(MapColor::TERRACOTTA_WHITE) .instrument(NoteBlockInstrument::BASEDRUM)  .requires_correct_tool_for_drops() .strength(0.75));
    pub const TINTED_GLASS: Block = register_with(registry, BlockItemIds::TINTED_GLASS, TintedGlassBlock::new, Properties::of_legacy_copy(&registry.get(Blocks::GLASS)) .map_color(MapColor::COLOR_GRAY) .no_occlusion()  .is_redstone_conductor(StatePredicate::State(None, never)) .is_suffocating(StatePredicate::State(None, never)));
    pub const POWDER_SNOW: Block = register_with(registry, BlockItemIds::POWDER_SNOW, PowderSnowBlock::new, Properties::of() .map_color(MapColor::SNOW) .strength(0.25)  .dynamic_shape() .no_occlusion() .is_redstone_conductor(StatePredicate::State(None, never)));
    pub const SCULK_SENSOR: Block = register_with(registry, BlockItemIds::SCULK_SENSOR, SculkSensorBlock::new, Properties::of().map_color(MapColor::COLOR_CYAN).strength(1.5).light_level(|_| 1).emissive_rendering(|state| SculkSensorBlock::get_phase(state) == SculkSensorPhase::ACTIVE));
    pub const CALIBRATED_SCULK_SENSOR: Block = register_with(registry, BlockItemIds::CALIBRATED_SCULK_SENSOR, CalibratedSculkSensorBlock::new, Properties::of_legacy_copy(&registry.get(Blocks::SCULK_SENSOR)));
    pub const SCULK: Block = register_with(registry, BlockItemIds::SCULK, SculkBlock::new, Properties::of().map_color(MapColor::COLOR_BLACK).strength(0.2));
    pub const SCULK_VEIN: Block = register_with(registry, BlockItemIds::SCULK_VEIN, SculkVeinBlock::new, Properties::of() .map_color(MapColor::COLOR_BLACK) .force_solid_on() .no_collision() .strength(0.2)  .push_reaction(PushReaction::POPPED));
    pub const SCULK_CATALYST: Block = register_with(registry, BlockItemIds::SCULK_CATALYST, SculkCatalystBlock::new, Properties::of().map_color(MapColor::COLOR_BLACK).strength_and_resistance(3.0, 3.0).light_level(|_| 6));
    pub const SCULK_SHRIEKER: Block = register_with(registry, BlockItemIds::SCULK_SHRIEKER, SculkShriekerBlock::new, Properties::of().map_color(MapColor::COLOR_BLACK).strength_and_resistance(3.0, 3.0));
    pub const COPPER_BLOCK: WeatheringCopperCollection<Block> = WeatheringCopperCollection::register_blocks(registry, BlockItemIds::COPPER_BLOCK, |_, p| Block::new(p), |_, p| WeatheringCopperFullBlock::new(p), |state| {
            let map_color = match state {
                WeatherState::UNAFFECTED => MapColor::COLOR_ORANGE,
                WeatherState::EXPOSED => MapColor::TERRACOTTA_LIGHT_GRAY,
                WeatherState::WEATHERED => MapColor::WARPED_STEM,
                WeatherState::OXIDIZED => MapColor::WARPED_NYLIUM,
            };
            Properties::of().map_color(map_color).requires_correct_tool_for_drops().strength_and_resistance(3.0, 6.0).instrument(match state {
                WeatherState::UNAFFECTED => NoteBlockInstrument::TRUMPET,
                WeatherState::EXPOSED => NoteBlockInstrument::TRUMPET_EXPOSED,
                WeatherState::WEATHERED => NoteBlockInstrument::TRUMPET_WEATHERED,
                WeatherState::OXIDIZED => NoteBlockInstrument::TRUMPET_OXIDIZED,
            })
        });
    pub const COPPER_ORE: Block = register_with(registry, BlockItemIds::COPPER_ORE, DropExperienceBlock::new, Properties::of_legacy_copy(&registry.get(Blocks::IRON_ORE)));
    pub const DEEPSLATE_COPPER_ORE: Block = register_with(registry, BlockItemIds::DEEPSLATE_COPPER_ORE, DropExperienceBlock::new, Properties::of_legacy_copy(&registry.get(Blocks::COPPER_ORE)).map_color(MapColor::DEEPSLATE).strength_and_resistance(4.5, 3.0));
    pub const CUT_COPPER: WeatheringCopperCollection<Block> = WeatheringCopperCollection::register_blocks(registry, BlockItemIds::CUT_COPPER, |_, p| Block::new(p), |_, p| WeatheringCopperFullBlock::new(p), |state| Properties::of_full_copy(&registry.get(*Blocks::COPPER_BLOCK.weathering().pick(state))));
    pub const CHISELED_COPPER: WeatheringCopperCollection<Block> = WeatheringCopperCollection::register_blocks(registry, BlockItemIds::CHISELED_COPPER, |_, p| Block::new(p), |_, p| WeatheringCopperFullBlock::new(p), |state| Properties::of_full_copy(&registry.get(*Blocks::COPPER_BLOCK.weathering().pick(state))));
    pub const CUT_COPPER_STAIRS: WeatheringCopperCollection<Block> = WeatheringCopperCollection::register_blocks(registry, BlockItemIds::CUT_COPPER_STAIRS, |_, p| StairBlock::new(p), |_, p| WeatheringCopperStairBlock::new(p), |state| Properties::of_full_copy(&registry.get(*Blocks::CUT_COPPER.weathering().pick(state))).is_view_blocking(NEAR_PLANE_INTERSECTS_OUTLINE));
    pub const CUT_COPPER_SLAB: WeatheringCopperCollection<Block> = WeatheringCopperCollection::register_blocks(registry, BlockItemIds::CUT_COPPER_SLAB, |_, p| SlabBlock::new(p), |_, p| WeatheringCopperSlabBlock::new(p), |state| Properties::of_full_copy(&registry.get(*Blocks::CUT_COPPER.weathering().pick(state))).is_view_blocking(NEAR_PLANE_INTERSECTS_OUTLINE));
    pub const COPPER_DOOR: WeatheringCopperCollection<Block> = WeatheringCopperCollection::register_blocks(registry, BlockItemIds::COPPER_DOOR, |_, p| DoorBlock::new(p), |_, p| WeatheringCopperDoorBlock::new(p), |state| {
            // Java asks the unwaxed copper block for its colour when each state is made: a constant one.
            let map_color = registry.get(*Blocks::COPPER_BLOCK.weathering().pick(state)).default_map_color();
            Properties::of().map_color_fn(move |_| map_color).strength_and_resistance(3.0, 6.0).no_occlusion().push_reaction(PushReaction::POPPED)
        });
    pub const COPPER_TRAPDOOR: WeatheringCopperCollection<Block> = WeatheringCopperCollection::register_blocks(registry, BlockItemIds::COPPER_TRAPDOOR, |_, p| TrapDoorBlock::new(p), |_, p| WeatheringCopperTrapDoorBlock::new(p), |state| {
            // Java asks the unwaxed copper block for its colour when each state is made: a constant one.
            let map_color = registry.get(*Blocks::COPPER_BLOCK.weathering().pick(state)).default_map_color();
            Properties::of().map_color_fn(move |_| map_color).strength_and_resistance(3.0, 6.0).requires_correct_tool_for_drops().no_occlusion()
        });
    pub const COPPER_GRATE: WeatheringCopperCollection<Block> = WeatheringCopperCollection::register_blocks(registry, BlockItemIds::COPPER_GRATE, |_, p| WaterloggedTransparentBlock::new(p), |_, p| WeatheringCopperGrateBlock::new(p), |state| {
            // Java asks the unwaxed copper block for its colour when each state is made: a constant one.
            let map_color = registry.get(*Blocks::COPPER_BLOCK.weathering().pick(state)).default_map_color();
            Properties::of().strength_and_resistance(3.0, 6.0).map_color_fn(move |_| map_color).no_occlusion().requires_correct_tool_for_drops().is_redstone_conductor(StatePredicate::State(None, never)).is_suffocating(StatePredicate::State(None, never))
        });
    pub const COPPER_BULB: WeatheringCopperCollection<Block> = WeatheringCopperCollection::register_blocks(registry, BlockItemIds::COPPER_BULB, |_, p| CopperBulbBlock::new(p), |_, p| WeatheringCopperBulbBlock::new(p), |p| {
            // Java asks the waxed copper block for its colour when each state is made: a constant one.
            let map_color = registry.get(*Blocks::COPPER_BLOCK.waxed().pick(p)).default_map_color();
            Properties::of().map_color_fn(move |_| map_color).strength_and_resistance(3.0, 6.0).requires_correct_tool_for_drops().is_redstone_conductor(StatePredicate::State(None, never)).light_level(lit_block_emission(match p {
                WeatherState::UNAFFECTED => 15,
                WeatherState::EXPOSED => 12,
                WeatherState::WEATHERED => 8,
                WeatherState::OXIDIZED => 4,
            }))
        });
    pub const COPPER_CHEST: WeatheringCopperCollection<Block> = WeatheringCopperCollection::register_blocks(registry, BlockItemIds::COPPER_CHEST, |_, p| CopperChestBlock::new(p), |_, p| WeatheringCopperChestBlock::new(p), |state| {
            // Java asks the unwaxed copper block for its colour when each state is made: a constant one.
            let map_color = registry.get(*Blocks::COPPER_BLOCK.weathering().pick(state)).default_map_color();
            Properties::of().map_color_fn(move |_| map_color).strength_and_resistance(3.0, 6.0).requires_correct_tool_for_drops()
        });
    pub const COPPER_GOLEM_STATUE: WeatheringCopperCollection<Block> = WeatheringCopperCollection::register_blocks(registry, BlockItemIds::COPPER_GOLEM_STATUE, |_, p| CopperGolemStatueBlock::new(p), |_, p| WeatheringCopperGolemStatueBlock::new(p), |state| {
            // Java asks the unwaxed copper block for its colour when each state is made: a constant one.
            let map_color = registry.get(*Blocks::COPPER_BLOCK.weathering().pick(state)).default_map_color();
            Properties::of().map_color_fn(move |_| map_color).strength_and_resistance(3.0, 6.0).push_reaction(PushReaction::POPPED).no_occlusion()
        });
    pub const LIGHTNING_ROD: WeatheringCopperCollection<Block> = WeatheringCopperCollection::register_blocks(registry, BlockItemIds::LIGHTNING_ROD, |_, p| LightningRodBlock::new(p), |_, p| WeatheringLightningRodBlock::new(p), |state| {
            // Java asks the unwaxed copper block for its colour when each state is made: a constant one.
            let map_color = registry.get(*Blocks::COPPER_BLOCK.weathering().pick(state)).default_map_color();
            Properties::of().map_color_fn(move |_| map_color).force_solid_on().requires_correct_tool_for_drops().strength_and_resistance(3.0, 6.0).no_occlusion()
        });
    pub const DRIPSTONE_BLOCK: Block = register(registry, BlockItemIds::DRIPSTONE_BLOCK, Properties::of() .map_color(MapColor::TERRACOTTA_BROWN) .instrument(NoteBlockInstrument::BASEDRUM)  .requires_correct_tool_for_drops() .strength_and_resistance(1.5, 1.0));
    pub const POINTED_DRIPSTONE: Block = register_with(registry, BlockItemIds::POINTED_DRIPSTONE, PointedDripstoneBlock::new, Properties::of() .map_color(MapColor::TERRACOTTA_BROWN) .force_solid_on() .instrument(NoteBlockInstrument::BASEDRUM) .no_occlusion()  .random_ticks() .strength_and_resistance(1.5, 3.0) .dynamic_shape() .offset_type(OffsetType::XZ) .push_reaction(PushReaction::POPPED) .is_redstone_conductor(StatePredicate::State(None, never)) .no_occlusion());
    pub const SULFUR_SPIKE: Block = register_with(registry, BlockItemIds::SULFUR_SPIKE, SulfurSpikeBlock::new, Properties::of() .map_color(MapColor::COLOR_YELLOW) .force_solid_on() .instrument(NoteBlockInstrument::BASEDRUM) .no_occlusion()  .random_ticks() .strength_and_resistance(1.5, 3.0) .dynamic_shape() .offset_type(OffsetType::XZ) .push_reaction(PushReaction::POPPED) .is_redstone_conductor(StatePredicate::State(None, never)) .no_occlusion());
    pub const CAVE_VINES: Block = register_with(registry, BlockItemIds::GLOW_BERRY_CROP, CaveVinesBlock::new, Properties::of() .map_color(MapColor::PLANT) .random_ticks() .no_collision() .light_level(CaveVines::emission(14)) .instabreak()  .push_reaction(PushReaction::POPPED));
    pub const CAVE_VINES_PLANT: Block = register_with(registry, BlockIds::CAVE_VINES_PLANT, CaveVinesPlantBlock::new, Properties::of() .map_color(MapColor::PLANT) .no_collision() .light_level(CaveVines::emission(14)) .instabreak()  .push_reaction(PushReaction::POPPED));
    pub const SPORE_BLOSSOM: Block = register_with(registry, BlockItemIds::SPORE_BLOSSOM, SporeBlossomBlock::new, Properties::of().map_color(MapColor::PLANT).instabreak().no_collision().push_reaction(PushReaction::POPPED));
    pub const AZALEA: Block = register_with(registry, BlockItemIds::AZALEA, AzaleaBlock::new, Properties::of() .map_color(MapColor::PLANT) .force_solid_off() .instabreak()  .no_occlusion() .push_reaction(PushReaction::POPPED));
    pub const FLOWERING_AZALEA: Block = register_with(registry, BlockItemIds::FLOWERING_AZALEA, AzaleaBlock::new, Properties::of() .map_color(MapColor::PLANT) .force_solid_off() .instabreak()  .no_occlusion() .push_reaction(PushReaction::POPPED));
    pub const MOSS_CARPET: Block = register_with(registry, BlockItemIds::MOSS_CARPET, CarpetBlock::new, Properties::of().map_color(MapColor::COLOR_GREEN).strength(0.1).push_reaction(PushReaction::POPPED));
    pub const PINK_PETALS: Block = register_with(registry, BlockItemIds::PINK_PETALS, |p| FlowerBedBlock::new(p, 3), Properties::of().map_color(MapColor::PLANT).no_collision().push_reaction(PushReaction::POPPED));
    pub const WILDFLOWERS: Block = register_with(registry, BlockItemIds::WILDFLOWERS, |p| FlowerBedBlock::new(p, 3), Properties::of().map_color(MapColor::PLANT).no_collision().push_reaction(PushReaction::POPPED));
    pub const LEAF_LITTER: Block = register_with(registry, BlockItemIds::LEAF_LITTER, LeafLitterBlock::new, Properties::of() .map_color(MapColor::COLOR_BROWN) .replaceable() .no_collision()  .push_reaction(PushReaction::POPPED));
    pub const MOSS_BLOCK: Block = register_with(registry, BlockItemIds::MOSS_BLOCK, BonemealableFeaturePlacerBlock::new, Properties::of().map_color(MapColor::COLOR_GREEN).strength(0.1).push_reaction(PushReaction::POPPED));
    pub const BIG_DRIPLEAF: Block = register_with(registry, BlockItemIds::BIG_DRIPLEAF, BigDripleafBlock::new, Properties::of().map_color(MapColor::PLANT).force_solid_off().strength(0.1).push_reaction(PushReaction::POPPED));
    pub const BIG_DRIPLEAF_STEM: Block = register_with(registry, BlockIds::BIG_DRIPLEAF_STEM, BigDripleafStemBlock::new, Properties::of().map_color(MapColor::PLANT).no_collision().strength(0.1).push_reaction(PushReaction::POPPED));
    pub const SMALL_DRIPLEAF: Block = register_with(registry, BlockItemIds::SMALL_DRIPLEAF, SmallDripleafBlock::new, Properties::of() .map_color(MapColor::PLANT) .no_collision() .instabreak()  .offset_type(OffsetType::XYZ) .push_reaction(PushReaction::POPPED));
    pub const HANGING_ROOTS: Block = register_with(registry, BlockItemIds::HANGING_ROOTS, HangingRootsBlock::new, Properties::of() .map_color(MapColor::DIRT) .replaceable() .no_collision() .instabreak()  .offset_type(OffsetType::XZ) .ignited_by_lava() .push_reaction(PushReaction::POPPED));
    pub const ROOTED_DIRT: Block = register_with(registry, BlockItemIds::ROOTED_DIRT, RootedDirtBlock::new, Properties::of().map_color(MapColor::DIRT).strength(0.5));
    pub const MUD: Block = register_with(registry, BlockItemIds::MUD, MudBlock::new, Properties::of_legacy_copy(&registry.get(Blocks::DIRT)) .map_color(MapColor::TERRACOTTA_CYAN)  .is_redstone_conductor(StatePredicate::State(None, always)) .is_suffocating(StatePredicate::State(None, always)) );
    pub const DEEPSLATE: Block = register_with(registry, BlockItemIds::DEEPSLATE, RotatedPillarBlock::new, Properties::of() .map_color(MapColor::DEEPSLATE) .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .strength_and_resistance(3.0, 6.0) );
    pub const COBBLED_DEEPSLATE: Block = register(registry, BlockItemIds::COBBLED_DEEPSLATE, Properties::of_legacy_copy(&registry.get(Blocks::DEEPSLATE)).strength_and_resistance(3.5, 6.0));
    pub const COBBLED_DEEPSLATE_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::COBBLED_DEEPSLATE_STAIRS, Blocks::COBBLED_DEEPSLATE);
    pub const COBBLED_DEEPSLATE_SLAB: Block = register_slab(registry, BlockItemIds::COBBLED_DEEPSLATE_SLAB, Blocks::COBBLED_DEEPSLATE);
    pub const COBBLED_DEEPSLATE_WALL: Block = register_wall(registry, BlockItemIds::COBBLED_DEEPSLATE_WALL, Blocks::COBBLED_DEEPSLATE);
    pub const POLISHED_DEEPSLATE: Block = register(registry, BlockItemIds::POLISHED_DEEPSLATE, Properties::of_legacy_copy(&registry.get(Blocks::COBBLED_DEEPSLATE)));
    pub const POLISHED_DEEPSLATE_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::POLISHED_DEEPSLATE_STAIRS, Blocks::POLISHED_DEEPSLATE);
    pub const POLISHED_DEEPSLATE_SLAB: Block = register_slab(registry, BlockItemIds::POLISHED_DEEPSLATE_SLAB, Blocks::POLISHED_DEEPSLATE);
    pub const POLISHED_DEEPSLATE_WALL: Block = register_wall(registry, BlockItemIds::POLISHED_DEEPSLATE_WALL, Blocks::POLISHED_DEEPSLATE);
    pub const DEEPSLATE_TILES: Block = register(registry, BlockItemIds::DEEPSLATE_TILES, Properties::of_legacy_copy(&registry.get(Blocks::COBBLED_DEEPSLATE)));
    pub const DEEPSLATE_TILE_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::DEEPSLATE_TILE_STAIRS, Blocks::DEEPSLATE_TILES);
    pub const DEEPSLATE_TILE_SLAB: Block = register_slab(registry, BlockItemIds::DEEPSLATE_TILE_SLAB, Blocks::DEEPSLATE_TILES);
    pub const DEEPSLATE_TILE_WALL: Block = register_wall(registry, BlockItemIds::DEEPSLATE_TILE_WALL, Blocks::DEEPSLATE_TILES);
    pub const DEEPSLATE_BRICKS: Block = register(registry, BlockItemIds::DEEPSLATE_BRICKS, Properties::of_legacy_copy(&registry.get(Blocks::COBBLED_DEEPSLATE)));
    pub const DEEPSLATE_BRICK_STAIRS: Block = register_legacy_stair(registry, BlockItemIds::DEEPSLATE_BRICK_STAIRS, Blocks::DEEPSLATE_BRICKS);
    pub const DEEPSLATE_BRICK_SLAB: Block = register_slab(registry, BlockItemIds::DEEPSLATE_BRICK_SLAB, Blocks::DEEPSLATE_BRICKS);
    pub const DEEPSLATE_BRICK_WALL: Block = register_wall(registry, BlockItemIds::DEEPSLATE_BRICK_WALL, Blocks::DEEPSLATE_BRICKS);
    pub const CHISELED_DEEPSLATE: Block = register(registry, BlockItemIds::CHISELED_DEEPSLATE, Properties::of_legacy_copy(&registry.get(Blocks::COBBLED_DEEPSLATE)));
    pub const CRACKED_DEEPSLATE_BRICKS: Block = register(registry, BlockItemIds::CRACKED_DEEPSLATE_BRICKS, Properties::of_legacy_copy(&registry.get(Blocks::DEEPSLATE_BRICKS)));
    pub const CRACKED_DEEPSLATE_TILES: Block = register(registry, BlockItemIds::CRACKED_DEEPSLATE_TILES, Properties::of_legacy_copy(&registry.get(Blocks::DEEPSLATE_TILES)));
    pub const INFESTED_DEEPSLATE: Block = register_with(registry, BlockItemIds::INFESTED_DEEPSLATE, |p| InfestedRotatedPillarBlock::new(&registry.get(Blocks::DEEPSLATE), p), Properties::of().map_color(MapColor::DEEPSLATE));
    pub const SMOOTH_BASALT: Block = register(registry, BlockItemIds::SMOOTH_BASALT, Properties::of_legacy_copy(&registry.get(Blocks::BASALT)));
    pub const RAW_IRON_BLOCK: Block = register(registry, BlockItemIds::RAW_IRON_BLOCK, Properties::of().map_color(MapColor::RAW_IRON).instrument(NoteBlockInstrument::BASEDRUM).requires_correct_tool_for_drops().strength_and_resistance(5.0, 6.0));
    pub const RAW_COPPER_BLOCK: Block = register(registry, BlockItemIds::RAW_COPPER_BLOCK, Properties::of() .map_color(MapColor::COLOR_ORANGE) .instrument(NoteBlockInstrument::BASEDRUM) .requires_correct_tool_for_drops() .strength_and_resistance(5.0, 6.0));
    pub const RAW_GOLD_BLOCK: Block = register(registry, BlockItemIds::RAW_GOLD_BLOCK, Properties::of().map_color(MapColor::GOLD).instrument(NoteBlockInstrument::BASEDRUM).requires_correct_tool_for_drops().strength_and_resistance(5.0, 6.0));
    pub const POTTED_AZALEA: Block = register_with(registry, BlockIds::POTTED_AZALEA_BUSH, FlowerPotBlock::new, flower_pot_properties());
    pub const POTTED_FLOWERING_AZALEA: Block = register_with(registry, BlockIds::POTTED_FLOWERING_AZALEA_BUSH, FlowerPotBlock::new, flower_pot_properties());
    pub const OCHRE_FROGLIGHT: Block = register_with(registry, BlockItemIds::OCHRE_FROGLIGHT, RotatedPillarBlock::new, Properties::of().map_color(MapColor::SAND).strength(0.3).light_level(|_| 15));
    pub const VERDANT_FROGLIGHT: Block = register_with(registry, BlockItemIds::VERDANT_FROGLIGHT, RotatedPillarBlock::new, Properties::of().map_color(MapColor::GLOW_LICHEN).strength(0.3).light_level(|_| 15));
    pub const PEARLESCENT_FROGLIGHT: Block = register_with(registry, BlockItemIds::PEARLESCENT_FROGLIGHT, RotatedPillarBlock::new, Properties::of().map_color(MapColor::COLOR_PINK).strength(0.3).light_level(|_| 15));
    pub const FROGSPAWN: Block = register_with(registry, BlockItemIds::FROGSPAWN, FrogspawnBlock::new, Properties::of() .map_color(MapColor::WATER) .instabreak() .no_occlusion() .no_collision()  .push_reaction(PushReaction::POPPED));
    pub const REINFORCED_DEEPSLATE: Block = register(registry, BlockItemIds::REINFORCED_DEEPSLATE, Properties::of() .map_color(MapColor::DEEPSLATE) .instrument(NoteBlockInstrument::BASEDRUM)  .strength_and_resistance(55.0, 1200.0) .push_reaction(PushReaction::IMMOVEABLE));
    pub const DECORATED_POT: Block = register_with(registry, BlockItemIds::DECORATED_POT, DecoratedPotBlock::new, Properties::of().map_color(MapColor::TERRACOTTA_RED).strength_and_resistance(0.0, 0.0).push_reaction(PushReaction::POPPED).no_occlusion());
    pub const CRAFTER: Block = register_with(registry, BlockItemIds::CRAFTER, CrafterBlock::new, Properties::of().map_color(MapColor::STONE).strength_and_resistance(1.5, 3.5));
    pub const TRIAL_SPAWNER: Block = register_with(registry, BlockItemIds::TRIAL_SPAWNER, TrialSpawnerBlock::new, Properties::of().map_color(MapColor::STONE).instrument(NoteBlockInstrument::BASEDRUM).light_level(|state| state.get_value(TrialSpawnerBlock::STATE).light_level()).strength(50.0).no_occlusion());
    pub const VAULT: Block = register_with(registry, BlockItemIds::VAULT, VaultBlock::new, Properties::of().map_color(MapColor::STONE).instrument(NoteBlockInstrument::BASEDRUM).no_occlusion().light_level(|state| state.get_value(VaultBlock::STATE).light_level()).strength(50.0));
    pub const HEAVY_CORE: Block = register_with(registry, BlockItemIds::HEAVY_CORE, HeavyCoreBlock::new, Properties::of() .map_color(MapColor::METAL) .instrument(NoteBlockInstrument::SNARE)  .strength(10.0) .push_reaction(PushReaction::PUSH_PULL) .explosion_resistance(1200.0));
    pub const PALE_MOSS_BLOCK: Block = register_with(registry, BlockItemIds::PALE_MOSS_BLOCK, BonemealableFeaturePlacerBlock::new, Properties::of() .ignited_by_lava() .map_color(MapColor::COLOR_LIGHT_GRAY) .strength(0.1)  .push_reaction(PushReaction::POPPED));
    pub const PALE_MOSS_CARPET: Block = register_with(registry, BlockItemIds::PALE_MOSS_CARPET, MossyCarpetBlock::new, Properties::of() .ignited_by_lava() .map_color(registry.get(Blocks::PALE_MOSS_BLOCK).default_map_color()) .strength(0.1)  .push_reaction(PushReaction::POPPED) .no_occlusion());
    pub const PALE_HANGING_MOSS: Block = register_with(registry, BlockItemIds::PALE_HANGING_MOSS, HangingMossBlock::new, Properties::of() .ignited_by_lava() .map_color(registry.get(Blocks::PALE_MOSS_BLOCK).default_map_color()) .no_collision()  .push_reaction(PushReaction::POPPED));
    pub const OPEN_EYEBLOSSOM: Block = register_with(registry, BlockItemIds::OPEN_EYEBLOSSOM, EyeblossomBlock::new, Properties::of() .map_color(registry.get(Blocks::CREAKING_HEART).default_map_color()) .no_collision() .instabreak()  .offset_type(OffsetType::XZ) .push_reaction(PushReaction::POPPED) .random_ticks());
    pub const CLOSED_EYEBLOSSOM: Block = register_with(registry, BlockItemIds::CLOSED_EYEBLOSSOM, EyeblossomBlock::new, Properties::of() .map_color(registry.get(Blocks::PALE_OAK_LEAVES).default_map_color()) .no_collision() .instabreak()  .offset_type(OffsetType::XZ) .push_reaction(PushReaction::POPPED) .random_ticks());
    pub const POTTED_OPEN_EYEBLOSSOM: Block = register_with(registry, BlockIds::POTTED_OPEN_EYEBLOSSOM, FlowerPotBlock::new, flower_pot_properties().random_ticks());
    pub const POTTED_CLOSED_EYEBLOSSOM: Block = register_with(registry, BlockIds::POTTED_CLOSED_EYEBLOSSOM, FlowerPotBlock::new, flower_pot_properties().random_ticks());
    pub const FIREFLY_BUSH: Block = register_with(registry, BlockItemIds::FIREFLY_BUSH, FireflyBushBlock::new, Properties::of() .map_color(MapColor::PLANT) .ignited_by_lava() .light_level(|_| 2) .no_collision() .instabreak()  .push_reaction(PushReaction::POPPED));
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::fs;
    use std::path::PathBuf;

    use serde_json::Value;

    use super::{Blocks, registry};
    use crate::golden::Golden;
    use crate::net::minecraft::world::level::block::block::Block;
    use crate::net::minecraft::world::level::block::state::state_holder::StateHolder;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.world.level.block.Blocks");
        golden.field("AIR", Blocks::AIR);
        golden.field("STONE", Blocks::STONE);
        golden.field("GRANITE", Blocks::GRANITE);
        golden.field("POLISHED_GRANITE", Blocks::POLISHED_GRANITE);
        golden.field("DIORITE", Blocks::DIORITE);
        golden.field("POLISHED_DIORITE", Blocks::POLISHED_DIORITE);
        golden.field("ANDESITE", Blocks::ANDESITE);
        golden.field("POLISHED_ANDESITE", Blocks::POLISHED_ANDESITE);
        golden.field("GRASS_BLOCK", Blocks::GRASS_BLOCK);
        golden.field("DIRT", Blocks::DIRT);
        golden.field("COARSE_DIRT", Blocks::COARSE_DIRT);
        golden.field("PODZOL", Blocks::PODZOL);
        golden.field("COBBLESTONE", Blocks::COBBLESTONE);
        golden.field("OAK_PLANKS", Blocks::OAK_PLANKS);
        golden.field("SPRUCE_PLANKS", Blocks::SPRUCE_PLANKS);
        golden.field("BIRCH_PLANKS", Blocks::BIRCH_PLANKS);
        golden.field("JUNGLE_PLANKS", Blocks::JUNGLE_PLANKS);
        golden.field("ACACIA_PLANKS", Blocks::ACACIA_PLANKS);
        golden.field("CHERRY_PLANKS", Blocks::CHERRY_PLANKS);
        golden.field("DARK_OAK_PLANKS", Blocks::DARK_OAK_PLANKS);
        golden.field("PALE_OAK_WOOD", Blocks::PALE_OAK_WOOD);
        golden.field("PALE_OAK_PLANKS", Blocks::PALE_OAK_PLANKS);
        golden.field("MANGROVE_PLANKS", Blocks::MANGROVE_PLANKS);
        golden.field("POPLAR_PLANKS", Blocks::POPLAR_PLANKS);
        golden.field("BAMBOO_PLANKS", Blocks::BAMBOO_PLANKS);
        golden.field("BAMBOO_MOSAIC", Blocks::BAMBOO_MOSAIC);
        golden.field("OAK_SAPLING", Blocks::OAK_SAPLING);
        golden.field("SPRUCE_SAPLING", Blocks::SPRUCE_SAPLING);
        golden.field("BIRCH_SAPLING", Blocks::BIRCH_SAPLING);
        golden.field("JUNGLE_SAPLING", Blocks::JUNGLE_SAPLING);
        golden.field("ACACIA_SAPLING", Blocks::ACACIA_SAPLING);
        golden.field("CHERRY_SAPLING", Blocks::CHERRY_SAPLING);
        golden.field("DARK_OAK_SAPLING", Blocks::DARK_OAK_SAPLING);
        golden.field("PALE_OAK_SAPLING", Blocks::PALE_OAK_SAPLING);
        golden.field("MANGROVE_PROPAGULE", Blocks::MANGROVE_PROPAGULE);
        golden.field("POPLAR_SAPLING", Blocks::POPLAR_SAPLING);
        golden.field("BEDROCK", Blocks::BEDROCK);
        golden.field("WATER", Blocks::WATER);
        golden.field("LAVA", Blocks::LAVA);
        golden.field("SAND", Blocks::SAND);
        golden.field("SUSPICIOUS_SAND", Blocks::SUSPICIOUS_SAND);
        golden.field("RED_SAND", Blocks::RED_SAND);
        golden.field("GRAVEL", Blocks::GRAVEL);
        golden.field("SUSPICIOUS_GRAVEL", Blocks::SUSPICIOUS_GRAVEL);
        golden.field("GOLD_ORE", Blocks::GOLD_ORE);
        golden.field("DEEPSLATE_GOLD_ORE", Blocks::DEEPSLATE_GOLD_ORE);
        golden.field("IRON_ORE", Blocks::IRON_ORE);
        golden.field("DEEPSLATE_IRON_ORE", Blocks::DEEPSLATE_IRON_ORE);
        golden.field("COAL_ORE", Blocks::COAL_ORE);
        golden.field("DEEPSLATE_COAL_ORE", Blocks::DEEPSLATE_COAL_ORE);
        golden.field("NETHER_GOLD_ORE", Blocks::NETHER_GOLD_ORE);
        golden.field("OAK_LOG", Blocks::OAK_LOG);
        golden.field("SPRUCE_LOG", Blocks::SPRUCE_LOG);
        golden.field("BIRCH_LOG", Blocks::BIRCH_LOG);
        golden.field("JUNGLE_LOG", Blocks::JUNGLE_LOG);
        golden.field("ACACIA_LOG", Blocks::ACACIA_LOG);
        golden.field("CHERRY_LOG", Blocks::CHERRY_LOG);
        golden.field("DARK_OAK_LOG", Blocks::DARK_OAK_LOG);
        golden.field("PALE_OAK_LOG", Blocks::PALE_OAK_LOG);
        golden.field("MANGROVE_LOG", Blocks::MANGROVE_LOG);
        golden.field("POPLAR_LOG", Blocks::POPLAR_LOG);
        golden.field("MANGROVE_ROOTS", Blocks::MANGROVE_ROOTS);
        golden.field("MUDDY_MANGROVE_ROOTS", Blocks::MUDDY_MANGROVE_ROOTS);
        golden.field("BAMBOO_BLOCK", Blocks::BAMBOO_BLOCK);
        golden.field("STRIPPED_SPRUCE_LOG", Blocks::STRIPPED_SPRUCE_LOG);
        golden.field("STRIPPED_BIRCH_LOG", Blocks::STRIPPED_BIRCH_LOG);
        golden.field("STRIPPED_JUNGLE_LOG", Blocks::STRIPPED_JUNGLE_LOG);
        golden.field("STRIPPED_ACACIA_LOG", Blocks::STRIPPED_ACACIA_LOG);
        golden.field("STRIPPED_CHERRY_LOG", Blocks::STRIPPED_CHERRY_LOG);
        golden.field("STRIPPED_DARK_OAK_LOG", Blocks::STRIPPED_DARK_OAK_LOG);
        golden.field("STRIPPED_PALE_OAK_LOG", Blocks::STRIPPED_PALE_OAK_LOG);
        golden.field("STRIPPED_OAK_LOG", Blocks::STRIPPED_OAK_LOG);
        golden.field("STRIPPED_MANGROVE_LOG", Blocks::STRIPPED_MANGROVE_LOG);
        golden.field("STRIPPED_POPLAR_LOG", Blocks::STRIPPED_POPLAR_LOG);
        golden.field("STRIPPED_BAMBOO_BLOCK", Blocks::STRIPPED_BAMBOO_BLOCK);
        golden.field("OAK_WOOD", Blocks::OAK_WOOD);
        golden.field("SPRUCE_WOOD", Blocks::SPRUCE_WOOD);
        golden.field("BIRCH_WOOD", Blocks::BIRCH_WOOD);
        golden.field("JUNGLE_WOOD", Blocks::JUNGLE_WOOD);
        golden.field("ACACIA_WOOD", Blocks::ACACIA_WOOD);
        golden.field("CHERRY_WOOD", Blocks::CHERRY_WOOD);
        golden.field("DARK_OAK_WOOD", Blocks::DARK_OAK_WOOD);
        golden.field("MANGROVE_WOOD", Blocks::MANGROVE_WOOD);
        golden.field("POPLAR_WOOD", Blocks::POPLAR_WOOD);
        golden.field("STRIPPED_OAK_WOOD", Blocks::STRIPPED_OAK_WOOD);
        golden.field("STRIPPED_SPRUCE_WOOD", Blocks::STRIPPED_SPRUCE_WOOD);
        golden.field("STRIPPED_BIRCH_WOOD", Blocks::STRIPPED_BIRCH_WOOD);
        golden.field("STRIPPED_JUNGLE_WOOD", Blocks::STRIPPED_JUNGLE_WOOD);
        golden.field("STRIPPED_ACACIA_WOOD", Blocks::STRIPPED_ACACIA_WOOD);
        golden.field("STRIPPED_CHERRY_WOOD", Blocks::STRIPPED_CHERRY_WOOD);
        golden.field("STRIPPED_DARK_OAK_WOOD", Blocks::STRIPPED_DARK_OAK_WOOD);
        golden.field("STRIPPED_PALE_OAK_WOOD", Blocks::STRIPPED_PALE_OAK_WOOD);
        golden.field("STRIPPED_MANGROVE_WOOD", Blocks::STRIPPED_MANGROVE_WOOD);
        golden.field("STRIPPED_POPLAR_WOOD", Blocks::STRIPPED_POPLAR_WOOD);
        golden.field("OAK_LEAVES", Blocks::OAK_LEAVES);
        golden.field("SPRUCE_LEAVES", Blocks::SPRUCE_LEAVES);
        golden.field("BIRCH_LEAVES", Blocks::BIRCH_LEAVES);
        golden.field("JUNGLE_LEAVES", Blocks::JUNGLE_LEAVES);
        golden.field("ACACIA_LEAVES", Blocks::ACACIA_LEAVES);
        golden.field("CHERRY_LEAVES", Blocks::CHERRY_LEAVES);
        golden.field("DARK_OAK_LEAVES", Blocks::DARK_OAK_LEAVES);
        golden.field("PALE_OAK_LEAVES", Blocks::PALE_OAK_LEAVES);
        golden.field("MANGROVE_LEAVES", Blocks::MANGROVE_LEAVES);
        golden.field("RED_POPLAR_LEAVES", Blocks::RED_POPLAR_LEAVES);
        golden.field("ORANGE_POPLAR_LEAVES", Blocks::ORANGE_POPLAR_LEAVES);
        golden.field("YELLOW_POPLAR_LEAVES", Blocks::YELLOW_POPLAR_LEAVES);
        golden.field("AZALEA_LEAVES", Blocks::AZALEA_LEAVES);
        golden.field("FLOWERING_AZALEA_LEAVES", Blocks::FLOWERING_AZALEA_LEAVES);
        golden.field("SPONGE", Blocks::SPONGE);
        golden.field("WET_SPONGE", Blocks::WET_SPONGE);
        golden.field("GLASS", Blocks::GLASS);
        golden.field("LAPIS_ORE", Blocks::LAPIS_ORE);
        golden.field("DEEPSLATE_LAPIS_ORE", Blocks::DEEPSLATE_LAPIS_ORE);
        golden.field("LAPIS_BLOCK", Blocks::LAPIS_BLOCK);
        golden.field("DISPENSER", Blocks::DISPENSER);
        golden.field("SANDSTONE", Blocks::SANDSTONE);
        golden.field("CHISELED_SANDSTONE", Blocks::CHISELED_SANDSTONE);
        golden.field("CUT_SANDSTONE", Blocks::CUT_SANDSTONE);
        golden.field("NOTE_BLOCK", Blocks::NOTE_BLOCK);
        golden.field("BED", Blocks::BED);
        golden.field("STRAW_BED", Blocks::STRAW_BED);
        golden.field("POWERED_RAIL", Blocks::POWERED_RAIL);
        golden.field("DETECTOR_RAIL", Blocks::DETECTOR_RAIL);
        golden.field("STICKY_PISTON", Blocks::STICKY_PISTON);
        golden.field("COBWEB", Blocks::COBWEB);
        golden.field("SHORT_GRASS", Blocks::SHORT_GRASS);
        golden.field("FERN", Blocks::FERN);
        golden.field("DEAD_BUSH", Blocks::DEAD_BUSH);
        golden.field("BUSH", Blocks::BUSH);
        golden.field("RED_SHRUB", Blocks::RED_SHRUB);
        golden.field("SHORT_DRY_GRASS", Blocks::SHORT_DRY_GRASS);
        golden.field("TALL_DRY_GRASS", Blocks::TALL_DRY_GRASS);
        golden.field("SEAGRASS", Blocks::SEAGRASS);
        golden.field("TALL_SEAGRASS", Blocks::TALL_SEAGRASS);
        golden.field("PISTON", Blocks::PISTON);
        golden.field("PISTON_HEAD", Blocks::PISTON_HEAD);
        golden.field("WOOL", Blocks::WOOL);
        golden.field("WOOL_STAIRS", Blocks::WOOL_STAIRS);
        golden.field("WOOL_SLAB", Blocks::WOOL_SLAB);
        golden.field("MOVING_PISTON", Blocks::MOVING_PISTON);
        golden.field("DANDELION", Blocks::DANDELION);
        golden.field("GOLDEN_DANDELION", Blocks::GOLDEN_DANDELION);
        golden.field("TORCHFLOWER", Blocks::TORCHFLOWER);
        golden.field("POPPY", Blocks::POPPY);
        golden.field("BLUE_ORCHID", Blocks::BLUE_ORCHID);
        golden.field("ALLIUM", Blocks::ALLIUM);
        golden.field("AZURE_BLUET", Blocks::AZURE_BLUET);
        golden.field("RED_TULIP", Blocks::RED_TULIP);
        golden.field("ORANGE_TULIP", Blocks::ORANGE_TULIP);
        golden.field("WHITE_TULIP", Blocks::WHITE_TULIP);
        golden.field("PINK_TULIP", Blocks::PINK_TULIP);
        golden.field("OXEYE_DAISY", Blocks::OXEYE_DAISY);
        golden.field("CORNFLOWER", Blocks::CORNFLOWER);
        golden.field("WITHER_ROSE", Blocks::WITHER_ROSE);
        golden.field("LILY_OF_THE_VALLEY", Blocks::LILY_OF_THE_VALLEY);
        golden.field("BROWN_MUSHROOM", Blocks::BROWN_MUSHROOM);
        golden.field("RED_MUSHROOM", Blocks::RED_MUSHROOM);
        golden.field("GOLD_BLOCK", Blocks::GOLD_BLOCK);
        golden.field("IRON_BLOCK", Blocks::IRON_BLOCK);
        golden.field("BRICKS", Blocks::BRICKS);
        golden.field("TNT", Blocks::TNT);
        golden.field("BOOKSHELF", Blocks::BOOKSHELF);
        golden.field("CHISELED_BOOKSHELF", Blocks::CHISELED_BOOKSHELF);
        golden.field("ACACIA_SHELF", Blocks::ACACIA_SHELF);
        golden.field("BAMBOO_SHELF", Blocks::BAMBOO_SHELF);
        golden.field("BIRCH_SHELF", Blocks::BIRCH_SHELF);
        golden.field("CHERRY_SHELF", Blocks::CHERRY_SHELF);
        golden.field("CRIMSON_SHELF", Blocks::CRIMSON_SHELF);
        golden.field("DARK_OAK_SHELF", Blocks::DARK_OAK_SHELF);
        golden.field("JUNGLE_SHELF", Blocks::JUNGLE_SHELF);
        golden.field("MANGROVE_SHELF", Blocks::MANGROVE_SHELF);
        golden.field("OAK_SHELF", Blocks::OAK_SHELF);
        golden.field("PALE_OAK_SHELF", Blocks::PALE_OAK_SHELF);
        golden.field("POPLAR_SHELF", Blocks::POPLAR_SHELF);
        golden.field("SPRUCE_SHELF", Blocks::SPRUCE_SHELF);
        golden.field("WARPED_SHELF", Blocks::WARPED_SHELF);
        golden.field("MOSSY_COBBLESTONE", Blocks::MOSSY_COBBLESTONE);
        golden.field("OBSIDIAN", Blocks::OBSIDIAN);
        golden.field("TORCH", Blocks::TORCH);
        golden.field("WALL_TORCH", Blocks::WALL_TORCH);
        golden.field("FIRE", Blocks::FIRE);
        golden.field("SOUL_FIRE", Blocks::SOUL_FIRE);
        golden.field("SPAWNER", Blocks::SPAWNER);
        golden.field("CREAKING_HEART", Blocks::CREAKING_HEART);
        golden.field("OAK_STAIRS", Blocks::OAK_STAIRS);
        golden.field("CHEST", Blocks::CHEST);
        golden.field("REDSTONE_WIRE", Blocks::REDSTONE_WIRE);
        golden.field("DIAMOND_ORE", Blocks::DIAMOND_ORE);
        golden.field("DEEPSLATE_DIAMOND_ORE", Blocks::DEEPSLATE_DIAMOND_ORE);
        golden.field("DIAMOND_BLOCK", Blocks::DIAMOND_BLOCK);
        golden.field("CRAFTING_TABLE", Blocks::CRAFTING_TABLE);
        golden.field("WHEAT", Blocks::WHEAT);
        golden.field("FARMLAND", Blocks::FARMLAND);
        golden.field("FURNACE", Blocks::FURNACE);
        golden.field("OAK_SIGN", Blocks::OAK_SIGN);
        golden.field("SPRUCE_SIGN", Blocks::SPRUCE_SIGN);
        golden.field("BIRCH_SIGN", Blocks::BIRCH_SIGN);
        golden.field("ACACIA_SIGN", Blocks::ACACIA_SIGN);
        golden.field("CHERRY_SIGN", Blocks::CHERRY_SIGN);
        golden.field("JUNGLE_SIGN", Blocks::JUNGLE_SIGN);
        golden.field("DARK_OAK_SIGN", Blocks::DARK_OAK_SIGN);
        golden.field("PALE_OAK_SIGN", Blocks::PALE_OAK_SIGN);
        golden.field("MANGROVE_SIGN", Blocks::MANGROVE_SIGN);
        golden.field("POPLAR_SIGN", Blocks::POPLAR_SIGN);
        golden.field("BAMBOO_SIGN", Blocks::BAMBOO_SIGN);
        golden.field("OAK_DOOR", Blocks::OAK_DOOR);
        golden.field("LADDER", Blocks::LADDER);
        golden.field("RAIL", Blocks::RAIL);
        golden.field("COBBLESTONE_STAIRS", Blocks::COBBLESTONE_STAIRS);
        golden.field("OAK_WALL_SIGN", Blocks::OAK_WALL_SIGN);
        golden.field("SPRUCE_WALL_SIGN", Blocks::SPRUCE_WALL_SIGN);
        golden.field("BIRCH_WALL_SIGN", Blocks::BIRCH_WALL_SIGN);
        golden.field("ACACIA_WALL_SIGN", Blocks::ACACIA_WALL_SIGN);
        golden.field("CHERRY_WALL_SIGN", Blocks::CHERRY_WALL_SIGN);
        golden.field("JUNGLE_WALL_SIGN", Blocks::JUNGLE_WALL_SIGN);
        golden.field("DARK_OAK_WALL_SIGN", Blocks::DARK_OAK_WALL_SIGN);
        golden.field("PALE_OAK_WALL_SIGN", Blocks::PALE_OAK_WALL_SIGN);
        golden.field("MANGROVE_WALL_SIGN", Blocks::MANGROVE_WALL_SIGN);
        golden.field("POPLAR_WALL_SIGN", Blocks::POPLAR_WALL_SIGN);
        golden.field("BAMBOO_WALL_SIGN", Blocks::BAMBOO_WALL_SIGN);
        golden.field("OAK_HANGING_SIGN", Blocks::OAK_HANGING_SIGN);
        golden.field("SPRUCE_HANGING_SIGN", Blocks::SPRUCE_HANGING_SIGN);
        golden.field("BIRCH_HANGING_SIGN", Blocks::BIRCH_HANGING_SIGN);
        golden.field("ACACIA_HANGING_SIGN", Blocks::ACACIA_HANGING_SIGN);
        golden.field("CHERRY_HANGING_SIGN", Blocks::CHERRY_HANGING_SIGN);
        golden.field("JUNGLE_HANGING_SIGN", Blocks::JUNGLE_HANGING_SIGN);
        golden.field("DARK_OAK_HANGING_SIGN", Blocks::DARK_OAK_HANGING_SIGN);
        golden.field("PALE_OAK_HANGING_SIGN", Blocks::PALE_OAK_HANGING_SIGN);
        golden.field("CRIMSON_HANGING_SIGN", Blocks::CRIMSON_HANGING_SIGN);
        golden.field("WARPED_HANGING_SIGN", Blocks::WARPED_HANGING_SIGN);
        golden.field("MANGROVE_HANGING_SIGN", Blocks::MANGROVE_HANGING_SIGN);
        golden.field("POPLAR_HANGING_SIGN", Blocks::POPLAR_HANGING_SIGN);
        golden.field("BAMBOO_HANGING_SIGN", Blocks::BAMBOO_HANGING_SIGN);
        golden.field("OAK_WALL_HANGING_SIGN", Blocks::OAK_WALL_HANGING_SIGN);
        golden.field("SPRUCE_WALL_HANGING_SIGN", Blocks::SPRUCE_WALL_HANGING_SIGN);
        golden.field("BIRCH_WALL_HANGING_SIGN", Blocks::BIRCH_WALL_HANGING_SIGN);
        golden.field("ACACIA_WALL_HANGING_SIGN", Blocks::ACACIA_WALL_HANGING_SIGN);
        golden.field("CHERRY_WALL_HANGING_SIGN", Blocks::CHERRY_WALL_HANGING_SIGN);
        golden.field("JUNGLE_WALL_HANGING_SIGN", Blocks::JUNGLE_WALL_HANGING_SIGN);
        golden.field(
            "DARK_OAK_WALL_HANGING_SIGN",
            Blocks::DARK_OAK_WALL_HANGING_SIGN,
        );
        golden.field(
            "PALE_OAK_WALL_HANGING_SIGN",
            Blocks::PALE_OAK_WALL_HANGING_SIGN,
        );
        golden.field(
            "MANGROVE_WALL_HANGING_SIGN",
            Blocks::MANGROVE_WALL_HANGING_SIGN,
        );
        golden.field("POPLAR_WALL_HANGING_SIGN", Blocks::POPLAR_WALL_HANGING_SIGN);
        golden.field(
            "CRIMSON_WALL_HANGING_SIGN",
            Blocks::CRIMSON_WALL_HANGING_SIGN,
        );
        golden.field("WARPED_WALL_HANGING_SIGN", Blocks::WARPED_WALL_HANGING_SIGN);
        golden.field("BAMBOO_WALL_HANGING_SIGN", Blocks::BAMBOO_WALL_HANGING_SIGN);
        golden.field("LEVER", Blocks::LEVER);
        golden.field("STONE_PRESSURE_PLATE", Blocks::STONE_PRESSURE_PLATE);
        golden.field("IRON_DOOR", Blocks::IRON_DOOR);
        golden.field("OAK_PRESSURE_PLATE", Blocks::OAK_PRESSURE_PLATE);
        golden.field("SPRUCE_PRESSURE_PLATE", Blocks::SPRUCE_PRESSURE_PLATE);
        golden.field("BIRCH_PRESSURE_PLATE", Blocks::BIRCH_PRESSURE_PLATE);
        golden.field("JUNGLE_PRESSURE_PLATE", Blocks::JUNGLE_PRESSURE_PLATE);
        golden.field("ACACIA_PRESSURE_PLATE", Blocks::ACACIA_PRESSURE_PLATE);
        golden.field("CHERRY_PRESSURE_PLATE", Blocks::CHERRY_PRESSURE_PLATE);
        golden.field("DARK_OAK_PRESSURE_PLATE", Blocks::DARK_OAK_PRESSURE_PLATE);
        golden.field("PALE_OAK_PRESSURE_PLATE", Blocks::PALE_OAK_PRESSURE_PLATE);
        golden.field("MANGROVE_PRESSURE_PLATE", Blocks::MANGROVE_PRESSURE_PLATE);
        golden.field("POPLAR_PRESSURE_PLATE", Blocks::POPLAR_PRESSURE_PLATE);
        golden.field("BAMBOO_PRESSURE_PLATE", Blocks::BAMBOO_PRESSURE_PLATE);
        golden.field("REDSTONE_ORE", Blocks::REDSTONE_ORE);
        golden.field("DEEPSLATE_REDSTONE_ORE", Blocks::DEEPSLATE_REDSTONE_ORE);
        golden.field("REDSTONE_TORCH", Blocks::REDSTONE_TORCH);
        golden.field("REDSTONE_WALL_TORCH", Blocks::REDSTONE_WALL_TORCH);
        golden.field("STONE_BUTTON", Blocks::STONE_BUTTON);
        golden.field("SNOW", Blocks::SNOW);
        golden.field("ICE", Blocks::ICE);
        golden.field("SNOW_BLOCK", Blocks::SNOW_BLOCK);
        golden.field("CACTUS", Blocks::CACTUS);
        golden.field("CACTUS_FLOWER", Blocks::CACTUS_FLOWER);
        golden.field("CLAY", Blocks::CLAY);
        golden.field("SUGAR_CANE", Blocks::SUGAR_CANE);
        golden.field("JUKEBOX", Blocks::JUKEBOX);
        golden.field("OAK_FENCE", Blocks::OAK_FENCE);
        golden.field("NETHERRACK", Blocks::NETHERRACK);
        golden.field("SOUL_SAND", Blocks::SOUL_SAND);
        golden.field("SOUL_SOIL", Blocks::SOUL_SOIL);
        golden.field("BASALT", Blocks::BASALT);
        golden.field("POLISHED_BASALT", Blocks::POLISHED_BASALT);
        golden.field("SOUL_TORCH", Blocks::SOUL_TORCH);
        golden.field("SOUL_WALL_TORCH", Blocks::SOUL_WALL_TORCH);
        golden.field("COPPER_TORCH", Blocks::COPPER_TORCH);
        golden.field("COPPER_WALL_TORCH", Blocks::COPPER_WALL_TORCH);
        golden.field("GLOWSTONE", Blocks::GLOWSTONE);
        golden.field("NETHER_PORTAL", Blocks::NETHER_PORTAL);
        golden.field("CARVED_PUMPKIN", Blocks::CARVED_PUMPKIN);
        golden.field("JACK_O_LANTERN", Blocks::JACK_O_LANTERN);
        golden.field("CAKE", Blocks::CAKE);
        golden.field("REPEATER", Blocks::REPEATER);
        golden.field("STAINED_GLASS", Blocks::STAINED_GLASS);
        golden.field("OAK_TRAPDOOR", Blocks::OAK_TRAPDOOR);
        golden.field("SPRUCE_TRAPDOOR", Blocks::SPRUCE_TRAPDOOR);
        golden.field("BIRCH_TRAPDOOR", Blocks::BIRCH_TRAPDOOR);
        golden.field("JUNGLE_TRAPDOOR", Blocks::JUNGLE_TRAPDOOR);
        golden.field("ACACIA_TRAPDOOR", Blocks::ACACIA_TRAPDOOR);
        golden.field("CHERRY_TRAPDOOR", Blocks::CHERRY_TRAPDOOR);
        golden.field("DARK_OAK_TRAPDOOR", Blocks::DARK_OAK_TRAPDOOR);
        golden.field("PALE_OAK_TRAPDOOR", Blocks::PALE_OAK_TRAPDOOR);
        golden.field("MANGROVE_TRAPDOOR", Blocks::MANGROVE_TRAPDOOR);
        golden.field("POPLAR_TRAPDOOR", Blocks::POPLAR_TRAPDOOR);
        golden.field("BAMBOO_TRAPDOOR", Blocks::BAMBOO_TRAPDOOR);
        golden.field("STONE_BRICKS", Blocks::STONE_BRICKS);
        golden.field("MOSSY_STONE_BRICKS", Blocks::MOSSY_STONE_BRICKS);
        golden.field("CRACKED_STONE_BRICKS", Blocks::CRACKED_STONE_BRICKS);
        golden.field("CHISELED_STONE_BRICKS", Blocks::CHISELED_STONE_BRICKS);
        golden.field("PACKED_MUD", Blocks::PACKED_MUD);
        golden.field("MUD_BRICKS", Blocks::MUD_BRICKS);
        golden.field("INFESTED_STONE", Blocks::INFESTED_STONE);
        golden.field("INFESTED_COBBLESTONE", Blocks::INFESTED_COBBLESTONE);
        golden.field("INFESTED_STONE_BRICKS", Blocks::INFESTED_STONE_BRICKS);
        golden.field(
            "INFESTED_MOSSY_STONE_BRICKS",
            Blocks::INFESTED_MOSSY_STONE_BRICKS,
        );
        golden.field(
            "INFESTED_CRACKED_STONE_BRICKS",
            Blocks::INFESTED_CRACKED_STONE_BRICKS,
        );
        golden.field(
            "INFESTED_CHISELED_STONE_BRICKS",
            Blocks::INFESTED_CHISELED_STONE_BRICKS,
        );
        golden.field("BROWN_MUSHROOM_BLOCK", Blocks::BROWN_MUSHROOM_BLOCK);
        golden.field("RED_MUSHROOM_BLOCK", Blocks::RED_MUSHROOM_BLOCK);
        golden.field("MUSHROOM_STEM", Blocks::MUSHROOM_STEM);
        golden.field("IRON_BARS", Blocks::IRON_BARS);
        golden.field("COPPER_BARS", Blocks::COPPER_BARS);
        golden.field("IRON_CHAIN", Blocks::IRON_CHAIN);
        golden.field("COPPER_CHAIN", Blocks::COPPER_CHAIN);
        golden.field("GLASS_PANE", Blocks::GLASS_PANE);
        golden.field("PUMPKIN", Blocks::PUMPKIN);
        golden.field("MELON", Blocks::MELON);
        golden.field("ATTACHED_PUMPKIN_STEM", Blocks::ATTACHED_PUMPKIN_STEM);
        golden.field("ATTACHED_MELON_STEM", Blocks::ATTACHED_MELON_STEM);
        golden.field("PUMPKIN_STEM", Blocks::PUMPKIN_STEM);
        golden.field("MELON_STEM", Blocks::MELON_STEM);
        golden.field("VINE", Blocks::VINE);
        golden.field("GLOW_LICHEN", Blocks::GLOW_LICHEN);
        golden.field("RESIN_CLUMP", Blocks::RESIN_CLUMP);
        golden.field("OAK_FENCE_GATE", Blocks::OAK_FENCE_GATE);
        golden.field("BRICK_STAIRS", Blocks::BRICK_STAIRS);
        golden.field("STONE_BRICK_STAIRS", Blocks::STONE_BRICK_STAIRS);
        golden.field("MUD_BRICK_STAIRS", Blocks::MUD_BRICK_STAIRS);
        golden.field("MYCELIUM", Blocks::MYCELIUM);
        golden.field("LILY_PAD", Blocks::LILY_PAD);
        golden.field("RESIN_BLOCK", Blocks::RESIN_BLOCK);
        golden.field("RESIN_BRICKS", Blocks::RESIN_BRICKS);
        golden.field("RESIN_BRICK_STAIRS", Blocks::RESIN_BRICK_STAIRS);
        golden.field("RESIN_BRICK_SLAB", Blocks::RESIN_BRICK_SLAB);
        golden.field("RESIN_BRICK_WALL", Blocks::RESIN_BRICK_WALL);
        golden.field("CHISELED_RESIN_BRICKS", Blocks::CHISELED_RESIN_BRICKS);
        golden.field("NETHER_BRICKS", Blocks::NETHER_BRICKS);
        golden.field("NETHER_BRICK_FENCE", Blocks::NETHER_BRICK_FENCE);
        golden.field("NETHER_BRICK_STAIRS", Blocks::NETHER_BRICK_STAIRS);
        golden.field("NETHER_WART", Blocks::NETHER_WART);
        golden.field("ENCHANTING_TABLE", Blocks::ENCHANTING_TABLE);
        golden.field("BREWING_STAND", Blocks::BREWING_STAND);
        golden.field("CAULDRON", Blocks::CAULDRON);
        golden.field("WATER_CAULDRON", Blocks::WATER_CAULDRON);
        golden.field("LAVA_CAULDRON", Blocks::LAVA_CAULDRON);
        golden.field("POWDER_SNOW_CAULDRON", Blocks::POWDER_SNOW_CAULDRON);
        golden.field("END_PORTAL", Blocks::END_PORTAL);
        golden.field("END_PORTAL_FRAME", Blocks::END_PORTAL_FRAME);
        golden.field("END_STONE", Blocks::END_STONE);
        golden.field("DRAGON_EGG", Blocks::DRAGON_EGG);
        golden.field("REDSTONE_LAMP", Blocks::REDSTONE_LAMP);
        golden.field("COCOA", Blocks::COCOA);
        golden.field("SHELF_MUSHROOM", Blocks::SHELF_MUSHROOM);
        golden.field("SANDSTONE_STAIRS", Blocks::SANDSTONE_STAIRS);
        golden.field("EMERALD_ORE", Blocks::EMERALD_ORE);
        golden.field("DEEPSLATE_EMERALD_ORE", Blocks::DEEPSLATE_EMERALD_ORE);
        golden.field("ENDER_CHEST", Blocks::ENDER_CHEST);
        golden.field("TRIPWIRE_HOOK", Blocks::TRIPWIRE_HOOK);
        golden.field("TRIPWIRE", Blocks::TRIPWIRE);
        golden.field("EMERALD_BLOCK", Blocks::EMERALD_BLOCK);
        golden.field("SPRUCE_STAIRS", Blocks::SPRUCE_STAIRS);
        golden.field("BIRCH_STAIRS", Blocks::BIRCH_STAIRS);
        golden.field("JUNGLE_STAIRS", Blocks::JUNGLE_STAIRS);
        golden.field("COMMAND_BLOCK", Blocks::COMMAND_BLOCK);
        golden.field("BEACON", Blocks::BEACON);
        golden.field("COBBLESTONE_WALL", Blocks::COBBLESTONE_WALL);
        golden.field("MOSSY_COBBLESTONE_WALL", Blocks::MOSSY_COBBLESTONE_WALL);
        golden.field("FLOWER_POT", Blocks::FLOWER_POT);
        golden.field("POTTED_TORCHFLOWER", Blocks::POTTED_TORCHFLOWER);
        golden.field("POTTED_OAK_SAPLING", Blocks::POTTED_OAK_SAPLING);
        golden.field("POTTED_SPRUCE_SAPLING", Blocks::POTTED_SPRUCE_SAPLING);
        golden.field("POTTED_BIRCH_SAPLING", Blocks::POTTED_BIRCH_SAPLING);
        golden.field("POTTED_JUNGLE_SAPLING", Blocks::POTTED_JUNGLE_SAPLING);
        golden.field("POTTED_ACACIA_SAPLING", Blocks::POTTED_ACACIA_SAPLING);
        golden.field("POTTED_CHERRY_SAPLING", Blocks::POTTED_CHERRY_SAPLING);
        golden.field("POTTED_DARK_OAK_SAPLING", Blocks::POTTED_DARK_OAK_SAPLING);
        golden.field("POTTED_PALE_OAK_SAPLING", Blocks::POTTED_PALE_OAK_SAPLING);
        golden.field("POTTED_POPLAR_SAPLING", Blocks::POTTED_POPLAR_SAPLING);
        golden.field(
            "POTTED_MANGROVE_PROPAGULE",
            Blocks::POTTED_MANGROVE_PROPAGULE,
        );
        golden.field("POTTED_FERN", Blocks::POTTED_FERN);
        golden.field("POTTED_DANDELION", Blocks::POTTED_DANDELION);
        golden.field("POTTED_GOLDEN_DANDELION", Blocks::POTTED_GOLDEN_DANDELION);
        golden.field("POTTED_POPPY", Blocks::POTTED_POPPY);
        golden.field("POTTED_BLUE_ORCHID", Blocks::POTTED_BLUE_ORCHID);
        golden.field("POTTED_ALLIUM", Blocks::POTTED_ALLIUM);
        golden.field("POTTED_AZURE_BLUET", Blocks::POTTED_AZURE_BLUET);
        golden.field("POTTED_RED_TULIP", Blocks::POTTED_RED_TULIP);
        golden.field("POTTED_ORANGE_TULIP", Blocks::POTTED_ORANGE_TULIP);
        golden.field("POTTED_WHITE_TULIP", Blocks::POTTED_WHITE_TULIP);
        golden.field("POTTED_PINK_TULIP", Blocks::POTTED_PINK_TULIP);
        golden.field("POTTED_OXEYE_DAISY", Blocks::POTTED_OXEYE_DAISY);
        golden.field("POTTED_CORNFLOWER", Blocks::POTTED_CORNFLOWER);
        golden.field(
            "POTTED_LILY_OF_THE_VALLEY",
            Blocks::POTTED_LILY_OF_THE_VALLEY,
        );
        golden.field("POTTED_WITHER_ROSE", Blocks::POTTED_WITHER_ROSE);
        golden.field("POTTED_RED_MUSHROOM", Blocks::POTTED_RED_MUSHROOM);
        golden.field("POTTED_BROWN_MUSHROOM", Blocks::POTTED_BROWN_MUSHROOM);
        golden.field("POTTED_DEAD_BUSH", Blocks::POTTED_DEAD_BUSH);
        golden.field("POTTED_CACTUS", Blocks::POTTED_CACTUS);
        golden.field("CARROTS", Blocks::CARROTS);
        golden.field("POTATOES", Blocks::POTATOES);
        golden.field("OAK_BUTTON", Blocks::OAK_BUTTON);
        golden.field("SPRUCE_BUTTON", Blocks::SPRUCE_BUTTON);
        golden.field("BIRCH_BUTTON", Blocks::BIRCH_BUTTON);
        golden.field("JUNGLE_BUTTON", Blocks::JUNGLE_BUTTON);
        golden.field("ACACIA_BUTTON", Blocks::ACACIA_BUTTON);
        golden.field("CHERRY_BUTTON", Blocks::CHERRY_BUTTON);
        golden.field("DARK_OAK_BUTTON", Blocks::DARK_OAK_BUTTON);
        golden.field("PALE_OAK_BUTTON", Blocks::PALE_OAK_BUTTON);
        golden.field("MANGROVE_BUTTON", Blocks::MANGROVE_BUTTON);
        golden.field("POPLAR_BUTTON", Blocks::POPLAR_BUTTON);
        golden.field("BAMBOO_BUTTON", Blocks::BAMBOO_BUTTON);
        golden.field("SKELETON_SKULL", Blocks::SKELETON_SKULL);
        golden.field("SKELETON_WALL_SKULL", Blocks::SKELETON_WALL_SKULL);
        golden.field("WITHER_SKELETON_SKULL", Blocks::WITHER_SKELETON_SKULL);
        golden.field(
            "WITHER_SKELETON_WALL_SKULL",
            Blocks::WITHER_SKELETON_WALL_SKULL,
        );
        golden.field("ZOMBIE_HEAD", Blocks::ZOMBIE_HEAD);
        golden.field("ZOMBIE_WALL_HEAD", Blocks::ZOMBIE_WALL_HEAD);
        golden.field("PLAYER_HEAD", Blocks::PLAYER_HEAD);
        golden.field("PLAYER_WALL_HEAD", Blocks::PLAYER_WALL_HEAD);
        golden.field("CREEPER_HEAD", Blocks::CREEPER_HEAD);
        golden.field("CREEPER_WALL_HEAD", Blocks::CREEPER_WALL_HEAD);
        golden.field("DRAGON_HEAD", Blocks::DRAGON_HEAD);
        golden.field("DRAGON_WALL_HEAD", Blocks::DRAGON_WALL_HEAD);
        golden.field("PIGLIN_HEAD", Blocks::PIGLIN_HEAD);
        golden.field("PIGLIN_WALL_HEAD", Blocks::PIGLIN_WALL_HEAD);
        golden.field("ANVIL", Blocks::ANVIL);
        golden.field("CHIPPED_ANVIL", Blocks::CHIPPED_ANVIL);
        golden.field("DAMAGED_ANVIL", Blocks::DAMAGED_ANVIL);
        golden.field("TRAPPED_CHEST", Blocks::TRAPPED_CHEST);
        golden.field(
            "LIGHT_WEIGHTED_PRESSURE_PLATE",
            Blocks::LIGHT_WEIGHTED_PRESSURE_PLATE,
        );
        golden.field(
            "HEAVY_WEIGHTED_PRESSURE_PLATE",
            Blocks::HEAVY_WEIGHTED_PRESSURE_PLATE,
        );
        golden.field("COMPARATOR", Blocks::COMPARATOR);
        golden.field("DAYLIGHT_DETECTOR", Blocks::DAYLIGHT_DETECTOR);
        golden.field("REDSTONE_BLOCK", Blocks::REDSTONE_BLOCK);
        golden.field("NETHER_QUARTZ_ORE", Blocks::NETHER_QUARTZ_ORE);
        golden.field("HOPPER", Blocks::HOPPER);
        golden.field("QUARTZ_BLOCK", Blocks::QUARTZ_BLOCK);
        golden.field("CHISELED_QUARTZ_BLOCK", Blocks::CHISELED_QUARTZ_BLOCK);
        golden.field("QUARTZ_PILLAR", Blocks::QUARTZ_PILLAR);
        golden.field("QUARTZ_STAIRS", Blocks::QUARTZ_STAIRS);
        golden.field("ACTIVATOR_RAIL", Blocks::ACTIVATOR_RAIL);
        golden.field("DROPPER", Blocks::DROPPER);
        golden.field("DYED_TERRACOTTA", Blocks::DYED_TERRACOTTA);
        golden.field("STAINED_GLASS_PANE", Blocks::STAINED_GLASS_PANE);
        golden.field("ACACIA_STAIRS", Blocks::ACACIA_STAIRS);
        golden.field("CHERRY_STAIRS", Blocks::CHERRY_STAIRS);
        golden.field("DARK_OAK_STAIRS", Blocks::DARK_OAK_STAIRS);
        golden.field("PALE_OAK_STAIRS", Blocks::PALE_OAK_STAIRS);
        golden.field("MANGROVE_STAIRS", Blocks::MANGROVE_STAIRS);
        golden.field("POPLAR_STAIRS", Blocks::POPLAR_STAIRS);
        golden.field("BAMBOO_STAIRS", Blocks::BAMBOO_STAIRS);
        golden.field("BAMBOO_MOSAIC_STAIRS", Blocks::BAMBOO_MOSAIC_STAIRS);
        golden.field("SLIME_BLOCK", Blocks::SLIME_BLOCK);
        golden.field("BARRIER", Blocks::BARRIER);
        golden.field("LIGHT", Blocks::LIGHT);
        golden.field("IRON_TRAPDOOR", Blocks::IRON_TRAPDOOR);
        golden.field("PRISMARINE", Blocks::PRISMARINE);
        golden.field("PRISMARINE_BRICKS", Blocks::PRISMARINE_BRICKS);
        golden.field("DARK_PRISMARINE", Blocks::DARK_PRISMARINE);
        golden.field("PRISMARINE_STAIRS", Blocks::PRISMARINE_STAIRS);
        golden.field("PRISMARINE_BRICK_STAIRS", Blocks::PRISMARINE_BRICK_STAIRS);
        golden.field("DARK_PRISMARINE_STAIRS", Blocks::DARK_PRISMARINE_STAIRS);
        golden.field("PRISMARINE_SLAB", Blocks::PRISMARINE_SLAB);
        golden.field("PRISMARINE_BRICK_SLAB", Blocks::PRISMARINE_BRICK_SLAB);
        golden.field("DARK_PRISMARINE_SLAB", Blocks::DARK_PRISMARINE_SLAB);
        golden.field("SEA_LANTERN", Blocks::SEA_LANTERN);
        golden.field("HAY_BLOCK", Blocks::HAY_BLOCK);
        golden.field("CARPET", Blocks::CARPET);
        golden.field("TERRACOTTA", Blocks::TERRACOTTA);
        golden.field("COAL_BLOCK", Blocks::COAL_BLOCK);
        golden.field("PACKED_ICE", Blocks::PACKED_ICE);
        golden.field("SUNFLOWER", Blocks::SUNFLOWER);
        golden.field("LILAC", Blocks::LILAC);
        golden.field("ROSE_BUSH", Blocks::ROSE_BUSH);
        golden.field("PEONY", Blocks::PEONY);
        golden.field("TALL_GRASS", Blocks::TALL_GRASS);
        golden.field("LARGE_FERN", Blocks::LARGE_FERN);
        golden.field("BANNER", Blocks::BANNER);
        golden.field("WALL_BANNER", Blocks::WALL_BANNER);
        golden.field("RED_SANDSTONE", Blocks::RED_SANDSTONE);
        golden.field("CHISELED_RED_SANDSTONE", Blocks::CHISELED_RED_SANDSTONE);
        golden.field("CUT_RED_SANDSTONE", Blocks::CUT_RED_SANDSTONE);
        golden.field("RED_SANDSTONE_STAIRS", Blocks::RED_SANDSTONE_STAIRS);
        golden.field("OAK_SLAB", Blocks::OAK_SLAB);
        golden.field("SPRUCE_SLAB", Blocks::SPRUCE_SLAB);
        golden.field("BIRCH_SLAB", Blocks::BIRCH_SLAB);
        golden.field("JUNGLE_SLAB", Blocks::JUNGLE_SLAB);
        golden.field("ACACIA_SLAB", Blocks::ACACIA_SLAB);
        golden.field("CHERRY_SLAB", Blocks::CHERRY_SLAB);
        golden.field("DARK_OAK_SLAB", Blocks::DARK_OAK_SLAB);
        golden.field("PALE_OAK_SLAB", Blocks::PALE_OAK_SLAB);
        golden.field("MANGROVE_SLAB", Blocks::MANGROVE_SLAB);
        golden.field("POPLAR_SLAB", Blocks::POPLAR_SLAB);
        golden.field("BAMBOO_SLAB", Blocks::BAMBOO_SLAB);
        golden.field("BAMBOO_MOSAIC_SLAB", Blocks::BAMBOO_MOSAIC_SLAB);
        golden.field("STONE_SLAB", Blocks::STONE_SLAB);
        golden.field("SANDSTONE_SLAB", Blocks::SANDSTONE_SLAB);
        golden.field("CUT_SANDSTONE_SLAB", Blocks::CUT_SANDSTONE_SLAB);
        golden.field("PETRIFIED_OAK_SLAB", Blocks::PETRIFIED_OAK_SLAB);
        golden.field("COBBLESTONE_SLAB", Blocks::COBBLESTONE_SLAB);
        golden.field("BRICK_SLAB", Blocks::BRICK_SLAB);
        golden.field("STONE_BRICK_SLAB", Blocks::STONE_BRICK_SLAB);
        golden.field("MUD_BRICK_SLAB", Blocks::MUD_BRICK_SLAB);
        golden.field("NETHER_BRICK_SLAB", Blocks::NETHER_BRICK_SLAB);
        golden.field("QUARTZ_SLAB", Blocks::QUARTZ_SLAB);
        golden.field("RED_SANDSTONE_SLAB", Blocks::RED_SANDSTONE_SLAB);
        golden.field("CUT_RED_SANDSTONE_SLAB", Blocks::CUT_RED_SANDSTONE_SLAB);
        golden.field("SMOOTH_STONE", Blocks::SMOOTH_STONE);
        golden.field("SMOOTH_STONE_SLAB", Blocks::SMOOTH_STONE_SLAB);
        golden.field("SMOOTH_SANDSTONE", Blocks::SMOOTH_SANDSTONE);
        golden.field("SMOOTH_QUARTZ", Blocks::SMOOTH_QUARTZ);
        golden.field("SMOOTH_RED_SANDSTONE", Blocks::SMOOTH_RED_SANDSTONE);
        golden.field("SPRUCE_FENCE_GATE", Blocks::SPRUCE_FENCE_GATE);
        golden.field("BIRCH_FENCE_GATE", Blocks::BIRCH_FENCE_GATE);
        golden.field("JUNGLE_FENCE_GATE", Blocks::JUNGLE_FENCE_GATE);
        golden.field("ACACIA_FENCE_GATE", Blocks::ACACIA_FENCE_GATE);
        golden.field("CHERRY_FENCE_GATE", Blocks::CHERRY_FENCE_GATE);
        golden.field("DARK_OAK_FENCE_GATE", Blocks::DARK_OAK_FENCE_GATE);
        golden.field("PALE_OAK_FENCE_GATE", Blocks::PALE_OAK_FENCE_GATE);
        golden.field("MANGROVE_FENCE_GATE", Blocks::MANGROVE_FENCE_GATE);
        golden.field("POPLAR_FENCE_GATE", Blocks::POPLAR_FENCE_GATE);
        golden.field("BAMBOO_FENCE_GATE", Blocks::BAMBOO_FENCE_GATE);
        golden.field("SPRUCE_FENCE", Blocks::SPRUCE_FENCE);
        golden.field("BIRCH_FENCE", Blocks::BIRCH_FENCE);
        golden.field("JUNGLE_FENCE", Blocks::JUNGLE_FENCE);
        golden.field("ACACIA_FENCE", Blocks::ACACIA_FENCE);
        golden.field("CHERRY_FENCE", Blocks::CHERRY_FENCE);
        golden.field("DARK_OAK_FENCE", Blocks::DARK_OAK_FENCE);
        golden.field("PALE_OAK_FENCE", Blocks::PALE_OAK_FENCE);
        golden.field("MANGROVE_FENCE", Blocks::MANGROVE_FENCE);
        golden.field("POPLAR_FENCE", Blocks::POPLAR_FENCE);
        golden.field("BAMBOO_FENCE", Blocks::BAMBOO_FENCE);
        golden.field("SPRUCE_DOOR", Blocks::SPRUCE_DOOR);
        golden.field("BIRCH_DOOR", Blocks::BIRCH_DOOR);
        golden.field("JUNGLE_DOOR", Blocks::JUNGLE_DOOR);
        golden.field("ACACIA_DOOR", Blocks::ACACIA_DOOR);
        golden.field("CHERRY_DOOR", Blocks::CHERRY_DOOR);
        golden.field("DARK_OAK_DOOR", Blocks::DARK_OAK_DOOR);
        golden.field("PALE_OAK_DOOR", Blocks::PALE_OAK_DOOR);
        golden.field("MANGROVE_DOOR", Blocks::MANGROVE_DOOR);
        golden.field("POPLAR_DOOR", Blocks::POPLAR_DOOR);
        golden.field("BAMBOO_DOOR", Blocks::BAMBOO_DOOR);
        golden.field("END_ROD", Blocks::END_ROD);
        golden.field("CHORUS_PLANT", Blocks::CHORUS_PLANT);
        golden.field("CHORUS_FLOWER", Blocks::CHORUS_FLOWER);
        golden.field("PURPUR_BLOCK", Blocks::PURPUR_BLOCK);
        golden.field("PURPUR_SLAB", Blocks::PURPUR_SLAB);
        golden.field("PURPUR_PILLAR", Blocks::PURPUR_PILLAR);
        golden.field("PURPUR_STAIRS", Blocks::PURPUR_STAIRS);
        golden.field("END_STONE_BRICKS", Blocks::END_STONE_BRICKS);
        golden.field("TORCHFLOWER_CROP", Blocks::TORCHFLOWER_CROP);
        golden.field("PITCHER_CROP", Blocks::PITCHER_CROP);
        golden.field("PITCHER_PLANT", Blocks::PITCHER_PLANT);
        golden.field("BEETROOTS", Blocks::BEETROOTS);
        golden.field("DIRT_PATH", Blocks::DIRT_PATH);
        golden.field("END_GATEWAY", Blocks::END_GATEWAY);
        golden.field("REPEATING_COMMAND_BLOCK", Blocks::REPEATING_COMMAND_BLOCK);
        golden.field("CHAIN_COMMAND_BLOCK", Blocks::CHAIN_COMMAND_BLOCK);
        golden.field("FROSTED_ICE", Blocks::FROSTED_ICE);
        golden.field("MAGMA_BLOCK", Blocks::MAGMA_BLOCK);
        golden.field("NETHER_WART_BLOCK", Blocks::NETHER_WART_BLOCK);
        golden.field("RED_NETHER_BRICKS", Blocks::RED_NETHER_BRICKS);
        golden.field("BONE_BLOCK", Blocks::BONE_BLOCK);
        golden.field("STRUCTURE_VOID", Blocks::STRUCTURE_VOID);
        golden.field("OBSERVER", Blocks::OBSERVER);
        golden.field("SHULKER_BOX", Blocks::SHULKER_BOX);
        golden.field("DYED_SHULKER_BOX", Blocks::DYED_SHULKER_BOX);
        golden.field("GLAZED_TERRACOTTA", Blocks::GLAZED_TERRACOTTA);
        golden.field("CONCRETE", Blocks::CONCRETE);
        golden.field("CONCRETE_STAIRS", Blocks::CONCRETE_STAIRS);
        golden.field("CONCRETE_SLAB", Blocks::CONCRETE_SLAB);
        golden.field("CONCRETE_POWDER", Blocks::CONCRETE_POWDER);
        golden.field("KELP", Blocks::KELP);
        golden.field("KELP_PLANT", Blocks::KELP_PLANT);
        golden.field("DRIED_KELP_BLOCK", Blocks::DRIED_KELP_BLOCK);
        golden.field("TURTLE_EGG", Blocks::TURTLE_EGG);
        golden.field("SNIFFER_EGG", Blocks::SNIFFER_EGG);
        golden.field("DRIED_GHAST", Blocks::DRIED_GHAST);
        golden.field("DEAD_TUBE_CORAL_BLOCK", Blocks::DEAD_TUBE_CORAL_BLOCK);
        golden.field("DEAD_BRAIN_CORAL_BLOCK", Blocks::DEAD_BRAIN_CORAL_BLOCK);
        golden.field("DEAD_BUBBLE_CORAL_BLOCK", Blocks::DEAD_BUBBLE_CORAL_BLOCK);
        golden.field("DEAD_FIRE_CORAL_BLOCK", Blocks::DEAD_FIRE_CORAL_BLOCK);
        golden.field("DEAD_HORN_CORAL_BLOCK", Blocks::DEAD_HORN_CORAL_BLOCK);
        golden.field("TUBE_CORAL_BLOCK", Blocks::TUBE_CORAL_BLOCK);
        golden.field("BRAIN_CORAL_BLOCK", Blocks::BRAIN_CORAL_BLOCK);
        golden.field("BUBBLE_CORAL_BLOCK", Blocks::BUBBLE_CORAL_BLOCK);
        golden.field("FIRE_CORAL_BLOCK", Blocks::FIRE_CORAL_BLOCK);
        golden.field("HORN_CORAL_BLOCK", Blocks::HORN_CORAL_BLOCK);
        golden.field("DEAD_TUBE_CORAL", Blocks::DEAD_TUBE_CORAL);
        golden.field("DEAD_BRAIN_CORAL", Blocks::DEAD_BRAIN_CORAL);
        golden.field("DEAD_BUBBLE_CORAL", Blocks::DEAD_BUBBLE_CORAL);
        golden.field("DEAD_FIRE_CORAL", Blocks::DEAD_FIRE_CORAL);
        golden.field("DEAD_HORN_CORAL", Blocks::DEAD_HORN_CORAL);
        golden.field("TUBE_CORAL", Blocks::TUBE_CORAL);
        golden.field("BRAIN_CORAL", Blocks::BRAIN_CORAL);
        golden.field("BUBBLE_CORAL", Blocks::BUBBLE_CORAL);
        golden.field("FIRE_CORAL", Blocks::FIRE_CORAL);
        golden.field("HORN_CORAL", Blocks::HORN_CORAL);
        golden.field("DEAD_TUBE_CORAL_FAN", Blocks::DEAD_TUBE_CORAL_FAN);
        golden.field("DEAD_BRAIN_CORAL_FAN", Blocks::DEAD_BRAIN_CORAL_FAN);
        golden.field("DEAD_BUBBLE_CORAL_FAN", Blocks::DEAD_BUBBLE_CORAL_FAN);
        golden.field("DEAD_FIRE_CORAL_FAN", Blocks::DEAD_FIRE_CORAL_FAN);
        golden.field("DEAD_HORN_CORAL_FAN", Blocks::DEAD_HORN_CORAL_FAN);
        golden.field("TUBE_CORAL_FAN", Blocks::TUBE_CORAL_FAN);
        golden.field("BRAIN_CORAL_FAN", Blocks::BRAIN_CORAL_FAN);
        golden.field("BUBBLE_CORAL_FAN", Blocks::BUBBLE_CORAL_FAN);
        golden.field("FIRE_CORAL_FAN", Blocks::FIRE_CORAL_FAN);
        golden.field("HORN_CORAL_FAN", Blocks::HORN_CORAL_FAN);
        golden.field("DEAD_TUBE_CORAL_WALL_FAN", Blocks::DEAD_TUBE_CORAL_WALL_FAN);
        golden.field(
            "DEAD_BRAIN_CORAL_WALL_FAN",
            Blocks::DEAD_BRAIN_CORAL_WALL_FAN,
        );
        golden.field(
            "DEAD_BUBBLE_CORAL_WALL_FAN",
            Blocks::DEAD_BUBBLE_CORAL_WALL_FAN,
        );
        golden.field("DEAD_FIRE_CORAL_WALL_FAN", Blocks::DEAD_FIRE_CORAL_WALL_FAN);
        golden.field("DEAD_HORN_CORAL_WALL_FAN", Blocks::DEAD_HORN_CORAL_WALL_FAN);
        golden.field("TUBE_CORAL_WALL_FAN", Blocks::TUBE_CORAL_WALL_FAN);
        golden.field("BRAIN_CORAL_WALL_FAN", Blocks::BRAIN_CORAL_WALL_FAN);
        golden.field("BUBBLE_CORAL_WALL_FAN", Blocks::BUBBLE_CORAL_WALL_FAN);
        golden.field("FIRE_CORAL_WALL_FAN", Blocks::FIRE_CORAL_WALL_FAN);
        golden.field("HORN_CORAL_WALL_FAN", Blocks::HORN_CORAL_WALL_FAN);
        golden.field("SEA_PICKLE", Blocks::SEA_PICKLE);
        golden.field("BLUE_ICE", Blocks::BLUE_ICE);
        golden.field("CONDUIT", Blocks::CONDUIT);
        golden.field("BAMBOO_SAPLING", Blocks::BAMBOO_SAPLING);
        golden.field("BAMBOO", Blocks::BAMBOO);
        golden.field("POTTED_BAMBOO", Blocks::POTTED_BAMBOO);
        golden.field("VOID_AIR", Blocks::VOID_AIR);
        golden.field("CAVE_AIR", Blocks::CAVE_AIR);
        golden.field("BUBBLE_COLUMN", Blocks::BUBBLE_COLUMN);
        golden.field("POLISHED_GRANITE_STAIRS", Blocks::POLISHED_GRANITE_STAIRS);
        golden.field(
            "SMOOTH_RED_SANDSTONE_STAIRS",
            Blocks::SMOOTH_RED_SANDSTONE_STAIRS,
        );
        golden.field("MOSSY_STONE_BRICK_STAIRS", Blocks::MOSSY_STONE_BRICK_STAIRS);
        golden.field("POLISHED_DIORITE_STAIRS", Blocks::POLISHED_DIORITE_STAIRS);
        golden.field("MOSSY_COBBLESTONE_STAIRS", Blocks::MOSSY_COBBLESTONE_STAIRS);
        golden.field("END_STONE_BRICK_STAIRS", Blocks::END_STONE_BRICK_STAIRS);
        golden.field("STONE_STAIRS", Blocks::STONE_STAIRS);
        golden.field("SMOOTH_SANDSTONE_STAIRS", Blocks::SMOOTH_SANDSTONE_STAIRS);
        golden.field("SMOOTH_QUARTZ_STAIRS", Blocks::SMOOTH_QUARTZ_STAIRS);
        golden.field("GRANITE_STAIRS", Blocks::GRANITE_STAIRS);
        golden.field("ANDESITE_STAIRS", Blocks::ANDESITE_STAIRS);
        golden.field("RED_NETHER_BRICK_STAIRS", Blocks::RED_NETHER_BRICK_STAIRS);
        golden.field("POLISHED_ANDESITE_STAIRS", Blocks::POLISHED_ANDESITE_STAIRS);
        golden.field("DIORITE_STAIRS", Blocks::DIORITE_STAIRS);
        golden.field("POLISHED_GRANITE_SLAB", Blocks::POLISHED_GRANITE_SLAB);
        golden.field(
            "SMOOTH_RED_SANDSTONE_SLAB",
            Blocks::SMOOTH_RED_SANDSTONE_SLAB,
        );
        golden.field("MOSSY_STONE_BRICK_SLAB", Blocks::MOSSY_STONE_BRICK_SLAB);
        golden.field("POLISHED_DIORITE_SLAB", Blocks::POLISHED_DIORITE_SLAB);
        golden.field("MOSSY_COBBLESTONE_SLAB", Blocks::MOSSY_COBBLESTONE_SLAB);
        golden.field("END_STONE_BRICK_SLAB", Blocks::END_STONE_BRICK_SLAB);
        golden.field("SMOOTH_SANDSTONE_SLAB", Blocks::SMOOTH_SANDSTONE_SLAB);
        golden.field("SMOOTH_QUARTZ_SLAB", Blocks::SMOOTH_QUARTZ_SLAB);
        golden.field("GRANITE_SLAB", Blocks::GRANITE_SLAB);
        golden.field("ANDESITE_SLAB", Blocks::ANDESITE_SLAB);
        golden.field("RED_NETHER_BRICK_SLAB", Blocks::RED_NETHER_BRICK_SLAB);
        golden.field("POLISHED_ANDESITE_SLAB", Blocks::POLISHED_ANDESITE_SLAB);
        golden.field("DIORITE_SLAB", Blocks::DIORITE_SLAB);
        golden.field("BRICK_WALL", Blocks::BRICK_WALL);
        golden.field("PRISMARINE_WALL", Blocks::PRISMARINE_WALL);
        golden.field("RED_SANDSTONE_WALL", Blocks::RED_SANDSTONE_WALL);
        golden.field("MOSSY_STONE_BRICK_WALL", Blocks::MOSSY_STONE_BRICK_WALL);
        golden.field("GRANITE_WALL", Blocks::GRANITE_WALL);
        golden.field("STONE_BRICK_WALL", Blocks::STONE_BRICK_WALL);
        golden.field("MUD_BRICK_WALL", Blocks::MUD_BRICK_WALL);
        golden.field("NETHER_BRICK_WALL", Blocks::NETHER_BRICK_WALL);
        golden.field("ANDESITE_WALL", Blocks::ANDESITE_WALL);
        golden.field("RED_NETHER_BRICK_WALL", Blocks::RED_NETHER_BRICK_WALL);
        golden.field("SANDSTONE_WALL", Blocks::SANDSTONE_WALL);
        golden.field("END_STONE_BRICK_WALL", Blocks::END_STONE_BRICK_WALL);
        golden.field("DIORITE_WALL", Blocks::DIORITE_WALL);
        golden.field("SCAFFOLDING", Blocks::SCAFFOLDING);
        golden.field("LOOM", Blocks::LOOM);
        golden.field("BARREL", Blocks::BARREL);
        golden.field("SMOKER", Blocks::SMOKER);
        golden.field("BLAST_FURNACE", Blocks::BLAST_FURNACE);
        golden.field("CARTOGRAPHY_TABLE", Blocks::CARTOGRAPHY_TABLE);
        golden.field("FLETCHING_TABLE", Blocks::FLETCHING_TABLE);
        golden.field("GRINDSTONE", Blocks::GRINDSTONE);
        golden.field("LECTERN", Blocks::LECTERN);
        golden.field("SMITHING_TABLE", Blocks::SMITHING_TABLE);
        golden.field("STONECUTTER", Blocks::STONECUTTER);
        golden.field("BELL", Blocks::BELL);
        golden.field("LANTERN", Blocks::LANTERN);
        golden.field("SOUL_LANTERN", Blocks::SOUL_LANTERN);
        golden.field("COPPER_LANTERN", Blocks::COPPER_LANTERN);
        golden.field("CAMPFIRE", Blocks::CAMPFIRE);
        golden.field("SOUL_CAMPFIRE", Blocks::SOUL_CAMPFIRE);
        golden.field("SWEET_BERRY_BUSH", Blocks::SWEET_BERRY_BUSH);
        golden.field("WARPED_STEM", Blocks::WARPED_STEM);
        golden.field("STRIPPED_WARPED_STEM", Blocks::STRIPPED_WARPED_STEM);
        golden.field("WARPED_HYPHAE", Blocks::WARPED_HYPHAE);
        golden.field("STRIPPED_WARPED_HYPHAE", Blocks::STRIPPED_WARPED_HYPHAE);
        golden.field("WARPED_NYLIUM", Blocks::WARPED_NYLIUM);
        golden.field("WARPED_FUNGUS", Blocks::WARPED_FUNGUS);
        golden.field("WARPED_WART_BLOCK", Blocks::WARPED_WART_BLOCK);
        golden.field("WARPED_ROOTS", Blocks::WARPED_ROOTS);
        golden.field("NETHER_SPROUTS", Blocks::NETHER_SPROUTS);
        golden.field("CRIMSON_STEM", Blocks::CRIMSON_STEM);
        golden.field("STRIPPED_CRIMSON_STEM", Blocks::STRIPPED_CRIMSON_STEM);
        golden.field("CRIMSON_HYPHAE", Blocks::CRIMSON_HYPHAE);
        golden.field("STRIPPED_CRIMSON_HYPHAE", Blocks::STRIPPED_CRIMSON_HYPHAE);
        golden.field("CRIMSON_NYLIUM", Blocks::CRIMSON_NYLIUM);
        golden.field("CRIMSON_FUNGUS", Blocks::CRIMSON_FUNGUS);
        golden.field("SHROOMLIGHT", Blocks::SHROOMLIGHT);
        golden.field("WEEPING_VINES", Blocks::WEEPING_VINES);
        golden.field("WEEPING_VINES_PLANT", Blocks::WEEPING_VINES_PLANT);
        golden.field("TWISTING_VINES", Blocks::TWISTING_VINES);
        golden.field("TWISTING_VINES_PLANT", Blocks::TWISTING_VINES_PLANT);
        golden.field("CRIMSON_ROOTS", Blocks::CRIMSON_ROOTS);
        golden.field("CRIMSON_PLANKS", Blocks::CRIMSON_PLANKS);
        golden.field("WARPED_PLANKS", Blocks::WARPED_PLANKS);
        golden.field("CRIMSON_SLAB", Blocks::CRIMSON_SLAB);
        golden.field("WARPED_SLAB", Blocks::WARPED_SLAB);
        golden.field("CRIMSON_PRESSURE_PLATE", Blocks::CRIMSON_PRESSURE_PLATE);
        golden.field("WARPED_PRESSURE_PLATE", Blocks::WARPED_PRESSURE_PLATE);
        golden.field("CRIMSON_FENCE", Blocks::CRIMSON_FENCE);
        golden.field("WARPED_FENCE", Blocks::WARPED_FENCE);
        golden.field("CRIMSON_TRAPDOOR", Blocks::CRIMSON_TRAPDOOR);
        golden.field("WARPED_TRAPDOOR", Blocks::WARPED_TRAPDOOR);
        golden.field("CRIMSON_FENCE_GATE", Blocks::CRIMSON_FENCE_GATE);
        golden.field("WARPED_FENCE_GATE", Blocks::WARPED_FENCE_GATE);
        golden.field("CRIMSON_STAIRS", Blocks::CRIMSON_STAIRS);
        golden.field("WARPED_STAIRS", Blocks::WARPED_STAIRS);
        golden.field("CRIMSON_BUTTON", Blocks::CRIMSON_BUTTON);
        golden.field("WARPED_BUTTON", Blocks::WARPED_BUTTON);
        golden.field("CRIMSON_DOOR", Blocks::CRIMSON_DOOR);
        golden.field("WARPED_DOOR", Blocks::WARPED_DOOR);
        golden.field("CRIMSON_SIGN", Blocks::CRIMSON_SIGN);
        golden.field("WARPED_SIGN", Blocks::WARPED_SIGN);
        golden.field("CRIMSON_WALL_SIGN", Blocks::CRIMSON_WALL_SIGN);
        golden.field("WARPED_WALL_SIGN", Blocks::WARPED_WALL_SIGN);
        golden.field("STRUCTURE_BLOCK", Blocks::STRUCTURE_BLOCK);
        golden.field("JIGSAW", Blocks::JIGSAW);
        golden.field("TEST_BLOCK", Blocks::TEST_BLOCK);
        golden.field("TEST_INSTANCE_BLOCK", Blocks::TEST_INSTANCE_BLOCK);
        golden.field("COMPOSTER", Blocks::COMPOSTER);
        golden.field("TARGET", Blocks::TARGET);
        golden.field("BEE_NEST", Blocks::BEE_NEST);
        golden.field("BEEHIVE", Blocks::BEEHIVE);
        golden.field("HONEY_BLOCK", Blocks::HONEY_BLOCK);
        golden.field("HONEYCOMB_BLOCK", Blocks::HONEYCOMB_BLOCK);
        golden.field("NETHERITE_BLOCK", Blocks::NETHERITE_BLOCK);
        golden.field("ANCIENT_DEBRIS", Blocks::ANCIENT_DEBRIS);
        golden.field("CRYING_OBSIDIAN", Blocks::CRYING_OBSIDIAN);
        golden.field("RESPAWN_ANCHOR", Blocks::RESPAWN_ANCHOR);
        golden.field("POTTED_CRIMSON_FUNGUS", Blocks::POTTED_CRIMSON_FUNGUS);
        golden.field("POTTED_WARPED_FUNGUS", Blocks::POTTED_WARPED_FUNGUS);
        golden.field("POTTED_CRIMSON_ROOTS", Blocks::POTTED_CRIMSON_ROOTS);
        golden.field("POTTED_WARPED_ROOTS", Blocks::POTTED_WARPED_ROOTS);
        golden.field("LODESTONE", Blocks::LODESTONE);
        golden.field("BLACKSTONE", Blocks::BLACKSTONE);
        golden.field("BLACKSTONE_STAIRS", Blocks::BLACKSTONE_STAIRS);
        golden.field("BLACKSTONE_WALL", Blocks::BLACKSTONE_WALL);
        golden.field("BLACKSTONE_SLAB", Blocks::BLACKSTONE_SLAB);
        golden.field("POLISHED_BLACKSTONE", Blocks::POLISHED_BLACKSTONE);
        golden.field(
            "POLISHED_BLACKSTONE_BRICKS",
            Blocks::POLISHED_BLACKSTONE_BRICKS,
        );
        golden.field(
            "CRACKED_POLISHED_BLACKSTONE_BRICKS",
            Blocks::CRACKED_POLISHED_BLACKSTONE_BRICKS,
        );
        golden.field(
            "CHISELED_POLISHED_BLACKSTONE",
            Blocks::CHISELED_POLISHED_BLACKSTONE,
        );
        golden.field(
            "POLISHED_BLACKSTONE_BRICK_SLAB",
            Blocks::POLISHED_BLACKSTONE_BRICK_SLAB,
        );
        golden.field(
            "POLISHED_BLACKSTONE_BRICK_STAIRS",
            Blocks::POLISHED_BLACKSTONE_BRICK_STAIRS,
        );
        golden.field(
            "POLISHED_BLACKSTONE_BRICK_WALL",
            Blocks::POLISHED_BLACKSTONE_BRICK_WALL,
        );
        golden.field("GILDED_BLACKSTONE", Blocks::GILDED_BLACKSTONE);
        golden.field(
            "POLISHED_BLACKSTONE_STAIRS",
            Blocks::POLISHED_BLACKSTONE_STAIRS,
        );
        golden.field("POLISHED_BLACKSTONE_SLAB", Blocks::POLISHED_BLACKSTONE_SLAB);
        golden.field(
            "POLISHED_BLACKSTONE_PRESSURE_PLATE",
            Blocks::POLISHED_BLACKSTONE_PRESSURE_PLATE,
        );
        golden.field(
            "POLISHED_BLACKSTONE_BUTTON",
            Blocks::POLISHED_BLACKSTONE_BUTTON,
        );
        golden.field("POLISHED_BLACKSTONE_WALL", Blocks::POLISHED_BLACKSTONE_WALL);
        golden.field("CHISELED_NETHER_BRICKS", Blocks::CHISELED_NETHER_BRICKS);
        golden.field("CRACKED_NETHER_BRICKS", Blocks::CRACKED_NETHER_BRICKS);
        golden.field("QUARTZ_BRICKS", Blocks::QUARTZ_BRICKS);
        golden.field("CANDLE", Blocks::CANDLE);
        golden.field("DYED_CANDLE", Blocks::DYED_CANDLE);
        golden.field("CANDLE_CAKE", Blocks::CANDLE_CAKE);
        golden.field("DYED_CANDLE_CAKE", Blocks::DYED_CANDLE_CAKE);
        golden.field("AMETHYST_BLOCK", Blocks::AMETHYST_BLOCK);
        golden.field("BUDDING_AMETHYST", Blocks::BUDDING_AMETHYST);
        golden.field("AMETHYST_CLUSTER", Blocks::AMETHYST_CLUSTER);
        golden.field("LARGE_AMETHYST_BUD", Blocks::LARGE_AMETHYST_BUD);
        golden.field("MEDIUM_AMETHYST_BUD", Blocks::MEDIUM_AMETHYST_BUD);
        golden.field("SMALL_AMETHYST_BUD", Blocks::SMALL_AMETHYST_BUD);
        golden.field("TUFF", Blocks::TUFF);
        golden.field("TUFF_SLAB", Blocks::TUFF_SLAB);
        golden.field("TUFF_STAIRS", Blocks::TUFF_STAIRS);
        golden.field("TUFF_WALL", Blocks::TUFF_WALL);
        golden.field("POLISHED_TUFF", Blocks::POLISHED_TUFF);
        golden.field("POLISHED_TUFF_SLAB", Blocks::POLISHED_TUFF_SLAB);
        golden.field("POLISHED_TUFF_STAIRS", Blocks::POLISHED_TUFF_STAIRS);
        golden.field("POLISHED_TUFF_WALL", Blocks::POLISHED_TUFF_WALL);
        golden.field("CHISELED_TUFF", Blocks::CHISELED_TUFF);
        golden.field("TUFF_BRICKS", Blocks::TUFF_BRICKS);
        golden.field("TUFF_BRICK_SLAB", Blocks::TUFF_BRICK_SLAB);
        golden.field("TUFF_BRICK_STAIRS", Blocks::TUFF_BRICK_STAIRS);
        golden.field("TUFF_BRICK_WALL", Blocks::TUFF_BRICK_WALL);
        golden.field("CHISELED_TUFF_BRICKS", Blocks::CHISELED_TUFF_BRICKS);
        golden.field("SULFUR", Blocks::SULFUR);
        golden.field("POTENT_SULFUR", Blocks::POTENT_SULFUR);
        golden.field("SULFUR_SLAB", Blocks::SULFUR_SLAB);
        golden.field("SULFUR_STAIRS", Blocks::SULFUR_STAIRS);
        golden.field("SULFUR_WALL", Blocks::SULFUR_WALL);
        golden.field("POLISHED_SULFUR", Blocks::POLISHED_SULFUR);
        golden.field("POLISHED_SULFUR_SLAB", Blocks::POLISHED_SULFUR_SLAB);
        golden.field("POLISHED_SULFUR_STAIRS", Blocks::POLISHED_SULFUR_STAIRS);
        golden.field("POLISHED_SULFUR_WALL", Blocks::POLISHED_SULFUR_WALL);
        golden.field("SULFUR_BRICKS", Blocks::SULFUR_BRICKS);
        golden.field("SULFUR_BRICK_SLAB", Blocks::SULFUR_BRICK_SLAB);
        golden.field("SULFUR_BRICK_STAIRS", Blocks::SULFUR_BRICK_STAIRS);
        golden.field("SULFUR_BRICK_WALL", Blocks::SULFUR_BRICK_WALL);
        golden.field("CHISELED_SULFUR", Blocks::CHISELED_SULFUR);
        golden.field("CINNABAR", Blocks::CINNABAR);
        golden.field("CINNABAR_SLAB", Blocks::CINNABAR_SLAB);
        golden.field("CINNABAR_STAIRS", Blocks::CINNABAR_STAIRS);
        golden.field("CINNABAR_WALL", Blocks::CINNABAR_WALL);
        golden.field("POLISHED_CINNABAR", Blocks::POLISHED_CINNABAR);
        golden.field("POLISHED_CINNABAR_SLAB", Blocks::POLISHED_CINNABAR_SLAB);
        golden.field("POLISHED_CINNABAR_STAIRS", Blocks::POLISHED_CINNABAR_STAIRS);
        golden.field("POLISHED_CINNABAR_WALL", Blocks::POLISHED_CINNABAR_WALL);
        golden.field("CINNABAR_BRICKS", Blocks::CINNABAR_BRICKS);
        golden.field("CINNABAR_BRICK_SLAB", Blocks::CINNABAR_BRICK_SLAB);
        golden.field("CINNABAR_BRICK_STAIRS", Blocks::CINNABAR_BRICK_STAIRS);
        golden.field("CINNABAR_BRICK_WALL", Blocks::CINNABAR_BRICK_WALL);
        golden.field("CHISELED_CINNABAR", Blocks::CHISELED_CINNABAR);
        golden.field("CALCITE", Blocks::CALCITE);
        golden.field("TINTED_GLASS", Blocks::TINTED_GLASS);
        golden.field("POWDER_SNOW", Blocks::POWDER_SNOW);
        golden.field("SCULK_SENSOR", Blocks::SCULK_SENSOR);
        golden.field("CALIBRATED_SCULK_SENSOR", Blocks::CALIBRATED_SCULK_SENSOR);
        golden.field("SCULK", Blocks::SCULK);
        golden.field("SCULK_VEIN", Blocks::SCULK_VEIN);
        golden.field("SCULK_CATALYST", Blocks::SCULK_CATALYST);
        golden.field("SCULK_SHRIEKER", Blocks::SCULK_SHRIEKER);
        golden.field("COPPER_BLOCK", Blocks::COPPER_BLOCK);
        golden.field("COPPER_ORE", Blocks::COPPER_ORE);
        golden.field("DEEPSLATE_COPPER_ORE", Blocks::DEEPSLATE_COPPER_ORE);
        golden.field("CUT_COPPER", Blocks::CUT_COPPER);
        golden.field("CHISELED_COPPER", Blocks::CHISELED_COPPER);
        golden.field("CUT_COPPER_STAIRS", Blocks::CUT_COPPER_STAIRS);
        golden.field("CUT_COPPER_SLAB", Blocks::CUT_COPPER_SLAB);
        golden.field("COPPER_DOOR", Blocks::COPPER_DOOR);
        golden.field("COPPER_TRAPDOOR", Blocks::COPPER_TRAPDOOR);
        golden.field("COPPER_GRATE", Blocks::COPPER_GRATE);
        golden.field("COPPER_BULB", Blocks::COPPER_BULB);
        golden.field("COPPER_CHEST", Blocks::COPPER_CHEST);
        golden.field("COPPER_GOLEM_STATUE", Blocks::COPPER_GOLEM_STATUE);
        golden.field("LIGHTNING_ROD", Blocks::LIGHTNING_ROD);
        golden.field("DRIPSTONE_BLOCK", Blocks::DRIPSTONE_BLOCK);
        golden.field("POINTED_DRIPSTONE", Blocks::POINTED_DRIPSTONE);
        golden.field("SULFUR_SPIKE", Blocks::SULFUR_SPIKE);
        golden.field("CAVE_VINES", Blocks::CAVE_VINES);
        golden.field("CAVE_VINES_PLANT", Blocks::CAVE_VINES_PLANT);
        golden.field("SPORE_BLOSSOM", Blocks::SPORE_BLOSSOM);
        golden.field("AZALEA", Blocks::AZALEA);
        golden.field("FLOWERING_AZALEA", Blocks::FLOWERING_AZALEA);
        golden.field("MOSS_CARPET", Blocks::MOSS_CARPET);
        golden.field("PINK_PETALS", Blocks::PINK_PETALS);
        golden.field("WILDFLOWERS", Blocks::WILDFLOWERS);
        golden.field("LEAF_LITTER", Blocks::LEAF_LITTER);
        golden.field("MOSS_BLOCK", Blocks::MOSS_BLOCK);
        golden.field("BIG_DRIPLEAF", Blocks::BIG_DRIPLEAF);
        golden.field("BIG_DRIPLEAF_STEM", Blocks::BIG_DRIPLEAF_STEM);
        golden.field("SMALL_DRIPLEAF", Blocks::SMALL_DRIPLEAF);
        golden.field("HANGING_ROOTS", Blocks::HANGING_ROOTS);
        golden.field("ROOTED_DIRT", Blocks::ROOTED_DIRT);
        golden.field("MUD", Blocks::MUD);
        golden.field("DEEPSLATE", Blocks::DEEPSLATE);
        golden.field("COBBLED_DEEPSLATE", Blocks::COBBLED_DEEPSLATE);
        golden.field("COBBLED_DEEPSLATE_STAIRS", Blocks::COBBLED_DEEPSLATE_STAIRS);
        golden.field("COBBLED_DEEPSLATE_SLAB", Blocks::COBBLED_DEEPSLATE_SLAB);
        golden.field("COBBLED_DEEPSLATE_WALL", Blocks::COBBLED_DEEPSLATE_WALL);
        golden.field("POLISHED_DEEPSLATE", Blocks::POLISHED_DEEPSLATE);
        golden.field(
            "POLISHED_DEEPSLATE_STAIRS",
            Blocks::POLISHED_DEEPSLATE_STAIRS,
        );
        golden.field("POLISHED_DEEPSLATE_SLAB", Blocks::POLISHED_DEEPSLATE_SLAB);
        golden.field("POLISHED_DEEPSLATE_WALL", Blocks::POLISHED_DEEPSLATE_WALL);
        golden.field("DEEPSLATE_TILES", Blocks::DEEPSLATE_TILES);
        golden.field("DEEPSLATE_TILE_STAIRS", Blocks::DEEPSLATE_TILE_STAIRS);
        golden.field("DEEPSLATE_TILE_SLAB", Blocks::DEEPSLATE_TILE_SLAB);
        golden.field("DEEPSLATE_TILE_WALL", Blocks::DEEPSLATE_TILE_WALL);
        golden.field("DEEPSLATE_BRICKS", Blocks::DEEPSLATE_BRICKS);
        golden.field("DEEPSLATE_BRICK_STAIRS", Blocks::DEEPSLATE_BRICK_STAIRS);
        golden.field("DEEPSLATE_BRICK_SLAB", Blocks::DEEPSLATE_BRICK_SLAB);
        golden.field("DEEPSLATE_BRICK_WALL", Blocks::DEEPSLATE_BRICK_WALL);
        golden.field("CHISELED_DEEPSLATE", Blocks::CHISELED_DEEPSLATE);
        golden.field("CRACKED_DEEPSLATE_BRICKS", Blocks::CRACKED_DEEPSLATE_BRICKS);
        golden.field("CRACKED_DEEPSLATE_TILES", Blocks::CRACKED_DEEPSLATE_TILES);
        golden.field("INFESTED_DEEPSLATE", Blocks::INFESTED_DEEPSLATE);
        golden.field("SMOOTH_BASALT", Blocks::SMOOTH_BASALT);
        golden.field("RAW_IRON_BLOCK", Blocks::RAW_IRON_BLOCK);
        golden.field("RAW_COPPER_BLOCK", Blocks::RAW_COPPER_BLOCK);
        golden.field("RAW_GOLD_BLOCK", Blocks::RAW_GOLD_BLOCK);
        golden.field("POTTED_AZALEA", Blocks::POTTED_AZALEA);
        golden.field("POTTED_FLOWERING_AZALEA", Blocks::POTTED_FLOWERING_AZALEA);
        golden.field("OCHRE_FROGLIGHT", Blocks::OCHRE_FROGLIGHT);
        golden.field("VERDANT_FROGLIGHT", Blocks::VERDANT_FROGLIGHT);
        golden.field("PEARLESCENT_FROGLIGHT", Blocks::PEARLESCENT_FROGLIGHT);
        golden.field("FROGSPAWN", Blocks::FROGSPAWN);
        golden.field("REINFORCED_DEEPSLATE", Blocks::REINFORCED_DEEPSLATE);
        golden.field("DECORATED_POT", Blocks::DECORATED_POT);
        golden.field("CRAFTER", Blocks::CRAFTER);
        golden.field("TRIAL_SPAWNER", Blocks::TRIAL_SPAWNER);
        golden.field("VAULT", Blocks::VAULT);
        golden.field("HEAVY_CORE", Blocks::HEAVY_CORE);
        golden.field("PALE_MOSS_BLOCK", Blocks::PALE_MOSS_BLOCK);
        golden.field("PALE_MOSS_CARPET", Blocks::PALE_MOSS_CARPET);
        golden.field("PALE_HANGING_MOSS", Blocks::PALE_HANGING_MOSS);
        golden.field("OPEN_EYEBLOSSOM", Blocks::OPEN_EYEBLOSSOM);
        golden.field("CLOSED_EYEBLOSSOM", Blocks::CLOSED_EYEBLOSSOM);
        golden.field("POTTED_OPEN_EYEBLOSSOM", Blocks::POTTED_OPEN_EYEBLOSSOM);
        golden.field("POTTED_CLOSED_EYEBLOSSOM", Blocks::POTTED_CLOSED_EYEBLOSSOM);
        golden.field("FIREFLY_BUSH", Blocks::FIREFLY_BUSH);
        golden.finish();
    }

    /// The data generator's report, `generated/reports/blocks.json`, agrees: each block's states
    /// by id, with their properties' values by name, and the default one.
    #[test]
    fn matches_the_blocks_report() {
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("vanilla")
            .join(include_str!("../../../../../../MINECRAFT_VERSION").trim())
            .join("generated/reports/blocks.json");
        let report: HashMap<String, Value> = serde_json::from_str(
            &fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("can't read {}: {error}", path.display())),
        )
        .expect("the report is a JSON object");
        assert_eq!(report.len(), registry().size(), "blocks");
        let mut states = 0;
        for block in registry().iter() {
            let name = registry().get_key(block).to_string();
            let entry = &report[&name];
            let definition = block.get_state_definition();
            let properties: Vec<(String, Vec<String>)> = definition
                .get_properties()
                .iter()
                .map(|property| {
                    let values = property
                        .get_possible_values()
                        .into_iter()
                        .map(|value| property.get_name_of(value))
                        .collect();
                    (property.get_name().to_owned(), values)
                })
                .collect();
            let reported_properties: Vec<(String, Vec<String>)> = entry
                .get("properties")
                .and_then(Value::as_object)
                .map(|object| {
                    object
                        .iter()
                        .map(|(name, values)| {
                            let values = values
                                .as_array()
                                .unwrap()
                                .iter()
                                .map(|value| value.as_str().unwrap().to_owned())
                                .collect();
                            (name.clone(), values)
                        })
                        .collect()
                })
                .unwrap_or_default();
            let mut sorted_report = reported_properties.clone();
            sorted_report.sort();
            let mut sorted_port = properties.clone();
            sorted_port.sort();
            assert_eq!(sorted_port, sorted_report, "{name}'s properties");
            let reported_states = entry["states"].as_array().unwrap();
            assert_eq!(
                reported_states.len(),
                definition.state_count() as usize,
                "{name}'s states"
            );
            for (state, reported) in definition.get_possible_states().zip(reported_states) {
                assert_eq!(
                    i64::from(state.id()),
                    reported["id"].as_i64().unwrap(),
                    "{state}'s id"
                );
                for (property, value) in state.get_values() {
                    assert_eq!(
                        reported["properties"][property.get_name()].as_str(),
                        Some(property.get_name_of(value).as_str()),
                        "{state}'s {}",
                        property.get_name()
                    );
                }
                let is_default = reported
                    .get("default")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                assert_eq!(
                    state == block.default_block_state(),
                    is_default,
                    "whether {state} is the default"
                );
                states += 1;
            }
        }
        assert_eq!(states, Block::block_state_registry().size(), "states");
        assert_eq!(Blocks::AIR.default_block_state().id(), 0);
    }
}
