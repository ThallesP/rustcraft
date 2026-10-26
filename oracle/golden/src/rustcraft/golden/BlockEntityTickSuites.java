package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonNull;
import com.google.gson.JsonPrimitive;
import java.io.IOException;
import java.lang.reflect.Field;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Random;
import java.util.UUID;
import java.util.function.Consumer;
import net.minecraft.client.Camera;
import net.minecraft.client.Minecraft;
import net.minecraft.client.OptionInstance;
import net.minecraft.client.Options;
import net.minecraft.client.multiplayer.ClientChunkCache;
import net.minecraft.client.multiplayer.ClientLevel;
import net.minecraft.client.particle.Particle;
import net.minecraft.client.particle.ParticleEngine;
import net.minecraft.client.player.AbstractClientPlayer;
import net.minecraft.client.player.LocalPlayer;
import net.minecraft.client.renderer.GameRenderer;
import net.minecraft.client.resources.sounds.SoundInstance;
import net.minecraft.client.sounds.SoundEngine;
import net.minecraft.client.sounds.SoundManager;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.core.Holder;
import net.minecraft.core.particles.ParticleOptions;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.core.registries.Registries;
import net.minecraft.network.protocol.game.ClientboundBlockEventPacket;
import net.minecraft.network.protocol.game.ClientboundLevelChunkWithLightPacket;
import net.minecraft.network.RegistryFriendlyByteBuf;
import net.minecraft.resources.Identifier;
import net.minecraft.resources.ResourceKey;
import net.minecraft.server.level.ParticleStatus;
import net.minecraft.util.AbortableIterationConsumer;
import net.minecraft.util.RandomSource;
import net.minecraft.world.Difficulty;
import net.minecraft.world.TickRateManager;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.entity.MoverType;
import net.minecraft.world.item.DyeColor;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.block.AbstractSkullBlock;
import net.minecraft.world.level.block.BellBlock;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.CampfireBlock;
import net.minecraft.world.level.block.ChestBlock;
import net.minecraft.world.level.block.ConduitBlock;
import net.minecraft.world.level.block.NoteBlock;
import net.minecraft.world.level.block.ShulkerBoxBlock;
import net.minecraft.world.level.block.entity.BeaconBeamOwner;
import net.minecraft.world.level.block.entity.BeaconBlockEntity;
import net.minecraft.world.level.block.entity.BellBlockEntity;
import net.minecraft.world.level.block.entity.BlockEntity;
import net.minecraft.world.level.block.entity.ChestBlockEntity;
import net.minecraft.world.level.block.entity.ChestLidController;
import net.minecraft.world.level.block.entity.ConduitBlockEntity;
import net.minecraft.world.level.block.entity.DecoratedPotBlockEntity;
import net.minecraft.world.level.block.entity.EnchantingTableBlockEntity;
import net.minecraft.world.level.block.entity.EnderChestBlockEntity;
import net.minecraft.world.level.block.entity.PotentSulfurBlockEntity;
import net.minecraft.world.level.block.entity.ShulkerBoxBlockEntity;
import net.minecraft.world.level.block.entity.SkullBlockEntity;
import net.minecraft.world.level.block.entity.SpawnerBlockEntity;
import net.minecraft.world.level.block.entity.TheEndGatewayBlockEntity;
import net.minecraft.world.level.block.entity.TickingBlockEntity;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.block.state.properties.BlockStateProperties;
import net.minecraft.world.level.block.state.properties.ChestType;
import net.minecraft.world.level.block.state.properties.NoteBlockInstrument;
import net.minecraft.world.level.block.state.properties.PotentSulfurState;
import net.minecraft.world.level.border.WorldBorder;
import net.minecraft.world.level.chunk.LevelChunk;
import net.minecraft.world.level.chunk.PalettedContainerFactory;
import net.minecraft.world.level.dimension.BuiltinDimensionTypes;
import net.minecraft.world.level.entity.EntityTypeTest;
import net.minecraft.world.level.entity.LevelEntityGetter;
import net.minecraft.world.phys.AABB;
import net.minecraft.world.phys.Vec3;

/**
 * The client's block entities ticking (TESTING.md, "Block entities"): {@code programs}, each a level made without its
 * constructor over the walk's 3 by 3 chunks around the spawn (a real {@code ClientLevel}, its {@code ClientChunkCache},
 * chunks and ticker lists), a course of block entity blocks set on it, then ticks with the server's block events and block
 * changes in between, each tick {@code tickBlockEntities}. A program is {@code [levelSeed, enchantingSeed, particles,
 * player, gameTime, ops]}, an op {@code ["set", x, y, z, state]} ({@code setBlock} with flags 18), {@code ["event", x, y, z,
 * b0, b1]} ({@code blockEvent}) or {@code ["tick"]}; its result a line for each tick, {@code [events, tickers, blockEntities]}:
 * the particles the particle engine was handed and the moves the player was handed meanwhile, the level's tickers in order
 * ({@code [type, x, y, z, removed]}), and each block entity of the course's chunks in each chunk's map order with its state
 * ({@code state}), then each random's next long. The player is a {@code LocalPlayer} made without its constructor whose
 * {@code move} records what it was handed, and the level's only entity.
 */
final class BlockEntityTickSuites {
    private BlockEntityTickSuites() {
    }

    static List<Suite> suites() {
        return List.of(new OpenHashMapSuite(), new BlockEventPacketSuite(), new TickSuite(), new LevelSuites.ClientLevelSuite());
    }

    private static JsonElement bits(final float value) {
        return new JsonPrimitive(String.format("%08x", Float.floatToRawIntBits(value)));
    }

    private static JsonElement bits(final double value) {
        return new JsonPrimitive(String.format("%016x", Double.doubleToRawLongBits(value)));
    }

    private static Object get(final Class<?> owner, final Object object, final String name) {
        try {
            Field field = owner.getDeclaredField(name);
            field.setAccessible(true);
            return field.get(object);
        } catch (ReflectiveOperationException e) {
            throw new IllegalStateException(e);
        }
    }

    private static void set(final Class<?> owner, final Object object, final String name, final Object value) {
        try {
            Field field = owner.getDeclaredField(name);
            field.setAccessible(true);
            field.set(object, value);
        } catch (ReflectiveOperationException e) {
            throw new IllegalStateException(e);
        }
    }

