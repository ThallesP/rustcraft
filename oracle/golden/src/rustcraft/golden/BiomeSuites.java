package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonNull;
import com.google.gson.JsonParser;
import com.google.gson.JsonPrimitive;
import com.mojang.serialization.JsonOps;
import io.netty.buffer.ByteBuf;
import io.netty.buffer.ByteBufOutputStream;
import io.netty.buffer.Unpooled;
import java.io.DataOutputStream;
import java.io.IOException;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.lang.reflect.Field;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Random;
import net.minecraft.client.resources.LegacyStuffWrapper;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Cursor3D;
import net.minecraft.core.Holder;
import net.minecraft.core.IdMapper;
import net.minecraft.core.QuartPos;
import net.minecraft.core.RegistryAccess;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.nbt.CompoundTag;
import net.minecraft.nbt.DoubleTag;
import net.minecraft.nbt.IntTag;
import net.minecraft.nbt.NbtIo;
import net.minecraft.nbt.NbtOps;
import net.minecraft.nbt.Tag;
import net.minecraft.network.FriendlyByteBuf;
import net.minecraft.network.protocol.game.ClientboundChunksBiomesPacket;
import net.minecraft.resources.Identifier;
import net.minecraft.resources.ResourceKey;
import net.minecraft.util.ARGB;
import net.minecraft.util.Mth;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.ColorResolver;
import net.minecraft.world.level.DryFoliageColor;
import net.minecraft.world.level.FoliageColor;
import net.minecraft.world.level.GrassColor;
import net.minecraft.world.level.biome.Biome;
import net.minecraft.world.level.biome.BiomeManager;
import net.minecraft.world.level.biome.BiomeSpecialEffects;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.chunk.LevelChunkSection;
import net.minecraft.world.level.chunk.PalettedContainer;
import net.minecraft.world.level.chunk.PalettedContainerFactory;
import net.minecraft.world.level.chunk.Strategy;
import net.minecraft.client.renderer.BiomeColors;

/**
 * The biomes and their colours, as the client reads them: each biome of the walk's registry decoded from the vanilla
 * pack's JSON by {@code Biome.NETWORK_CODEC} (and from NBT, as a server sends it), the colormaps the resource reload
 * loads, {@code BiomeManager}'s zoomed lookup, and {@code ClientLevel.calculateBlockTint}'s blend over the walk's
 * chunks with biomes laid out on them. See TESTING.md ("Biomes and tints").
 */
final class BiomeSuites {
    // The static registries, which a biome's attributes name sounds and particles of, as the client's registry ops do.
    private static final RegistryAccess BUILT_IN = RegistryAccess.fromRegistryOfRegistries(BuiltInRegistries.REGISTRY);

    private BiomeSuites() {
    }

    static List<Suite> suites() {
        return List.of(
            new ColormapSuite(GrassColor.class.getName(), "grass", GrassColor::get), new ColormapSuite(FoliageColor.class.getName(), "foliage", FoliageColor::get),
            new ColormapSuite(DryFoliageColor.class.getName(), "dry_foliage", DryFoliageColor::get), new BiomeSuite(), new BiomeManagerSuite(), new BlockColorsSuite()
        );
    }

    interface Colormap {
        int get(double temp, double rain);
    }

    private static boolean colormapsLoaded;

    /** The three colormaps, loaded like their reload listeners load them. */
    static synchronized void loadColormaps() {
        if (!colormapsLoaded) {
            try {
                GrassColor.init(LegacyStuffWrapper.getPixels(TextureSuites.vanillaResources(), Identifier.withDefaultNamespace("textures/colormap/grass.png")));
                FoliageColor.init(LegacyStuffWrapper.getPixels(TextureSuites.vanillaResources(), Identifier.withDefaultNamespace("textures/colormap/foliage.png")));
                DryFoliageColor.init(LegacyStuffWrapper.getPixels(TextureSuites.vanillaResources(), Identifier.withDefaultNamespace("textures/colormap/dry_foliage.png")));
            } catch (IOException e) {
                throw new IllegalStateException(e);
            }
            colormapsLoaded = true;
        }
    }

