//! net/minecraft/core/registries/Registries.java

use std::sync::LazyLock;

use crate::net::minecraft::resources::identifier::Identifier;
use crate::net::minecraft::resources::resource_key::ResourceKey;

static ROOT_REGISTRY_NAME: LazyLock<Identifier> =
    LazyLock::new(|| Identifier::with_default_namespace("root"));
static ACTIVITY: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("activity"));
static ATTRIBUTE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("attribute"));
static BIOME_SOURCE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("worldgen/biome_source"));
static BLOCK_ENTITY_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("block_entity_type"));
static BLOCK_PREDICATE_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("block_predicate_type"));
static BLOCK_STATE_PROVIDER_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("worldgen/block_state_provider_type"));
static BLOCK: LazyLock<ResourceKey> = LazyLock::new(|| Registries::create_registry_key("block"));
static CARVER_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("worldgen/carver_type"));
static CHUNK_GENERATOR: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("worldgen/chunk_generator"));
static CHUNK_STATUS: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("chunk_status"));
static COMMAND_ARGUMENT_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("command_argument_type"));
static CONSUME_EFFECT_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("consume_effect_type"));
static CREATIVE_MODE_TAB: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("creative_mode_tab"));
static CUSTOM_STAT: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("custom_stat"));
static DATA_COMPONENT_PREDICATE_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("data_component_predicate_type"));
static DATA_COMPONENT_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("data_component_type"));
static GAME_RULE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("game_rule"));
static DEBUG_SUBSCRIPTION: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("debug_subscription"));
static DENSITY_FUNCTION_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("worldgen/density_function_type"));
static DIALOG_BODY_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("dialog_body_type"));
static DIALOG_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("dialog_type"));
static ENCHANTMENT_EFFECT_COMPONENT_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("enchantment_effect_component_type"));
static ENCHANTMENT_ENTITY_EFFECT_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("enchantment_entity_effect_type"));
static ENCHANTMENT_LEVEL_BASED_VALUE_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("enchantment_level_based_value_type"));
static ENCHANTMENT_LOCATION_BASED_EFFECT_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("enchantment_location_based_effect_type"));
static ENCHANTMENT_PROVIDER_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("enchantment_provider_type"));
static ENCHANTMENT_VALUE_EFFECT_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("enchantment_value_effect_type"));
static ENTITY_SUB_PREDICATE_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("entity_sub_predicate_type"));
static ENTITY_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("entity_type"));
static ENVIRONMENT_ATTRIBUTE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("environment_attribute"));
static ATTRIBUTE_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("attribute_type"));
static FEATURE_SIZE_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("worldgen/feature_size_type"));
static FEATURE_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("worldgen/feature_type"));
static FLOAT_PROVIDER_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("float_provider_type"));
static FLUID: LazyLock<ResourceKey> = LazyLock::new(|| Registries::create_registry_key("fluid"));
static FOLIAGE_PLACER_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("worldgen/foliage_placer_type"));
static GAME_EVENT: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("game_event"));
static HEIGHT_PROVIDER_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("height_provider_type"));
static INPUT_CONTROL_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("input_control_type"));
static INT_PROVIDER_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("int_provider_type"));
static ITEM: LazyLock<ResourceKey> = LazyLock::new(|| Registries::create_registry_key("item"));
static SLOT_SOURCE_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("slot_source_type"));
static LOOT_CONDITION_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("loot_condition_type"));
static LOOT_FUNCTION_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("loot_function_type"));
static LOOT_NBT_PROVIDER_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("loot_nbt_provider_type"));
static CONTEXT_FLOAT_PROVIDER_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("context_float_provider_type"));
static CONTEXT_INT_PROVIDER_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("context_int_provider_type"));
static LOOT_POOL_ENTRY_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("loot_pool_entry_type"));
static LOOT_SCORE_PROVIDER_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("loot_score_provider_type"));
static MAP_DECORATION_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("map_decoration_type"));
static MATERIAL_CONDITION_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("worldgen/material_condition_type"));
static MATERIAL_RULE_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("worldgen/material_rule_type"));
static MEMORY_MODULE_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("memory_module_type"));
static MENU: LazyLock<ResourceKey> = LazyLock::new(|| Registries::create_registry_key("menu"));
static MOB_EFFECT: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("mob_effect"));
static NUMBER_FORMAT_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("number_format_type"));
static PARTICLE_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("particle_type"));
static PLACEMENT_MODIFIER_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("worldgen/placement_modifier_type"));
static POINT_OF_INTEREST_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("point_of_interest_type"));
static POOL_ALIAS_BINDING: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("worldgen/pool_alias_binding"));
static POSITION_SOURCE_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("position_source_type"));
static POS_RULE_TEST: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("pos_rule_test"));
static POTION: LazyLock<ResourceKey> = LazyLock::new(|| Registries::create_registry_key("potion"));
static RECIPE_BOOK_CATEGORY: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("recipe_book_category"));
static RECIPE_DISPLAY: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("recipe_display"));
static RECIPE_SERIALIZER: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("recipe_serializer"));
static RECIPE_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("recipe_type"));
static ROOT_PLACER_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("worldgen/root_placer_type"));
static RULE_BLOCK_ENTITY_MODIFIER: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("rule_block_entity_modifier"));
static RULE_TEST: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("rule_test"));
static SENSOR_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("sensor_type"));
static SLOT_DISPLAY: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("slot_display"));
static SOUND_EVENT: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("sound_event"));
static SPAWN_CONDITION_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("spawn_condition_type"));
static STAT_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("stat_type"));
static STRUCTURE_PIECE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("worldgen/structure_piece"));
static STRUCTURE_PLACEMENT: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("worldgen/structure_placement"));
static STRUCTURE_POOL_ELEMENT: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("worldgen/structure_pool_element"));
static STRUCTURE_PROCESSOR: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("worldgen/structure_processor"));
static STRUCTURE_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("worldgen/structure_type"));
static DIALOG_ACTION_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("dialog_action_type"));
static TEST_ENVIRONMENT_DEFINITION_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("test_environment_definition_type"));
static TEST_FUNCTION: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("test_function"));
static TEST_INSTANCE_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("test_instance_type"));
static TICKET_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("ticket_type"));
static TREE_DECORATOR_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("worldgen/tree_decorator_type"));
static TRUNK_PLACER_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("worldgen/trunk_placer_type"));
static VILLAGER_PROFESSION: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("villager_profession"));
static VILLAGER_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("villager_type"));
static INCOMING_RPC_METHOD: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("incoming_rpc_methods"));
static OUTGOING_RPC_METHOD: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("outgoing_rpc_methods"));
static PERMISSION_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("permission_type"));
static PERMISSION_CHECK_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("permission_check_type"));
static CONTEXT_KEY_SET: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("context_key_set"));
static BANNER_PATTERN: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("banner_pattern"));
static BIOME: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("worldgen/biome"));
static BLOCK_STATE_PROVIDER: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("worldgen/block_state_provider"));
static CAT_SOUND_VARIANT: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("cat_sound_variant"));
static CAT_VARIANT: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("cat_variant"));
static CARVER: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("worldgen/carver"));
static CHAT_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("chat_type"));
static CHICKEN_SOUND_VARIANT: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("chicken_sound_variant"));
static CHICKEN_VARIANT: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("chicken_variant"));
static ZOMBIE_NAUTILUS_VARIANT: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("zombie_nautilus_variant"));
static FEATURE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("worldgen/feature"));
static COW_SOUND_VARIANT: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("cow_sound_variant"));
static COW_VARIANT: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("cow_variant"));
static DAMAGE_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("damage_type"));
static DENSITY_FUNCTION: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("worldgen/density_function"));
static DIALOG: LazyLock<ResourceKey> = LazyLock::new(|| Registries::create_registry_key("dialog"));
static DIMENSION_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("dimension_type"));
static ENCHANTMENT_PROVIDER: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("enchantment_provider"));
static ENCHANTMENT: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("enchantment"));
static FLAT_LEVEL_GENERATOR_PRESET: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("worldgen/flat_level_generator_preset"));
static FROG_VARIANT: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("frog_variant"));
static INSTRUMENT: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("instrument"));
static JUKEBOX_SONG: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("jukebox_song"));
static MATERIAL_CONDITION: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("worldgen/material_condition"));
static MATERIAL_RULE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("worldgen/material_rule"));
static MULTI_NOISE_BIOME_SOURCE_PARAMETER_LIST: LazyLock<ResourceKey> = LazyLock::new(|| {
    Registries::create_registry_key("worldgen/multi_noise_biome_source_parameter_list")
});
static NOISE_SETTINGS: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("worldgen/noise_settings"));
static NOISE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("worldgen/noise"));
static PAINTING_VARIANT: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("painting_variant"));
static PIG_SOUND_VARIANT: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("pig_sound_variant"));
static PIG_VARIANT: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("pig_variant"));
static PLACED_FEATURE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("worldgen/placed_feature"));
static PROCESSOR_LIST: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("worldgen/processor_list"));
static STRUCTURE_SET: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("worldgen/structure_set"));
static STRUCTURE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("worldgen/structure"));
static SULFUR_CUBE_ARCHETYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("sulfur_cube_archetype"));
static TEMPLATE_POOL: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("worldgen/template_pool"));
static TEST_ENVIRONMENT: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("test_environment"));
static TEST_INSTANCE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("test_instance"));
static TIMELINE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("timeline"));
static TRADE_SET: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("trade_set"));
static TRIAL_SPAWNER_CONFIG: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("trial_spawner"));
static TRIGGER_TYPE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("trigger_type"));
static TRIM_MATERIAL: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("trim_material"));
static TRIM_PATTERN: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("trim_pattern"));
static VILLAGER_TRADE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("villager_trade"));
static WOLF_VARIANT: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("wolf_variant"));
static WOLF_SOUND_VARIANT: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("wolf_sound_variant"));
static WORLD_CLOCK: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("world_clock"));
static WORLD_PRESET: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("worldgen/world_preset"));
static DECORATED_POT_PATTERN: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("decorated_pot_pattern"));
static BLOCK_TRANSFORMER: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("block_transformer"));
static DIMENSION: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("dimension"));
static LEVEL_STEM: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("dimension"));
static LOOT_TABLE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("loot_table"));
static ITEM_MODIFIER: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("item_modifier"));
static PREDICATE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("predicate"));
static SLOT_SOURCE: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("slot_source"));
static CONTEXT_FLOAT_PROVIDER: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("context_float_provider"));
static CONTEXT_INT_PROVIDER: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("context_int_provider"));
static ADVANCEMENT: LazyLock<ResourceKey> =
    LazyLock::new(|| Registries::create_registry_key("advancement"));
