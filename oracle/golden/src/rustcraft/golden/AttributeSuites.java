package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonParser;
import com.google.gson.JsonPrimitive;
import com.mojang.serialization.JsonOps;
import com.mojang.serialization.Lifecycle;
import java.io.IOException;
import java.lang.reflect.Field;
import java.lang.reflect.Method;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Random;
import java.util.Set;
import java.util.stream.Stream;
import net.minecraft.client.multiplayer.ClientLevel;
import net.minecraft.core.Holder;
import net.minecraft.core.HolderLookup;
import net.minecraft.core.MappedRegistry;
import net.minecraft.core.RegistrationInfo;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.core.registries.Registries;
import net.minecraft.data.registries.VanillaRegistries;
import net.minecraft.resources.Identifier;
import net.minecraft.resources.RegistryOps;
import net.minecraft.resources.ResourceKey;
import net.minecraft.world.attribute.AttributeTypes;
import net.minecraft.world.attribute.EnvironmentAttribute;
import net.minecraft.world.attribute.EnvironmentAttributeProbe;
import net.minecraft.world.attribute.EnvironmentAttributeSystem;
import net.minecraft.world.attribute.EnvironmentAttributes;
import net.minecraft.world.attribute.GaussianSampler;
import net.minecraft.world.attribute.SpatialAttributeInterpolator;
import net.minecraft.world.attribute.WeatherAttributes;
import net.minecraft.world.clock.ClockInstance;
import net.minecraft.world.clock.ClockManager;
import net.minecraft.world.clock.WorldClock;
import net.minecraft.world.level.Level;
import net.minecraft.world.level.MoonPhase;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.biome.BiomeManager;
import net.minecraft.world.level.dimension.DimensionType;
import net.minecraft.world.phys.Vec3;
import net.minecraft.world.timeline.Timeline;
import org.joml.Vector3fc;
import org.joml.Vector4fc;

/**
 * The environment attributes the sky, fog and lightmap read (TESTING.md, "Environment attributes"): an
 * EnvironmentAttributeSystem built the way {@code addDefaultLayers} builds a level's, over the vanilla pack's dimension
 * types, biomes and timelines decoded the way the client's synchronized registries decode its known pack (the NETWORK
 * codecs, which keep the syncable attributes), and the camera's EnvironmentAttributeProbe over it.
 */
final class AttributeSuites {
    private AttributeSuites() {
    }

    static List<Suite> suites() {
        return List.of(new SystemSuite(), new ProbeSuite());
    }

    // Every attribute of a type the port has, in the registry's order.
    private static final Set<Object> PORTED_TYPES = Set.of(
        AttributeTypes.BOOLEAN, AttributeTypes.FLOAT, AttributeTypes.ANGLE_DEGREES, AttributeTypes.RGB_COLOR, AttributeTypes.ARGB_COLOR, AttributeTypes.MOON_PHASE
    );

    static List<EnvironmentAttribute<?>> attributes() {
        return BuiltInRegistries.ENVIRONMENT_ATTRIBUTE.stream().filter(attribute -> PORTED_TYPES.contains(attribute.type())).toList();
    }

    static JsonArray attributeNames() {
        JsonArray names = new JsonArray();
        attributes().forEach(attribute -> names.add(BuiltInRegistries.ENVIRONMENT_ATTRIBUTE.getKey(attribute).toString()));
        return names;
    }

    static JsonElement encode(final Object value) {
        return switch (value) {
            case Boolean bool -> new JsonPrimitive(bool);
            case Float f -> new JsonPrimitive(String.format("%08x", Float.floatToRawIntBits(f)));
            case Vector3fc v -> RenderSuites.array(hex(v.x()), hex(v.y()), hex(v.z()));
            case Vector4fc v -> RenderSuites.array(hex(v.x()), hex(v.y()), hex(v.z()), hex(v.w()));
            case MoonPhase phase -> new JsonPrimitive(phase.getSerializedName());
            default -> throw new IllegalArgumentException("not a ported attribute value: " + value);
        };
    }