    /** The biomes of the walk's registry, in its order, read from the vanilla pack like a known pack's elements. */
    private static List<Biome> biomes;

    static List<Biome> biomes() {
        if (biomes == null) {
            List<Biome> values = new ArrayList<>();
            for (ResourceKey<Object> key : LevelChunkSuites.biomeKeys()) {
                Identifier id = key.identifier();
                String path = "/data/" + id.getNamespace() + "/worldgen/biome/" + id.getPath() + ".json";
                try (InputStream stream = Biome.class.getResourceAsStream(path)) {
                    JsonElement json = JsonParser.parseReader(new InputStreamReader(stream, StandardCharsets.UTF_8));
                    values.add(Biome.NETWORK_CODEC.parse(BUILT_IN.createSerializationContext(JsonOps.INSTANCE), json).getOrThrow());
                } catch (IOException e) {
                    throw new IllegalStateException(e);
                }
            }
            biomes = values;
        }
        return biomes;
    }

    private static Object field(final Object object, final String name) {
        try {
            Field field = object.getClass().getDeclaredField(name);
            field.setAccessible(true);
            return field.get(object);
        } catch (ReflectiveOperationException e) {
            throw new IllegalStateException(e);
        }
    }

    private static JsonElement optional(final java.util.Optional<Integer> value) {
        return value.<JsonElement>map(JsonPrimitive::new).orElse(JsonNull.INSTANCE);
    }

    /**
     * A biome: {@code [hasPrecipitation, temperature, temperatureModifier, downfall, waterColor, foliageColorOverride,
     * dryFoliageColorOverride, grassColorOverride, grassColorModifier]}, the climate's private fields read by reflection,
     * floats as their bits.
     */
    static JsonArray fields(final Biome biome) {
        Object climate = field(biome, "climateSettings");
        BiomeSpecialEffects effects = biome.getSpecialEffects();
        return ChunkSuites.array(
            (Boolean)field(climate, "hasPrecipitation"), String.format("%08x", Float.floatToRawIntBits((Float)field(climate, "temperature"))),
            ((Biome.TemperatureModifier)field(climate, "temperatureModifier")).getSerializedName(),
            String.format("%08x", Float.floatToRawIntBits((Float)field(climate, "downfall"))), effects.waterColor(), optional(effects.foliageColorOverride()),
            optional(effects.dryFoliageColorOverride()), optional(effects.grassColorOverride()), effects.grassColorModifier().getSerializedName()
        );
    }

    static String nbt(final Tag tag) {
        ByteBuf out = Unpooled.buffer();
        try (DataOutputStream stream = new DataOutputStream(new ByteBufOutputStream(out))) {
            NbtIo.writeAnyTag(tag, stream);
        } catch (IOException e) {
            throw new IllegalStateException(e);
        }
        byte[] bytes = new byte[out.readableBytes()];
        out.getBytes(0, bytes);
        return ChunkSuites.hex(bytes);
    }

    /**
     * {@code pixels}, the colormap as {@code LegacyStuffWrapper.getPixels} loads it from {@code textures/colormap/}, and
     * {@code get(DD)I} on temperatures and downfalls from below 0 to past 1 in 64ths, at the colormap's grid lines and
     * either side, NaN, and random ones.
     */
    static final class ColormapSuite implements Suite {
        private final String name;
        private final String texture;
        private final Colormap colormap;

        ColormapSuite(final String name, final String texture, final Colormap colormap) {
            this.name = name;
            this.texture = texture;
            this.colormap = colormap;
        }

        @Override
        public String name() {
            return this.name;
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            int[] pixels = LegacyStuffWrapper.getPixels(TextureSuites.vanillaResources(), Identifier.withDefaultNamespace("textures/colormap/" + this.texture + ".png"));
            writer.field("pixels", ChunkSuites.ints(pixels));
            loadColormaps();
            List<Double> values = new ArrayList<>();
            for (int i = -4; i <= 68; i++) {
                values.add(i / 64.0);
            }
            for (int i = 0; i <= 255; i += 15) {
                double edge = 1.0 - i / 255.0;
                values.add(edge);
                values.add(Math.nextUp(edge));
                values.add(Math.nextDown(edge));
            }
            values.add(Double.NaN);
            Random random = new Random(this.texture.hashCode());
            for (int i = 0; i < 40; i++) {
                values.add(random.nextDouble());
            }
            for (double temp : values) {
                for (double rain : values) {
                    JsonArray args = ChunkSuites.array(hex(temp), hex(rain));
                    try {
                        writer.result("get(DD)I", args, new JsonPrimitive(this.colormap.get(temp, rain)), null);
                    } catch (RuntimeException e) {
                        writer.threw("get(DD)I", args, e, null);
                    }
                }
            }
        }
    }