static RECIPE: LazyLock<ResourceKey> = LazyLock::new(|| Registries::create_registry_key("recipe"));

/// The keys of the game's registries, each in the root registry. `DIMENSION` and `LEVEL_STEM` are
/// the same key: a level and the stem it's made from share their registry's name.
pub struct Registries;

impl Registries {
    pub const ROOT_REGISTRY_NAME: &'static LazyLock<Identifier> = &ROOT_REGISTRY_NAME;
    pub const ACTIVITY: &'static LazyLock<ResourceKey> = &ACTIVITY;
    pub const ATTRIBUTE: &'static LazyLock<ResourceKey> = &ATTRIBUTE;
    pub const BIOME_SOURCE: &'static LazyLock<ResourceKey> = &BIOME_SOURCE;
    pub const BLOCK_ENTITY_TYPE: &'static LazyLock<ResourceKey> = &BLOCK_ENTITY_TYPE;
    pub const BLOCK_PREDICATE_TYPE: &'static LazyLock<ResourceKey> = &BLOCK_PREDICATE_TYPE;
    pub const BLOCK_STATE_PROVIDER_TYPE: &'static LazyLock<ResourceKey> =
        &BLOCK_STATE_PROVIDER_TYPE;
    pub const BLOCK: &'static LazyLock<ResourceKey> = &BLOCK;
    pub const CARVER_TYPE: &'static LazyLock<ResourceKey> = &CARVER_TYPE;
    pub const CHUNK_GENERATOR: &'static LazyLock<ResourceKey> = &CHUNK_GENERATOR;
    pub const CHUNK_STATUS: &'static LazyLock<ResourceKey> = &CHUNK_STATUS;
    pub const COMMAND_ARGUMENT_TYPE: &'static LazyLock<ResourceKey> = &COMMAND_ARGUMENT_TYPE;
    pub const CONSUME_EFFECT_TYPE: &'static LazyLock<ResourceKey> = &CONSUME_EFFECT_TYPE;
    pub const CREATIVE_MODE_TAB: &'static LazyLock<ResourceKey> = &CREATIVE_MODE_TAB;
    pub const CUSTOM_STAT: &'static LazyLock<ResourceKey> = &CUSTOM_STAT;
    pub const DATA_COMPONENT_PREDICATE_TYPE: &'static LazyLock<ResourceKey> =
        &DATA_COMPONENT_PREDICATE_TYPE;
    pub const DATA_COMPONENT_TYPE: &'static LazyLock<ResourceKey> = &DATA_COMPONENT_TYPE;
    pub const GAME_RULE: &'static LazyLock<ResourceKey> = &GAME_RULE;
    pub const DEBUG_SUBSCRIPTION: &'static LazyLock<ResourceKey> = &DEBUG_SUBSCRIPTION;
    pub const DENSITY_FUNCTION_TYPE: &'static LazyLock<ResourceKey> = &DENSITY_FUNCTION_TYPE;
    pub const DIALOG_BODY_TYPE: &'static LazyLock<ResourceKey> = &DIALOG_BODY_TYPE;
    pub const DIALOG_TYPE: &'static LazyLock<ResourceKey> = &DIALOG_TYPE;
    pub const ENCHANTMENT_EFFECT_COMPONENT_TYPE: &'static LazyLock<ResourceKey> =
        &ENCHANTMENT_EFFECT_COMPONENT_TYPE;
    pub const ENCHANTMENT_ENTITY_EFFECT_TYPE: &'static LazyLock<ResourceKey> =
        &ENCHANTMENT_ENTITY_EFFECT_TYPE;
    pub const ENCHANTMENT_LEVEL_BASED_VALUE_TYPE: &'static LazyLock<ResourceKey> =
        &ENCHANTMENT_LEVEL_BASED_VALUE_TYPE;
    pub const ENCHANTMENT_LOCATION_BASED_EFFECT_TYPE: &'static LazyLock<ResourceKey> =
        &ENCHANTMENT_LOCATION_BASED_EFFECT_TYPE;
    pub const ENCHANTMENT_PROVIDER_TYPE: &'static LazyLock<ResourceKey> =
        &ENCHANTMENT_PROVIDER_TYPE;
    pub const ENCHANTMENT_VALUE_EFFECT_TYPE: &'static LazyLock<ResourceKey> =
        &ENCHANTMENT_VALUE_EFFECT_TYPE;
    pub const ENTITY_SUB_PREDICATE_TYPE: &'static LazyLock<ResourceKey> =
        &ENTITY_SUB_PREDICATE_TYPE;
    pub const ENTITY_TYPE: &'static LazyLock<ResourceKey> = &ENTITY_TYPE;
    pub const ENVIRONMENT_ATTRIBUTE: &'static LazyLock<ResourceKey> = &ENVIRONMENT_ATTRIBUTE;
    pub const ATTRIBUTE_TYPE: &'static LazyLock<ResourceKey> = &ATTRIBUTE_TYPE;
    pub const FEATURE_SIZE_TYPE: &'static LazyLock<ResourceKey> = &FEATURE_SIZE_TYPE;
    pub const FEATURE_TYPE: &'static LazyLock<ResourceKey> = &FEATURE_TYPE;
    pub const FLOAT_PROVIDER_TYPE: &'static LazyLock<ResourceKey> = &FLOAT_PROVIDER_TYPE;
    pub const FLUID: &'static LazyLock<ResourceKey> = &FLUID;
    pub const FOLIAGE_PLACER_TYPE: &'static LazyLock<ResourceKey> = &FOLIAGE_PLACER_TYPE;
    pub const GAME_EVENT: &'static LazyLock<ResourceKey> = &GAME_EVENT;
    pub const HEIGHT_PROVIDER_TYPE: &'static LazyLock<ResourceKey> = &HEIGHT_PROVIDER_TYPE;
    pub const INPUT_CONTROL_TYPE: &'static LazyLock<ResourceKey> = &INPUT_CONTROL_TYPE;
    pub const INT_PROVIDER_TYPE: &'static LazyLock<ResourceKey> = &INT_PROVIDER_TYPE;
    pub const ITEM: &'static LazyLock<ResourceKey> = &ITEM;
    pub const SLOT_SOURCE_TYPE: &'static LazyLock<ResourceKey> = &SLOT_SOURCE_TYPE;
    pub const LOOT_CONDITION_TYPE: &'static LazyLock<ResourceKey> = &LOOT_CONDITION_TYPE;
    pub const LOOT_FUNCTION_TYPE: &'static LazyLock<ResourceKey> = &LOOT_FUNCTION_TYPE;
    pub const LOOT_NBT_PROVIDER_TYPE: &'static LazyLock<ResourceKey> = &LOOT_NBT_PROVIDER_TYPE;
    pub const CONTEXT_FLOAT_PROVIDER_TYPE: &'static LazyLock<ResourceKey> =
        &CONTEXT_FLOAT_PROVIDER_TYPE;
    pub const CONTEXT_INT_PROVIDER_TYPE: &'static LazyLock<ResourceKey> =
        &CONTEXT_INT_PROVIDER_TYPE;
    pub const LOOT_POOL_ENTRY_TYPE: &'static LazyLock<ResourceKey> = &LOOT_POOL_ENTRY_TYPE;
    pub const LOOT_SCORE_PROVIDER_TYPE: &'static LazyLock<ResourceKey> = &LOOT_SCORE_PROVIDER_TYPE;
    pub const MAP_DECORATION_TYPE: &'static LazyLock<ResourceKey> = &MAP_DECORATION_TYPE;
    pub const MATERIAL_CONDITION_TYPE: &'static LazyLock<ResourceKey> = &MATERIAL_CONDITION_TYPE;
    pub const MATERIAL_RULE_TYPE: &'static LazyLock<ResourceKey> = &MATERIAL_RULE_TYPE;
    pub const MEMORY_MODULE_TYPE: &'static LazyLock<ResourceKey> = &MEMORY_MODULE_TYPE;
    pub const MENU: &'static LazyLock<ResourceKey> = &MENU;
    pub const MOB_EFFECT: &'static LazyLock<ResourceKey> = &MOB_EFFECT;
    pub const NUMBER_FORMAT_TYPE: &'static LazyLock<ResourceKey> = &NUMBER_FORMAT_TYPE;
    pub const PARTICLE_TYPE: &'static LazyLock<ResourceKey> = &PARTICLE_TYPE;
    pub const PLACEMENT_MODIFIER_TYPE: &'static LazyLock<ResourceKey> = &PLACEMENT_MODIFIER_TYPE;
    pub const POINT_OF_INTEREST_TYPE: &'static LazyLock<ResourceKey> = &POINT_OF_INTEREST_TYPE;
    pub const POOL_ALIAS_BINDING: &'static LazyLock<ResourceKey> = &POOL_ALIAS_BINDING;
    pub const POSITION_SOURCE_TYPE: &'static LazyLock<ResourceKey> = &POSITION_SOURCE_TYPE;
    pub const POS_RULE_TEST: &'static LazyLock<ResourceKey> = &POS_RULE_TEST;
    pub const POTION: &'static LazyLock<ResourceKey> = &POTION;
    pub const RECIPE_BOOK_CATEGORY: &'static LazyLock<ResourceKey> = &RECIPE_BOOK_CATEGORY;
    pub const RECIPE_DISPLAY: &'static LazyLock<ResourceKey> = &RECIPE_DISPLAY;
    pub const RECIPE_SERIALIZER: &'static LazyLock<ResourceKey> = &RECIPE_SERIALIZER;
    pub const RECIPE_TYPE: &'static LazyLock<ResourceKey> = &RECIPE_TYPE;
    pub const ROOT_PLACER_TYPE: &'static LazyLock<ResourceKey> = &ROOT_PLACER_TYPE;
    pub const RULE_BLOCK_ENTITY_MODIFIER: &'static LazyLock<ResourceKey> =
        &RULE_BLOCK_ENTITY_MODIFIER;
    pub const RULE_TEST: &'static LazyLock<ResourceKey> = &RULE_TEST;
    pub const SENSOR_TYPE: &'static LazyLock<ResourceKey> = &SENSOR_TYPE;
    pub const SLOT_DISPLAY: &'static LazyLock<ResourceKey> = &SLOT_DISPLAY;
    pub const SOUND_EVENT: &'static LazyLock<ResourceKey> = &SOUND_EVENT;
    pub const SPAWN_CONDITION_TYPE: &'static LazyLock<ResourceKey> = &SPAWN_CONDITION_TYPE;
    pub const STAT_TYPE: &'static LazyLock<ResourceKey> = &STAT_TYPE;
    pub const STRUCTURE_PIECE: &'static LazyLock<ResourceKey> = &STRUCTURE_PIECE;
    pub const STRUCTURE_PLACEMENT: &'static LazyLock<ResourceKey> = &STRUCTURE_PLACEMENT;
    pub const STRUCTURE_POOL_ELEMENT: &'static LazyLock<ResourceKey> = &STRUCTURE_POOL_ELEMENT;
    pub const STRUCTURE_PROCESSOR: &'static LazyLock<ResourceKey> = &STRUCTURE_PROCESSOR;
    pub const STRUCTURE_TYPE: &'static LazyLock<ResourceKey> = &STRUCTURE_TYPE;
    pub const DIALOG_ACTION_TYPE: &'static LazyLock<ResourceKey> = &DIALOG_ACTION_TYPE;
    pub const TEST_ENVIRONMENT_DEFINITION_TYPE: &'static LazyLock<ResourceKey> =
        &TEST_ENVIRONMENT_DEFINITION_TYPE;
    pub const TEST_FUNCTION: &'static LazyLock<ResourceKey> = &TEST_FUNCTION;
    pub const TEST_INSTANCE_TYPE: &'static LazyLock<ResourceKey> = &TEST_INSTANCE_TYPE;
    pub const TICKET_TYPE: &'static LazyLock<ResourceKey> = &TICKET_TYPE;
    pub const TREE_DECORATOR_TYPE: &'static LazyLock<ResourceKey> = &TREE_DECORATOR_TYPE;
    pub const TRUNK_PLACER_TYPE: &'static LazyLock<ResourceKey> = &TRUNK_PLACER_TYPE;
    pub const VILLAGER_PROFESSION: &'static LazyLock<ResourceKey> = &VILLAGER_PROFESSION;
    pub const VILLAGER_TYPE: &'static LazyLock<ResourceKey> = &VILLAGER_TYPE;
    pub const INCOMING_RPC_METHOD: &'static LazyLock<ResourceKey> = &INCOMING_RPC_METHOD;
    pub const OUTGOING_RPC_METHOD: &'static LazyLock<ResourceKey> = &OUTGOING_RPC_METHOD;
    pub const PERMISSION_TYPE: &'static LazyLock<ResourceKey> = &PERMISSION_TYPE;
    pub const PERMISSION_CHECK_TYPE: &'static LazyLock<ResourceKey> = &PERMISSION_CHECK_TYPE;
    pub const CONTEXT_KEY_SET: &'static LazyLock<ResourceKey> = &CONTEXT_KEY_SET;
    pub const BANNER_PATTERN: &'static LazyLock<ResourceKey> = &BANNER_PATTERN;
    pub const BIOME: &'static LazyLock<ResourceKey> = &BIOME;
    pub const BLOCK_STATE_PROVIDER: &'static LazyLock<ResourceKey> = &BLOCK_STATE_PROVIDER;
    pub const CAT_SOUND_VARIANT: &'static LazyLock<ResourceKey> = &CAT_SOUND_VARIANT;
    pub const CAT_VARIANT: &'static LazyLock<ResourceKey> = &CAT_VARIANT;
    pub const CARVER: &'static LazyLock<ResourceKey> = &CARVER;
    pub const CHAT_TYPE: &'static LazyLock<ResourceKey> = &CHAT_TYPE;
    pub const CHICKEN_SOUND_VARIANT: &'static LazyLock<ResourceKey> = &CHICKEN_SOUND_VARIANT;
    pub const CHICKEN_VARIANT: &'static LazyLock<ResourceKey> = &CHICKEN_VARIANT;
    pub const ZOMBIE_NAUTILUS_VARIANT: &'static LazyLock<ResourceKey> = &ZOMBIE_NAUTILUS_VARIANT;
    pub const FEATURE: &'static LazyLock<ResourceKey> = &FEATURE;
    pub const COW_SOUND_VARIANT: &'static LazyLock<ResourceKey> = &COW_SOUND_VARIANT;
    pub const COW_VARIANT: &'static LazyLock<ResourceKey> = &COW_VARIANT;
    pub const DAMAGE_TYPE: &'static LazyLock<ResourceKey> = &DAMAGE_TYPE;
    pub const DENSITY_FUNCTION: &'static LazyLock<ResourceKey> = &DENSITY_FUNCTION;
    pub const DIALOG: &'static LazyLock<ResourceKey> = &DIALOG;
    pub const DIMENSION_TYPE: &'static LazyLock<ResourceKey> = &DIMENSION_TYPE;
    pub const ENCHANTMENT_PROVIDER: &'static LazyLock<ResourceKey> = &ENCHANTMENT_PROVIDER;
    pub const ENCHANTMENT: &'static LazyLock<ResourceKey> = &ENCHANTMENT;
    pub const FLAT_LEVEL_GENERATOR_PRESET: &'static LazyLock<ResourceKey> =
        &FLAT_LEVEL_GENERATOR_PRESET;
    pub const FROG_VARIANT: &'static LazyLock<ResourceKey> = &FROG_VARIANT;
    pub const INSTRUMENT: &'static LazyLock<ResourceKey> = &INSTRUMENT;
    pub const JUKEBOX_SONG: &'static LazyLock<ResourceKey> = &JUKEBOX_SONG;
    pub const MATERIAL_CONDITION: &'static LazyLock<ResourceKey> = &MATERIAL_CONDITION;
    pub const MATERIAL_RULE: &'static LazyLock<ResourceKey> = &MATERIAL_RULE;
    pub const MULTI_NOISE_BIOME_SOURCE_PARAMETER_LIST: &'static LazyLock<ResourceKey> =
        &MULTI_NOISE_BIOME_SOURCE_PARAMETER_LIST;
    pub const NOISE_SETTINGS: &'static LazyLock<ResourceKey> = &NOISE_SETTINGS;
    pub const NOISE: &'static LazyLock<ResourceKey> = &NOISE;
    pub const PAINTING_VARIANT: &'static LazyLock<ResourceKey> = &PAINTING_VARIANT;
    pub const PIG_SOUND_VARIANT: &'static LazyLock<ResourceKey> = &PIG_SOUND_VARIANT;
    pub const PIG_VARIANT: &'static LazyLock<ResourceKey> = &PIG_VARIANT;
    pub const PLACED_FEATURE: &'static LazyLock<ResourceKey> = &PLACED_FEATURE;
    pub const PROCESSOR_LIST: &'static LazyLock<ResourceKey> = &PROCESSOR_LIST;
    pub const STRUCTURE_SET: &'static LazyLock<ResourceKey> = &STRUCTURE_SET;
    pub const STRUCTURE: &'static LazyLock<ResourceKey> = &STRUCTURE;
    pub const SULFUR_CUBE_ARCHETYPE: &'static LazyLock<ResourceKey> = &SULFUR_CUBE_ARCHETYPE;
    pub const TEMPLATE_POOL: &'static LazyLock<ResourceKey> = &TEMPLATE_POOL;
    pub const TEST_ENVIRONMENT: &'static LazyLock<ResourceKey> = &TEST_ENVIRONMENT;
    pub const TEST_INSTANCE: &'static LazyLock<ResourceKey> = &TEST_INSTANCE;
    pub const TIMELINE: &'static LazyLock<ResourceKey> = &TIMELINE;
    pub const TRADE_SET: &'static LazyLock<ResourceKey> = &TRADE_SET;
    pub const TRIAL_SPAWNER_CONFIG: &'static LazyLock<ResourceKey> = &TRIAL_SPAWNER_CONFIG;
    pub const TRIGGER_TYPE: &'static LazyLock<ResourceKey> = &TRIGGER_TYPE;
    pub const TRIM_MATERIAL: &'static LazyLock<ResourceKey> = &TRIM_MATERIAL;
    pub const TRIM_PATTERN: &'static LazyLock<ResourceKey> = &TRIM_PATTERN;
    pub const VILLAGER_TRADE: &'static LazyLock<ResourceKey> = &VILLAGER_TRADE;
    pub const WOLF_VARIANT: &'static LazyLock<ResourceKey> = &WOLF_VARIANT;
    pub const WOLF_SOUND_VARIANT: &'static LazyLock<ResourceKey> = &WOLF_SOUND_VARIANT;
    pub const WORLD_CLOCK: &'static LazyLock<ResourceKey> = &WORLD_CLOCK;
    pub const WORLD_PRESET: &'static LazyLock<ResourceKey> = &WORLD_PRESET;
    pub const DECORATED_POT_PATTERN: &'static LazyLock<ResourceKey> = &DECORATED_POT_PATTERN;
    pub const BLOCK_TRANSFORMER: &'static LazyLock<ResourceKey> = &BLOCK_TRANSFORMER;
    pub const DIMENSION: &'static LazyLock<ResourceKey> = &DIMENSION;
    pub const LEVEL_STEM: &'static LazyLock<ResourceKey> = &LEVEL_STEM;
    pub const LOOT_TABLE: &'static LazyLock<ResourceKey> = &LOOT_TABLE;
    pub const ITEM_MODIFIER: &'static LazyLock<ResourceKey> = &ITEM_MODIFIER;
    pub const PREDICATE: &'static LazyLock<ResourceKey> = &PREDICATE;
    pub const SLOT_SOURCE: &'static LazyLock<ResourceKey> = &SLOT_SOURCE;
    pub const CONTEXT_FLOAT_PROVIDER: &'static LazyLock<ResourceKey> = &CONTEXT_FLOAT_PROVIDER;
    pub const CONTEXT_INT_PROVIDER: &'static LazyLock<ResourceKey> = &CONTEXT_INT_PROVIDER;
    pub const ADVANCEMENT: &'static LazyLock<ResourceKey> = &ADVANCEMENT;
    pub const RECIPE: &'static LazyLock<ResourceKey> = &RECIPE;

