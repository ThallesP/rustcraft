//! net/minecraft/world/level/block/entity/BlockEntity.java
//!
//! A block's extra state, kept by its chunk at its position: its type, position and block state,
//! and what its class loads (`loadAdditional`). Java's subclasses are a class each; the client
//! tells them apart with `instanceof` (the shapes, the renderers), so here a block entity is one
//! struct whose `data` is an enum of the classes ported (`BlockEntityData`), the others holding
//! nothing more. `loadWithComponents` is `loadAdditional` by class, and `triggerEvent` the class's
//! block event. Java tells block entities apart by reference (a ticker holds its own); here each
//! has an identity, a number no other block entity the game makes has. The components
//! (`DataComponentMap.CODEC`, item data the client shows in tooltips and on items picked from the
//! block) aren't ported, nor are saving (`saveWithFullMetadata`…, the server's), `setChanged`, the
//! level back-reference (`setLevel`, `hasLevel`: the chunk that holds a block entity is its
//! level's), `preRemoveSideEffects`, `getUpdatePacket` and the crash report details.

use std::sync::atomic::{AtomicU64, Ordering};

use std::fmt;

use crate::net::minecraft::client::multiplayer::client_level::LevelTicking;
use crate::net::minecraft::core::block_pos::BlockPos;
use crate::net::minecraft::world::level::block::entity::banner_block_entity::BannerBlockEntity;
use crate::net::minecraft::world::level::block::entity::beacon_block_entity::BeaconBlockEntity;
use crate::net::minecraft::world::level::block::entity::bell_block_entity::BellBlockEntity;
use crate::net::minecraft::world::level::block::entity::block_entity_type::BlockEntityType;
use crate::net::minecraft::world::level::block::entity::brushable_block_entity::BrushableBlockEntity;
use crate::net::minecraft::world::level::block::entity::campfire_block_entity::CampfireBlockEntity;
use crate::net::minecraft::world::level::block::entity::chest_block_entity::ChestBlockEntity;
use crate::net::minecraft::world::level::block::entity::conduit_block_entity::ConduitBlockEntity;
use crate::net::minecraft::world::level::block::entity::decorated_pot_block_entity::DecoratedPotBlockEntity;
use crate::net::minecraft::world::level::block::entity::enchanting_table_block_entity::EnchantingTableBlockEntity;
use crate::net::minecraft::world::level::block::entity::ender_chest_block_entity::EnderChestBlockEntity;
use crate::net::minecraft::world::level::block::entity::potent_sulfur_block_entity::PotentSulfurBlockEntity;
use crate::net::minecraft::world::level::block::entity::shulker_box_block_entity::ShulkerBoxBlockEntity;
use crate::net::minecraft::world::level::block::entity::sign_block_entity::SignBlockEntity;
use crate::net::minecraft::world::level::block::entity::skull_block_entity::SkullBlockEntity;
use crate::net::minecraft::world::level::block::entity::spawner_block_entity::SpawnerBlockEntity;
use crate::net::minecraft::world::level::block::entity::the_end_gateway_block_entity::TheEndGatewayBlockEntity;
use crate::net::minecraft::world::level::block::piston::piston_moving_block_entity::PistonMovingBlockEntity;
use crate::net::minecraft::world::level::block::state::block_state::BlockState;
use crate::net::minecraft::world::level::storage::tag_value_input::TagValueInput;

/// The identities handed out so far.
static IDENTITIES: AtomicU64 = AtomicU64::new(0);

pub struct BlockEntity {
    identity: u64,
    r#type: BlockEntityType,
    world_position: BlockPos,
    remove: bool,
    block_state: BlockState,
    /// What the block entity's class adds.
    pub data: BlockEntityData,
}

