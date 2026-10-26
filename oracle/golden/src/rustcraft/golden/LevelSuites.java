package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonNull;
import com.google.gson.JsonPrimitive;
import it.unimi.dsi.fastutil.objects.Object2ObjectArrayMap;
import java.util.ArrayList;
import java.util.Map;
import net.minecraft.client.Camera;
import net.minecraft.client.Minecraft;
import net.minecraft.client.OptionInstance;
import net.minecraft.client.Options;
import net.minecraft.client.color.block.BlockColors;
import net.minecraft.client.color.block.BlockTintCache;
import net.minecraft.client.particle.Particle;
import net.minecraft.client.particle.ParticleEngine;
import net.minecraft.client.player.LocalPlayer;
import net.minecraft.client.renderer.BiomeColors;
import net.minecraft.client.renderer.GameRenderer;
import net.minecraft.client.resources.sounds.AbstractSoundInstance;
import net.minecraft.client.resources.sounds.EntityBoundSoundInstance;
import net.minecraft.client.resources.sounds.SoundInstance;
import net.minecraft.client.sounds.SoundEngine;
import net.minecraft.client.sounds.SoundManager;
import net.minecraft.core.QuartPos;
import net.minecraft.util.Mth;
import net.minecraft.core.SectionPos;
import net.minecraft.core.particles.BlockParticleOption;
import net.minecraft.core.particles.ColorParticleOption;
import net.minecraft.core.particles.DustColorTransitionOptions;
import net.minecraft.core.particles.DustParticleOptions;
import net.minecraft.core.particles.ParticleOptions;
import net.minecraft.core.particles.SimpleParticleType;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.network.protocol.game.ClientboundLevelChunkWithLightPacket;
import net.minecraft.server.level.ParticleStatus;
import net.minecraft.util.RandomSource;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.attribute.EnvironmentAttributeSystem;
import net.minecraft.world.level.ColorResolver;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.biome.BiomeManager;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.EntityBlock;
import net.minecraft.world.level.block.LiquidBlock;
import net.minecraft.world.level.block.entity.BlockEntity;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.ProtoChunk;
import net.minecraft.world.level.dimension.BuiltinDimensionTypes;
import net.minecraft.world.level.levelgen.Heightmap;
import net.minecraft.world.level.lighting.LevelLightEngine;
import net.minecraft.world.level.material.FluidState;
import net.minecraft.world.level.material.Fluids;
import java.io.IOException;
import java.lang.reflect.Field;
import java.lang.reflect.Method;
import java.util.HashMap;
import java.util.List;
import java.util.Random;
import net.minecraft.client.ClientClockManager;
import net.minecraft.client.multiplayer.ClientLevel;
import net.minecraft.client.multiplayer.ClientPacketListener;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Holder;
import net.minecraft.core.registries.Registries;
import net.minecraft.resources.Identifier;
import net.minecraft.resources.ResourceKey;
import net.minecraft.world.Difficulty;
import net.minecraft.world.clock.ClockNetworkState;
import net.minecraft.world.clock.WorldClock;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.border.WorldBorder;
import net.minecraft.world.level.dimension.DimensionType;
import net.minecraft.world.phys.AABB;
import net.minecraft.world.phys.Vec3;

/**
 * The level's own ticking, as the client runs it between the server's packets: the world border's moves
 * ({@code WorldBorder}), and the game time, the clocks and the weather of a {@code ClientLevel}. See TESTING.md ("The
 * level's ticking").
 */
final class LevelSuites {
    private LevelSuites() {
    }

    static List<Suite> suites() {
        // ClientLevelSuite runs last, with BlockEntityTickSuites, after the fluids' tags are bound.
        return List.of(new WorldBorderSuite());
    }

    private static JsonElement bits(final double value) {
        return Values.encode(value, double.class);
    }

    private static JsonElement bits(final float value) {
        return Values.encode(value, float.class);
    }

    /**
     * {@code programs}: {@code [settings, ops]} → each op's output, on a border made with the settings (null for
     * {@code new WorldBorder()}). The ops set it ({@code setCenter}, {@code setSize}, {@code lerpSizeBetween},
     * {@code setAbsoluteMaxSize}, the warnings, the damage and safe zone, {@code applyInitialSettings}), step it
     * ({@code tick}), and read it: {@code state} with a partial tick (its bounds then and now, center, size, the move's
     * ticks, target and speed, its status, its other fields and its {@code Settings}), {@code point} (every
     * {@code isWithinBounds}, the distance to it and the clamps at a position) and {@code shape} (the collision shape's
     * boxes). Centers and sizes go up to the largest a border takes, and the maximum size is sometimes small, which clamps.
     */
    static final class WorldBorderSuite implements Suite {
        private static final int PROGRAMS = 300;

