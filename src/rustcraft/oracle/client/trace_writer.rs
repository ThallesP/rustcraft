//! oracle/client/src/main/java/rustcraft/oracle/client/TraceWriter.java: a tick's line of
//! `trace.jsonl`, the player's state after the tick (TESTING.md), floats as their raw bits.

use crate::net::minecraft::client::player::local_player::LocalPlayer;
use crate::net::minecraft::world::entity::entity::EntityClass;

/// The line for the player after `tick` ran.
pub fn line(tick: i32, player: &LocalPlayer) -> String {
    let delta = player.get_delta_movement();
    format!(
        "{{\"tick\":{tick},\"pos\":[{},{},{}],\"delta\":[{},{},{}],\"y_rot\":{},\"x_rot\":{},\"on_ground\":{},\"horizontal_collision\":{},\"vertical_collision\":{},\"sprinting\":{},\"crouching\":{},\"pose\":\"{}\"}}",
        bits_f64(player.get_x()),
        bits_f64(player.get_y()),
        bits_f64(player.get_z()),
        bits_f64(delta.x),
        bits_f64(delta.y),
        bits_f64(delta.z),
        bits_f32(player.get_y_rot()),
        bits_f32(player.get_x_rot()),
        player.on_ground(),
        player.horizontal_collision,
        player.vertical_collision,
        player.is_sprinting(),
        player.is_crouching(),
        player.get_pose().name(),
    )
}

fn bits_f64(value: f64) -> String {
    format!("\"{:016x}\"", value.to_bits())
}