    /// The level a dimension's stem makes: the same identifier, in `DIMENSION`.
    pub fn level_stem_to_level(level_stem: &ResourceKey) -> ResourceKey {
        ResourceKey::create(Registries::DIMENSION, level_stem.identifier().clone())
    }

    pub fn level_to_level_stem(level: &ResourceKey) -> ResourceKey {
        ResourceKey::create(Registries::LEVEL_STEM, level.identifier().clone())
    }

    fn create_registry_key(name: &str) -> ResourceKey {
        ResourceKey::create_registry_key(Identifier::with_default_namespace(name))
    }

    /// Where a data pack keeps the registry's elements: its identifier's path, whatever the
    /// namespace.
    fn registry_dir_path(registry_key: &ResourceKey) -> String {
        registry_key.identifier().get_path().to_owned()
    }

    pub fn elements_dir_path(registry_key: &ResourceKey) -> String {
        Registries::registry_dir_path(registry_key)
    }

    pub fn tags_dir_path(registry_key: &ResourceKey) -> String {
        format!("tags/{}", Registries::registry_dir_path(registry_key))
    }

    pub fn components_dir_path(registry_key: &ResourceKey) -> String {
        format!("components/{}", Registries::registry_dir_path(registry_key))
    }
}

#[cfg(test)]
mod tests {
    use super::Registries;
    use crate::golden::Golden;
    use crate::net::minecraft::resources::resource_key::ResourceKey;