        @Override
        public String name() {
            return WorldBorder.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            Random random = new Random(0xb0de5);
            for (int program = 0; program < PROGRAMS; program++) {
                WorldBorder.Settings settings = random.nextInt(4) == 0 ? new WorldBorder.Settings(
                    center(random), center(random), random.nextDouble(), random.nextDouble() * 10.0, random.nextInt(20), random.nextInt(400), size(random),
                    random.nextInt(3) == 0 ? 0L : random.nextInt(100), size(random)
                ) : null;
                WorldBorder border = settings == null ? new WorldBorder() : new WorldBorder(settings);
                JsonArray ops = new JsonArray();
                JsonArray outputs = new JsonArray();
                long gameTime = random.nextInt(100000);
                for (int step = 10 + random.nextInt(50); step > 0; step--) {
                    JsonArray op;
                    JsonElement output = JsonNull.INSTANCE;
                    switch (random.nextInt(14)) {
                        case 0 -> {
                            double x = center(random);
                            double z = center(random);
                            op = ChunkSuites.array("setCenter", bits(x), bits(z));
                            border.setCenter(x, z);
                        }
                        case 1 -> {
                            double size = size(random);
                            op = ChunkSuites.array("setSize", bits(size));
                            border.setSize(size);
                        }
                        case 2 -> {
                            double from = size(random);
                            double to = random.nextInt(8) == 0 ? from : size(random);
                            long ticks = random.nextInt(4) == 0 ? 1 + random.nextInt(3) : 1 + random.nextInt(60);
                            op = ChunkSuites.array("lerpSizeBetween", bits(from), bits(to), ticks, gameTime);
                            border.lerpSizeBetween(from, to, ticks, gameTime);
                        }
                        case 3, 4, 5 -> {
                            op = ChunkSuites.array("tick");
                            border.tick();
                            gameTime++;
                        }
                        case 6 -> {
                            int max = random.nextInt(3) == 0 ? random.nextInt(2000) : 29999984;
                            op = ChunkSuites.array("setAbsoluteMaxSize", max);
                            border.setAbsoluteMaxSize(max);
                        }
                        case 7 -> {
                            int value = random.nextInt(500) - 10;
                            switch (random.nextInt(4)) {
                                case 0 -> {
                                    op = ChunkSuites.array("setWarningTime", value);
                                    border.setWarningTime(value);
                                }
                                case 1 -> {
                                    op = ChunkSuites.array("setWarningBlocks", value);
                                    border.setWarningBlocks(value);
                                }
                                case 2 -> {
                                    double safeZone = random.nextDouble() * 10.0;
                                    op = ChunkSuites.array("setSafeZone", bits(safeZone));
                                    border.setSafeZone(safeZone);
                                }
                                default -> {
                                    double damage = random.nextDouble();
                                    op = ChunkSuites.array("setDamagePerBlock", bits(damage));
                                    border.setDamagePerBlock(damage);
                                }
                            }
                        }
                        case 8 -> {
                            op = ChunkSuites.array("applyInitialSettings", gameTime);
                            border.applyInitialSettings(gameTime);
                        }
                        case 9, 10 -> {
                            float partial = random.nextInt(4) == 0 ? random.nextInt(2) : random.nextFloat();
                            op = ChunkSuites.array("state", bits(partial));
                            output = state(border, partial);
                        }
                        case 11, 12 -> {
                            double x = near(random, border.getCenterX(), border.getSize());
                            double z = near(random, border.getCenterZ(), border.getSize());
                            double margin = random.nextInt(3) == 0 ? 0.0 : random.nextDouble() * 4.0;
                            op = ChunkSuites.array("point", bits(x), bits(z), bits(margin));
                            output = point(border, x, z, margin);
                        }
                        default -> {
                            op = ChunkSuites.array("shape");
                            JsonArray boxes = new JsonArray();
                            for (AABB box : border.getCollisionShape().toAabbs()) {
                                boxes.add(ChunkSuites.array(bits(box.minX), bits(box.minY), bits(box.minZ), bits(box.maxX), bits(box.maxY), bits(box.maxZ)));
                            }
                            output = boxes;
                        }
                    }
                    ops.add(op);
                    outputs.add(output);
                }
                writer.result("programs", ChunkSuites.array(settings == null ? JsonNull.INSTANCE : settings(settings), ops), outputs, null);
            }
        }

        private static double center(final Random random) {
            return switch (random.nextInt(5)) {
                case 0 -> 0.0;
                case 1 -> random.nextInt(2000) - 1000 + random.nextInt(2) * 0.5;
                case 2 -> (random.nextDouble() * 2.0 - 1.0) * 2.9999984E7;
                default -> (random.nextDouble() * 2.0 - 1.0) * 500.0;
            };
        }

        private static double size(final Random random) {
            return switch (random.nextInt(6)) {
                case 0 -> 5.999997E7F;
                case 1 -> 1 + random.nextInt(64);
                case 2 -> random.nextDouble() * 10.0;
                case 3 -> random.nextDouble() * 6.0E7;
                default -> random.nextInt(2000) + random.nextDouble();
            };
        }

        private static double near(final Random random, final double center, final double size) {
            double half = Math.min(size, 1.0E8) / 2.0;
            return switch (random.nextInt(4)) {
                case 0 -> center + half + (random.nextDouble() - 0.5) * 4.0;
                case 1 -> center - half + (random.nextDouble() - 0.5) * 4.0;
                case 2 -> center + (random.nextDouble() * 2.0 - 1.0) * half;
                default -> (random.nextDouble() * 2.0 - 1.0) * 3.0E7;
            };
        }