fn bits_f32(value: f32) -> String {
    format!("\"{:08x}\"", value.to_bits())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use md5::{Digest, Md5};

    use crate::net::minecraft::client::options::Options;
    use crate::net::minecraft::client::player::local_player::LocalPlayer;
    use crate::net::minecraft::core::block_pos::BlockPos;
    use crate::net::minecraft::world::entity::entity::EntityClass;
    use crate::net::minecraft::world::level::block::blocks::Blocks;
    use crate::net::minecraft::world::level::block::state::block_state::BlockState;
    use crate::net::minecraft::world::level::block_getter::BlockGetter;
    use crate::net::minecraft::world::level::border::world_border::WorldBorder;
    use crate::net::minecraft::world::level::collision_getter::CollisionGetter;
    use crate::net::minecraft::world::level::level::Level;
    use crate::net::minecraft::world::phys::aabb::AABB;
    use crate::net::minecraft::world::phys::shapes::voxel_shape::VoxelShape;
    use crate::net::minecraft::world::phys::vec3::Vec3;
    use crate::rustcraft::oracle::client::scenario::Scenario;

    /// The MD5 of vanilla's `vanilla/26.3/oracle/walk/trace.jsonl`, which isn't committed (it's
    /// vanilla's output) and which CI doesn't record (it runs no oracle client). Where the file
    /// is, the test compares line by line, and checks the digest is the file's.
    const VANILLA_WALK_TRACE_MD5: &str = "b4da2320a6595f98e58b7590d2107891";

    /// The walk scenario's world, the flat preset: bedrock at y = -64, two dirt, grass at -61,
    /// air above, every chunk loaded, no fluids, no other entities, the default border.
    struct FlatLevel {
        world_border: WorldBorder,
    }

    impl BlockGetter for FlatLevel {
        fn get_block_state(&self, pos: &BlockPos) -> BlockState {
            match pos.get_y() {
                -64 => Blocks::BEDROCK.default_block_state(),
                -63 | -62 => Blocks::DIRT.default_block_state(),
                -61 => Blocks::GRASS_BLOCK.default_block_state(),
                _ => Blocks::AIR.default_block_state(),
            }
        }
    }

    impl CollisionGetter for FlatLevel {
        fn get_world_border(&self) -> &WorldBorder {
            &self.world_border
        }

        fn get_chunk_for_collisions(
            &self,
            _chunk_x: i32,
            _chunk_z: i32,
        ) -> Option<&dyn BlockGetter> {
            Some(self)
        }

        fn get_entity_collisions(
            &self,
            _source: Option<&dyn EntityClass>,
            _test_area: &AABB,
        ) -> Vec<VoxelShape> {
            Vec::new()
        }
    }

    impl Level for FlatLevel {
        fn is_client_side(&self) -> bool {
            true
        }

        fn get_min_y(&self) -> i32 {
            -64
        }

        fn has_chunk_at(&self, _pos: &BlockPos) -> bool {
            true
        }

        fn chunk_has_fluid(&self, _: i32, _: i32, _: i32, _: i32) -> Option<bool> {
            Some(false)
        }
    }

    /// The keys a step holds, as the oracle client holds them (`holdOnly`).
    fn hold_only(options: &mut Options, keys: &[String]) {
        let held = |name: &str| keys.iter().any(|key| key == name);
        options.key_up.set_down(held("forward"));
        options.key_down.set_down(held("back"));
        options.key_left.set_down(held("left"));
        options.key_right.set_down(held("right"));
        options.key_jump.set_down(held("jump"));
        options.key_shift.set_down(held("sneak"));
        options.key_sprint.set_down(held("sprint"));
    }

    /// One client tick of the player: `ClientLevel.tickNonPassenger`, and what `sendPosition`
    /// does to the player (auto-jump from the options).
    fn tick(player: &mut LocalPlayer, level: &FlatLevel, options: &Options) {
        player.common_tick();
        player.tick(level, options);
        player.set_auto_jump_enabled(options.auto_jump());
    }

    /// Replays a scenario like the oracle client: the player teleported to its position, left
    /// to land (the oracle settles for 20 ticks and until it's on the ground), then each tick
    /// with the step's keys held and its look set first, one trace line after each.
    fn replay(scenario: &Scenario) -> Vec<String> {
        let level = FlatLevel {
            world_border: WorldBorder::new(),
        };
        let mut options = Options::new();
        let mut player = LocalPlayer::new(None);
        player.set_auto_jump_enabled(options.auto_jump());
        player.set_pos(Vec3::new(
            scenario.player.x,
            scenario.player.y,
            scenario.player.z,
        ));
        player.set_y_rot(scenario.player.y_rot);
        player.set_x_rot(scenario.player.x_rot);
        for _ in 0..20 {
            tick(&mut player, &level, &options);
        }
        assert!(player.on_ground(), "the player landed");

        let mut trace = Vec::new();
        for step in &scenario.steps {
            if step.screenshot.is_some() {
                continue;
            }
            hold_only(&mut options, &step.keys);
            for _ in 0..step.ticks {
                if let Some([y_rot, x_rot]) = step.look {
                    player.set_y_rot(y_rot);
                    player.set_x_rot(x_rot);
                }
                tick(&mut player, &level, &options);
                trace.push(super::line(trace.len() as i32, &player));
            }
        }
        trace
    }

    #[test]
    fn replays_vanilla_s_walk_trace() {
        let scenario = Scenario::load(Path::new("oracle/scenarios/walk.json")).unwrap();
        let trace = replay(&scenario);
        let text: String = trace.iter().map(|line| format!("{line}\n")).collect();

        let vanilla = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!(
            "vanilla/{}/oracle/walk/trace.jsonl",
            include_str!("../../../../MINECRAFT_VERSION").trim()
        ));
        if let Ok(expected) = std::fs::read_to_string(&vanilla) {
            assert_eq!(
                format!("{:x}", Md5::digest(expected.as_bytes())),
                VANILLA_WALK_TRACE_MD5,
                "vanilla's trace changed: update the digest"
            );
            for (index, (port, vanilla)) in trace.iter().zip(expected.lines()).enumerate() {
                assert_eq!(port, vanilla, "tick {index} differs");
            }
            assert_eq!(trace.len(), expected.lines().count());
        }
        assert_eq!(
            format!("{:x}", Md5::digest(text.as_bytes())),
            VANILLA_WALK_TRACE_MD5
        );
    }
}