    /**
     * fastutil's {@code Object2ObjectOpenHashMap}, a chunk's block entities by position, whose order the client iterates them in:
     * {@code programs}, ops on {@code new Object2ObjectOpenHashMap<BlockPos, Integer>()}, each op's output: {@code ["put", x, y,
     * z, value]} → the value it replaced or null, {@code ["remove", x, y, z]} → the value or null, {@code ["get", x, y, z]},
     * {@code ["clear"]} → null, {@code ["size"]}, {@code ["iterate"]} → the entries in order, {@code [x, y, z, value]}. Keys are
     * blocks of a chunk, from a small pool now and then so they're put and removed again; programs put up to 300 keys, so the
     * table grows, and remove most of them, so it shrinks.
     */
    static final class OpenHashMapSuite implements Suite {
        @Override
        public String name() {
            return it.unimi.dsi.fastutil.objects.Object2ObjectOpenHashMap.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            Random random = new Random(0x0b2068L);
            for (int program = 0; program < 240; program++) {
                it.unimi.dsi.fastutil.objects.Object2ObjectOpenHashMap<BlockPos, Integer> map = new it.unimi.dsi.fastutil.objects.Object2ObjectOpenHashMap<>();
                List<BlockPos> pool = new ArrayList<>();
                for (int i = 0; i < 8; i++) {
                    pool.add(key(random));
                }
                List<BlockPos> added = new ArrayList<>();
                int length = 20 + random.nextInt(program % 4 == 0 ? 700 : 120);
                JsonArray ops = new JsonArray();
                JsonArray outputs = new JsonArray();
                for (int i = 0; i < length; i++) {
                    int roll = random.nextInt(100);
                    BlockPos pos = random.nextInt(4) == 0 ? pool.get(random.nextInt(pool.size()))
                        : roll >= 50 && roll < 85 && !added.isEmpty() ? added.get(random.nextInt(added.size())) : key(random);
                    // Programs grow the map for their first half, then mostly empty it.
                    boolean growing = i < length / 2;
                    JsonElement output;
                    if (roll < (growing ? 55 : 15)) {
                        int value = random.nextInt(1000);
                        ops.add(ChunkSuites.array("put", pos.getX(), pos.getY(), pos.getZ(), value));
                        Integer previous = map.put(pos, value);
                        added.add(pos);
                        output = previous == null ? JsonNull.INSTANCE : new JsonPrimitive(previous);
                    } else if (roll < (growing ? 75 : 85)) {
                        ops.add(ChunkSuites.array("remove", pos.getX(), pos.getY(), pos.getZ()));
                        Integer previous = map.remove(pos);
                        output = previous == null ? JsonNull.INSTANCE : new JsonPrimitive(previous);
                    } else if (roll < 90) {
                        ops.add(ChunkSuites.array("get", pos.getX(), pos.getY(), pos.getZ()));
                        Integer value = map.get(pos);
                        output = value == null ? JsonNull.INSTANCE : new JsonPrimitive(value);
                    } else if (roll < 91) {
                        ops.add(ChunkSuites.array("clear"));
                        map.clear();
                        output = JsonNull.INSTANCE;
                    } else if (roll < 94) {
                        ops.add(ChunkSuites.array("size"));
                        output = new JsonPrimitive(map.size());
                    } else {
                        ops.add(ChunkSuites.array("iterate"));
                        output = entries(map);
                    }
                    outputs.add(output);
                }
                ops.add(ChunkSuites.array("iterate"));
                outputs.add(entries(map));
                writer.result("programs", ChunkSuites.array(ops), outputs, null);
            }
        }

        /** A block of a chunk at the walk's spawn, near the ground mostly. */
        private static BlockPos key(final Random random) {
            int y = random.nextInt(3) == 0 ? -64 + random.nextInt(384) : -64 + random.nextInt(16);
            return new BlockPos(random.nextInt(16), y, random.nextInt(16));
        }

        private static JsonArray entries(final it.unimi.dsi.fastutil.objects.Object2ObjectOpenHashMap<BlockPos, Integer> map) {
            JsonArray entries = new JsonArray();
            for (Map.Entry<BlockPos, Integer> entry : map.entrySet()) {
                entries.add(ChunkSuites.array(entry.getKey().getX(), entry.getKey().getY(), entry.getKey().getZ(), entry.getValue()));
            }
            return entries;
        }
    }

    /**
     * ClientboundBlockEventPacket: decodes positions (blocks around the walk's spawn, at the packing's limits, any longs),
     * both bytes (every one of {@code b0}, and edges of {@code b1}) and block ids (every block's edges, the registry's size,
     * -1 and any), cut short or with more after: {@code [[x, y, z, b0, b1, block], unread, written]}.
     */
    static final class BlockEventPacketSuite implements Suite {
        @Override
        public String name() {
            return ClientboundBlockEventPacket.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            Random random = new Random(0xb1e7);
            int size = BuiltInRegistries.BLOCK.size();
            List<Integer> ids = new ArrayList<>(List.of(0, 1, size - 1, size, -1, Integer.MAX_VALUE, Integer.MIN_VALUE));
            for (int i = 0; i < 150; i++) {
                ids.add(random.nextInt(size));
            }
            for (int b0 = 0; b0 < 256; b0++) {
                ids.add(random.nextInt(size));
            }
            for (int i = 0; i < ids.size(); i++) {
                long pos = switch (i % 4) {
                    case 0 -> new BlockPos(random.nextInt(33) - 16, -64 + random.nextInt(8), random.nextInt(33) - 16).asLong();
                    case 1 -> new BlockPos(33554431 * (random.nextBoolean() ? 1 : -1), 2047, -33554432).asLong();
                    default -> random.nextLong();
                };
                RegistryFriendlyByteBuf out = LevelChunkSuites.writer();
                out.writeLong(pos);
                out.writeByte(i < 413 && i >= 157 ? i - 157 : random.nextInt(256));
                out.writeByte(switch (i % 5) {
                    case 0 -> 0;
                    case 1 -> 1;
                    case 2 -> 255;
                    default -> random.nextInt(256);
                });
                out.writeVarInt(ids.get(i));
                byte[] bytes = LevelChunkSuites.bytes(out);
                for (byte[] variant : i % 6 == 0 ? LevelChunkSuites.variants(random, bytes) : List.of(bytes)) {
                    LevelChunkSuites.decodeCase(
                        writer, "STREAM_CODEC.decode", variant, LevelChunkSuites::reader, LevelChunkSuites::writer, ClientboundBlockEventPacket.STREAM_CODEC,
                        packet -> ChunkSuites.array(
                            packet.getPos().getX(), packet.getPos().getY(), packet.getPos().getZ(), packet.getB0(), packet.getB1(),
                            BuiltInRegistries.BLOCK.getKey(packet.getBlock()).toString()
                        )
                    );
                }
            }
        }
    }

    /** What the particle engine and the player were handed since the last tick's line. */
    private static final List<JsonArray> EVENTS = new ArrayList<>();

    /** The game's particle engine, made without its constructor: it records what it's handed. */
    static class Particles extends ParticleEngine {
        Particles() {
            super(null, null);
        }

        @Override
        public Particle createParticle(
            final ParticleOptions options, final double x, final double y, final double z, final double xd, final double yd, final double zd
        ) {
            EVENTS.add(ChunkSuites.array(
                "particle", BuiltInRegistries.PARTICLE_TYPE.getKey(options.getType()).toString(), bits(x), bits(y), bits(z), bits(xd), bits(yd), bits(zd)
            ));
            return null;
        }
    }

    /** The game's sound manager, made without its constructor: it records what it's handed. */
    static class Sounds extends SoundManager {
        Sounds() {
            super(null);
        }

        @Override
        public SoundEngine.PlayResult play(final SoundInstance instance) {
            EVENTS.add(LevelSuites.AnimateTick.sound(instance, -1));
            return null;
        }

        @Override
        public void playDelayed(final SoundInstance instance, final int delay) {
            EVENTS.add(LevelSuites.AnimateTick.sound(instance, delay));
        }
    }

    /** The player, made without its constructor: it records the moves it's handed, and is no spectator and alive. */
    static class Player extends LocalPlayer {
        Player() {
            super(null, null, null, null, null, null, false, null, null);
        }

        @Override
        public void move(final MoverType moverType, final Vec3 delta) {
            EVENTS.add(ChunkSuites.array("move", moverType.name(), bits(delta.x), bits(delta.y), bits(delta.z)));
        }

        /** The blocks' effects on what a piston moves, which the port doesn't have (PistonMovingBlockEntity's header). */
        @Override
        public void applyEffectsFromBlocks(final Vec3 from, final Vec3 to) {
        }

        @Override
        public void removeLatestMovementRecording() {
        }

        /** A slime block's bounce, recorded. */
        @Override
        public void setDeltaMovement(final Vec3 deltaMovement) {
            EVENTS.add(ChunkSuites.array("motion", bits(deltaMovement.x), bits(deltaMovement.y), bits(deltaMovement.z)));
            super.setDeltaMovement(deltaMovement);
        }

        @Override
        public boolean isSpectator() {
            return false;
        }