    private static JsonPrimitive hex(final float value) {
        return new JsonPrimitive(String.format("%08x", Float.floatToRawIntBits(value)));
    }

    static JsonPrimitive hexDouble(final double value) {
        return new JsonPrimitive(String.format("%016x", Double.doubleToRawLongBits(value)));
    }

    /** The vanilla pack's registries, as the client reads its known pack's synchronized ones. */
    static final class Data {
        final HolderLookup.Provider lookup = VanillaRegistries.createWorldLookup();
        final RegistryOps<JsonElement> ops = this.lookup.createSerializationContext(JsonOps.INSTANCE);
        final MappedRegistry<Biome> biomes = new MappedRegistry<>(Registries.BIOME, Lifecycle.stable());
        final Map<String, DimensionType> dimensionTypes = new LinkedHashMap<>();
        final Map<String, Holder<Timeline>> timelines = new LinkedHashMap<>();
        private static Data data;

        static synchronized Data get() throws IOException {
            if (data == null) {
                data = new Data();
            }
            return data;
        }

        private Data() throws IOException {
            Path root = Path.of("resources/data/minecraft");
            for (Path file : files(root.resolve("worldgen/biome"))) {
                Biome biome = Biome.NETWORK_CODEC.parse(this.ops, json(file)).getOrThrow();
                this.biomes.register(ResourceKey.create(Registries.BIOME, Identifier.withDefaultNamespace(name(file))), biome, RegistrationInfo.BUILT_IN);
            }
            this.biomes.freeze();
            for (Path file : files(root.resolve("dimension_type"))) {
                this.dimensionTypes.put(name(file), DimensionType.NETWORK_CODEC.parse(this.ops, json(file)).getOrThrow());
            }
            for (Path file : files(root.resolve("timeline"))) {
                this.timelines.put(name(file), Holder.direct(Timeline.NETWORK_CODEC.parse(this.ops, json(file)).getOrThrow()));
            }
        }

        private static List<Path> files(final Path directory) throws IOException {
            try (Stream<Path> files = Files.list(directory)) {
                return files.filter(file -> file.toString().endsWith(".json")).sorted().toList();
            }
        }

        private static String name(final Path file) {
            String name = file.getFileName().toString();
            return name.substring(0, name.length() - ".json".length());
        }

        private static JsonElement json(final Path file) throws IOException {
            return JsonParser.parseString(Files.readString(file));
        }

        Holder<Biome> biome(final String name) {
            return this.biomes.getOrThrow(ResourceKey.create(Registries.BIOME, Identifier.withDefaultNamespace(name)));
        }

        /**
         * A dimension type's timelines tag, read from the pack's tag files in their order, {@code #universal} first: tags
         * aren't bound in the lookup the generator builds.
         */
        List<Holder<Timeline>> timelinesOf(final String tag) throws IOException {
            List<Holder<Timeline>> result = new ArrayList<>();
            for (JsonElement value : JsonParser.parseString(Files.readString(Path.of("resources/data/minecraft/tags/timeline/" + tag + ".json")))
                .getAsJsonObject()
                .getAsJsonArray("values")) {
                String entry = value.getAsString();
                if (entry.startsWith("#")) {
                    result.addAll(this.timelinesOf(Identifier.parse(entry.substring(1)).getPath()));
                } else {
                    result.add(this.timelines.get(Identifier.parse(entry).getPath()));
                }
            }
            return result;
        }
    }

    static final class Clock implements ClockManager {
        long ticks;

        @Override
        public ClockInstance getInstance(final Holder<WorldClock> definition) {
            long ticks = this.ticks;
            return new ClockInstance() {
                @Override
                public long totalTicks() {
                    return ticks;
                }

                @Override
                public float partialTick() {
                    return 0.0F;
                }

                @Override
                public float rate() {
                    return 1.0F;
                }

                @Override
                public boolean isPaused() {
                    return false;
                }
            };
        }
    }

    static final class Weather implements WeatherAttributes.WeatherAccess {
        float rain;
        float thunder;

        @Override
        public float rainLevel() {
            return this.rain;
        }