    static String hex(final double value) {
        return String.format("%016x", Double.doubleToRawLongBits(value));
    }

    /**
     * {@code json}: each biome of the walk's registry, by its identifier, decoded from the vanilla pack's JSON →
     * {@code [fields, nbt]}, its fields and its NBT as {@code NETWORK_CODEC} writes it with {@code NbtOps}
     * ({@code NbtIo.writeAnyTag}, in hex). {@code nbt}: NBT the codec reads → the fields: each biome's, and plains' with
     * a colour as an int and the temperature as a double. {@code colors}: {@code [id, x, z]} → its grass, foliage, dry
     * foliage and water colours there, with the colormaps loaded, at blocks near and far (the swamp's grass reads the
     * noise there).
     */
    static final class BiomeSuite implements Suite {
        @Override
        public String name() {
            return Biome.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            loadColormaps();
            writer.field("BIOMES", LevelChunkSuites.biomeNames());
            List<ResourceKey<Object>> keys = LevelChunkSuites.biomeKeys();
            Random random = new Random(0xb104e);
            for (int i = 0; i < keys.size(); i++) {
                Biome biome = biomes().get(i);
                String id = keys.get(i).identifier().toString();
                Tag tag = Biome.NETWORK_CODEC.encodeStart(BUILT_IN.createSerializationContext(NbtOps.INSTANCE), biome).getOrThrow();
                writer.result("json", ChunkSuites.array(id), ChunkSuites.array(fields(biome), nbt(tag)), null);
                writer.result("nbt", ChunkSuites.array(nbt(tag)), fields(Biome.NETWORK_CODEC.parse(BUILT_IN.createSerializationContext(NbtOps.INSTANCE), tag).getOrThrow()), null);
                if (id.equals("minecraft:plains")) {
                    CompoundTag variant = ((CompoundTag)tag).copy();
                    variant.putDouble("temperature", 0.8);
                    CompoundTag effects = variant.getCompoundOrEmpty("effects");
                    effects.putInt("water_color", 0x123456);
                    variant.put("effects", effects);
                    writer.result("nbt", ChunkSuites.array(nbt(variant)), fields(Biome.NETWORK_CODEC.parse(BUILT_IN.createSerializationContext(NbtOps.INSTANCE), variant).getOrThrow()), null);
                }
                for (int j = 0; j < 24; j++) {
                    double x = j < 8 ? random.nextInt(64) - 32 : (random.nextDouble() - 0.5) * (j < 16 ? 2000.0 : 6.0E7);
                    double z = j < 8 ? random.nextInt(64) - 32 : (random.nextDouble() - 0.5) * (j < 16 ? 2000.0 : 6.0E7);
                    writer.result(
                        "colors", ChunkSuites.array(id, hex(x), hex(z)),
                        ChunkSuites.array(biome.getGrassColor(x, z), biome.getFoliageColor(), biome.getDryFoliageColor(), biome.getWaterColor()), null
                    );
                }
            }
        }
    }