        @Override
        public boolean isAlive() {
            return true;
        }
    }

    /**
     * The level: a {@code ClientLevel} made without its constructor, whose chunks, block entities and tickers are its own,
     * with its renderer's hooks left out and the player as its only entity.
     */
    static class Level extends ClientLevel {
        private Level() {
            super(null, null, null, null, 0, 0, null, false, 0L, 0);
        }

        Player player;

        @Override
        public void sendBlockUpdated(final BlockPos pos, final BlockState old, final BlockState current, final int updateFlags) {
        }

        @Override
        public void setBlocksDirty(final BlockPos pos, final BlockState oldState, final BlockState newState) {
        }

        @Override
        public void setSectionDirtyWithNeighbors(final int chunkX, final int chunkY, final int chunkZ) {
        }

        @Override
        public void setSectionRangeDirty(final int minSectionX, final int minSectionY, final int minSectionZ, final int maxSectionX, final int maxSectionY, final int maxSectionZ) {
        }

        @Override
        public void onBlockEntityAdded(final BlockEntity blockEntity) {
        }

        @Override
        public void onChunkLoaded(final ChunkPos pos) {
        }

        @Override
        public List<AbstractClientPlayer> players() {
            return List.of(this.player);
        }

        @Override
        protected LevelEntityGetter<Entity> getEntities() {
            Player player = this.player;
            return new LevelEntityGetter<>() {
                @Override
                public Entity get(final int id) {
                    return id == player.getId() ? player : null;
                }

                @Override
                public Entity get(final UUID id) {
                    return null;
                }

                @Override
                public Iterable<Entity> getAll() {
                    return List.of(player);
                }

                @Override
                public <U extends Entity> void get(final EntityTypeTest<Entity, U> type, final AbortableIterationConsumer<U> consumer) {
                    U entity = type.tryCast(player);
                    if (entity != null) {
                        consumer.accept(entity);
                    }
                }

                @Override
                public void get(final AABB bb, final Consumer<Entity> output) {
                    if (player.getBoundingBox().intersects(bb)) {
                        output.accept(player);
                    }
                }

                @Override
                public <U extends Entity> void get(final EntityTypeTest<Entity, U> type, final AABB bb, final AbortableIterationConsumer<U> consumer) {
                    U entity = type.tryCast(player);
                    if (entity != null && player.getBoundingBox().intersects(bb)) {
                        consumer.accept(entity);
                    }
                }
            };
        }
    }

    /** A block entity's fields its ticker and block events change, by class, after its type, position and whether it's removed. */
    static JsonElement state(final net.minecraft.world.level.Level level, final BlockEntity entity) {
        JsonElement fields = switch (entity) {
            case ChestBlockEntity chest -> lid((ChestLidController)get(ChestBlockEntity.class, chest, "chestLidController"));
            case EnderChestBlockEntity chest -> lid((ChestLidController)get(EnderChestBlockEntity.class, chest, "chestLidController"));
            case ShulkerBoxBlockEntity box -> {
                BlockState state = box.getBlockState();
                yield ChunkSuites.array(
                    box.getAnimationStatus().name(), bits(box.getProgress(0.0F)), bits(box.getProgress(1.0F)), (Integer)get(ShulkerBoxBlockEntity.class, box, "openCount"),
                    boxes(state.getShape(level, box.getBlockPos())), boxes(state.getBlockSupportShape(level, box.getBlockPos()))
                );
            }
            case BellBlockEntity bell -> ChunkSuites.array(
                bell.ticks, bell.shaking, bell.clickDirection == null ? null : bell.clickDirection.name(), (Boolean)get(BellBlockEntity.class, bell, "resonating"),
                (Integer)get(BellBlockEntity.class, bell, "resonationTicks"), (Long)get(BellBlockEntity.class, bell, "lastRingTimestamp")
            );
            case TheEndGatewayBlockEntity gateway -> ChunkSuites.array(
                (Long)get(TheEndGatewayBlockEntity.class, gateway, "age"), (Integer)get(TheEndGatewayBlockEntity.class, gateway, "teleportCooldown")
            );
            case DecoratedPotBlockEntity pot -> ChunkSuites.array(pot.wobbleStartedAtTick, pot.lastWobbleStyle == null ? null : pot.lastWobbleStyle.name());
            case SpawnerBlockEntity spawner -> ChunkSuites.array(
                (Integer)get(net.minecraft.world.level.BaseSpawner.class, spawner.getSpawner(), "spawnDelay"), bits(spawner.getSpawner().getSpin()),
                bits(spawner.getSpawner().getOSpin())
            );
            case SkullBlockEntity skull -> ChunkSuites.array(
                (Boolean)get(SkullBlockEntity.class, skull, "isAnimating"), (Integer)get(SkullBlockEntity.class, skull, "animationTickCount")
            );
            case ConduitBlockEntity conduit -> {
                JsonArray blocks = new JsonArray();
                @SuppressWarnings("unchecked")
                List<BlockPos> effectBlocks = (List<BlockPos>)get(ConduitBlockEntity.class, conduit, "effectBlocks");
                effectBlocks.forEach(pos -> blocks.add(ChunkSuites.array(pos.getX(), pos.getY(), pos.getZ())));
                yield ChunkSuites.array(conduit.tickCount, bits((Float)get(ConduitBlockEntity.class, conduit, "activeRotation")), conduit.isActive(), conduit.isHunting(), blocks);
            }
            case BeaconBlockEntity beacon -> {
                @SuppressWarnings("unchecked")
                List<BeaconBeamOwner.Section> checking = (List<BeaconBeamOwner.Section>)get(BeaconBlockEntity.class, beacon, "checkingBeamSections");
                yield ChunkSuites.array(
                    sections(beacon.getBeamSections()), sections(checking), (Integer)get(BeaconBlockEntity.class, beacon, "levels"),
                    (Integer)get(BeaconBlockEntity.class, beacon, "lastCheckY")
                );
            }
            case EnchantingTableBlockEntity table -> ChunkSuites.array(
                table.time, bits(table.flip), bits(table.oFlip), bits(table.flipT), bits(table.flipA), bits(table.open), bits(table.oOpen), bits(table.rot),
                bits(table.oRot), bits(table.tRot)
            );
            case PotentSulfurBlockEntity sulfur -> ChunkSuites.array(sulfur.eruptionTick);
            case net.minecraft.world.level.block.piston.PistonMovingBlockEntity piston -> ChunkSuites.array(
                Block.getId(piston.getMovedState()), piston.getDirection().name(), piston.isExtending(), piston.isSourcePiston(),
                bits(piston.getProgress(0.0F)), bits(piston.getProgress(1.0F)), piston.getLastTicked(),
                (Integer)get(net.minecraft.world.level.block.piston.PistonMovingBlockEntity.class, piston, "deathTicks"),
                boxes(piston.getCollisionShape(level, piston.getBlockPos()))
            );
            default -> JsonNull.INSTANCE;
        };
        BlockPos pos = entity.getBlockPos();
        return ChunkSuites.array(
            BuiltInRegistries.BLOCK_ENTITY_TYPE.getKey(entity.getType()).toString(), pos.getX(), pos.getY(), pos.getZ(), entity.isRemoved(), fields
        );
    }

    private static JsonElement lid(final ChestLidController lid) {
        return ChunkSuites.array((Boolean)get(ChestLidController.class, lid, "shouldBeOpen"), bits(lid.getOpenness(0.0F)), bits(lid.getOpenness(1.0F)));
    }

    private static JsonElement sections(final List<BeaconBeamOwner.Section> sections) {
        JsonArray result = new JsonArray();
        sections.forEach(section -> result.add(ChunkSuites.array(section.getColor(), section.getHeight())));
        return result;
    }