        @Override
        public float thunderLevel() {
            return this.thunder;
        }
    }

    // The biomes a "mixed" source picks from, by a hash of the quart position over blocks of 2 by 4 by 2 quarts.
    static final List<String> MIXED_BIOMES = List.of(
        "plains", "desert", "swamp", "cherry_grove", "pale_garden", "nether_wastes", "crimson_forest", "basalt_deltas", "the_end", "deep_dark",
        "mushroom_fields", "snowy_plains", "warm_ocean", "soul_sand_valley"
    );

    static int mixedIndex(final int quartX, final int quartY, final int quartZ) {
        int h = (quartX >> 1) * 73856093 ^ (quartY >> 2) * 19349663 ^ (quartZ >> 1) * 83492791;
        h ^= h >>> 13;
        h *= 0x5bd1e995;
        h ^= h >>> 15;
        return Math.floorMod(h, MIXED_BIOMES.size());
    }

    static BiomeManager biomeManager(final Data data, final String source) {
        if (source.equals("mixed")) {
            List<Holder<Biome>> biomes = MIXED_BIOMES.stream().map(data::biome).toList();
            return new BiomeManager((x, y, z) -> biomes.get(mixedIndex(x, y, z)), 0L);
        }
        Holder<Biome> biome = data.biome(source);
        return new BiomeManager((x, y, z) -> biome, 0L);
    }

    /**
     * EnvironmentAttributeSystem.Builder.addDefaultLayers for a level of the dimension type: the dimension's constant
     * layer and the biomes' positional ones (the system's own private statics), the timelines' layers in the tag's
     * order, then the weather's where the level has weather.
     */
    static EnvironmentAttributeSystem system(
        final Data data, final String dimensionType, final String timelineTag, final boolean canHaveWeather, final BiomeManager biomeManager,
        final Clock clock, final Weather weather
    ) throws ReflectiveOperationException, IOException {
        EnvironmentAttributeSystem.Builder builder = EnvironmentAttributeSystem.builder();
        Method addDimensionLayer = EnvironmentAttributeSystem.class.getDeclaredMethod("addDimensionLayer", EnvironmentAttributeSystem.Builder.class, DimensionType.class);
        addDimensionLayer.setAccessible(true);
        addDimensionLayer.invoke(null, builder, data.dimensionTypes.get(dimensionType));
        Method addBiomeLayer = EnvironmentAttributeSystem.class.getDeclaredMethod(
            "addBiomeLayer", EnvironmentAttributeSystem.Builder.class, HolderLookup.class, BiomeManager.class
        );
        addBiomeLayer.setAccessible(true);
        addBiomeLayer.invoke(null, builder, data.biomes, biomeManager);
        for (Holder<Timeline> timeline : data.timelinesOf(timelineTag)) {
            builder.addTimelineLayer(timeline, clock);
        }
        if (canHaveWeather) {
            WeatherAttributes.addBuiltinLayers(builder, weather);
        }
        return builder.build();
    }

    // The levels the suites build: a dimension type, its timelines tag, whether it has weather, and its biome sources.
    record LevelKind(String dimensionType, String timelineTag, boolean canHaveWeather, List<String> sources) {
    }

    static final List<LevelKind> LEVELS = List.of(
        new LevelKind("overworld", "in_overworld", true, List.of("plains", "mixed")),
        new LevelKind("the_nether", "in_nether", false, List.of("nether_wastes", "mixed")),
        new LevelKind("the_end", "in_end", false, List.of("the_end", "mixed"))
    );

    static final long[] TIMES = {
        0, 1, 133, 500, 1000, 3000, 6000, 9000, 11867, 12000, 12542, 12600, 13000, 13140, 13670, 15000, 18000, 21000, 22330, 22860, 23000, 23460,
        23999, 24000, 24000L * 3 + 6000, 24000L * 5 + 13000, 192000L + 1000, -1000, 1234567
    };
    static final float[][] WEATHER = {{0.0F, 0.0F}, {0.3F, 0.0F}, {1.0F, 0.5F}, {1.0F, 1.0F}};
    static final double[][] POSITIONS = {{0.5, -60.0 + 1.62F, 0.5}, {123.45, 70.0, -67.8}, {-1000.25, 5.5, 2000.75}};