        private static JsonArray settings(final WorldBorder.Settings settings) {
            return ChunkSuites.array(
                bits(settings.centerX()), bits(settings.centerZ()), bits(settings.damagePerBlock()), bits(settings.safeZone()), settings.warningBlocks(),
                settings.warningTime(), bits(settings.size()), settings.lerpTime(), bits(settings.lerpTarget())
            );
        }

        private static JsonArray state(final WorldBorder border, final float partial) {
            return ChunkSuites.array(
                bits(border.getMinX(partial)), bits(border.getMinZ(partial)), bits(border.getMaxX(partial)), bits(border.getMaxZ(partial)),
                bits(border.getMinX()), bits(border.getMinZ()), bits(border.getMaxX()), bits(border.getMaxZ()), bits(border.getCenterX()),
                bits(border.getCenterZ()), bits(border.getSize()), border.getLerpTime(), bits(border.getLerpTarget()), bits(border.getLerpSpeed()),
                border.getStatus().name(), border.getStatus().getColor(), border.getAbsoluteMaxSize(), border.getWarningTime(), border.getWarningBlocks(),
                bits(border.getSafeZone()), bits(border.getDamagePerBlock()), settings(new WorldBorder.Settings(border))
            );
        }

        private static JsonArray point(final WorldBorder border, final double x, final double z, final double margin) {
            BlockPos clamped = border.clampToBounds(x, 64.5, z);
            Vec3 clampedVec = border.clampVec3ToBound(new Vec3(x, 64.5, z));
            BlockPos block = BlockPos.containing(x, 64.0, z);
            ChunkPos chunk = ChunkPos.containing(block);
            AABB box = new AABB(x - 0.3, 64.0, z - 0.3, x + 0.3, 65.8, z + 0.3);
            BlockPos clampedBlock = border.clampToBounds(block);
            return ChunkSuites.array(
                border.isWithinBounds(x, z), border.isWithinBounds(x, z, margin), border.isWithinBounds(block), border.isWithinBounds(new Vec3(x, 64.0, z)),
                border.isWithinBounds(chunk), border.isWithinBounds(box), bits(border.getDistanceToBorder(x, z)),
                ChunkSuites.array(clamped.getX(), clamped.getY(), clamped.getZ()),
                ChunkSuites.array(bits(clampedVec.x), bits(clampedVec.y), bits(clampedVec.z)),
                ChunkSuites.array(clampedBlock.getX(), clampedBlock.getY(), clampedBlock.getZ())
            );
        }
    }

    /**
     * {@code net.minecraft.client.multiplayer.ClientLevel}: the tints ({@code BiomeSuites.TintSuite}), then {@code ticks},
     * {@code [dimension, ops]} → each op's output, on a ClientLevel made without its constructor that has what the ops read:
     * its level data, its dimension and dimension type, and a packet listener (also made without its constructor) with
     * its clocks. The ops are the level's {@code tickTime}, {@code handleSetTime}'s part ({@code setTimeFromServer} and
     * the clocks' {@code handleUpdates}), the weather's setters ({@code setRainLevel}, {@code setThunderLevel}, which
     * clamp) and reads: {@code weather} with a partial tick ({@code getRainLevel}, {@code getThunderLevel},
     * {@code isRaining}, {@code isThundering}) and {@code time} (the game time and each clock's ticks, partial tick and
     * rate).
     */
    static final class ClientLevelSuite implements Suite {
        private static final int PROGRAMS = 120;
        private static final String[] DIMENSIONS = {"overworld", "the_nether", "the_end", "overworld_caves"};

        @Override
        public String name() {
            return ClientLevel.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            BlockEntityTickSuites.bindFluidTags();
            new BiomeSuites.TintSuite().write(writer);
            try {
                this.writeTicks(writer);
                AnimateTick.write(writer);
            } catch (ReflectiveOperationException e) {
                throw new IllegalStateException(e);
            }
            BlockEntityTickSuites.Weather.write(writer);
        }