    #[test]
    fn matches_vanilla() {
        let golden = Golden::load("net.minecraft.core.registries.Registries");
        golden.field(
            "ROOT_REGISTRY_NAME",
            (**Registries::ROOT_REGISTRY_NAME).clone(),
        );
        for (name, key) in [
            ("ACTIVITY", Registries::ACTIVITY),
            ("ATTRIBUTE", Registries::ATTRIBUTE),
            ("BIOME_SOURCE", Registries::BIOME_SOURCE),
            ("BLOCK_ENTITY_TYPE", Registries::BLOCK_ENTITY_TYPE),
            ("BLOCK_PREDICATE_TYPE", Registries::BLOCK_PREDICATE_TYPE),
            (
                "BLOCK_STATE_PROVIDER_TYPE",
                Registries::BLOCK_STATE_PROVIDER_TYPE,
            ),
            ("BLOCK", Registries::BLOCK),
            ("CARVER_TYPE", Registries::CARVER_TYPE),
            ("CHUNK_GENERATOR", Registries::CHUNK_GENERATOR),
            ("CHUNK_STATUS", Registries::CHUNK_STATUS),
            ("COMMAND_ARGUMENT_TYPE", Registries::COMMAND_ARGUMENT_TYPE),
            ("CONSUME_EFFECT_TYPE", Registries::CONSUME_EFFECT_TYPE),
            ("CREATIVE_MODE_TAB", Registries::CREATIVE_MODE_TAB),
            ("CUSTOM_STAT", Registries::CUSTOM_STAT),
            (
                "DATA_COMPONENT_PREDICATE_TYPE",
                Registries::DATA_COMPONENT_PREDICATE_TYPE,
            ),
            ("DATA_COMPONENT_TYPE", Registries::DATA_COMPONENT_TYPE),
            ("GAME_RULE", Registries::GAME_RULE),
            ("DEBUG_SUBSCRIPTION", Registries::DEBUG_SUBSCRIPTION),
            ("DENSITY_FUNCTION_TYPE", Registries::DENSITY_FUNCTION_TYPE),
            ("DIALOG_BODY_TYPE", Registries::DIALOG_BODY_TYPE),
            ("DIALOG_TYPE", Registries::DIALOG_TYPE),
            (
                "ENCHANTMENT_EFFECT_COMPONENT_TYPE",
                Registries::ENCHANTMENT_EFFECT_COMPONENT_TYPE,
            ),
            (
                "ENCHANTMENT_ENTITY_EFFECT_TYPE",
                Registries::ENCHANTMENT_ENTITY_EFFECT_TYPE,
            ),
            (
                "ENCHANTMENT_LEVEL_BASED_VALUE_TYPE",
                Registries::ENCHANTMENT_LEVEL_BASED_VALUE_TYPE,
            ),
            (
                "ENCHANTMENT_LOCATION_BASED_EFFECT_TYPE",
                Registries::ENCHANTMENT_LOCATION_BASED_EFFECT_TYPE,
            ),
            (
                "ENCHANTMENT_PROVIDER_TYPE",
                Registries::ENCHANTMENT_PROVIDER_TYPE,
            ),
            (
                "ENCHANTMENT_VALUE_EFFECT_TYPE",
                Registries::ENCHANTMENT_VALUE_EFFECT_TYPE,
            ),
            (
                "ENTITY_SUB_PREDICATE_TYPE",
                Registries::ENTITY_SUB_PREDICATE_TYPE,
            ),
            ("ENTITY_TYPE", Registries::ENTITY_TYPE),
            ("ENVIRONMENT_ATTRIBUTE", Registries::ENVIRONMENT_ATTRIBUTE),
            ("ATTRIBUTE_TYPE", Registries::ATTRIBUTE_TYPE),
            ("FEATURE_SIZE_TYPE", Registries::FEATURE_SIZE_TYPE),
            ("FEATURE_TYPE", Registries::FEATURE_TYPE),
            ("FLOAT_PROVIDER_TYPE", Registries::FLOAT_PROVIDER_TYPE),
            ("FLUID", Registries::FLUID),
            ("FOLIAGE_PLACER_TYPE", Registries::FOLIAGE_PLACER_TYPE),
            ("GAME_EVENT", Registries::GAME_EVENT),
            ("HEIGHT_PROVIDER_TYPE", Registries::HEIGHT_PROVIDER_TYPE),
            ("INPUT_CONTROL_TYPE", Registries::INPUT_CONTROL_TYPE),
            ("INT_PROVIDER_TYPE", Registries::INT_PROVIDER_TYPE),
            ("ITEM", Registries::ITEM),
            ("SLOT_SOURCE_TYPE", Registries::SLOT_SOURCE_TYPE),
            ("LOOT_CONDITION_TYPE", Registries::LOOT_CONDITION_TYPE),
            ("LOOT_FUNCTION_TYPE", Registries::LOOT_FUNCTION_TYPE),
            ("LOOT_NBT_PROVIDER_TYPE", Registries::LOOT_NBT_PROVIDER_TYPE),
            (
                "CONTEXT_FLOAT_PROVIDER_TYPE",
                Registries::CONTEXT_FLOAT_PROVIDER_TYPE,
            ),
            (
                "CONTEXT_INT_PROVIDER_TYPE",
                Registries::CONTEXT_INT_PROVIDER_TYPE,
            ),
            ("LOOT_POOL_ENTRY_TYPE", Registries::LOOT_POOL_ENTRY_TYPE),
            (
                "LOOT_SCORE_PROVIDER_TYPE",
                Registries::LOOT_SCORE_PROVIDER_TYPE,
            ),
            ("MAP_DECORATION_TYPE", Registries::MAP_DECORATION_TYPE),
            (
                "MATERIAL_CONDITION_TYPE",
                Registries::MATERIAL_CONDITION_TYPE,
            ),
            ("MATERIAL_RULE_TYPE", Registries::MATERIAL_RULE_TYPE),
            ("MEMORY_MODULE_TYPE", Registries::MEMORY_MODULE_TYPE),
            ("MENU", Registries::MENU),
            ("MOB_EFFECT", Registries::MOB_EFFECT),
            ("NUMBER_FORMAT_TYPE", Registries::NUMBER_FORMAT_TYPE),
            ("PARTICLE_TYPE", Registries::PARTICLE_TYPE),
            (
                "PLACEMENT_MODIFIER_TYPE",
                Registries::PLACEMENT_MODIFIER_TYPE,
            ),
            ("POINT_OF_INTEREST_TYPE", Registries::POINT_OF_INTEREST_TYPE),
            ("POOL_ALIAS_BINDING", Registries::POOL_ALIAS_BINDING),
            ("POSITION_SOURCE_TYPE", Registries::POSITION_SOURCE_TYPE),
            ("POS_RULE_TEST", Registries::POS_RULE_TEST),
            ("POTION", Registries::POTION),
            ("RECIPE_BOOK_CATEGORY", Registries::RECIPE_BOOK_CATEGORY),
            ("RECIPE_DISPLAY", Registries::RECIPE_DISPLAY),
            ("RECIPE_SERIALIZER", Registries::RECIPE_SERIALIZER),
            ("RECIPE_TYPE", Registries::RECIPE_TYPE),
            ("ROOT_PLACER_TYPE", Registries::ROOT_PLACER_TYPE),
            (
                "RULE_BLOCK_ENTITY_MODIFIER",
                Registries::RULE_BLOCK_ENTITY_MODIFIER,
            ),
            ("RULE_TEST", Registries::RULE_TEST),
            ("SENSOR_TYPE", Registries::SENSOR_TYPE),
            ("SLOT_DISPLAY", Registries::SLOT_DISPLAY),
            ("SOUND_EVENT", Registries::SOUND_EVENT),
            ("SPAWN_CONDITION_TYPE", Registries::SPAWN_CONDITION_TYPE),
            ("STAT_TYPE", Registries::STAT_TYPE),
            ("STRUCTURE_PIECE", Registries::STRUCTURE_PIECE),
            ("STRUCTURE_PLACEMENT", Registries::STRUCTURE_PLACEMENT),
            ("STRUCTURE_POOL_ELEMENT", Registries::STRUCTURE_POOL_ELEMENT),
            ("STRUCTURE_PROCESSOR", Registries::STRUCTURE_PROCESSOR),
            ("STRUCTURE_TYPE", Registries::STRUCTURE_TYPE),
            ("DIALOG_ACTION_TYPE", Registries::DIALOG_ACTION_TYPE),
            (
                "TEST_ENVIRONMENT_DEFINITION_TYPE",
                Registries::TEST_ENVIRONMENT_DEFINITION_TYPE,
            ),
            ("TEST_FUNCTION", Registries::TEST_FUNCTION),
            ("TEST_INSTANCE_TYPE", Registries::TEST_INSTANCE_TYPE),
            ("TICKET_TYPE", Registries::TICKET_TYPE),
            ("TREE_DECORATOR_TYPE", Registries::TREE_DECORATOR_TYPE),
            ("TRUNK_PLACER_TYPE", Registries::TRUNK_PLACER_TYPE),
            ("VILLAGER_PROFESSION", Registries::VILLAGER_PROFESSION),
            ("VILLAGER_TYPE", Registries::VILLAGER_TYPE),
            ("INCOMING_RPC_METHOD", Registries::INCOMING_RPC_METHOD),
            ("OUTGOING_RPC_METHOD", Registries::OUTGOING_RPC_METHOD),
            ("PERMISSION_TYPE", Registries::PERMISSION_TYPE),
            ("PERMISSION_CHECK_TYPE", Registries::PERMISSION_CHECK_TYPE),
            ("CONTEXT_KEY_SET", Registries::CONTEXT_KEY_SET),
            ("BANNER_PATTERN", Registries::BANNER_PATTERN),
            ("BIOME", Registries::BIOME),
            ("BLOCK_STATE_PROVIDER", Registries::BLOCK_STATE_PROVIDER),
            ("CAT_SOUND_VARIANT", Registries::CAT_SOUND_VARIANT),
            ("CAT_VARIANT", Registries::CAT_VARIANT),
            ("CARVER", Registries::CARVER),
            ("CHAT_TYPE", Registries::CHAT_TYPE),
            ("CHICKEN_SOUND_VARIANT", Registries::CHICKEN_SOUND_VARIANT),
            ("CHICKEN_VARIANT", Registries::CHICKEN_VARIANT),
            (
                "ZOMBIE_NAUTILUS_VARIANT",
                Registries::ZOMBIE_NAUTILUS_VARIANT,
            ),
            ("FEATURE", Registries::FEATURE),
            ("COW_SOUND_VARIANT", Registries::COW_SOUND_VARIANT),
            ("COW_VARIANT", Registries::COW_VARIANT),
            ("DAMAGE_TYPE", Registries::DAMAGE_TYPE),
            ("DENSITY_FUNCTION", Registries::DENSITY_FUNCTION),
            ("DIALOG", Registries::DIALOG),
            ("DIMENSION_TYPE", Registries::DIMENSION_TYPE),
            ("ENCHANTMENT_PROVIDER", Registries::ENCHANTMENT_PROVIDER),
            ("ENCHANTMENT", Registries::ENCHANTMENT),
            (
                "FLAT_LEVEL_GENERATOR_PRESET",
                Registries::FLAT_LEVEL_GENERATOR_PRESET,
            ),
            ("FROG_VARIANT", Registries::FROG_VARIANT),
            ("INSTRUMENT", Registries::INSTRUMENT),
            ("JUKEBOX_SONG", Registries::JUKEBOX_SONG),
            ("MATERIAL_CONDITION", Registries::MATERIAL_CONDITION),
            ("MATERIAL_RULE", Registries::MATERIAL_RULE),
            (
                "MULTI_NOISE_BIOME_SOURCE_PARAMETER_LIST",
                Registries::MULTI_NOISE_BIOME_SOURCE_PARAMETER_LIST,
            ),
            ("NOISE_SETTINGS", Registries::NOISE_SETTINGS),
            ("NOISE", Registries::NOISE),
            ("PAINTING_VARIANT", Registries::PAINTING_VARIANT),
            ("PIG_SOUND_VARIANT", Registries::PIG_SOUND_VARIANT),
            ("PIG_VARIANT", Registries::PIG_VARIANT),
            ("PLACED_FEATURE", Registries::PLACED_FEATURE),
            ("PROCESSOR_LIST", Registries::PROCESSOR_LIST),
            ("STRUCTURE_SET", Registries::STRUCTURE_SET),
            ("STRUCTURE", Registries::STRUCTURE),
            ("SULFUR_CUBE_ARCHETYPE", Registries::SULFUR_CUBE_ARCHETYPE),
            ("TEMPLATE_POOL", Registries::TEMPLATE_POOL),
            ("TEST_ENVIRONMENT", Registries::TEST_ENVIRONMENT),
            ("TEST_INSTANCE", Registries::TEST_INSTANCE),
            ("TIMELINE", Registries::TIMELINE),
            ("TRADE_SET", Registries::TRADE_SET),
            ("TRIAL_SPAWNER_CONFIG", Registries::TRIAL_SPAWNER_CONFIG),
            ("TRIGGER_TYPE", Registries::TRIGGER_TYPE),
            ("TRIM_MATERIAL", Registries::TRIM_MATERIAL),
            ("TRIM_PATTERN", Registries::TRIM_PATTERN),
            ("VILLAGER_TRADE", Registries::VILLAGER_TRADE),
            ("WOLF_VARIANT", Registries::WOLF_VARIANT),
            ("WOLF_SOUND_VARIANT", Registries::WOLF_SOUND_VARIANT),
            ("WORLD_CLOCK", Registries::WORLD_CLOCK),
            ("WORLD_PRESET", Registries::WORLD_PRESET),
            ("DECORATED_POT_PATTERN", Registries::DECORATED_POT_PATTERN),
            ("BLOCK_TRANSFORMER", Registries::BLOCK_TRANSFORMER),
            ("DIMENSION", Registries::DIMENSION),
            ("LEVEL_STEM", Registries::LEVEL_STEM),
            ("LOOT_TABLE", Registries::LOOT_TABLE),
            ("ITEM_MODIFIER", Registries::ITEM_MODIFIER),
            ("PREDICATE", Registries::PREDICATE),
            ("SLOT_SOURCE", Registries::SLOT_SOURCE),
            ("CONTEXT_FLOAT_PROVIDER", Registries::CONTEXT_FLOAT_PROVIDER),
            ("CONTEXT_INT_PROVIDER", Registries::CONTEXT_INT_PROVIDER),
            ("ADVANCEMENT", Registries::ADVANCEMENT),
            ("RECIPE", Registries::RECIPE),
        ] {
            golden.field(name, (**key).clone());
        }
        golden.check(
            "levelStemToLevel(Lnet/minecraft/resources/ResourceKey;)Lnet/minecraft/resources/ResourceKey;",
            |level_stem: ResourceKey| Registries::level_stem_to_level(&level_stem),
        );
        golden.check(
            "levelToLevelStem(Lnet/minecraft/resources/ResourceKey;)Lnet/minecraft/resources/ResourceKey;",
            |level: ResourceKey| Registries::level_to_level_stem(&level),
        );
        golden.check(
            "elementsDirPath(Lnet/minecraft/resources/ResourceKey;)Ljava/lang/String;",
            |registry_key: ResourceKey| Registries::elements_dir_path(&registry_key),
        );
        golden.check(
            "tagsDirPath(Lnet/minecraft/resources/ResourceKey;)Ljava/lang/String;",
            |registry_key: ResourceKey| Registries::tags_dir_path(&registry_key),
        );
        golden.check(
            "componentsDirPath(Lnet/minecraft/resources/ResourceKey;)Ljava/lang/String;",
            |registry_key: ResourceKey| Registries::components_dir_path(&registry_key),
        );
        golden.finish();
    }
}