    private static JsonElement boxes(final net.minecraft.world.phys.shapes.VoxelShape shape) {
        JsonArray result = new JsonArray();
        for (AABB box : shape.toAabbs()) {
            result.add(ChunkSuites.array(bits(box.minX), bits(box.minY), bits(box.minZ), bits(box.maxX), bits(box.maxY), bits(box.maxZ)));
        }
        return result;
    }

    /**
     * {@code ClientLevel.tickWeatherEffects} ({@code weather}, in ClientLevel's file): {@code [levelSeed, particles,
     * weatherRadius, camera, gameTime, course, ticks]} → for each tick (each {@code [rainLevel, camera]}), the particles and
     * sounds it makes and {@code rainSoundTime} after, then the level random's next long. The level is the tick suite's
     * {@code ClientLevel} over the walk's chunks (its biome manager over them, plains), a course of blocks the rain falls on
     * set first: slabs, fences, glass, leaves, torches, carpets, snow, water and lava at any level, magma, campfires lit or
     * not, pillars up past the camera's ten blocks either way, and a roof of glass over the camera a quarter of the time.
     */
    static final class Weather {
        private Weather() {
        }

        static void write(final GoldenWriter writer) throws IOException {
            sun.misc.Unsafe unsafe;
            try {
                Field theUnsafe = sun.misc.Unsafe.class.getDeclaredField("theUnsafe");
                theUnsafe.setAccessible(true);
                unsafe = (sun.misc.Unsafe)theUnsafe.get(null);
            } catch (ReflectiveOperationException e) {
                throw new IllegalStateException(e);
            }
            ShapeSuites.BlockStateShapeSuite.bindBlockTags();
            bindFluidTags();

            Minecraft minecraft;
            OptionInstance<?> particles;
            OptionInstance<?> weatherRadius;
            Camera camera;
            try {
                minecraft = (Minecraft)unsafe.allocateInstance(Minecraft.class);
                Options options = (Options)unsafe.allocateInstance(Options.class);
                particles = (OptionInstance<?>)unsafe.allocateInstance(OptionInstance.class);
                weatherRadius = (OptionInstance<?>)unsafe.allocateInstance(OptionInstance.class);
                set(Options.class, options, "particles", particles);
                set(Options.class, options, "weatherRadius", weatherRadius);
                GameRenderer gameRenderer = (GameRenderer)unsafe.allocateInstance(GameRenderer.class);
                camera = (Camera)unsafe.allocateInstance(Camera.class);
                set(GameRenderer.class, gameRenderer, "mainCamera", camera);
                set(Minecraft.class, minecraft, "options", options);
                set(Minecraft.class, minecraft, "gameRenderer", gameRenderer);
                set(Minecraft.class, minecraft, "particleEngine", unsafe.allocateInstance(Particles.class));
                set(Minecraft.class, minecraft, "soundManager", unsafe.allocateInstance(Sounds.class));
            } catch (ReflectiveOperationException e) {
                throw new IllegalStateException(e);
            }
            Field instance;
            Object previousInstance;
            try {
                instance = Minecraft.class.getDeclaredField("instance");
                instance.setAccessible(true);
                previousInstance = instance.get(null);
                instance.set(null, minecraft);
            } catch (ReflectiveOperationException e) {
                throw new IllegalStateException(e);
            }

            Random random = new Random(0x4a1e);
            ParticleStatus[] statuses = ParticleStatus.values();
            List<BlockState> blocks = List.of(
                Blocks.STONE.defaultBlockState(), Blocks.OAK_SLAB.defaultBlockState(),
                Blocks.OAK_SLAB.defaultBlockState().setValue(BlockStateProperties.SLAB_TYPE, net.minecraft.world.level.block.state.properties.SlabType.TOP),
                Blocks.OAK_FENCE.defaultBlockState(), Blocks.GLASS.defaultBlockState(), Blocks.OAK_LEAVES.defaultBlockState(), Blocks.TORCH.defaultBlockState(),
                Blocks.MOSS_CARPET.defaultBlockState(), Blocks.SNOW.defaultBlockState().setValue(BlockStateProperties.LAYERS, 3),
                Blocks.WATER.defaultBlockState(), Blocks.WATER.defaultBlockState().setValue(BlockStateProperties.LEVEL, 3),
                Blocks.LAVA.defaultBlockState(), Blocks.LAVA.defaultBlockState().setValue(BlockStateProperties.LEVEL, 5),
                Blocks.MAGMA_BLOCK.defaultBlockState(), Blocks.CAMPFIRE.defaultBlockState(),
                Blocks.CAMPFIRE.defaultBlockState().setValue(CampfireBlock.LIT, false), Blocks.SOUL_CAMPFIRE.defaultBlockState(),
                Blocks.HOPPER.defaultBlockState(), Blocks.OAK_STAIRS.defaultBlockState(), Blocks.FARMLAND.defaultBlockState()
            );
            try {
                for (int program = 0; program < 64; program++) {
                    long levelSeed = random.nextLong();
                    ParticleStatus status = statuses[program % statuses.length];
                    int radius = program % 5 == 0 ? 3 + random.nextInt(7) : 10;
                    long gameTime = random.nextInt(100000);
                    Level level = TickSuite.level(unsafe, minecraft, levelSeed, gameTime);
                    set(net.minecraft.world.level.Level.class, level, "biomeManager", new net.minecraft.world.level.biome.BiomeManager(level, 0L));
                    set(ClientLevel.class, level, "seaLevel", 63);
                    set(OptionInstance.class, particles, "value", status);
                    set(OptionInstance.class, weatherRadius, "value", radius);

                    // The course: blocks on the ground, some on pillars, around the camera's columns.
                    JsonArray course = new JsonArray();
                    for (int i = 0; i < 160; i++) {
                        int x = random.nextInt(25) - 12;
                        int z = random.nextInt(25) - 12;
                        int height = random.nextInt(6) == 0 ? random.nextInt(14) : 0;
                        for (int y = -60; y < -60 + height; y++) {
                            course.add(ChunkSuites.array(x, y, z, Block.getId(Blocks.STONE.defaultBlockState())));
                        }
                        course.add(ChunkSuites.array(x, -60 + height, z, Block.getId(blocks.get(random.nextInt(blocks.size())))));
                    }
                    Vec3 camera0 = new Vec3(random.nextDouble() * 10.0 - 5.0, -58.5 + random.nextDouble() * (random.nextInt(4) == 0 ? 20.0 : 3.0), random.nextDouble() * 10.0 - 5.0);
                    if (program % 4 == 1) {
                        // A roof over the camera, which the rain sounds above (WEATHER_RAIN_ABOVE).
                        BlockPos center = BlockPos.containing(camera0);
                        for (int x = -3; x <= 3; x++) {
                            for (int z = -3; z <= 3; z++) {
                                course.add(ChunkSuites.array(center.getX() + x, center.getY() + 4, center.getZ() + z, Block.getId(Blocks.GLASS.defaultBlockState())));
                            }
                        }
                    }
                    for (JsonElement placement : course) {
                        JsonArray place = placement.getAsJsonArray();
                        level.setBlock(new BlockPos(place.get(0).getAsInt(), place.get(1).getAsInt(), place.get(2).getAsInt()), Block.stateById(place.get(3).getAsInt()), 18);
                    }

                    JsonArray ticks = new JsonArray();
                    JsonArray results = new JsonArray();
                    EVENTS.clear();
                    for (int tick = 0; tick < 24; tick++) {
                        float rain = switch (random.nextInt(6)) {
                            case 0 -> 0.0F;
                            case 1 -> random.nextFloat();
                            case 2 -> 0.3F;
                            default -> 1.0F;
                        };
                        Vec3 cameraPosition = random.nextInt(8) == 0 ? camera0.add(random.nextDouble() * 4.0 - 2.0, 0.0, random.nextDouble() * 4.0 - 2.0) : camera0;
                        ((ClientLevel.ClientLevelData)level.getLevelData()).setGameTime(gameTime + tick);
                        level.setRainLevel(rain);
                        set(Camera.class, camera, "position", cameraPosition);
                        level.tickWeatherEffects();
                        JsonArray events = new JsonArray();
                        EVENTS.forEach(events::add);
                        EVENTS.clear();
                        results.add(ChunkSuites.array(events, (Integer)get(ClientLevel.class, level, "rainSoundTime")));
                        ticks.add(ChunkSuites.array(bits(rain), ChunkSuites.array(bits(cameraPosition.x), bits(cameraPosition.y), bits(cameraPosition.z))));
                    }
                    results.add(ChunkSuites.array("end", level.getRandom().nextLong()));
                    writer.result("weather", ChunkSuites.array(levelSeed, status.name(), radius, gameTime, course, ticks), results, null);
                }
            } finally {
                try {
                    instance.set(null, previousInstance);
                } catch (ReflectiveOperationException e) {
                    throw new IllegalStateException(e);
                }
            }
        }
    }