        @SuppressWarnings("unchecked")
        private void writeTicks(final GoldenWriter writer) throws IOException, ReflectiveOperationException {
            Random random = new Random(0x71c5);
            List<Object> clocks = RegistryValues.holders(Registries.WORLD_CLOCK).edges();
            float[] rates = {0.0F, 1.0F, 0.5F, 2.0F, 72.0F};
            Method tickTime = ClientLevel.class.getDeclaredMethod("tickTime");
            tickTime.setAccessible(true);
            for (int program = 0; program < PROGRAMS; program++) {
                String dimension = DIMENSIONS[program % DIMENSIONS.length];
                ClientClockManager clockManager = new ClientClockManager();
                ClientLevel level = level(dimension, clockManager);
                JsonArray ops = new JsonArray();
                JsonArray outputs = new JsonArray();
                for (int step = 10 + random.nextInt(40); step > 0; step--) {
                    JsonArray op;
                    JsonElement output = JsonNull.INSTANCE;
                    switch (random.nextInt(8)) {
                        case 0, 1 -> {
                            op = ChunkSuites.array("tickTime");
                            tickTime.invoke(level);
                        }
                        case 2 -> {
                            long gameTime = random.nextInt(4) == 0 ? random.nextLong() : random.nextInt(48000);
                            HashMap<Holder<WorldClock>, ClockNetworkState> updates = new HashMap<>();
                            JsonArray entries = new JsonArray();
                            if (random.nextInt(4) != 0) {
                                Holder<?> clock = (Holder<?>)clocks.get(random.nextInt(clocks.size()));
                                ClockNetworkState state = new ClockNetworkState(random.nextInt(48000), random.nextFloat(), rates[random.nextInt(rates.length)]);
                                updates.put((Holder<WorldClock>)clock, state);
                                entries.add(ChunkSuites.array(RegistryValues.encode(clock), Values.encode(state, ClockNetworkState.class)));
                            }
                            op = ChunkSuites.array("handleSetTime", gameTime, entries);
                            level.setTimeFromServer(gameTime);
                            clockManager.handleUpdates(gameTime, updates);
                        }
                        case 3 -> {
                            float rain = weather(random);
                            op = ChunkSuites.array("setRainLevel", bits(rain));
                            level.setRainLevel(rain);
                        }
                        case 4 -> {
                            float thunder = weather(random);
                            op = ChunkSuites.array("setThunderLevel", bits(thunder));
                            level.setThunderLevel(thunder);
                        }
                        case 5, 6 -> {
                            float partial = random.nextInt(3) == 0 ? random.nextInt(2) : random.nextFloat();
                            op = ChunkSuites.array("weather", bits(partial));
                            output = ChunkSuites.array(
                                bits(level.getRainLevel(partial)), bits(level.getThunderLevel(partial)), level.isRaining(), level.isThundering()
                            );
                        }
                        default -> {
                            op = ChunkSuites.array("time");
                            JsonArray state = new JsonArray();
                            state.add(level.getGameTime());
                            for (Object clock : clocks) {
                                ClientClockManager.ClientClockInstance instance = clockManager.getInstance((Holder<WorldClock>)clock);
                                state.add(ChunkSuites.array(instance.totalTicks(), bits(instance.partialTick()), bits(instance.rate())));
                            }
                            output = state;
                        }
                    }
                    ops.add(op);
                    outputs.add(output);
                }
                writer.result("ticks", ChunkSuites.array(dimension, ops), outputs, null);
            }
        }

        // The ends, out of range, random, and isRaining's and isThundering's thresholds (a float compared with a double)
        // with their neighbours.
        private static final float[] WEATHER_EDGES = {
            0.0F, 1.0F, 0.2F, Math.nextUp(0.2F), Math.nextDown(0.2F), 0.9F, Math.nextUp(0.9F), Math.nextDown(0.9F), -0.0F, 1.5F, -1.0F
        };

        private static float weather(final Random random) {
            return switch (random.nextInt(4)) {
                case 0, 1 -> WEATHER_EDGES[random.nextInt(WEATHER_EDGES.length)];
                case 2 -> random.nextFloat() * 3.0F - 1.0F;
                default -> random.nextFloat();
            };
        }

        /** A ClientLevel with what the ops read, in the dimension (its dimension type of the same name). */
        private static ClientLevel level(final String dimension, final ClientClockManager clockManager) throws ReflectiveOperationException {
            Field theUnsafe = sun.misc.Unsafe.class.getDeclaredField("theUnsafe");
            theUnsafe.setAccessible(true);
            sun.misc.Unsafe unsafe = (sun.misc.Unsafe)theUnsafe.get(null);
            ClientLevel level = (ClientLevel)unsafe.allocateInstance(ClientLevel.class);
            ClientPacketListener connection = (ClientPacketListener)unsafe.allocateInstance(ClientPacketListener.class);
            set(ClientPacketListener.class, connection, "clockManager", clockManager);
            ClientLevel.ClientLevelData levelData = new ClientLevel.ClientLevelData(Difficulty.NORMAL, false, false);
            set(ClientLevel.class, level, "clientLevelData", levelData);
            set(ClientLevel.class, level, "connection", connection);
            set(net.minecraft.world.level.Level.class, level, "levelData", levelData);
            set(net.minecraft.world.level.Level.class, level, "dimension", ResourceKey.create(Registries.DIMENSION, Identifier.withDefaultNamespace(dimension)));
            Holder<DimensionType> dimensionType = RegistryValues.registries()
                .lookupOrThrow(Registries.DIMENSION_TYPE)
                .getOrThrow(ResourceKey.create(Registries.DIMENSION_TYPE, Identifier.withDefaultNamespace(dimension)));
            set(net.minecraft.world.level.Level.class, level, "dimensionTypeRegistration", dimensionType);
            return level;
        }

        static void set(final Class<?> owner, final Object object, final String name, final Object value) throws ReflectiveOperationException {
            Field field = owner.getDeclaredField(name);
            field.setAccessible(true);
            field.set(object, value);
        }
    }