    /**
     * {@code getBiome(III)}: {@code [seed, x, y, z]} → the quart {@code [x, y, z]} whose biome it picks, from a source that
     * says which it was asked for. Seeds are the walk's (its world seed's hash, as the login sends it), 0 and any longs;
     * blocks around the spawn, anywhere in the world, and at the int limits.
     */
    static final class BiomeManagerSuite implements Suite {
        @Override
        public String name() {
            return BiomeManager.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            Random random = new Random(0xb10b);
            List<Long> seeds = new ArrayList<>(List.of(BiomeManager.obfuscateSeed(0L), 0L, -1L, Long.MIN_VALUE, Long.MAX_VALUE));
            for (int i = 0; i < 6; i++) {
                seeds.add(random.nextLong());
            }
            int[] asked = new int[3];
            for (long seed : seeds) {
                BiomeManager manager = new BiomeManager((qx, qy, qz) -> {
                    asked[0] = qx;
                    asked[1] = qy;
                    asked[2] = qz;
                    return null;
                }, seed);
                for (int i = 0; i < 400; i++) {
                    int x;
                    int y;
                    int z;
                    switch (i % 4) {
                        case 0 -> {
                            x = random.nextInt(64) - 32;
                            y = random.nextInt(64) - 64;
                            z = random.nextInt(64) - 32;
                        }
                        case 1 -> {
                            x = random.nextInt(60000000) - 30000000;
                            y = random.nextInt(400) - 64;
                            z = random.nextInt(60000000) - 30000000;
                        }
                        case 2 -> {
                            int[] limits = {Integer.MIN_VALUE, Integer.MIN_VALUE + 1, -1, 0, 1, 2, 3, Integer.MAX_VALUE - 1, Integer.MAX_VALUE};
                            x = limits[random.nextInt(limits.length)];
                            y = limits[random.nextInt(limits.length)];
                            z = limits[random.nextInt(limits.length)];
                        }
                        default -> {
                            x = random.nextInt();
                            y = random.nextInt();
                            z = random.nextInt();
                        }
                    }
                    manager.getBiome(x, y, z);
                    writer.result("getBiome(III)Lnet/minecraft/core/Holder;", ChunkSuites.array(seed, x, y, z), ChunkSuites.array(asked[0], asked[1], asked[2]), null);
                }
            }
        }
    }

    /**
     * {@code layouts}: {@code [radius, chunksBiomes, columns]} → the four resolvers' tints ({@code GRASS}, {@code FOLIAGE},
     * {@code DRY_FOLIAGE}, {@code WATER}) at each column's blocks, as {@code ClientLevel.calculateBlockTint} blends them
     * with that biome blend radius over the walk's chunks once a {@code chunks_biomes} packet (in hex) has laid biomes on
     * them: patches of a few biomes per section, every biome of the registry somewhere, swamps among them. A column is
     * {@code [x, z]}, its blocks at y -61, 0 and 70, and the chunks the walk doesn't have answer plains
     * ({@code getUncachedNoiseBiome}). The zoom seed is the walk's, its world seed's hash ({@code ZOOM_SEED}).
     */
    static final class TintSuite implements Suite {
        private static final List<ColorResolver> RESOLVERS = List.of(
            BiomeColors.GRASS_COLOR_RESOLVER, BiomeColors.FOLIAGE_COLOR_RESOLVER, BiomeColors.DRY_FOLIAGE_COLOR_RESOLVER, BiomeColors.WATER_COLOR_RESOLVER
        );
        private static final int[] HEIGHTS = {-61, 0, 70};

        @Override
        public String name() {
            return "net.minecraft.client.multiplayer.ClientLevel";
        }