    /**
     * Binds the fluids' tags from the vanilla pack, as the server's {@code update_tags} binds them (a conduit's water is
     * {@code isWaterAt}'s tag, the rain smokes on lava). The other suites bind only the blocks', so these suites run last,
     * with ClientLevel's.
     */
    static void bindFluidTags() {
        net.minecraft.server.packs.resources.CloseableResourceManager resources = new net.minecraft.server.packs.resources.MultiPackResourceManager(
            net.minecraft.server.packs.PackType.SERVER_DATA,
            net.minecraft.server.packs.repository.ServerPacksSource.createVanillaPackSource().asResourcesSupplier().openResources(null, null).toList()
        );
        net.minecraft.tags.TagLoader.loadTagsForExistingRegistries(
                resources, net.minecraft.server.RegistryLayer.createRegistryAccess().getLayer(net.minecraft.server.RegistryLayer.STATIC)
            )
            .stream()
            .filter(tags -> tags.key() == Registries.FLUID)
            .forEach(net.minecraft.core.Registry.PendingTags::apply);
    }

    static final class TickSuite implements Suite {
        /** The course's corner and size: blocks set within it, on the walk's flat ground (its grass at y -61). */
        private static final int MIN_X = -8;
        private static final int MIN_Z = -8;
        private static final int SIZE = 17;

        @Override
        public String name() {
            return TickingBlockEntity.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            Field theUnsafe;
            sun.misc.Unsafe unsafe;
            try {
                theUnsafe = sun.misc.Unsafe.class.getDeclaredField("theUnsafe");
                theUnsafe.setAccessible(true);
                unsafe = (sun.misc.Unsafe)theUnsafe.get(null);
            } catch (ReflectiveOperationException e) {
                throw new IllegalStateException(e);
            }
            ShapeSuites.BlockStateShapeSuite.bindBlockTags();
            bindFluidTags();
            BlockEntitySuites.bindComponents();

            Minecraft minecraft;
            OptionInstance<?> particles;
            Camera camera;
            try {
                minecraft = (Minecraft)unsafe.allocateInstance(Minecraft.class);
                Options options = (Options)unsafe.allocateInstance(Options.class);
                particles = (OptionInstance<?>)unsafe.allocateInstance(OptionInstance.class);
                set(Options.class, options, "particles", particles);
                GameRenderer gameRenderer = (GameRenderer)unsafe.allocateInstance(GameRenderer.class);
                camera = (Camera)unsafe.allocateInstance(Camera.class);
                set(GameRenderer.class, gameRenderer, "mainCamera", camera);
                set(Minecraft.class, minecraft, "options", options);
                set(Minecraft.class, minecraft, "gameRenderer", gameRenderer);
                set(Minecraft.class, minecraft, "particleEngine", unsafe.allocateInstance(Particles.class));
                set(Minecraft.class, minecraft, "soundManager", unsafe.allocateInstance(Sounds.class));
            } catch (ReflectiveOperationException e) {
                throw new IllegalStateException(e);
            }
            Field instance;
            Object previousInstance;
            try {
                instance = Minecraft.class.getDeclaredField("instance");
                instance.setAccessible(true);
                previousInstance = instance.get(null);
                instance.set(null, minecraft);
            } catch (ReflectiveOperationException e) {
                throw new IllegalStateException(e);
            }
            RandomSource enchantingRandom = (RandomSource)get(EnchantingTableBlockEntity.class, null, "RANDOM");

            Random random = new Random(0xb10cce17L);
            ParticleStatus[] statuses = ParticleStatus.values();
            try {
                for (int program = 0; program < 48; program++) {
                    long levelSeed = random.nextLong();
                    long enchantingSeed = random.nextLong();
                    ParticleStatus status = statuses[program % statuses.length];
                    long gameTime = random.nextInt(4) == 0 ? 0L : random.nextInt(2000);
                    Level level = level(unsafe, minecraft, levelSeed, gameTime);
                    Player player;
                    try {
                        player = (Player)unsafe.allocateInstance(Player.class);
                    } catch (ReflectiveOperationException e) {
                        throw new IllegalStateException(e);
                    }
                    set(Entity.class, player, "id", 7);
                    set(Entity.class, player, "type", net.minecraft.world.entity.EntityTypes.PLAYER);
                    level.player = player;
                    // The game's player, which a sound played for no one (`playSeededSound(null, …)`) isn't played for.
                    minecraft.player = player;
                    set(OptionInstance.class, particles, "value", status);
                    enchantingRandom.setSeed(enchantingSeed);

                    List<JsonArray> ops = new ArrayList<>();
                    List<BlockPos> course = course(random, ops);
                    // The player: on a shulker box opening upwards, beside one opening towards it, on another block of the
                    // course, or far from it. A box it's near gets most of the block events.
                    BlockPos stand = course.get(2 + random.nextInt(course.size() - 2));
                    BlockPos box = null;
                    Vec3 position;
                    switch (program % 3) {
                        case 0 -> {
                            box = stand;
                            place(ops, box, Blocks.SHULKER_BOX.defaultBlockState().setValue(ShulkerBoxBlock.FACING, Direction.UP));
                            position = new Vec3(box.getX() + 0.5 + (random.nextDouble() - 0.5) * 0.4, box.getY() + 1.0, box.getZ() + 0.5 + (random.nextDouble() - 0.5) * 0.4);
                        }
                        case 1 -> {
                            box = stand;
                            Direction facing = Direction.Plane.HORIZONTAL.getRandomDirection(RandomSource.create(random.nextLong()));
                            place(ops, box, Blocks.SHULKER_BOX.defaultBlockState().setValue(ShulkerBoxBlock.FACING, facing));
                            BlockPos beside = box.relative(facing);
                            position = new Vec3(
                                beside.getX() + 0.5 - facing.getStepX() * 0.15 * random.nextDouble(), beside.getY(),
                                beside.getZ() + 0.5 - facing.getStepZ() * 0.15 * random.nextDouble()
                            );
                        }
                        default -> position = random.nextInt(4) == 0
                            ? new Vec3(40.5, -60.0, 40.5)
                            : new Vec3(stand.getX() + 0.5 + (random.nextDouble() - 0.5) * 0.4, stand.getY() + 1.0, stand.getZ() + 0.5 + (random.nextDouble() - 0.5) * 0.4);
                    }
                    set(Entity.class, player, "position", position);
                    set(Entity.class, player, "bb", new AABB(position.x - 0.3, position.y, position.z - 0.3, position.x + 0.3, position.y + 1.8, position.z + 0.3));
                    set(Camera.class, camera, "position", position.add(0.0, 1.62, 0.0));
                    int ticks = 40;
                    for (int tick = 0; tick < ticks; tick++) {
                        for (int i = random.nextInt(4); i > 0; i--) {
                            if (random.nextInt(4) == 0) {
                                change(random, course, ops, level);
                            } else {
                                BlockPos pos = box != null && random.nextBoolean() ? box : course.get(random.nextInt(course.size()));
                                int b0 = new int[]{1, 1, 1, 0, 2, 3}[random.nextInt(6)];
                                int b1 = pos == box ? random.nextInt(3) : new int[]{0, 1, 1, 2, 3, 5, 255, random.nextInt(256)}[random.nextInt(8)];
                                ops.add(ChunkSuites.array("event", pos.getX(), pos.getY(), pos.getZ(), b0, b1));
                            }
                        }
                        ops.add(ChunkSuites.array("tick"));
                    }

                    JsonArray results = new JsonArray();
                    EVENTS.clear();
                    long time = gameTime;
                    for (JsonArray op : ops) {
                        switch (op.get(0).getAsString()) {
                            case "set" -> level.setBlock(
                                new BlockPos(op.get(1).getAsInt(), op.get(2).getAsInt(), op.get(3).getAsInt()), Block.stateById(op.get(4).getAsInt()), 18
                            );
                            case "event" -> {
                                BlockPos pos = new BlockPos(op.get(1).getAsInt(), op.get(2).getAsInt(), op.get(3).getAsInt());
                                level.blockEvent(pos, level.getBlockState(pos).getBlock(), op.get(4).getAsInt(), op.get(5).getAsInt());
                            }
                            default -> {
                                ((ClientLevel.ClientLevelData)level.getLevelData()).setGameTime(time);
                                level.tickBlockEntities();
                                time++;
                                JsonArray events = new JsonArray();
                                EVENTS.forEach(events::add);
                                EVENTS.clear();
                                results.add(ChunkSuites.array(events, tickers(level), blockEntities(level)));
                            }
                        }
                    }
                    results.add(ChunkSuites.array("end", level.getRandom().nextLong(), enchantingRandom.nextLong()));
                    JsonArray opsJson = new JsonArray();
                    ops.forEach(opsJson::add);
                    writer.result(
                        "programs",
                        ChunkSuites.array(levelSeed, enchantingSeed, status.name(), ChunkSuites.array(bits(position.x), bits(position.y), bits(position.z)), gameTime, opsJson),
                        results,
                        null
                    );
                }
                pistons(writer, unsafe, minecraft, camera, particles);
            } finally {
                try {
                    instance.set(null, previousInstance);
                } catch (ReflectiveOperationException e) {
                    throw new IllegalStateException(e);
                }
            }
        }