/// The fields of the classes whose data is ported, which the client reads.
#[derive(Clone, Debug)]
pub enum BlockEntityData {
    /// A class whose fields aren't ported, or one with none.
    None,
    Sign(SignBlockEntity),
    Banner(BannerBlockEntity),
    Skull(SkullBlockEntity),
    Spawner(SpawnerBlockEntity),
    Campfire(CampfireBlockEntity),
    Brushable(BrushableBlockEntity),
    ShulkerBox(ShulkerBoxBlockEntity),
    /// The chest's, the copper chests' and the trapped chest's.
    Chest(ChestBlockEntity),
    EnderChest(EnderChestBlockEntity),
    Bell(BellBlockEntity),
    EndGateway(TheEndGatewayBlockEntity),
    DecoratedPot(DecoratedPotBlockEntity),
    Conduit(ConduitBlockEntity),
    Beacon(BeaconBlockEntity),
    EnchantingTable(EnchantingTableBlockEntity),
    PotentSulfur(PotentSulfurBlockEntity),
    Piston(PistonMovingBlockEntity),
}

impl BlockEntity {
    /// `BlockEntity(type, worldPosition, blockState)`, with its class's fields as its constructor
    /// sets them. Panics where Java throws `IllegalStateException`, for a state the type isn't
    /// valid for.
    pub fn new(
        r#type: BlockEntityType,
        world_position: BlockPos,
        block_state: BlockState,
    ) -> BlockEntity {
        let entity = BlockEntity {
            identity: IDENTITIES.fetch_add(1, Ordering::Relaxed),
            r#type,
            world_position,
            remove: false,
            block_state,
            data: BlockEntityData::None,
        };
        entity.validate_block_state(block_state);
        entity
    }

    fn validate_block_state(&self, block_state: BlockState) {
        if !self.is_valid_block_state(block_state) {
            panic!(
                "Invalid block entity {} state at {}, got {block_state}",
                self.get_name_for_reporting(),
                self.world_position
            );
        }
    }

    pub fn is_valid_block_state(&self, block_state: BlockState) -> bool {
        self.r#type.is_valid(block_state)
    }

    /// `setLevel(level)`: what a class works out of its level when its chunk takes it (a beacon
    /// starts its beam's check from the bottom, a potent sulfur takes the time as its last
    /// eruption's), the level as far as a class reads it: its lowest block's height and its game
    /// time. The moving piston's checks its block is in the level's registry, which every block
    /// is.
    pub fn set_level(&mut self, min_y: i32, game_time: i64) {
        match &mut self.data {
            BlockEntityData::Beacon(beacon) => beacon.set_level(min_y),
            BlockEntityData::PotentSulfur(sulfur) if sulfur.eruption_tick == -1 => {
                sulfur.eruption_tick = game_time;
            }
            _ => {}
        }
    }

    /// Which block entity it is: Java's reference to it.
    pub fn identity(&self) -> u64 {
        self.identity
    }

    /// `triggerEvent(b0, b1)`: what a block event does to it, by class, whether it was handled
    /// (`BlockEntity`'s handles nothing).
    pub fn trigger_event(&mut self, level: &LevelTicking<'_>, b0: i32, b1: i32) -> bool {
        let pos = self.world_position;
        match &mut self.data {
            BlockEntityData::Chest(chest) => chest.trigger_event(b0, b1),
            BlockEntityData::EnderChest(ender_chest) => ender_chest.trigger_event(b0, b1),
            BlockEntityData::ShulkerBox(shulker_box) => shulker_box.trigger_event(b0, b1),
            BlockEntityData::Bell(bell) => bell.trigger_event(level, &pos, b0, b1),
            BlockEntityData::EndGateway(end_gateway) => end_gateway.trigger_event(b0, b1),
            BlockEntityData::DecoratedPot(decorated_pot) => {
                decorated_pot.trigger_event(level.level, b0, b1)
            }
            BlockEntityData::Spawner(spawner) => spawner.trigger_event(b0, b1),
            _ => false,
        }
    }