    /**
     * {@code animateTick}: the course ({@code COURSE}, {@code [x, y, z, state]} set in order on the walk's chunks, their
     * light run to the end, every block an animated class's in any state among the blocks around them) and {@code programs},
     * {@code [levelSeed, animateSeed, particles, camera, center, skyDarken, gameTime, clock, rain, thunder]} → the events
     * {@code ClientLevel.animateTick}'s 667 pairs of {@code doAnimateTick} make, with the level's random and the animations'
     * seeded, then each random's next long. The level is a ClientLevel made without its constructor, whose blocks, fluids,
     * light, heightmaps and block entities are the light suites' harness's chunks ({@code LightSuites.Harness}, which a
     * subclass reads them from), with its game's particle engine and sound manager recording what they're handed, its
     * options' particles, its camera and its player; its environment attributes are the overworld's
     * ({@code AttributeSuites.system}, the clocks at the program's time, its weather) over the chunks' biomes.
     */
    static final class AnimateTick {
        private static final int[] MIN = {-13, -60, -13};
        private static final int[] MAX = {13, -46, 13};
        // What an event's options add: nothing for a simple type, the state's id, the colour, or the dust's.
        private static final List<JsonArray> EVENTS = new ArrayList<>();

        /** The level: the harness's chunks, and the block entities their blocks make, made as they're asked for. */
        static class Level extends ClientLevel {
            LightSuites.Harness harness;
            Map<BlockPos, BlockEntity> blockEntities;

            Level() {
                super(null, null, null, null, 0, 0, null, false, 0L, 0);
            }

            private ProtoChunk chunkAt(final BlockPos pos) {
                if (this.isOutsideBuildHeight(pos)) {
                    return null;
                }
                return this.harness.loadedChunk(SectionPos.blockToSectionCoord(pos.getX()), SectionPos.blockToSectionCoord(pos.getZ()));
            }

            @Override
            public BlockState getBlockState(final BlockPos pos) {
                ProtoChunk chunk = this.chunkAt(pos);
                return chunk == null ? Blocks.VOID_AIR.defaultBlockState() : chunk.getBlockState(pos);
            }

            @Override
            public FluidState getFluidState(final BlockPos pos) {
                ProtoChunk chunk = this.chunkAt(pos);
                return chunk == null ? Fluids.EMPTY.defaultFluidState() : chunk.getFluidState(pos);
            }

            @Override
            public BlockEntity getBlockEntity(final BlockPos pos) {
                BlockPos key = pos.immutable();
                if (!this.blockEntities.containsKey(key)) {
                    BlockState state = this.getBlockState(key);
                    BlockEntity blockEntity = state.getBlock() instanceof EntityBlock entityBlock ? entityBlock.newBlockEntity(key, state) : null;
                    if (blockEntity != null) {
                        blockEntity.setLevel(this);
                    }
                    this.blockEntities.put(key, blockEntity);
                }
                return this.blockEntities.get(key);
            }

            @Override
            public LevelLightEngine getLightEngine() {
                return this.harness.lightEngine;
            }

            /** Level.getHeight, with the harness's chunks' heightmaps. */
            @Override
            public int getHeight(final Heightmap.Types type, final int x, final int z) {
                if (x >= -30000000 && z >= -30000000 && x < 30000000 && z < 30000000) {
                    ProtoChunk chunk = this.harness.loadedChunk(SectionPos.blockToSectionCoord(x), SectionPos.blockToSectionCoord(z));
                    return chunk == null ? this.getMinY() : chunk.getHeight(type, x & 15, z & 15) + 1;
                }
                return this.getSeaLevel() + 1;
            }

            @Override
            public int getMinY() {
                return -64;
            }

            @Override
            public int getHeight() {
                return 384;
            }

            @Override
            public int getSeaLevel() {
                return 63;
            }
        }

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
                    "particle", BuiltInRegistries.PARTICLE_TYPE.getKey(options.getType()).toString(), options(options), bits(x), bits(y), bits(z),
                    bits(xd), bits(yd), bits(zd)
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
                EVENTS.add(sound(instance, -1));
                return null;
            }