        /**
         * {@code pistons}: programs like {@code programs}' on courses of pistons, each with a line of blocks in front of it, the
         * player standing on one pushed up or in one's way; each tick the pistons extended and retracted by their block events,
         * worked out as the program runs from what's there ({@code ["event", x, y, z, 0, facing]} for a retracted piston,
         * {@code 1} or {@code 2} for an extended one). A line adds the blocks in front of each piston: {@code [events, tickers,
         * blockEntities, blocks]}, the blocks each piston's position and the four in front of it, by id.
         */
        private static void pistons(
            final GoldenWriter writer, final sun.misc.Unsafe unsafe, final Minecraft minecraft, final Camera camera, final OptionInstance<?> particles
        ) throws IOException {
            Random random = new Random(0x915701L);
            for (int program = 0; program < 64; program++) {
                long levelSeed = random.nextLong();
                long gameTime = random.nextInt(2000);
                Level level = level(unsafe, minecraft, levelSeed, gameTime);
                Player player;
                try {
                    player = (Player)unsafe.allocateInstance(Player.class);
                } catch (ReflectiveOperationException e) {
                    throw new IllegalStateException(e);
                }
                set(Entity.class, player, "id", 7);
                set(Entity.class, player, "type", net.minecraft.world.entity.EntityTypes.PLAYER);
                level.player = player;
                minecraft.player = player;
                set(OptionInstance.class, particles, "value", ParticleStatus.ALL);

                List<JsonArray> ops = new ArrayList<>();
                List<BlockPos> pistons = new ArrayList<>();
                List<Direction> facings = new ArrayList<>();
                List<BlockPos> lines = new ArrayList<>();
                List<Vec3> spots = new ArrayList<>();
                for (int slot = 0; slot < 9; slot++) {
                    if (random.nextInt(3) == 0) {
                        continue;
                    }
                    Direction facing = random.nextInt(3) == 0 ? Direction.UP : random.nextInt(6) == 0 ? Direction.DOWN : Direction.Plane.HORIZONTAL.getRandomDirection(RandomSource.create(random.nextLong()));
                    BlockPos piston = new BlockPos(-10 + 10 * (slot % 3), facing == Direction.DOWN ? -55 : -60, -10 + 10 * (slot / 3));
                    boolean sticky = random.nextBoolean();
                    place(ops, piston, (sticky ? Blocks.STICKY_PISTON : Blocks.PISTON).defaultBlockState().setValue(BlockStateProperties.FACING, facing));
                    pistons.add(piston);
                    facings.add(facing);
                    int length = random.nextInt(5);
                    BlockPos last = piston;
                    for (int i = 1; i <= length; i++) {
                        BlockPos pos = piston.relative(facing, i);
                        BlockState state = List.of(
                            Blocks.STONE, Blocks.DIRT, Blocks.SLIME_BLOCK, Blocks.HONEY_BLOCK, Blocks.GLASS, Blocks.OAK_PLANKS, Blocks.OBSIDIAN, Blocks.STONE
                        ).get(random.nextInt(8)).defaultBlockState();
                        place(ops, pos, state);
                        last = pos;
                        if ((state.is(Blocks.SLIME_BLOCK) || state.is(Blocks.HONEY_BLOCK)) && random.nextBoolean()) {
                            // A block stuck to its side.
                            Direction side = facing.getAxis() == Direction.Axis.Y ? Direction.NORTH : Direction.UP;
                            place(ops, pos.relative(side), (random.nextBoolean() ? Blocks.STONE : Blocks.SLIME_BLOCK).defaultBlockState());
                        }
                    }
                    lines.add(last);
                    // Where the player can be: on what an upward piston pushes, in a sideways one's way, on what it pushes.
                    if (facing == Direction.UP) {
                        spots.add(new Vec3(piston.getX() + 0.5, last.getY() + 1.0, piston.getZ() + 0.5));
                    } else if (facing != Direction.DOWN) {
                        BlockPos ahead = last.relative(facing);
                        spots.add(new Vec3(ahead.getX() + 0.5 - facing.getStepX() * 0.2, -60.0, ahead.getZ() + 0.5 - facing.getStepZ() * 0.2));
                        if (last != piston) {
                            spots.add(new Vec3(last.getX() + 0.5, last.getY() + 1.0, last.getZ() + 0.5));
                        }
                    }
                }
                Vec3 position = spots.isEmpty() || random.nextInt(8) == 0
                    ? new Vec3(40.5, -60.0, 40.5)
                    : spots.get(random.nextInt(spots.size())).add((random.nextDouble() - 0.5) * 0.2, 0.0, (random.nextDouble() - 0.5) * 0.2);
                set(Entity.class, player, "position", position);
                set(Entity.class, player, "bb", new AABB(position.x - 0.3, position.y, position.z - 0.3, position.x + 0.3, position.y + 1.8, position.z + 0.3));
                set(Camera.class, camera, "position", position.add(0.0, 1.62, 0.0));
                // Players stand on what they're pushed onto, which the moves the harness records don't change.
                set(Entity.class, player, "onGround", true);
                set(Entity.class, player, "mainSupportingBlockPos", java.util.Optional.empty());
                set(Entity.class, player, "deltaMovement", new Vec3(0.0, -0.0784000015258789, 0.0));
                // Its level, which says it moves itself (canSimulateMovement).
                set(Entity.class, player, "level", level);

                JsonArray results = new JsonArray();
                EVENTS.clear();
                for (JsonArray op : ops) {
                    level.setBlock(new BlockPos(op.get(1).getAsInt(), op.get(2).getAsInt(), op.get(3).getAsInt()), Block.stateById(op.get(4).getAsInt()), 18);
                }
                long time = gameTime;
                for (int tick = 0; tick < 30; tick++) {
                    for (int i = 0; i < pistons.size(); i++) {
                        if (random.nextInt(5) != 0) {
                            continue;
                        }
                        BlockPos piston = pistons.get(i);
                        BlockState state = level.getBlockState(piston);
                        if (!(state.getBlock() instanceof net.minecraft.world.level.block.piston.PistonBaseBlock)) {
                            continue;
                        }
                        int b0 = state.getValue(net.minecraft.world.level.block.piston.PistonBaseBlock.EXTENDED) ? (random.nextInt(4) == 0 ? 2 : 1) : 0;
                        int b1 = facings.get(i).get3DDataValue();
                        ops.add(ChunkSuites.array("event", piston.getX(), piston.getY(), piston.getZ(), b0, b1));
                        level.blockEvent(piston, state.getBlock(), b0, b1);
                    }
                    ops.add(ChunkSuites.array("tick"));
                    ((ClientLevel.ClientLevelData)level.getLevelData()).setGameTime(time);
                    level.tickBlockEntities();
                    time++;
                    JsonArray events = new JsonArray();
                    EVENTS.forEach(events::add);
                    EVENTS.clear();
                    JsonArray blocks = new JsonArray();
                    for (int i = 0; i < pistons.size(); i++) {
                        for (int j = 0; j <= 4; j++) {
                            blocks.add(Block.getId(level.getBlockState(pistons.get(i).relative(facings.get(i), j))));
                        }
                    }
                    results.add(ChunkSuites.array(events, tickers(level), blockEntities(level), blocks));
                }
                results.add(ChunkSuites.array("end", level.getRandom().nextLong()));
                JsonArray opsJson = new JsonArray();
                ops.forEach(opsJson::add);
                JsonArray watched = new JsonArray();
                for (int i = 0; i < pistons.size(); i++) {
                    watched.add(ChunkSuites.array(pistons.get(i).getX(), pistons.get(i).getY(), pistons.get(i).getZ(), facings.get(i).name()));
                }
                writer.result(
                    "pistons",
                    ChunkSuites.array(levelSeed, ChunkSuites.array(bits(position.x), bits(position.y), bits(position.z)), gameTime, watched, opsJson),
                    results,
                    null
                );
            }
        }