    /// `loadWithComponents`: `loadAdditional`, the class's own fields.
    pub fn load_with_components(&mut self, input: &TagValueInput<'_>) {
        match &mut self.data {
            BlockEntityData::None
            | BlockEntityData::Chest(_)
            | BlockEntityData::EnderChest(_)
            | BlockEntityData::Bell(_)
            | BlockEntityData::DecoratedPot(_)
            | BlockEntityData::Conduit(_)
            | BlockEntityData::Beacon(_)
            | BlockEntityData::EnchantingTable(_)
            | BlockEntityData::PotentSulfur(_) => {}
            BlockEntityData::EndGateway(end_gateway) => end_gateway.load_additional(input),
            BlockEntityData::Piston(piston) => piston.load_additional(input),
            BlockEntityData::Sign(sign) => sign.load_additional(input),
            BlockEntityData::Banner(banner) => banner.load_additional(input),
            BlockEntityData::Skull(skull) => skull.load_additional(input),
            BlockEntityData::Spawner(spawner) => spawner.load_additional(input),
            BlockEntityData::Campfire(campfire) => campfire.load_additional(input),
            BlockEntityData::Brushable(brushable) => brushable.load_additional(input),
            BlockEntityData::ShulkerBox(shulker_box) => shulker_box.load_additional(input),
        }
    }

    pub fn get_block_pos(&self) -> BlockPos {
        self.world_position
    }

    pub fn get_block_state(&self) -> BlockState {
        self.block_state
    }

    pub fn get_type(&self) -> BlockEntityType {
        self.r#type
    }

    pub fn is_removed(&self) -> bool {
        self.remove
    }

    pub fn set_removed(&mut self) {
        self.remove = true;
    }

    pub fn clear_removed(&mut self) {
        self.remove = false;
    }

    /// `setBlockState`: panics like the constructor for a state the type isn't valid for.
    pub fn set_block_state(&mut self, block_state: BlockState) {
        self.validate_block_state(block_state);
        self.block_state = block_state;
    }

    /// `getNameForReporting`: the type's registered name and the class's (by the data's kind).
    pub fn get_name_for_reporting(&self) -> String {
        format!("{} // {}", self.r#type.key(), self.r#type.class_name())
    }
}

/// `Object.toString`: the class's name and an identity hash; the hash, which changes from run to
/// run, is left out.
impl fmt::Debug for BlockEntity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BlockEntity")
            .field("type", &self.r#type)
            .field("pos", &self.world_position)
            .field("data", &self.data)
            .finish_non_exhaustive()
    }
}