        @Override
        @SuppressWarnings({"unchecked", "rawtypes"})
        public void write(final GoldenWriter writer) throws IOException {
            loadColormaps();
            List<Holder<Biome>> holders = biomes().stream().map(Holder::direct).toList();
            IdMapper<Holder<Biome>> registry = new IdMapper<>();
            holders.forEach(registry::add);
            Holder<Biome> plains = holders.get(LevelChunkSuites.biomeKeys().stream().map(key -> key.identifier().toString()).toList().indexOf("minecraft:plains"));
            PalettedContainerFactory factory = new PalettedContainerFactory(
                Strategy.createForBlockStates(Block.BLOCK_STATE_REGISTRY), Blocks.AIR.defaultBlockState(), null, Strategy.createForBiomes((IdMapper)registry), plains, null
            );
            long seed = BiomeManager.obfuscateSeed(0L);
            writer.field("ZOOM_SEED", new JsonPrimitive(seed));
            Random random = new Random(0x7e17);
            int[] radii = {2, 2, 0, 7};
            for (int layout = 0; layout < radii.length; layout++) {
                // Each chunk's sections' biomes, in patches of up to four biomes a section.
                Map<Long, LevelChunkSection[]> sections = new HashMap<>();
                List<ClientboundChunksBiomesPacket.ChunkBiomeData> data = new ArrayList<>();
                for (ClientboundLevelChunkWithLightPacketPos pos : chunkPositions()) {
                    LevelChunkSection[] chunk = new LevelChunkSection[LevelChunkSuites.SECTION_COUNT];
                    FriendlyByteBuf buffer = new FriendlyByteBuf(Unpooled.buffer());
                    for (int s = 0; s < chunk.length; s++) {
                        LevelChunkSection section = new LevelChunkSection(factory);
                        PalettedContainer<Holder<Biome>> biomes = (PalettedContainer)section.getBiomes();
                        List<Holder<Biome>> palette = new ArrayList<>();
                        for (int p = 1 + random.nextInt(4); p > 0; p--) {
                            palette.add(holders.get(random.nextInt(holders.size())));
                        }
                        for (int x = 0; x < 4; x++) {
                            for (int y = 0; y < 4; y++) {
                                for (int z = 0; z < 4; z++) {
                                    biomes.getAndSetUnchecked(x, y, z, palette.get(((x / 2) + (z / 2) * 2 + random.nextInt(2)) % palette.size()));
                                }
                            }
                        }
                        biomes.write(buffer);
                        chunk[s] = section;
                    }
                    byte[] bytes = new byte[buffer.readableBytes()];
                    buffer.getBytes(0, bytes);
                    data.add(new ClientboundChunksBiomesPacket.ChunkBiomeData(new ChunkPos(pos.x, pos.z), bytes));
                    sections.put(ChunkPos.pack(pos.x, pos.z), chunk);
                }
                ByteBuf out = Unpooled.buffer();
                ClientboundChunksBiomesPacket.STREAM_CODEC.encode(out, new ClientboundChunksBiomesPacket(data));
                byte[] packet = new byte[out.readableBytes()];
                out.getBytes(0, packet);

                // ChunkAccess.getNoiseBiome of the level's chunks, or LevelReader's plains where it has none.
                BiomeManager manager = new BiomeManager((qx, qy, qz) -> {
                    LevelChunkSection[] chunk = sections.get(ChunkPos.pack(QuartPos.toSection(qx), QuartPos.toSection(qz)));
                    if (chunk == null) {
                        return plains;
                    }
                    int quartMinY = QuartPos.fromBlock(-64);
                    int quartMaxY = quartMinY + QuartPos.fromBlock(384) - 1;
                    int clampedQuartY = Mth.clamp(qy, quartMinY, quartMaxY);
                    int sectionIndex = (QuartPos.toBlock(clampedQuartY) >> 4) - (-64 >> 4);
                    return chunk[sectionIndex].getNoiseBiome(qx & 3, clampedQuartY & 3, qz & 3);
                }, seed);
                int radius = radii[layout];
                JsonArray columns = new JsonArray();
                JsonArray tints = new JsonArray();
                int span = layout < 2 ? 2 : 1;
                for (int chunkX = -span; chunkX <= span; chunkX++) {
                    for (int chunkZ = -span; chunkZ <= span; chunkZ++) {
                        for (int z = 0; z < 16; z++) {
                            for (int x = 0; x < 16; x++) {
                                int blockX = chunkX * 16 + x;
                                int blockZ = chunkZ * 16 + z;
                                columns.add(ChunkSuites.array(blockX, blockZ));
                                for (int y : HEIGHTS) {
                                    for (ColorResolver resolver : RESOLVERS) {
                                        tints.add(calculateBlockTint(manager, new BlockPos(blockX, y, blockZ), resolver, radius));
                                    }
                                }
                            }
                        }
                    }
                }
                // And the walk's edge, where the blend reaches chunks it doesn't have.
                for (int i = 0; i < 64; i++) {
                    int blockX = random.nextBoolean() ? random.nextInt(32) + 80 : -random.nextInt(32) - 81;
                    int blockZ = random.nextInt(200) - 100;
                    columns.add(ChunkSuites.array(blockX, blockZ));
                    for (int y : HEIGHTS) {
                        for (ColorResolver resolver : RESOLVERS) {
                            tints.add(calculateBlockTint(manager, new BlockPos(blockX, y, blockZ), resolver, radius));
                        }
                    }
                }
                writer.result("layouts", ChunkSuites.array(radius, ChunkSuites.hex(packet), columns), tints, null);
            }
        }