        /** A new level over the walk's 3 by 3 chunks around the spawn, its random seeded, at the game time. */
        static Level level(final sun.misc.Unsafe unsafe, final Minecraft minecraft, final long seed, final long gameTime) {
            Level level;
            try {
                level = (Level)unsafe.allocateInstance(Level.class);
            } catch (ReflectiveOperationException e) {
                throw new IllegalStateException(e);
            }
            Class<net.minecraft.world.level.Level> base = net.minecraft.world.level.Level.class;
            ClientLevel.ClientLevelData levelData = new ClientLevel.ClientLevelData(Difficulty.NORMAL, false, false);
            levelData.setGameTime(gameTime);
            set(ClientLevel.class, level, "clientLevelData", levelData);
            set(base, level, "levelData", levelData);
            set(base, level, "isClientSide", true);
            set(base, level, "dimension", net.minecraft.world.level.Level.OVERWORLD);
            Holder<?> dimensionType = RegistryValues.registries().lookupOrThrow(Registries.DIMENSION_TYPE).getOrThrow(BuiltinDimensionTypes.OVERWORLD);
            set(base, level, "dimensionTypeRegistration", dimensionType);
            set(base, level, "registryAccess", RegistryValues.registries());
            set(base, level, "palettedContainerFactory", PalettedContainerFactory.create(RegistryValues.registries()));
            set(base, level, "random", RandomSource.create(seed));
            // What the sounds a client doesn't play are seeded from (a beacon's, removed).
            set(base, level, "soundSeedGenerator", RandomSource.createThreadSafe());
            set(base, level, "blockEntityTickers", new ArrayList<>());
            set(base, level, "neighborUpdater", new net.minecraft.world.level.redstone.CollectingNeighborUpdater(level, 1000000));
            set(base, level, "pendingBlockEntityTickers", new ArrayList<>());
            set(ClientLevel.class, level, "minecraft", minecraft);
            // The listener, for the vanilla features a moving piston's block is checked against.
            try {
                net.minecraft.client.multiplayer.ClientPacketListener connection =
                    (net.minecraft.client.multiplayer.ClientPacketListener)unsafe.allocateInstance(net.minecraft.client.multiplayer.ClientPacketListener.class);
                set(net.minecraft.client.multiplayer.ClientPacketListener.class, connection, "enabledFeatures", net.minecraft.world.flag.FeatureFlags.DEFAULT_FLAGS);
                set(ClientLevel.class, level, "connection", connection);
            } catch (ReflectiveOperationException e) {
                throw new IllegalStateException(e);
            }
            set(ClientLevel.class, level, "tickRateManager", new TickRateManager());
            set(ClientLevel.class, level, "worldBorder", new WorldBorder());
            set(ClientLevel.class, level, "dragonParts", new ArrayList<>());
            set(ClientLevel.class, level, "blockStatePredictionHandler", new net.minecraft.client.multiplayer.prediction.BlockStatePredictionHandler());
            set(ClientLevel.class, level, "chunkSource", new ClientChunkCache(level, 8));
            for (ClientboundLevelChunkWithLightPacket packet : LightSuites.chunks().values()) {
                if (Math.abs(packet.x()) <= 1 && Math.abs(packet.z()) <= 1) {
                    level.getChunkSource().replaceWithPacketData(packet.x(), packet.z(), packet.chunkData());
                }
            }
            return level;
        }

        /** The level's tickers in order: {@code [type, x, y, z, removed]}. */
        private static JsonArray tickers(final Level level) {
            @SuppressWarnings("unchecked")
            List<TickingBlockEntity> tickers = (List<TickingBlockEntity>)get(net.minecraft.world.level.Level.class, level, "blockEntityTickers");
            JsonArray result = new JsonArray();
            for (TickingBlockEntity ticker : tickers) {
                BlockPos pos = ticker.getPos();
                result.add(ChunkSuites.array(ticker.getType(), pos.getX(), pos.getY(), pos.getZ(), ticker.isRemoved()));
            }
            return result;
        }

        /** Each chunk's block entities in its map's order, the chunks from (-1, -1) by x then z. */
        private static JsonArray blockEntities(final Level level) {
            JsonArray result = new JsonArray();
            for (int x = -1; x <= 1; x++) {
                for (int z = -1; z <= 1; z++) {
                    LevelChunk chunk = level.getChunkSource().getChunk(x, z, false);
                    for (Map.Entry<BlockPos, BlockEntity> entry : chunk.getBlockEntities().entrySet()) {
                        result.add(state(level, entry.getValue()));
                    }
                }
            }
            return result;
        }