#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use super::{BlockEntity, BlockEntityData};
    use crate::golden::{Golden, unhex};
    use crate::io::netty::buffer::byte_buf_input_stream::ByteBufInputStream;
    use crate::io::netty::buffer::unpooled;
    use crate::net::minecraft::core::block_pos::BlockPos;
    use crate::net::minecraft::core::registry_access;
    use crate::net::minecraft::nbt::nbt_accounter::NbtAccounter;
    use crate::net::minecraft::nbt::nbt_io::NbtIo;
    use crate::net::minecraft::nbt::tag::Tag;
    use crate::net::minecraft::network::chat::component::Component;
    use crate::net::minecraft::util::problem_reporter;
    use crate::net::minecraft::util::string_representable::StringRepresentable;
    use crate::net::minecraft::world::level::block::block::Block;
    use crate::net::minecraft::world::level::block::entity::block_entity_type::BlockEntityType;
    use crate::net::minecraft::world::level::block::entity::block_entity_types::BY_BLOCK;
    use crate::net::minecraft::world::level::block::entity::sign_text::SignText;
    use crate::net::minecraft::world::level::block::state::block_state::BlockState;
    use crate::net::minecraft::world::level::storage::tag_value_input::TagValueInput;

    fn component(component: Option<&Component>) -> Value {
        component.map_or(Value::Null, |component| json!(component.to_string()))
    }

    fn sign_text(text: &SignText) -> Value {
        let messages = |filtered| {
            text.get_messages(filtered)
                .iter()
                .map(|message| message.to_string())
                .collect::<Vec<_>>()
        };
        json!([
            messages(false),
            messages(true),
            text.get_color().get_serialized_name(),
            text.has_glowing_text()
        ])
    }

    /// The suite's fields: `[type, state, isRemoved, data]`.
    fn fields(entity: &BlockEntity) -> Value {
        let data = match &entity.data {
            BlockEntityData::None => Value::Null,
            BlockEntityData::Sign(sign) => json!([
                sign_text(sign.get_front_text()),
                sign_text(sign.get_back_text()),
                sign.is_waxed()
            ]),
            BlockEntityData::Banner(banner) => json!([
                banner.get_base_color().get_serialized_name(),
                component(banner.get_custom_name()),
                banner
                    .get_patterns()
                    .layers()
                    .iter()
                    .map(|layer| {
                        json!([
                            layer
                                .pattern()
                                .unwrap_key()
                                .expect("a reference")
                                .identifier()
                                .to_string(),
                            layer.color().get_serialized_name()
                        ])
                    })
                    .collect::<Vec<_>>()
            ]),
            BlockEntityData::Skull(skull) => json!([
                skull.get_note_block_sound().map(ToString::to_string),
                component(skull.get_custom_name())
            ]),
            BlockEntityData::Spawner(spawner) => {
                let base = spawner.get_spawner();
                json!([
                    base.spawn_delay(),
                    base.min_spawn_delay(),
                    base.max_spawn_delay(),
                    base.spawn_count(),
                    base.max_nearby_entities(),
                    base.required_player_range(),
                    base.spawn_range(),
                    base.next_spawn_data()
                        .map(|data| data.entity_to_spawn().to_string())
                ])
            }
            BlockEntityData::Campfire(campfire) => {
                json!([campfire.cooking_progress(), campfire.cooking_time()])
            }
            BlockEntityData::Brushable(brushable) => brushable
                .get_hit_direction()
                .map_or(Value::Null, |direction| json!(direction.to_string())),
            BlockEntityData::ShulkerBox(shulker_box) => json!([
                format!("{:?}", shulker_box.get_animation_status()),
                format!("{:08x}", shulker_box.get_progress(1.0).to_bits())
            ]),
            // The classes the suite writes no fields of.
            _ => Value::Null,
        };
        json!([
            entity.get_type().key(),
            entity.get_block_state().id(),
            entity.is_removed(),
            data
        ])
    }

    /// The block entity a state's block makes, or for the moving piston, which makes none, the
    /// type that's valid for it.
    fn create(pos: BlockPos, state: BlockState) -> BlockEntity {
        let r#type = BY_BLOCK[state.get_block().0 as usize].unwrap_or_else(|| {
            (0..BlockEntityType::registry_size())
                .map(BlockEntityType::by_id_or_throw)
                .find(|r#type| r#type.is_valid(state))
                .expect("a type valid for the block")
        });
        r#type.create(pos, state)
    }

    /// Every type made empty, and the tags a server sends of signs, banners, skulls, spawners,
    /// campfires, brushable blocks and shulker boxes, loaded like vanilla's client loads them.
    #[test]
    fn block_entities_load_like_vanillas() {
        let golden = Golden::load("net.minecraft.world.level.block.entity.BlockEntity");
        let registries = registry_access::tests::golden();
        let pos = BlockPos::new(1, 64, -2);
        let cases = golden.cases("load");
        assert!(cases.len() > 90);
        for (args, expected) in cases {
            let state = Block::block_state_registry()
                .by_id(args[0].as_i64().expect("a state") as i32)
                .expect("a state the registry has");
            let mut buffer = unpooled::wrapped_buffer(unhex(args[1].as_str().expect("hex")));
            let Tag::Compound(tag) = NbtIo::read_any_tag(
                &mut ByteBufInputStream::new(&mut buffer),
                &mut NbtAccounter::default_quota(),
            ) else {
                panic!("a compound");
            };
            let mut entity = create(pos, state);
            entity.load_with_components(&TagValueInput::create(
                &problem_reporter::DISCARDING,
                &registries,
                &tag,
            ));
            assert_eq!(fields(&entity), expected, "{state} {tag}");
        }
        golden.finish();
    }
}
