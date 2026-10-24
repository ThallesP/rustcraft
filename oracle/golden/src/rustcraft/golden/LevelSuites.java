package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonNull;
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
import net.minecraft.world.level.Level;
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
        return List.of(new WorldBorderSuite(), new ClientLevelSuite());
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
            new BiomeSuites.TintSuite().write(writer);
            try {
                this.writeTicks(writer);
            } catch (ReflectiveOperationException e) {
                throw new IllegalStateException(e);
            }
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
            set(Level.class, level, "levelData", levelData);
            set(Level.class, level, "dimension", ResourceKey.create(Registries.DIMENSION, Identifier.withDefaultNamespace(dimension)));
            Holder<DimensionType> dimensionType = RegistryValues.registries()
                .lookupOrThrow(Registries.DIMENSION_TYPE)
                .getOrThrow(ResourceKey.create(Registries.DIMENSION_TYPE, Identifier.withDefaultNamespace(dimension)));
            set(Level.class, level, "dimensionTypeRegistration", dimensionType);
            return level;
        }

        private static void set(final Class<?> owner, final Object object, final String name, final Object value) throws ReflectiveOperationException {
            Field field = owner.getDeclaredField(name);
            field.setAccessible(true);
            field.set(object, value);
        }
    }
}