        /** The blocks a course is made of: each block entity's class the client ticks or handles events for, in their states. */
        private static BlockState pick(final Random random) {
            Direction[] horizontal = {Direction.NORTH, Direction.EAST, Direction.SOUTH, Direction.WEST};
            Direction facing = horizontal[random.nextInt(4)];
            return switch (random.nextInt(17)) {
                case 0 -> List.of(Blocks.CHEST, Blocks.TRAPPED_CHEST, Blocks.COPPER_CHEST.weathering().unaffected(), Blocks.COPPER_CHEST.waxed().oxidized()).get(random.nextInt(4))
                    .defaultBlockState()
                    .setValue(ChestBlock.FACING, facing)
                    .setValue(ChestBlock.TYPE, ChestType.values()[random.nextInt(3)]);
                case 1 -> Blocks.ENDER_CHEST.defaultBlockState().setValue(BlockStateProperties.HORIZONTAL_FACING, facing);
                case 2, 3 -> (random.nextBoolean() ? Blocks.SHULKER_BOX : Blocks.DYED_SHULKER_BOX.pick(DyeColor.values()[random.nextInt(16)]))
                    .defaultBlockState()
                    .setValue(ShulkerBoxBlock.FACING, Direction.values()[random.nextInt(6)]);
                case 4 -> Blocks.BELL.defaultBlockState().setValue(BellBlock.FACING, facing);
                case 5 -> Blocks.END_GATEWAY.defaultBlockState();
                case 6 -> Blocks.DECORATED_POT.defaultBlockState();
                case 7 -> Blocks.SPAWNER.defaultBlockState();
                case 8 -> (random.nextBoolean() ? Blocks.CAMPFIRE : Blocks.SOUL_CAMPFIRE)
                    .defaultBlockState()
                    .setValue(CampfireBlock.LIT, random.nextInt(4) != 0)
                    .setValue(CampfireBlock.SIGNAL_FIRE, random.nextBoolean());
                case 9 -> List.of(Blocks.DRAGON_HEAD, Blocks.PIGLIN_HEAD, Blocks.DRAGON_WALL_HEAD, Blocks.PIGLIN_WALL_HEAD, Blocks.ZOMBIE_HEAD, Blocks.PLAYER_HEAD)
                    .get(random.nextInt(6))
                    .defaultBlockState()
                    .setValue(AbstractSkullBlock.POWERED, random.nextBoolean());
                case 10 -> Blocks.ENCHANTING_TABLE.defaultBlockState();
                case 11 -> List.of(Blocks.OAK_SIGN, Blocks.SPRUCE_WALL_SIGN, Blocks.CHERRY_HANGING_SIGN, Blocks.BAMBOO_WALL_HANGING_SIGN).get(random.nextInt(4)).defaultBlockState();
                case 12 -> Blocks.NOTE_BLOCK.defaultBlockState()
                    .setValue(NoteBlock.INSTRUMENT, NoteBlockInstrument.values()[random.nextInt(NoteBlockInstrument.values().length)])
                    .setValue(NoteBlock.NOTE, random.nextInt(25));
                case 13 -> Blocks.COMPARATOR.defaultBlockState();
                case 14 -> Blocks.POTENT_SULFUR.defaultBlockState().setValue(BlockStateProperties.POTENT_SULFUR_STATE, PotentSulfurState.DRY);
                case 15 -> Blocks.BARREL.defaultBlockState();
                default -> Blocks.FURNACE.defaultBlockState();
            };
        }

        /**
         * The course: a beacon on a pyramid of 0 to 4 levels with stained glass of random colours over it, a conduit in water
         * with a frame of up to 42 prismarine blocks, and up to 30 block entity blocks at random places on the ground. Returns
         * the block entities' places.
         */
        private static List<BlockPos> course(final Random random, final List<JsonArray> ops) {
            List<BlockPos> places = new ArrayList<>();
            // The beacon, at a corner of the course.
            int levels = random.nextInt(5);
            BlockPos beacon = new BlockPos(MIN_X + 4, -61 + levels, MIN_Z + 4);
            for (int step = 1; step <= levels; step++) {
                for (int x = -step; x <= step; x++) {
                    for (int z = -step; z <= step; z++) {
                        place(ops, beacon.offset(x, -step, z), (random.nextInt(30) == 0 ? Blocks.STONE : Blocks.IRON_BLOCK).defaultBlockState());
                    }
                }
            }
            place(ops, beacon, Blocks.BEACON.defaultBlockState());
            places.add(beacon);
            int y = beacon.getY() + 1 + random.nextInt(3);
            for (int i = random.nextInt(6); i > 0; i--, y += 1 + random.nextInt(3)) {
                BlockState glass = random.nextInt(4) == 0
                    ? Blocks.STAINED_GLASS_PANE.pick(DyeColor.values()[random.nextInt(16)]).defaultBlockState()
                    : Blocks.STAINED_GLASS.pick(DyeColor.values()[random.nextInt(16)]).defaultBlockState();
                place(ops, new BlockPos(beacon.getX(), y, beacon.getZ()), random.nextInt(8) == 0 ? Blocks.STONE.defaultBlockState() : glass);
            }

            // The conduit, in water held up in the air at the other corner, its frame of prismarine.
            BlockPos conduit = new BlockPos(MIN_X + SIZE - 4, -52, MIN_Z + SIZE - 4);
            for (int x = -2; x <= 2; x++) {
                for (int dy = -2; dy <= 2; dy++) {
                    for (int z = -2; z <= 2; z++) {
                        int ax = Math.abs(x);
                        int ay = Math.abs(dy);
                        int az = Math.abs(z);
                        BlockPos pos = conduit.offset(x, dy, z);
                        if (ax <= 1 && ay <= 1 && az <= 1) {
                            if (!pos.equals(conduit) && random.nextInt(40) != 0) {
                                place(ops, pos, Blocks.WATER.defaultBlockState());
                            }
                        } else if ((x == 0 && (ay == 2 || az == 2) || dy == 0 && (ax == 2 || az == 2) || z == 0 && (ax == 2 || ay == 2)) && random.nextInt(4) != 0) {
                            place(ops, pos, List.of(Blocks.PRISMARINE, Blocks.PRISMARINE_BRICKS, Blocks.DARK_PRISMARINE, Blocks.SEA_LANTERN).get(random.nextInt(4)).defaultBlockState());
                        }
                    }
                }
            }
            place(ops, conduit, Blocks.CONDUIT.defaultBlockState().setValue(ConduitBlock.WATERLOGGED, true));
            places.add(conduit);

            // Two blocks apart on the ground, so no two are neighbours: a shulker box's lid updates its neighbours' shapes
            // and tells them it changed, which the port doesn't (PORTING.md), and nothing here changes for it.
            for (int i = 0; i < 30; i++) {
                BlockPos pos = new BlockPos(MIN_X + 2 * random.nextInt(SIZE / 2 + 1), -60, MIN_Z + 2 * random.nextInt(SIZE / 2 + 1));
                if (Math.abs(pos.getX() - beacon.getX()) <= 5 && Math.abs(pos.getZ() - beacon.getZ()) <= 5 || places.contains(pos)) {
                    continue;
                }
                place(ops, pos, pick(random));
                places.add(pos);
            }
            return places;
        }

        /** A block change among the course's: another state of the same class, a new block, stone or air. */
        private static void change(final Random random, final List<BlockPos> course, final List<JsonArray> ops, final Level level) {
            BlockPos pos = course.get(random.nextInt(course.size()));
            BlockState state = switch (random.nextInt(6)) {
                case 0 -> Blocks.STONE.defaultBlockState();
                case 1 -> Blocks.AIR.defaultBlockState();
                default -> pick(random);
            };
            place(ops, pos, state);
        }

        private static void place(final List<JsonArray> ops, final BlockPos pos, final BlockState state) {
            ops.add(ChunkSuites.array("set", pos.getX(), pos.getY(), pos.getZ(), Block.getId(state)));
        }
    }
}