            @Override
            public void playDelayed(final SoundInstance instance, final int delay) {
                EVENTS.add(sound(instance, delay));
            }
        }

        static void write(final GoldenWriter writer) throws IOException, ReflectiveOperationException {
            Field theUnsafe = sun.misc.Unsafe.class.getDeclaredField("theUnsafe");
            theUnsafe.setAccessible(true);
            sun.misc.Unsafe unsafe = (sun.misc.Unsafe)theUnsafe.get(null);
            ShapeSuites.BlockStateShapeSuite.bindBlockTags();
            BiomeSuites.loadColormaps();

            // The walk's chunks, their light run to the end, and their heightmaps (the packet's, the others new).
            LightSuites.Harness harness = new LightSuites.Harness();
            for (ClientboundLevelChunkWithLightPacket packet : LightSuites.chunks().values()) {
                harness.chunk(packet);
                ProtoChunk chunk = harness.loadedChunk(packet.x(), packet.z());
                packet.chunkData().getHeightmaps().forEach(chunk::setHeightmap);
                for (Heightmap.Types type : HEIGHTMAPS) {
                    chunk.getOrCreateHeightmapUnprimed(type);
                }
            }
            settle(harness);

            // The course: in layers, every other one of animated blocks among blocks and fluids around them.
            Random random = new Random(0xa717);
            List<List<BlockState>> animated = animatedStates();
            List<BlockState> around = List.of(
                Blocks.STONE.defaultBlockState(), Blocks.GLASS.defaultBlockState(), Blocks.WATER.defaultBlockState(), Blocks.LAVA.defaultBlockState(),
                Blocks.WATER.defaultBlockState().setValue(LiquidBlock.LEVEL, 3), Blocks.WATER.defaultBlockState().setValue(LiquidBlock.LEVEL, 8),
                Blocks.LAVA.defaultBlockState().setValue(LiquidBlock.LEVEL, 5), Blocks.BOOKSHELF.defaultBlockState(), Blocks.PALE_OAK_LOG.defaultBlockState(),
                Blocks.PALE_OAK_LEAVES.defaultBlockState(), Blocks.SAND.defaultBlockState(), Blocks.RED_SAND.defaultBlockState(),
                Blocks.TERRACOTTA.defaultBlockState(), Blocks.MUD.defaultBlockState(), Blocks.SOUL_SAND.defaultBlockState(),
                Blocks.PALE_MOSS_BLOCK.defaultBlockState(), Blocks.OAK_PLANKS.defaultBlockState(), Blocks.WOOL.pick(net.minecraft.world.item.DyeColor.WHITE).defaultBlockState(),
                Blocks.OAK_SLAB.defaultBlockState(), Blocks.HOPPER.defaultBlockState(), Blocks.OAK_FENCE.defaultBlockState(),
                Blocks.POINTED_DRIPSTONE.defaultBlockState(), Blocks.DRIPSTONE_BLOCK.defaultBlockState(), Blocks.AIR.defaultBlockState()
            );
            JsonArray course = new JsonArray();
            for (int y = MIN[1]; y <= MAX[1]; y++) {
                for (int x = MIN[0]; x <= MAX[0]; x++) {
                    for (int z = MIN[2]; z <= MAX[2]; z++) {
                        int roll = random.nextInt(100);
                        BlockState state;
                        if ((y & 1) == 0 ? roll < 55 : roll < 6) {
                            List<BlockState> states = animated.get(random.nextInt(animated.size()));
                            state = states.get(random.nextInt(states.size()));
                        } else if ((y & 1) == 0 ? roll < 60 : roll < 22) {
                            state = around.get(random.nextInt(around.size()));
                        } else {
                            continue;
                        }
                        BlockPos pos = new BlockPos(x, y, z);
                        place(harness, pos, state);
                        course.add(ChunkSuites.array(x, y, z, Block.getId(state)));
                    }
                }
            }
            settle(harness);
            writer.field("COURSE", course);

            // The level and its game.
            AttributeSuites.Data data = AttributeSuites.Data.get();
            Holder<Biome> plains = data.biome("plains");
            net.minecraft.world.level.biome.BiomeResolver source = (quartX, quartY, quartZ) -> {
                ProtoChunk chunk = harness.loadedChunk(QuartPos.toSection(quartX), QuartPos.toSection(quartZ));
                if (chunk == null) {
                    return plains;
                }
                // ChunkAccess.getNoiseBiome, which a ProtoChunk refuses before its biomes' generation step.
                int minQuartY = QuartPos.fromBlock(chunk.getMinY());
                int maxQuartY = minQuartY + QuartPos.fromBlock(chunk.getHeight()) - 1;
                int clampedQuartY = Mth.clamp(quartY, minQuartY, maxQuartY);
                Holder<?> biome = chunk.getSection(chunk.getSectionIndex(QuartPos.toBlock(clampedQuartY)))
                    .getNoiseBiome(quartX & 3, clampedQuartY & 3, quartZ & 3);
                return data.biome(((ResourceKey<?>)biome.value()).identifier().getPath());
            };
            BiomeManager biomeManager = new BiomeManager(source, BiomeManager.obfuscateSeed(0L));
            AttributeSuites.Clock clock = new AttributeSuites.Clock();
            AttributeSuites.Weather weather = new AttributeSuites.Weather();
            EnvironmentAttributeSystem system = AttributeSuites.system(data, "overworld", "in_overworld", true, biomeManager, clock, weather);

            Level level = (Level)unsafe.allocateInstance(Level.class);
            level.harness = harness;
            level.blockEntities = new HashMap<>();
            ClientLevel.ClientLevelData levelData = new ClientLevel.ClientLevelData(Difficulty.NORMAL, false, false);
            set(ClientLevel.class, level, "clientLevelData", levelData);
            set(net.minecraft.world.level.Level.class, level, "levelData", levelData);
            set(net.minecraft.world.level.Level.class, level, "dimension", net.minecraft.world.level.Level.OVERWORLD);
            set(
                net.minecraft.world.level.Level.class, level, "dimensionTypeRegistration",
                RegistryValues.registries().lookupOrThrow(Registries.DIMENSION_TYPE).getOrThrow(BuiltinDimensionTypes.OVERWORLD)
            );
            set(net.minecraft.world.level.Level.class, level, "biomeManager", biomeManager);
            set(ClientLevel.class, level, "environmentAttributes", system);
            Object2ObjectArrayMap<ColorResolver, BlockTintCache> tintCaches = new Object2ObjectArrayMap<>(3);
            for (ColorResolver resolver : List.of(
                BiomeColors.GRASS_COLOR_RESOLVER, BiomeColors.FOLIAGE_COLOR_RESOLVER, BiomeColors.DRY_FOLIAGE_COLOR_RESOLVER, BiomeColors.WATER_COLOR_RESOLVER
            )) {
                tintCaches.put(resolver, new BlockTintCache(pos -> level.calculateBlockTint(pos, resolver)));
            }
            set(ClientLevel.class, level, "tintCaches", tintCaches);

            Minecraft minecraft = (Minecraft)unsafe.allocateInstance(Minecraft.class);
            Options options = (Options)unsafe.allocateInstance(Options.class);
            OptionInstance<?> particles = (OptionInstance<?>)unsafe.allocateInstance(OptionInstance.class);
            OptionInstance<?> biomeBlendRadius = (OptionInstance<?>)unsafe.allocateInstance(OptionInstance.class);
            set(OptionInstance.class, biomeBlendRadius, "value", 2);
            set(Options.class, options, "particles", particles);
            set(Options.class, options, "biomeBlendRadius", biomeBlendRadius);
            GameRenderer gameRenderer = (GameRenderer)unsafe.allocateInstance(GameRenderer.class);
            Camera camera = (Camera)unsafe.allocateInstance(Camera.class);
            set(GameRenderer.class, gameRenderer, "mainCamera", camera);
            LocalPlayer player = (LocalPlayer)unsafe.allocateInstance(LocalPlayer.class);
            set(Entity.class, player, "id", 7);
            set(Entity.class, player, "position", new Vec3(0.5, -60.0, 0.5));
            set(Minecraft.class, minecraft, "options", options);
            set(Minecraft.class, minecraft, "gameRenderer", gameRenderer);
            set(Minecraft.class, minecraft, "particleEngine", unsafe.allocateInstance(Particles.class));
            set(Minecraft.class, minecraft, "soundManager", unsafe.allocateInstance(Sounds.class));
            set(Minecraft.class, minecraft, "blockColors", BlockColors.createDefault());
            minecraft.player = player;
            set(ClientLevel.class, level, "minecraft", minecraft);
            Field instance = Minecraft.class.getDeclaredField("instance");
            instance.setAccessible(true);
            Object previousInstance = instance.get(null);
            instance.set(null, minecraft);

            ParticleStatus[] statuses = ParticleStatus.values();
            long[] clocks = {6000L, 18000L, 13000L, 23000L};
            int[] skyDarkens = {0, 4, 11, 15};
            try {
                for (int program = 0; program < 64; program++) {
                    long levelSeed = random.nextLong();
                    long animateSeed = random.nextLong();
                    ParticleStatus status = statuses[program % statuses.length];
                    Vec3 cameraPosition = new Vec3(random.nextDouble() * 80.0 - 40.0, -60.0 + random.nextDouble() * 30.0, random.nextDouble() * 80.0 - 40.0);
                    int xt = random.nextInt(9) - 4;
                    int yt = -53 + random.nextInt(9) - 4;
                    int zt = random.nextInt(9) - 4;
                    int skyDarken = skyDarkens[random.nextInt(skyDarkens.length)];
                    long gameTime = random.nextInt(100000);
                    long clockTime = clocks[random.nextInt(clocks.length)];
                    boolean storm = program % 4 == 3;
                    float rain = storm ? 1.0F : 0.0F;
                    float thunder = storm ? 1.0F : 0.0F;

                    set(OptionInstance.class, particles, "value", status);
                    set(Camera.class, camera, "position", cameraPosition);
                    set(net.minecraft.world.level.Level.class, level, "random", RandomSource.create(levelSeed));
                    set(net.minecraft.world.level.Level.class, level, "skyDarken", skyDarken);
                    levelData.setGameTime(gameTime);
                    level.setRainLevel(rain);
                    level.setThunderLevel(thunder);
                    clock.ticks = clockTime;
                    weather.rain = rain;
                    weather.thunder = thunder;
                    system.invalidateTickCache();
                    RandomSource animateRandom = RandomSource.createThreadLocalInstance(animateSeed);
                    BlockPos.MutableBlockPos pos = new BlockPos.MutableBlockPos();
                    EVENTS.clear();
                    for (int i = 0; i < 667; i++) {
                        level.doAnimateTick(xt, yt, zt, 16, animateRandom, null, pos);
                        level.doAnimateTick(xt, yt, zt, 32, animateRandom, null, pos);
                    }
                    JsonArray events = new JsonArray();
                    EVENTS.forEach(events::add);
                    events.add(ChunkSuites.array("end", level.getRandom().nextLong(), animateRandom.nextLong()));
                    JsonArray args = ChunkSuites.array(
                        levelSeed, animateSeed, status.name(), ChunkSuites.array(bits(cameraPosition.x), bits(cameraPosition.y), bits(cameraPosition.z)),
                        ChunkSuites.array(xt, yt, zt), skyDarken, gameTime, clockTime, bits(rain), bits(thunder)
                    );
                    writer.result("programs", args, events, null);
                }
            } finally {
                instance.set(null, previousInstance);
            }
        }

        private static final Heightmap.Types[] HEIGHTMAPS = {
            Heightmap.Types.WORLD_SURFACE, Heightmap.Types.OCEAN_FLOOR, Heightmap.Types.MOTION_BLOCKING, Heightmap.Types.MOTION_BLOCKING_NO_LEAVES
        };

        /** The harness's set, with LevelChunk.setBlockState's heightmaps. */
        private static void place(final LightSuites.Harness harness, final BlockPos pos, final BlockState state) {
            ProtoChunk chunk = harness.loadedChunk(SectionPos.blockToSectionCoord(pos.getX()), SectionPos.blockToSectionCoord(pos.getZ()));
            BlockState old = chunk.getBlockState(pos);
            harness.set(pos, state);
            if (old != state) {
                for (Heightmap.Types type : HEIGHTMAPS) {
                    chunk.getOrCreateHeightmapUnprimed(type).update(pos.getX() & 15, pos.getY(), pos.getZ() & 15, state);
                }
            }
        }

        /** Frames until no light update is queued and the light engine has nothing left to do. */
        private static void settle(final LightSuites.Harness harness) {
            while (harness.queued() > 0 || harness.lightEngine.hasLightWork()) {
                harness.frame();
            }
        }

        /** Each block whose class animates (overrides Block.animateTick, or its parent does), with its states. */
        private static List<List<BlockState>> animatedStates() throws ReflectiveOperationException {
            List<List<BlockState>> blocks = new ArrayList<>();
            for (Block block : BuiltInRegistries.BLOCK) {
                Method method = block.getClass().getMethod("animateTick", BlockState.class, net.minecraft.world.level.Level.class, BlockPos.class, RandomSource.class);
                if (method.getDeclaringClass() != Block.class) {
                    blocks.add(block.getStateDefinition().getPossibleStates());
                }
            }
            return blocks;
        }

        private static JsonElement options(final ParticleOptions options) {
            try {
                return switch (options) {
                    case SimpleParticleType simple -> JsonNull.INSTANCE;
                    case BlockParticleOption block -> new JsonPrimitive(Block.getId(block.getState()));
                    case ColorParticleOption color -> new JsonPrimitive((Integer)get(ColorParticleOption.class, color, "color"));
                    case DustParticleOptions dust -> ChunkSuites.array((Integer)get(DustParticleOptions.class, dust, "color"), bits(dust.getScale()));
                    case DustColorTransitionOptions dust -> ChunkSuites.array(
                        (Integer)get(DustColorTransitionOptions.class, dust, "fromColor"), (Integer)get(DustColorTransitionOptions.class, dust, "toColor"),
                        bits(dust.getScale())
                    );
                    default -> throw new IllegalArgumentException("no encoding for " + options.getClass().getName());
                };
            } catch (ReflectiveOperationException e) {
                throw new IllegalStateException(e);
            }
        }

        /** A sound: its event, source, volume, pitch, its random's first long, and where it is (or its entity). */
        static JsonArray sound(final SoundInstance instance, final int delay) {
            try {
                long first = ((RandomSource)get(AbstractSoundInstance.class, instance, "random")).nextLong();
                // The volume and pitch it was made with: its getters scale them by the sound resolved, which isn't.
                float volume = (Float)get(AbstractSoundInstance.class, instance, "volume");
                float pitch = (Float)get(AbstractSoundInstance.class, instance, "pitch");
                if (instance instanceof EntityBoundSoundInstance) {
                    Entity entity = (Entity)get(EntityBoundSoundInstance.class, instance, "entity");
                    return ChunkSuites.array(
                        "entitySound", instance.getIdentifier().toString(), instance.getSource().getName(), bits(volume), bits(pitch), entity.getId(),
                        first, delay
                    );
                }
                return ChunkSuites.array(
                    "sound", instance.getIdentifier().toString(), instance.getSource().getName(), bits(volume), bits(pitch), bits(instance.getX()),
                    bits(instance.getY()), bits(instance.getZ()), first, delay
                );
            } catch (ReflectiveOperationException e) {
                throw new IllegalStateException(e);
            }
        }

        private static Object get(final Class<?> owner, final Object object, final String name) throws ReflectiveOperationException {
            Field field = owner.getDeclaredField(name);
            field.setAccessible(true);
            return field.get(object);
        }

        private static void set(final Class<?> owner, final Object object, final String name, final Object value) throws ReflectiveOperationException {
            ClientLevelSuite.set(owner, object, name, value);
        }
    }
}