    /**
     * Every ported attribute's value from the system, at times through the day and the moon's cycle, with weather where
     * the level has it, at a few positions. "getValue" args are {@code [dimensionType, source, ticks, rain, thunder, x, y,
     * z]}, results {@code [plain, interpolated]}: each attribute's {@code getValue(attribute, pos, null)} (the biome at the
     * position), and with a SpatialAttributeInterpolator filled the way the probe fills one, in ATTRIBUTES' order. The
     * system's tick cache is invalidated before each case, as a level's tick does.
     */
    static final class SystemSuite implements Suite {
        @Override
        public String name() {
            return EnvironmentAttributeSystem.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            writer.field("ATTRIBUTES", attributeNames());
            Data data = Data.get();
            List<EnvironmentAttribute<?>> attributes = attributes();
            try {
                for (LevelKind level : LEVELS) {
                    for (String source : level.sources()) {
                        Clock clock = new Clock();
                        Weather weather = new Weather();
                        BiomeManager biomeManager = biomeManager(data, source);
                        EnvironmentAttributeSystem system = system(data, level.dimensionType(), level.timelineTag(), level.canHaveWeather(), biomeManager, clock, weather);
                        for (long ticks : TIMES) {
                            for (float[] rainAndThunder : level.canHaveWeather() ? WEATHER : new float[][]{WEATHER[0]}) {
                                for (double[] position : POSITIONS) {
                                    clock.ticks = ticks;
                                    weather.rain = rainAndThunder[0];
                                    weather.thunder = rainAndThunder[1];
                                    system.invalidateTickCache();
                                    Vec3 pos = new Vec3(position[0], position[1], position[2]);
                                    SpatialAttributeInterpolator interpolator = new SpatialAttributeInterpolator();
                                    GaussianSampler.sample(
                                        pos.scale(0.25), biomeManager::getNoiseBiomeAtQuart, (weight, biome) -> interpolator.accumulate(weight, biome.value().getAttributes())
                                    );
                                    JsonArray plain = new JsonArray();
                                    JsonArray interpolated = new JsonArray();
                                    for (EnvironmentAttribute<?> attribute : attributes) {
                                        plain.add(encode(system.getValue(attribute, pos, null)));
                                        interpolated.add(encode(system.getValue(attribute, pos, interpolator)));
                                    }
                                    writer.result(
                                        "getValue",
                                        RenderSuites.array(
                                            new JsonPrimitive(level.dimensionType()),
                                            new JsonPrimitive(source),
                                            new JsonPrimitive(ticks),
                                            hex(rainAndThunder[0]),
                                            hex(rainAndThunder[1]),
                                            hexDouble(position[0]),
                                            hexDouble(position[1]),
                                            hexDouble(position[2])
                                        ),
                                        RenderSuites.array(plain, interpolated),
                                        null
                                    );
                                }
                            }
                        }
                    }
                }
            } catch (ReflectiveOperationException e) {
                throw new IllegalStateException(e);
            }
        }
    }