        /** ClientLevel.calculateBlockTint, with the blend radius given. */
        private static int calculateBlockTint(final BiomeManager manager, final BlockPos pos, final ColorResolver colorResolver, final int dist) {
            if (dist == 0) {
                return colorResolver.getColor(manager.getBiome(pos).value(), pos.getX(), pos.getZ());
            }

            int count = (dist * 2 + 1) * (dist * 2 + 1);
            int totalRed = 0;
            int totalGreen = 0;
            int totalBlue = 0;
            Cursor3D cursor = new Cursor3D(pos.getX() - dist, pos.getY(), pos.getZ() - dist, pos.getX() + dist, pos.getY(), pos.getZ() + dist);
            BlockPos.MutableBlockPos nextPos = new BlockPos.MutableBlockPos();

            while (cursor.advance()) {
                nextPos.set(cursor.nextX(), cursor.nextY(), cursor.nextZ());
                int color = colorResolver.getColor(manager.getBiome(nextPos).value(), nextPos.getX(), nextPos.getZ());
                totalRed += ARGB.red(color);
                totalGreen += ARGB.green(color);
                totalBlue += ARGB.blue(color);
            }

            return ARGB.color(totalRed / count, totalGreen / count, totalBlue / count);
        }

        record ClientboundLevelChunkWithLightPacketPos(int x, int z) {
        }

        static List<ClientboundLevelChunkWithLightPacketPos> chunkPositions() {
            List<ClientboundLevelChunkWithLightPacketPos> positions = new ArrayList<>();
            LightSuites.chunks().keySet().forEach(pos -> positions.add(new ClientboundLevelChunkWithLightPacketPos(ChunkPos.getX(pos), ChunkPos.getZ(pos))));
            return positions;
        }
    }

    /**
     * {@code BlockColors.createDefault()}: {@code sources}, each block by name → how many tint layers it has, and
     * {@code colors}, each state of a block with layers → each layer's {@code [color, colorInWorld,
     * colorAsTerrainParticle]}, the last two at (1, 2, 3) of {@code BlockAndTintGetter.EMPTY}, whose tints are -1, with
     * the colormaps loaded.
     */
    static final class BlockColorsSuite implements Suite {
        @Override
        public String name() {
            return net.minecraft.client.color.block.BlockColors.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            loadColormaps();
            net.minecraft.client.color.block.BlockColors colors = net.minecraft.client.color.block.BlockColors.createDefault();
            net.minecraft.client.renderer.block.BlockAndTintGetter level = net.minecraft.client.renderer.block.BlockAndTintGetter.EMPTY;
            BlockPos pos = new BlockPos(1, 2, 3);
            for (Block block : net.minecraft.core.registries.BuiltInRegistries.BLOCK) {
                List<net.minecraft.client.color.block.BlockTintSource> layers = colors.getTintSources(block.defaultBlockState());
                writer.result("sources", ChunkSuites.array(net.minecraft.core.registries.BuiltInRegistries.BLOCK.getKey(block).toString()), new JsonPrimitive(layers.size()), null);
                if (layers.isEmpty()) {
                    continue;
                }
                for (net.minecraft.world.level.block.state.BlockState state : block.getStateDefinition().getPossibleStates()) {
                    JsonArray result = new JsonArray();
                    for (net.minecraft.client.color.block.BlockTintSource source : colors.getTintSources(state)) {
                        result.add(ChunkSuites.array(source.color(state), source.colorInWorld(state, level, pos), source.colorAsTerrainParticle(state, level, pos)));
                    }
                    writer.result("colors", ChunkSuites.array(Block.getId(state)), result, null);
                }
            }
        }
    }
}