    /**
     * EnvironmentAttributeProbe, the camera's, ticked at a position and read with partial ticks, over the overworld's
     * system with the mixed biomes. A program's ops are {@code ["tick", ticks, rain, thunder, x, y, z]} (the clock and the
     * weather set, the system's tick cache invalidated as the level's tick does, then the probe ticked at the position)
     * and {@code ["get", attribute, partialTicks]}, whose values are the result in order. The probe reads the level
     * through {@code environmentAttributes()} and {@code getBiomeManager()}, so the level is a ClientLevel made without
     * its constructor that has those two fields set.
     */
    static final class ProbeSuite implements Suite {
        @Override
        public String name() {
            return EnvironmentAttributeProbe.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            Data data = Data.get();
            List<EnvironmentAttribute<?>> attributes = attributes();
            Random random = new Random(0x5e7);
            try {
                for (int program = 0; program < 48; program++) {
                    Clock clock = new Clock();
                    Weather weather = new Weather();
                    BiomeManager biomeManager = biomeManager(data, program % 3 == 0 ? "plains" : "mixed");
                    EnvironmentAttributeSystem system = system(data, "overworld", "in_overworld", true, biomeManager, clock, weather);
                    Level level = level(system, biomeManager);
                    EnvironmentAttributeProbe probe = new EnvironmentAttributeProbe();
                    JsonArray ops = new JsonArray();
                    JsonArray results = new JsonArray();
                    long ticks = random.nextInt(24000 * 8);
                    double x = random.nextDouble() * 200.0 - 100.0;
                    double y = random.nextDouble() * 100.0 - 60.0;
                    double z = random.nextDouble() * 200.0 - 100.0;
                    // Some reads before the first tick, which the probe answers with defaults.
                    for (int step = 0; step < 30; step++) {
                        if (step > 0) {
                            ticks += random.nextInt(5) == 0 ? random.nextInt(3000) : random.nextInt(3);
                            x += random.nextGaussian() * 3.0;
                            y += random.nextGaussian() * 2.0;
                            z += random.nextGaussian() * 3.0;
                            float rain = random.nextInt(4) == 0 ? random.nextFloat() : 0.0F;
                            float thunder = random.nextInt(8) == 0 ? random.nextFloat() * rain : 0.0F;
                            clock.ticks = ticks;
                            weather.rain = rain;
                            weather.thunder = thunder;
                            system.invalidateTickCache();
                            probe.tick(level, new Vec3(x, y, z));
                            ops.add(RenderSuites.array(
                                new JsonPrimitive("tick"), new JsonPrimitive(ticks), hex(rain), hex(thunder), hexDouble(x), hexDouble(y), hexDouble(z)
                            ));
                        }
                        // A few attributes each step, so some go unread for a tick and their probes are dropped, and often
                        // the angles, which blend between ticks along the shorter way round.
                        List<EnvironmentAttribute<?>> reads = new ArrayList<>();
                        for (int read = random.nextInt(6); read > 0; read--) {
                            reads.add(attributes.get(random.nextInt(attributes.size())));
                        }
                        if (random.nextBoolean()) {
                            reads.addAll(List.of(EnvironmentAttributes.SUN_ANGLE, EnvironmentAttributes.MOON_ANGLE, EnvironmentAttributes.STAR_ANGLE));
                        }
                        for (EnvironmentAttribute<?> attribute : reads) {
                            float partialTicks = random.nextInt(3) == 0 ? random.nextInt(2) : random.nextFloat();
                            ops.add(RenderSuites.array(
                                new JsonPrimitive("get"),
                                new JsonPrimitive(BuiltInRegistries.ENVIRONMENT_ATTRIBUTE.getKey(attribute).toString()),
                                hex(partialTicks)
                            ));
                            results.add(encode(probe.getValue(attribute, partialTicks)));
                        }
                    }
                    writer.result("program", RenderSuites.array(new JsonPrimitive(program % 3 == 0 ? "plains" : "mixed"), ops), results, null);
                }
            } catch (ReflectiveOperationException e) {
                throw new IllegalStateException(e);
            }
        }

        private static Level level(final EnvironmentAttributeSystem system, final BiomeManager biomeManager) throws ReflectiveOperationException {
            Field theUnsafe = sun.misc.Unsafe.class.getDeclaredField("theUnsafe");
            theUnsafe.setAccessible(true);
            ClientLevel level = (ClientLevel)((sun.misc.Unsafe)theUnsafe.get(null)).allocateInstance(ClientLevel.class);
            Field environmentAttributes = ClientLevel.class.getDeclaredField("environmentAttributes");
            environmentAttributes.setAccessible(true);
            environmentAttributes.set(level, system);
            Field biomes = Level.class.getDeclaredField("biomeManager");
            biomes.setAccessible(true);
            biomes.set(level, biomeManager);
            return level;
        }
    }
}
