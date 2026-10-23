package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonNull;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import com.google.gson.JsonPrimitive;
import io.netty.buffer.ByteBuf;
import io.netty.buffer.Unpooled;
import java.io.IOException;
import java.lang.reflect.Constructor;
import java.lang.reflect.Field;
import java.lang.reflect.Modifier;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.BitSet;
import java.util.EnumMap;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.Random;
import java.util.function.Function;
import java.util.function.Supplier;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Holder;
import net.minecraft.core.IdMapper;
import net.minecraft.core.RegistryAccess;
import net.minecraft.core.SectionPos;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.core.registries.Registries;
import net.minecraft.nbt.CompoundTag;
import net.minecraft.nbt.IntTag;
import net.minecraft.nbt.ListTag;
import net.minecraft.nbt.StringTag;
import net.minecraft.network.FriendlyByteBuf;
import net.minecraft.network.RegistryFriendlyByteBuf;
import net.minecraft.network.VarInt;
import net.minecraft.network.codec.StreamCodec;
import net.minecraft.network.protocol.configuration.ClientboundRegistryDataPacket;
import net.minecraft.network.protocol.game.ClientboundBlockUpdatePacket;
import net.minecraft.network.protocol.game.ClientboundChunksBiomesPacket;
import net.minecraft.network.protocol.game.ClientboundForgetLevelChunkPacket;
import net.minecraft.network.protocol.game.ClientboundLevelChunkPacketData;
import net.minecraft.network.protocol.game.ClientboundLevelChunkWithLightPacket;
import net.minecraft.network.protocol.game.ClientboundLightUpdatePacket;
import net.minecraft.network.protocol.game.ClientboundLightUpdatePacketData;
import net.minecraft.network.protocol.game.ClientboundSectionBlocksUpdatePacket;
import net.minecraft.resources.ResourceKey;
import net.minecraft.world.level.ChunkPos;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.Blocks;
import net.minecraft.world.level.block.entity.BlockEntityType;
import net.minecraft.world.level.block.entity.BlockEntityTypes;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.DataLayer;
import net.minecraft.world.level.chunk.LevelChunkSection;
import net.minecraft.world.level.chunk.PalettedContainer;
import net.minecraft.world.level.chunk.PalettedContainerFactory;
import net.minecraft.world.level.chunk.Strategy;
import net.minecraft.world.level.levelgen.Heightmap;

/**
 * The client's chunk data: what the server's chunk, light, block and biome packets carry, decoded by vanilla's own
 * classes, and the sections and light layers they fill. See TESTING.md ("Chunk data"). The chunks are the walk
 * scenario's: every {@code level_chunk_with_light} payload in {@code oracle/walk/capture.jsonl}, keyed by its own bytes,
 * so the goldens don't depend on how many there are or in what order they came. Its biomes are the capture's
 * synchronized {@code worldgen/biome} registry, in the order the server sent them.
 */
final class LevelChunkSuites {
    // The oracle client's capture of the walk scenario, or the one RUSTCRAFT_CHUNK_CAPTURE names: CI has no oracle
    // captures, so it records the walk's server with the port's own client (tools/port_client.py capture walk). The
    // chunk bytes are the server's either way: the same 117 chunks, which only differ in their heightmaps' order.
    private static final Path CAPTURE = Path.of(System.getenv().getOrDefault("RUSTCRAFT_CHUNK_CAPTURE", "oracle/walk/capture.jsonl"));
    // The overworld's (data/minecraft/dimension_type/overworld.json: min_y -64, height 384), which the walk's login names.
    static final int SECTION_COUNT = 384 / 16;
    private static final RegistryAccess BUILT_IN = RegistryAccess.fromRegistryOfRegistries(BuiltInRegistries.REGISTRY);

    private LevelChunkSuites() {
    }

    static List<Suite> suites() {
        return List.of(
            new BlockEntityTypesSuite(), new DataLayerSuite(), new LevelChunkSectionSuite(), new ChunkPacketSuite(), new LightUpdatePacketSuite(),
            new ForgetLevelChunkPacketSuite(), new BlockUpdatePacketSuite(), new SectionBlocksUpdatePacketSuite(), new ChunksBiomesPacketSuite()
        );
    }

    // The capture, read once.
    private static List<JsonObject> capture;

    static List<JsonObject> capture() {
        if (capture == null) {
            List<JsonObject> packets = new ArrayList<>();
            try {
                for (String line : Files.readAllLines(CAPTURE)) {
                    packets.add(JsonParser.parseString(line).getAsJsonObject());
                }
            } catch (IOException e) {
                throw new IllegalStateException(
                    "can't read " + CAPTURE.toAbsolutePath() + ": record the walk scenario first (oracle_client.py capture walk, or port_client.py "
                        + "capture walk with RUSTCRAFT_CHUNK_CAPTURE naming its capture.jsonl)", e
                );
            }
            capture = packets;
        }
        return capture;
    }

    static List<byte[]> payloads(final String name) {
        List<byte[]> payloads = new ArrayList<>();
        for (JsonObject packet : capture()) {
            if (packet.get("name").getAsString().equals(name)) {
                payloads.add(ChunkSuites.unhex(packet.get("payload").getAsString()));
            }
        }
        return payloads;
    }

    private static List<ResourceKey<Object>> biomeKeys;

    /** The biome registry's keys in the order the capture's registry data sent them. */
    @SuppressWarnings({"unchecked", "rawtypes"})
    static List<ResourceKey<Object>> biomeKeys() {
        if (biomeKeys == null) {
            List<ResourceKey<Object>> keys = new ArrayList<>();
            for (byte[] payload : payloads("minecraft:registry_data")) {
                ClientboundRegistryDataPacket packet = ClientboundRegistryDataPacket.STREAM_CODEC.decode(new FriendlyByteBuf(Unpooled.wrappedBuffer(payload)));
                if (packet.registry().equals(Registries.BIOME)) {
                    packet.entries().forEach(entry -> keys.add((ResourceKey)ResourceKey.create(Registries.BIOME, entry.id())));
                }
            }
            biomeKeys = keys;
        }
        return biomeKeys;
    }

    /** The biome registry's identifiers, in its order: what a check builds the registry from. */
    static JsonArray biomeNames() {
        JsonArray names = new JsonArray();
        biomeKeys().forEach(key -> names.add(key.identifier().toString()));
        return names;
    }

    private static List<Holder<Object>> biomes;

    /**
     * The biomes, each a holder of its key: their palettes only need the registry's ids, and a holder of the registry is
     * its key's, one for one.
     */
    static List<Holder<Object>> biomes() {
        if (biomes == null) {
            biomes = biomeKeys().stream().map(key -> (Holder<Object>)Holder.direct((Object)key)).toList();
        }
        return biomes;
    }

    private static PalettedContainerFactory factory;

    /** PalettedContainerFactory.create's, with the capture's biomes. The codecs, the disk format's, are none. */
    @SuppressWarnings({"unchecked", "rawtypes"})
    static PalettedContainerFactory factory() {
        if (factory == null) {
            IdMapper<Holder<Object>> registry = new IdMapper<>();
            biomes().forEach(registry::add);
            Holder plains = biomes().get(biomeKeys().stream().map(key -> key.identifier().toString()).toList().indexOf("minecraft:plains"));
            Strategy blockStates = Strategy.createForBlockStates(Block.BLOCK_STATE_REGISTRY);
            Strategy biomeStrategy = Strategy.createForBiomes((IdMapper)registry);
            factory = new PalettedContainerFactory(blockStates, Blocks.AIR.defaultBlockState(), null, biomeStrategy, plains, null);
        }
        return factory;
    }

    static int biomeId(final Object biome) {
        return biomes().indexOf(biome);
    }

    static RegistryFriendlyByteBuf reader(final byte[] bytes) {
        return new RegistryFriendlyByteBuf(Unpooled.wrappedBuffer(bytes), BUILT_IN);
    }

    static RegistryFriendlyByteBuf writer() {
        return new RegistryFriendlyByteBuf(Unpooled.buffer(), BUILT_IN);
    }

    static JsonPrimitive hex(final ByteBuf buffer) {
        return new JsonPrimitive(ChunkSuites.hex(bytes(buffer)));
    }

    static JsonArray hexes(final List<byte[]> arrays) {
        JsonArray array = new JsonArray();
        arrays.forEach(bytes -> array.add(ChunkSuites.hex(bytes)));
        return array;
    }

    static JsonArray bitSet(final BitSet bits) {
        return ChunkSuites.longs(bits.toLongArray());
    }

    private static final Map<String, Field> FIELDS = new HashMap<>();

    static int shortField(final Object object, final String name) {
        try {
            Field field = FIELDS.computeIfAbsent(name, n -> {
                try {
                    Field f = object.getClass().getDeclaredField(n);
                    f.setAccessible(true);
                    return f;
                } catch (NoSuchFieldException e) {
                    throw new IllegalStateException(e);
                }
            });
            return (Short)field.get(object);
        } catch (IllegalAccessException e) {
            throw new IllegalStateException(e);
        }
    }

    static JsonArray counts(final LevelChunkSection section) {
        return ChunkSuites.array(
            shortField(section, "nonEmptyBlockCount"), shortField(section, "fluidCount"), shortField(section, "tickingBlockCount"),
            shortField(section, "tickingFluidCount")
        );
    }

    /**
     * A section: {@code [counts, statesBits, states, biomesBits, biomes]}, its four counts read by reflection, and its block
     * states (ids) and biomes (ids) in index order as runs.
     */
    static JsonArray section(final LevelChunkSection section) {
        JsonArray states = runsOf(16, (x, y, z) -> new JsonPrimitive(Block.getId(section.getBlockState(x, y, z))));
        JsonArray biomes = runsOf(4, (x, y, z) -> new JsonPrimitive(biomeId(section.getNoiseBiome(x, y, z))));
        return ChunkSuites.array(counts(section), section.getStates().bitsPerEntry(), states, section.getBiomes().bitsPerEntry(), biomes);
    }

    interface Lookup {
        JsonElement at(int x, int y, int z);
    }

    static JsonArray runsOf(final int side, final Lookup lookup) {
        List<JsonElement> values = new ArrayList<>();
        for (int y = 0; y < side; y++) {
            for (int z = 0; z < side; z++) {
                for (int x = 0; x < side; x++) {
                    values.add(lookup.at(x, y, z));
                }
            }
        }
        return ChunkSuites.runs(values.iterator(), Function.identity());
    }

    /** Decodes a buffer with the codec: {@code [fields(packet), unread, written]}, or what it threw. */
    static <B extends ByteBuf, P> void decodeCase(
        final GoldenWriter writer, final String method, final byte[] bytes, final Function<byte[], B> reader, final Supplier<B> output,
        final StreamCodec<B, P> codec, final Function<P, JsonElement> fields
    ) throws IOException {
        JsonArray args = ChunkSuites.array(ChunkSuites.hex(bytes));
        JsonElement result;
        try {
            B input = reader.apply(bytes);
            P packet = codec.decode(input);
            int unread = input.readableBytes();
            JsonElement decoded = fields.apply(packet);
            JsonElement written;
            try {
                B out = output.get();
                codec.encode(out, packet);
                written = hex(out);
            } catch (RuntimeException e) {
                written = ChunkSuites.thrown(e);
            }
            result = ChunkSuites.array(decoded, unread, written);
        } catch (RuntimeException e) {
            writer.threw(method, args, e, null);
            return;
        }
        writer.result(method, args, result, null);
    }

    static byte[] bytes(final ByteBuf buffer) {
        byte[] bytes = new byte[buffer.readableBytes()];
        buffer.getBytes(buffer.readerIndex(), bytes);
        return bytes;
    }

    // A buffer's variants: whole, cut short by a byte, cut anywhere, and with a byte after.
    static List<byte[]> variants(final Random random, final byte[] bytes) {
        List<byte[]> variants = new ArrayList<>();
        variants.add(bytes);
        if (bytes.length > 0) {
            variants.add(Arrays.copyOf(bytes, bytes.length - 1));
            variants.add(Arrays.copyOf(bytes, random.nextInt(bytes.length)));
        }
        byte[] longer = Arrays.copyOf(bytes, bytes.length + 1);
        longer[bytes.length] = 0x2a;
        variants.add(longer);
        return variants;
    }

    static FriendlyByteBuf friendly(final byte[] bytes) {
        return new FriendlyByteBuf(Unpooled.wrappedBuffer(bytes));
    }

    static FriendlyByteBuf friendly() {
        return new FriendlyByteBuf(Unpooled.buffer());
    }

    /** BlockEntityTypes' fields, each {@code [id, key]} in BuiltInRegistries.BLOCK_ENTITY_TYPE, and its size. */
    static final class BlockEntityTypesSuite implements Suite {
        @Override
        public String name() {
            return BlockEntityTypes.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            for (Field field : BlockEntityTypes.class.getFields()) {
                if (Modifier.isStatic(field.getModifiers()) && field.getType() == BlockEntityType.class) {
                    try {
                        BlockEntityType<?> type = (BlockEntityType<?>)field.get(null);
                        writer.field(field.getName(), ChunkSuites.array(
                            BuiltInRegistries.BLOCK_ENTITY_TYPE.getId(type), BuiltInRegistries.BLOCK_ENTITY_TYPE.getKey(type).toString()
                        ));
                    } catch (IllegalAccessException e) {
                        throw new IllegalStateException(e);
                    }
                }
            }
            writer.field("size", new JsonPrimitive(BuiltInRegistries.BLOCK_ENTITY_TYPE.size()));
        }
    }

    /**
     * DataLayer, a section's light: programs on a layer made empty ({@code ["new"]}), filled ({@code ["default", v]}) or
     * from bytes ({@code ["data", hex]}, which throws unless there are 2048), each step {@code [output, isEmpty,
     * isDefinitelyHomogenous]}: get, set (any value, masked to 4 bits), fill, getData (which makes the array), copy,
     * isDefinitelyFilledWith, and toString and layerToString now and then.
     */
    static final class DataLayerSuite implements Suite {
        @Override
        public String name() {
            return DataLayer.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            Random random = new Random(0xda7a);
            writer.field("LAYER_COUNT", new JsonPrimitive(DataLayer.LAYER_COUNT));
            writer.field("LAYER_SIZE", new JsonPrimitive(DataLayer.LAYER_SIZE));
            writer.field("SIZE", new JsonPrimitive(DataLayer.SIZE));
            List<JsonArray> inits = new ArrayList<>(List.of(
                ChunkSuites.array("new"), ChunkSuites.array("default", 0), ChunkSuites.array("default", 15), ChunkSuites.array("default", 7),
                ChunkSuites.array("default", 16), ChunkSuites.array("default", -1), ChunkSuites.array("data", ChunkSuites.hex(new byte[2047])),
                ChunkSuites.array("data", ChunkSuites.hex(new byte[2049])), ChunkSuites.array("data", ""), ChunkSuites.array("data", ChunkSuites.hex(filled(0x11)))
            ));
            // The captured chunks' light, the way the client makes its layers.
            for (byte[] payload : payloads("minecraft:level_chunk_with_light")) {
                ClientboundLevelChunkWithLightPacket packet = ClientboundLevelChunkWithLightPacket.STREAM_CODEC.decode(reader(payload));
                packet.lightData().skyUpdates().stream().limit(1).forEach(bytes -> inits.add(ChunkSuites.array("data", ChunkSuites.hex(bytes))));
                if (inits.size() > 30) {
                    break;
                }
            }
            for (int i = 0; i < 40; i++) {
                byte[] bytes = new byte[2048];
                random.nextBytes(bytes);
                inits.add(ChunkSuites.array("data", ChunkSuites.hex(bytes)));
            }
            int programs = inits.size() + 120;
            for (int i = 0; i < programs; i++) {
                JsonArray init = i < inits.size() ? inits.get(i) : inits.get(random.nextInt(inits.size()));
                JsonArray ops = program(random);
                writer.result("programs", ChunkSuites.array(init, ops), run(init, ops), null);
            }
        }

        private static byte[] filled(final int value) {
            byte[] bytes = new byte[2048];
            Arrays.fill(bytes, (byte)value);
            return bytes;
        }

        private static JsonArray program(final Random random) {
            JsonArray ops = new JsonArray();
            int length = 10 + random.nextInt(120);
            for (int i = 0; i < length; i++) {
                int x = coordinate(random);
                int y = coordinate(random);
                int z = coordinate(random);
                int value = random.nextInt(20) == 0 ? random.nextInt() : random.nextInt(16);
                int kind = random.nextInt(100);
                ops.add(
                    kind < 40 ? ChunkSuites.array("get", x, y, z)
                        : kind < 80 ? ChunkSuites.array("set", x, y, z, value)
                        : kind < 84 ? ChunkSuites.array("fill", value)
                        : kind < 88 ? ChunkSuites.array("getData")
                        : kind < 92 ? ChunkSuites.array("copy")
                        : kind < 97 ? ChunkSuites.array("isDefinitelyFilledWith", value)
                        : kind < 99 ? ChunkSuites.array("layerToString", random.nextInt(16))
                        : ChunkSuites.array("toString")
                );
            }
            ops.add(ChunkSuites.array("getData"));
            return ops;
        }

        // In a section mostly, and past it now and then, where an index lands on another position or past the array.
        private static int coordinate(final Random random) {
            return random.nextInt(1000) == 0 ? new int[]{-1, 16, 17, 255, 256, -16}[random.nextInt(6)] : random.nextInt(16);
        }

        private static JsonArray run(final JsonArray init, final JsonArray ops) {
            JsonArray steps = new JsonArray();
            DataLayer layer;
            try {
                layer = switch (init.get(0).getAsString()) {
                    case "new" -> new DataLayer();
                    case "default" -> new DataLayer(init.get(1).getAsInt());
                    default -> new DataLayer(ChunkSuites.unhex(init.get(1).getAsString()));
                };
            } catch (RuntimeException e) {
                steps.add(ChunkSuites.thrown(e));
                return steps;
            }
            for (JsonElement element : ops) {
                JsonArray op = element.getAsJsonArray();
                try {
                    JsonElement output = switch (op.get(0).getAsString()) {
                        case "get" -> new JsonPrimitive(layer.get(op.get(1).getAsInt(), op.get(2).getAsInt(), op.get(3).getAsInt()));
                        case "set" -> {
                            layer.set(op.get(1).getAsInt(), op.get(2).getAsInt(), op.get(3).getAsInt(), op.get(4).getAsInt());
                            yield JsonNull.INSTANCE;
                        }
                        case "fill" -> {
                            layer.fill(op.get(1).getAsInt());
                            yield JsonNull.INSTANCE;
                        }
                        case "getData" -> new JsonPrimitive(ChunkSuites.hex(layer.getData()));
                        case "copy" -> {
                            layer = layer.copy();
                            yield JsonNull.INSTANCE;
                        }
                        case "isDefinitelyFilledWith" -> new JsonPrimitive(layer.isDefinitelyFilledWith(op.get(1).getAsInt()));
                        case "layerToString" -> new JsonPrimitive(layer.layerToString(op.get(1).getAsInt()));
                        case "toString" -> new JsonPrimitive(layer.toString());
                        default -> throw new IllegalArgumentException(op.toString());
                    };
                    steps.add(ChunkSuites.array(output, layer.isEmpty(), layer.isDefinitelyHomogenous()));
                } catch (RuntimeException e) {
                    steps.add(ChunkSuites.thrown(e));
                    break;
                }
            }
            return steps;
        }
    }

    /**
     * LevelChunkSection: programs on a new section (air, plains), each step {@code [output, counts, hasOnlyAir, hasFluid,
     * isRandomlyTicking, isRandomlyTickingBlocks, isRandomlyTickingFluids]} with its four counts read by reflection. The
     * ops set states from pools of states that count differently, get states and fluid states (encoded like BlockSuites'
     * fluid states), get biomes, recalcBlockCounts, write, read (the captured chunks' sections, and others vanilla wrote,
     * whose counts it takes as they are), readBiomes, getSerializedSize, maybeHas and copy.
     */
    static final class LevelChunkSectionSuite implements Suite {
        @Override
        public String name() {
            return LevelChunkSection.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            Random random = new Random(0x5ec7);
            writer.field("BIOME_CONTAINER_BITS", new JsonPrimitive(LevelChunkSection.BIOME_CONTAINER_BITS));
            writer.field("BIOMES", biomeNames());
            List<BlockState> special = specialStates();
            List<byte[]> sections = new ArrayList<>(capturedSections());
            List<byte[]> biomes = new ArrayList<>();
            for (int i = 0; i < 24; i++) {
                LevelChunkSection section = randomSection(random, special, i);
                FriendlyByteBuf buffer = friendly();
                section.write(buffer);
                sections.add(bytes(buffer));
                FriendlyByteBuf biome = friendly();
                section.getBiomes().write(biome);
                biomes.add(bytes(biome));
            }
            for (int program = 0; program < 160; program++) {
                List<BlockState> pool = pool(random, special, 1 + random.nextInt(program % 4 == 0 ? 300 : 24));
                JsonArray ops = program(random, pool, sections, biomes);
                writer.result("programs", ChunkSuites.array(ops), run(ops), null);
            }
        }

        // States that count differently: air, cave air and void air; ticking blocks; water and lava at each level; and
        // waterlogged blocks, plants that are always water, and kelp, which is water and ticks.
        static List<BlockState> specialStates() {
            List<BlockState> states = new ArrayList<>();
            states.add(Blocks.AIR.defaultBlockState());
            states.add(Blocks.CAVE_AIR.defaultBlockState());
            states.add(Blocks.VOID_AIR.defaultBlockState());
            states.add(Blocks.STONE.defaultBlockState());
            states.add(Blocks.GRASS_BLOCK.defaultBlockState());
            states.add(Blocks.WHEAT.defaultBlockState());
            states.addAll(Blocks.WATER.getStateDefinition().getPossibleStates());
            states.addAll(Blocks.LAVA.getStateDefinition().getPossibleStates());
            states.addAll(Blocks.OAK_STAIRS.getStateDefinition().getPossibleStates().subList(0, 4));
            states.add(Blocks.SEAGRASS.defaultBlockState());
            states.add(Blocks.KELP.defaultBlockState());
            states.add(Blocks.KELP_PLANT.defaultBlockState());
            states.add(Blocks.BUBBLE_COLUMN.defaultBlockState());
            states.addAll(Blocks.OAK_LEAVES.getStateDefinition().getPossibleStates().subList(0, 8));
            return states;
        }

        static List<BlockState> pool(final Random random, final List<BlockState> special, final int size) {
            List<BlockState> pool = new ArrayList<>();
            while (pool.size() < size) {
                BlockState state = random.nextInt(3) == 0
                    ? Block.BLOCK_STATE_REGISTRY.byId(random.nextInt(Block.BLOCK_STATE_REGISTRY.size()))
                    : special.get(random.nextInt(special.size()));
                if (!pool.contains(state)) {
                    pool.add(state);
                }
            }
            return pool;
        }

        @SuppressWarnings("unchecked")
        static LevelChunkSection randomSection(final Random random, final List<BlockState> special, final int index) {
            LevelChunkSection section = new LevelChunkSection(factory());
            List<BlockState> pool = pool(random, special, 1 + random.nextInt(index % 3 == 0 ? 300 : 12));
            int filled = index % 4 == 0 ? 4096 : random.nextInt(4096);
            for (int i = 0; i < filled; i++) {
                section.setBlockState(random.nextInt(16), random.nextInt(16), random.nextInt(16), pool.get(random.nextInt(pool.size())));
            }
            int biomesUsed = 1 + random.nextInt(index % 2 == 0 ? 3 : 20);
            PalettedContainer<Object> biomes = (PalettedContainer<Object>)(PalettedContainer<?>)section.getBiomes().recreate();
            for (int i = 0; i < 64 && biomesUsed > 1; i++) {
                biomes.set(random.nextInt(4), random.nextInt(4), random.nextInt(4), biomes().get(random.nextInt(biomesUsed)));
            }
            FriendlyByteBuf buffer = friendly();
            biomes.write(buffer);
            section.readBiomes(buffer);
            return section;
        }

        // Each captured chunk's sections, cut out of its buffer by reading them, each once.
        static List<byte[]> capturedSections() {
            List<byte[]> sections = new ArrayList<>();
            for (byte[] payload : payloads("minecraft:level_chunk_with_light")) {
                ClientboundLevelChunkWithLightPacket packet = ClientboundLevelChunkWithLightPacket.STREAM_CODEC.decode(reader(payload));
                FriendlyByteBuf buffer = packet.chunkData().getReadBuffer();
                for (int i = 0; i < SECTION_COUNT; i++) {
                    int start = buffer.readerIndex();
                    new LevelChunkSection(factory()).read(buffer);
                    byte[] bytes = new byte[buffer.readerIndex() - start];
                    buffer.getBytes(start, bytes);
                    if (sections.stream().noneMatch(other -> Arrays.equals(other, bytes))) {
                        sections.add(bytes);
                    }
                }
            }
            return sections;
        }

        private static JsonArray program(final Random random, final List<BlockState> pool, final List<byte[]> sections, final List<byte[]> biomes) {
            JsonArray ops = new JsonArray();
            int length = 20 + random.nextInt(400);
            for (int i = 0; i < length; i++) {
                int x = random.nextInt(16);
                int y = random.nextInt(16);
                int z = random.nextInt(16);
                int state = Block.getId(pool.get(random.nextInt(pool.size())));
                int kind = random.nextInt(200);
                ops.add(
                    kind < 110 ? ChunkSuites.array("setBlockState", x, y, z, state)
                        : kind < 140 ? ChunkSuites.array("getBlockState", x, y, z)
                        : kind < 160 ? ChunkSuites.array("getFluidState", x, y, z)
                        : kind < 165 ? ChunkSuites.array("getNoiseBiome", random.nextInt(4), random.nextInt(4), random.nextInt(4))
                        : kind < 170 ? ChunkSuites.array("recalcBlockCounts")
                        : kind < 174 ? ChunkSuites.array("write")
                        : kind < 178 ? ChunkSuites.array("read", ChunkSuites.hex(sections.get(random.nextInt(sections.size()))))
                        : kind < 181 ? ChunkSuites.array("readBiomes", ChunkSuites.hex(biomes.get(random.nextInt(biomes.size()))))
                        : kind < 185 ? ChunkSuites.array("getSerializedSize")
                        : kind < 190 ? ChunkSuites.array("maybeHas", state)
                        : ChunkSuites.array("copy")
                );
            }
            ops.add(ChunkSuites.array("write"));
            return ops;
        }

        static JsonArray run(final JsonArray ops) {
            LevelChunkSection section = new LevelChunkSection(factory());
            JsonArray steps = new JsonArray();
            for (JsonElement element : ops) {
                JsonArray op = element.getAsJsonArray();
                try {
                    JsonElement output = switch (op.get(0).getAsString()) {
                        case "setBlockState" -> new JsonPrimitive(Block.getId(section.setBlockState(
                            op.get(1).getAsInt(), op.get(2).getAsInt(), op.get(3).getAsInt(), Block.BLOCK_STATE_REGISTRY.byIdOrThrow(op.get(4).getAsInt())
                        )));
                        case "getBlockState" -> new JsonPrimitive(Block.getId(section.getBlockState(op.get(1).getAsInt(), op.get(2).getAsInt(), op.get(3).getAsInt())));
                        case "getFluidState" -> BlockSuites.fluidState(section.getFluidState(op.get(1).getAsInt(), op.get(2).getAsInt(), op.get(3).getAsInt()));
                        case "getNoiseBiome" -> new JsonPrimitive(biomeId(section.getNoiseBiome(op.get(1).getAsInt(), op.get(2).getAsInt(), op.get(3).getAsInt())));
                        case "recalcBlockCounts" -> {
                            section.recalcBlockCounts();
                            yield JsonNull.INSTANCE;
                        }
                        case "write" -> {
                            FriendlyByteBuf buffer = friendly();
                            section.write(buffer);
                            yield hex(buffer);
                        }
                        case "read" -> {
                            FriendlyByteBuf buffer = friendly(ChunkSuites.unhex(op.get(1).getAsString()));
                            section.read(buffer);
                            yield new JsonPrimitive(buffer.readableBytes());
                        }
                        case "readBiomes" -> {
                            FriendlyByteBuf buffer = friendly(ChunkSuites.unhex(op.get(1).getAsString()));
                            section.readBiomes(buffer);
                            yield new JsonPrimitive(buffer.readableBytes());
                        }
                        case "getSerializedSize" -> new JsonPrimitive(section.getSerializedSize());
                        case "maybeHas" -> {
                            BlockState state = Block.BLOCK_STATE_REGISTRY.byIdOrThrow(op.get(1).getAsInt());
                            yield new JsonPrimitive(section.maybeHas(candidate -> candidate == state));
                        }
                        case "copy" -> {
                            section = section.copy();
                            yield JsonNull.INSTANCE;
                        }
                        default -> throw new IllegalArgumentException(op.toString());
                    };
                    steps.add(ChunkSuites.array(
                        output, counts(section), section.hasOnlyAir(), section.hasFluid(), section.isRandomlyTicking(), section.isRandomlyTickingBlocks(),
                        section.isRandomlyTickingFluids()
                    ));
                } catch (RuntimeException e) {
                    steps.add(ChunkSuites.thrown(e));
                    break;
                }
            }
            return steps;
        }
    }

    /**
     * ClientboundLevelChunkWithLightPacket and the data it carries. "captured" decodes each payload of the capture, the
     * fields being {@code [x, z, heightmaps, blockEntities, light, sections, unreadSections, sectionsWritten]}: the
     * heightmaps {@code [[type, longs], …]} in the EnumMap's order (the server writes its HashMap's), the block entities
     * {@code [[x, y, z, type, tag], …]} as forEachBlockEntityTag hands them out, the light {@code [skyYMask, blockYMask,
     * emptySkyYMask, emptyBlockYMask, skyUpdates, blockUpdates]}, the level's sections each read from the buffer by a new
     * section (see section()) with recalcBlockCounts' counts after, what's left of the buffer, and every section written
     * back. "generated" decodes what vanilla writes of packets it's made with any heightmaps, sections it wrote (one
     * fewer or more than the level has now and then) and block entities of any type, cut short or with more after, and
     * buffers with ids it refuses.
     */
    static final class ChunkPacketSuite implements Suite {
        @Override
        public String name() {
            return ClientboundLevelChunkWithLightPacket.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            writer.field("BIOMES", biomeNames());
            writer.field("SECTION_COUNT", new JsonPrimitive(SECTION_COUNT));
            for (byte[] payload : payloads("minecraft:level_chunk_with_light")) {
                decode(writer, "captured", payload);
            }
            Random random = new Random(0xc4c4);
            List<BlockState> special = LevelChunkSectionSuite.specialStates();
            for (int i = 0; i < 32; i++) {
                byte[] bytes = generated(random, special, i);
                for (byte[] variant : i % 8 == 1 ? variants(random, bytes) : List.of(bytes)) {
                    decode(writer, "generated", variant);
                }
            }
            for (byte[] bytes : crafted()) {
                decode(writer, "generated", bytes);
            }
        }

        private static void decode(final GoldenWriter writer, final String method, final byte[] bytes) throws IOException {
            decodeCase(writer, method, bytes, LevelChunkSuites::reader, LevelChunkSuites::writer, ClientboundLevelChunkWithLightPacket.STREAM_CODEC, ChunkPacketSuite::fields);
        }

        static JsonElement fields(final ClientboundLevelChunkWithLightPacket packet) {
            ClientboundLevelChunkPacketData data = packet.chunkData();
            JsonArray heightmaps = new JsonArray();
            data.getHeightmaps().forEach((type, longs) -> heightmaps.add(ChunkSuites.array(type.getSerializationKey(), ChunkSuites.longs(longs))));
            JsonArray blockEntities = new JsonArray();
            data.forEachBlockEntityTag(packet.x(), packet.z(), (pos, type, tag) -> blockEntities.add(ChunkSuites.array(
                pos.getX(), pos.getY(), pos.getZ(), BuiltInRegistries.BLOCK_ENTITY_TYPE.getKey(type).toString(), tag == null ? null : NbtValues.encode(tag)
            )));
            JsonArray sections = new JsonArray();
            FriendlyByteBuf buffer = data.getReadBuffer();
            List<LevelChunkSection> read = new ArrayList<>();
            JsonElement unreadSections;
            try {
                for (int i = 0; i < SECTION_COUNT; i++) {
                    LevelChunkSection section = new LevelChunkSection(factory());
                    section.read(buffer);
                    read.add(section);
                    JsonArray state = section(section);
                    section.recalcBlockCounts();
                    state.add(counts(section));
                    sections.add(state);
                }
                unreadSections = new JsonPrimitive(buffer.readableBytes());
            } catch (RuntimeException e) {
                unreadSections = ChunkSuites.thrown(e);
            }
            FriendlyByteBuf written = friendly();
            read.forEach(section -> section.write(written));
            return ChunkSuites.array(packet.x(), packet.z(), heightmaps, blockEntities, light(packet.lightData()), sections, unreadSections, hex(written));
        }

        static JsonArray light(final ClientboundLightUpdatePacketData light) {
            return ChunkSuites.array(
                bitSet(light.skyYMask()), bitSet(light.blockYMask()), bitSet(light.emptySkyYMask()), bitSet(light.emptyBlockYMask()),
                hexes(light.skyUpdates()), hexes(light.blockUpdates())
            );
        }

        // A packet vanilla writes: any heightmaps (the worldgen ones too, which the server never sends), sections it wrote,
        // block entities of any type at any position with tags or none, and light data.
        private static byte[] generated(final Random random, final List<BlockState> special, final int index) {
            Map<Heightmap.Types, long[]> heightmaps = new EnumMap<>(Heightmap.Types.class);
            for (Heightmap.Types type : Heightmap.Types.values()) {
                if (random.nextInt(3) != 0) {
                    heightmaps.put(type, ChunkSuites.randomLongs(random, random.nextInt(4) == 0 ? random.nextInt(40) : 37));
                }
            }
            FriendlyByteBuf sections = friendly();
            int count = index % 7 == 3 ? SECTION_COUNT - 1 : index % 7 == 5 ? SECTION_COUNT + 1 : SECTION_COUNT;
            // Mostly air, like the level's sections above the ground, and a section of many states in one packet of eight
            // (whose global palette's longs would make the file big).
            for (int i = 0; i < count; i++) {
                LevelChunkSection section = random.nextInt(6) == 0 || index % 8 == 0 && i == 0
                    ? LevelChunkSectionSuite.randomSection(random, special, index % 8 == 0 && i == 0 ? 0 : 1 + random.nextInt(2) * 3 + 1)
                    : new LevelChunkSection(factory());
                section.write(sections);
            }
            List<Object> blockEntities = new ArrayList<>();
            for (int i = random.nextInt(index % 3 == 0 ? 12 : 2); i > 0; i--) {
                BlockEntityType<?> type = BuiltInRegistries.BLOCK_ENTITY_TYPE.byId(random.nextInt(BuiltInRegistries.BLOCK_ENTITY_TYPE.size()));
                blockEntities.add(blockEntityInfo((byte)random.nextInt(), (short)(random.nextInt(400) - 64), type, randomTag(random)));
            }
            ClientboundLevelChunkPacketData data = chunkData(heightmaps, bytes(sections), blockEntities);
            ClientboundLevelChunkWithLightPacket packet = new ClientboundLevelChunkWithLightPacket(
                random.nextInt(65) - 32, random.nextInt(65) - 32, data, LightUpdatePacketSuite.randomLight(random)
            );
            RegistryFriendlyByteBuf out = writer();
            ClientboundLevelChunkWithLightPacket.STREAM_CODEC.encode(out, packet);
            return bytes(out);
        }

        static Optional<CompoundTag> randomTag(final Random random) {
            if (random.nextInt(3) == 0) {
                return Optional.empty();
            }
            CompoundTag tag = new CompoundTag();
            tag.putString("id", "minecraft:chest");
            tag.putInt("x", random.nextInt());
            if (random.nextBoolean()) {
                ListTag items = new ListTag();
                for (int i = random.nextInt(4); i > 0; i--) {
                    CompoundTag item = new CompoundTag();
                    item.putString("id", "minecraft:stone");
                    item.put("count", IntTag.valueOf(random.nextInt(64)));
                    items.add(item);
                }
                tag.put("Items", items);
            }
            if (random.nextBoolean()) {
                tag.put("CustomName", StringTag.valueOf("é" + random.nextInt()));
            }
            return Optional.of(tag);
        }

        // Buffers with what the packet's codecs refuse or read oddly: a heightmap of a type id past the types (ZERO's
        // strategy reads it as the first type), a section buffer of 2 MiB and one past it, and block entity type ids at and
        // past the registry's end.
        private static List<byte[]> crafted() {
            List<byte[]> buffers = new ArrayList<>();
            for (int typeId : new int[]{0, 6, -1, 100}) {
                FriendlyByteBuf out = friendly();
                out.writeInt(1);
                out.writeInt(-2);
                out.writeVarInt(1);
                out.writeVarInt(typeId);
                out.writeLongArray(new long[]{1L, 2L});
                out.writeVarInt(0);
                out.writeVarInt(0);
                ClientboundLightUpdatePacketData.STREAM_CODEC.encode(out, LightUpdatePacketSuite.emptyLight());
                buffers.add(bytes(out));
            }
            for (int length : new int[]{2097152, 2097153, -1}) {
                FriendlyByteBuf out = friendly();
                out.writeInt(0);
                out.writeInt(0);
                out.writeVarInt(0);
                out.writeVarInt(length);
                buffers.add(bytes(out));
            }
            int types = BuiltInRegistries.BLOCK_ENTITY_TYPE.size();
            for (int typeId : new int[]{0, types - 1, types, -1}) {
                FriendlyByteBuf out = friendly();
                out.writeInt(0);
                out.writeInt(0);
                out.writeVarInt(0);
                out.writeVarInt(0);
                out.writeVarInt(1);
                out.writeByte(0x3a);
                out.writeShort(-60);
                out.writeVarInt(typeId);
                out.writeByte(0);
                ClientboundLightUpdatePacketData.STREAM_CODEC.encode(out, LightUpdatePacketSuite.emptyLight());
                buffers.add(bytes(out));
            }
            return buffers;
        }

        static ClientboundLevelChunkPacketData chunkData(final Map<Heightmap.Types, long[]> heightmaps, final byte[] buffer, final List<Object> blockEntities) {
            try {
                Constructor<ClientboundLevelChunkPacketData> constructor = ClientboundLevelChunkPacketData.class.getDeclaredConstructor(Map.class, byte[].class, List.class);
                constructor.setAccessible(true);
                return constructor.newInstance(heightmaps, buffer, blockEntities);
            } catch (ReflectiveOperationException e) {
                throw new IllegalStateException(e);
            }
        }

        static Object blockEntityInfo(final byte packedXZ, final short y, final BlockEntityType<?> type, final Optional<CompoundTag> tag) {
            try {
                Class<?> info = Class.forName(ClientboundLevelChunkPacketData.class.getName() + "$BlockEntityInfo");
                Constructor<?> constructor = info.getDeclaredConstructor(byte.class, short.class, BlockEntityType.class, Optional.class);
                constructor.setAccessible(true);
                return constructor.newInstance(packedXZ, y, type, tag);
            } catch (ReflectiveOperationException e) {
                throw new IllegalStateException(e);
            }
        }
    }

    /**
     * ClientboundLightUpdatePacket: decodes what vanilla writes of packets with any masks and layers (2048 bytes mostly,
     * and up to 2048), the captured chunks' light behind their positions, and buffers with a layer of 2049 bytes or
     * negative, and counts negative or past what's there: {@code [[x, z, light], unread, written]}.
     */
    static final class LightUpdatePacketSuite implements Suite {
        @Override
        public String name() {
            return ClientboundLightUpdatePacket.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            Random random = new Random(0x1197);
            List<byte[]> buffers = new ArrayList<>();
            for (byte[] payload : payloads("minecraft:level_chunk_with_light")) {
                ClientboundLevelChunkWithLightPacket chunk = ClientboundLevelChunkWithLightPacket.STREAM_CODEC.decode(reader(payload));
                buffers.add(encode(new ClientboundLightUpdatePacket(chunk.x(), chunk.z(), chunk.lightData())));
                if (buffers.size() >= 12) {
                    break;
                }
            }
            for (int i = 0; i < 60; i++) {
                byte[] bytes = encode(new ClientboundLightUpdatePacket(random.nextInt(), random.nextInt(65) - 32, randomLight(random)));
                buffers.addAll(i % 5 == 0 ? variants(random, bytes) : List.of(bytes));
            }
            for (int length : new int[]{2048, 2049, 0, -1}) {
                ByteBuf out = Unpooled.buffer();
                VarInt.write(out, 3);
                VarInt.write(out, -4);
                for (int mask = 0; mask < 4; mask++) {
                    FriendlyByteBuf.writeByteArray(out, new byte[]{1});
                }
                VarInt.write(out, 1);
                VarInt.write(out, length);
                out.writeBytes(new byte[Math.max(0, length)]);
                VarInt.write(out, 0);
                buffers.add(bytes(out));
            }
            for (int count : new int[]{-1, 65536, 3}) {
                ByteBuf out = Unpooled.buffer();
                VarInt.write(out, 0);
                VarInt.write(out, 0);
                for (int mask = 0; mask < 4; mask++) {
                    FriendlyByteBuf.writeByteArray(out, new byte[0]);
                }
                VarInt.write(out, count);
                buffers.add(bytes(out));
            }
            for (byte[] bytes : buffers) {
                decodeCase(writer, "STREAM_CODEC.decode", bytes, b -> Unpooled.wrappedBuffer(b), Unpooled::buffer, ClientboundLightUpdatePacket.STREAM_CODEC,
                    packet -> ChunkSuites.array(packet.x(), packet.z(), ChunkPacketSuite.light(packet.lightData())));
            }
        }

        private static byte[] encode(final ClientboundLightUpdatePacket packet) {
            ByteBuf out = Unpooled.buffer();
            ClientboundLightUpdatePacket.STREAM_CODEC.encode(out, packet);
            return bytes(out);
        }

        static ClientboundLightUpdatePacketData emptyLight() {
            return new ClientboundLightUpdatePacketData(new BitSet(), new BitSet(), new BitSet(), new BitSet(), List.of(), List.of());
        }

        // Masks of the overworld's 26 light sections (or any bits now and then, or none), and as many layers as the masks
        // say (up to 4, which keeps the files small), or a few.
        static ClientboundLightUpdatePacketData randomLight(final Random random) {
            BitSet[] masks = new BitSet[4];
            for (int i = 0; i < 4; i++) {
                long word = random.nextInt(4) == 0 ? random.nextLong() : random.nextLong() & random.nextLong() & ((1L << 26) - 1);
                masks[i] = BitSet.valueOf(new long[]{random.nextInt(6) == 0 ? 0L : word});
            }
            List<byte[]> sky = layers(random, random.nextInt(4) == 0 ? random.nextInt(5) : masks[0].cardinality());
            List<byte[]> block = layers(random, random.nextInt(4) == 0 ? random.nextInt(5) : masks[1].cardinality());
            return new ClientboundLightUpdatePacketData(masks[0], masks[1], masks[2], masks[3], sky, block);
        }

        private static List<byte[]> layers(final Random random, final int count) {
            List<byte[]> layers = new ArrayList<>();
            for (int i = 0; i < Math.min(count, 4); i++) {
                byte[] layer = new byte[random.nextInt(10) == 0 ? random.nextInt(2049) : 2048];
                if (random.nextBoolean()) {
                    random.nextBytes(layer);
                } else {
                    Arrays.fill(layer, (byte)random.nextInt(256));
                }
                layers.add(layer);
            }
            return layers;
        }
    }

    /** ClientboundForgetLevelChunkPacket: decodes chunk positions (any longs), cut short or with more after. */
    static final class ForgetLevelChunkPacketSuite implements Suite {
        @Override
        public String name() {
            return ClientboundForgetLevelChunkPacket.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            Random random = new Random(0xf06e);
            List<Long> longs = new ArrayList<>(List.of(0L, -1L, ChunkPos.pack(1, -1), ChunkPos.pack(-3, 2), Long.MIN_VALUE, Long.MAX_VALUE, ChunkPos.INVALID_CHUNK_POS));
            for (int i = 0; i < 100; i++) {
                longs.add(random.nextBoolean() ? random.nextLong() : ChunkPos.pack(random.nextInt(65) - 32, random.nextInt(65) - 32));
            }
            for (long packed : longs) {
                ByteBuf out = Unpooled.buffer();
                out.writeLong(packed);
                for (byte[] bytes : variants(random, bytes(out))) {
                    decodeCase(writer, "STREAM_CODEC.decode", bytes, LevelChunkSuites::friendly, LevelChunkSuites::friendly,
                        ClientboundForgetLevelChunkPacket.STREAM_CODEC, packet -> ChunkSuites.array(packet.pos().x(), packet.pos().z()));
                }
            }
        }
    }

    /**
     * ClientboundBlockUpdatePacket: decodes positions (blocks around the walk's spawn, at the packing's limits, any longs)
     * and state ids (every state's edges, the registry's size, -1, any), cut short or with more after: {@code [[x, y, z,
     * state], unread, written]}.
     */
    static final class BlockUpdatePacketSuite implements Suite {
        @Override
        public String name() {
            return ClientboundBlockUpdatePacket.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            Random random = new Random(0xb10c);
            int size = Block.BLOCK_STATE_REGISTRY.size();
            List<Integer> ids = new ArrayList<>(List.of(0, 1, size - 1, size, -1, Integer.MAX_VALUE, Integer.MIN_VALUE));
            for (int i = 0; i < 150; i++) {
                ids.add(random.nextInt(size));
            }
            for (int i = 0; i < ids.size(); i++) {
                long pos = switch (i % 4) {
                    case 0 -> new BlockPos(random.nextInt(33) - 16, -64 + random.nextInt(8), random.nextInt(33) - 16).asLong();
                    case 1 -> new BlockPos(33554431 * (random.nextBoolean() ? 1 : -1), 2047, -33554432).asLong();
                    default -> random.nextLong();
                };
                RegistryFriendlyByteBuf out = writer();
                out.writeLong(pos);
                out.writeVarInt(ids.get(i));
                for (byte[] bytes : i % 6 == 0 ? variants(random, bytes(out)) : List.of(bytes(out))) {
                    decodeCase(writer, "STREAM_CODEC.decode", bytes, LevelChunkSuites::reader, LevelChunkSuites::writer, ClientboundBlockUpdatePacket.STREAM_CODEC,
                        packet -> ChunkSuites.array(packet.getPos().getX(), packet.getPos().getY(), packet.getPos().getZ(), Block.getId(packet.getBlockState())));
                }
            }
        }
    }

    /**
     * ClientboundSectionBlocksUpdatePacket: decodes section positions, counts (0 to 4096, and negative ones, which throw)
     * and changes (positions with bits past 12, which the packet drops, and state ids, -1 and past the registry, which
     * it reads as null and writes as 0), cut short or with more after: {@code [[x, y, z, [[x, y, z, state], …]], unread,
     * written]}, the changes as runUpdates hands them out.
     */
    static final class SectionBlocksUpdatePacketSuite implements Suite {
        @Override
        public String name() {
            return ClientboundSectionBlocksUpdatePacket.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            Random random = new Random(0x5bc0);
            int size = Block.BLOCK_STATE_REGISTRY.size();
            for (int i = 0; i < 120; i++) {
                FriendlyByteBuf out = friendly();
                long section = i % 3 == 0 ? random.nextLong() : SectionPos.asLong(random.nextInt(9) - 4, random.nextInt(24) - 4, random.nextInt(9) - 4);
                out.writeLong(section);
                int count = i < 4 ? new int[]{0, -1, 4096, 1}[i] : random.nextInt(4) == 0 ? random.nextInt(4097) : random.nextInt(20);
                out.writeVarInt(count);
                for (int change = 0; change < count; change++) {
                    long position = random.nextInt(10) == 0 ? random.nextInt(1 << 16) : random.nextInt(4096);
                    long state = switch (random.nextInt(12)) {
                        case 0 -> size;
                        case 1 -> -1L;
                        case 2 -> random.nextLong() >>> 12;
                        default -> random.nextInt(size);
                    };
                    out.writeVarLong(state << 12 | position);
                }
                for (byte[] bytes : i % 5 == 0 ? variants(random, bytes(out)) : List.of(bytes(out))) {
                    decodeCase(writer, "STREAM_CODEC.decode", bytes, LevelChunkSuites::friendly, LevelChunkSuites::friendly,
                        ClientboundSectionBlocksUpdatePacket.STREAM_CODEC, SectionBlocksUpdatePacketSuite::fields);
                }
            }
        }

        private static JsonElement fields(final ClientboundSectionBlocksUpdatePacket packet) {
            JsonArray changes = new JsonArray();
            packet.runUpdates((pos, state) -> changes.add(ChunkSuites.array(pos.getX(), pos.getY(), pos.getZ(), state == null ? null : Block.getId(state))));
            try {
                Field sectionPos = ClientboundSectionBlocksUpdatePacket.class.getDeclaredField("sectionPos");
                sectionPos.setAccessible(true);
                SectionPos pos = (SectionPos)sectionPos.get(packet);
                return ChunkSuites.array(pos.x(), pos.y(), pos.z(), changes);
            } catch (ReflectiveOperationException e) {
                throw new IllegalStateException(e);
            }
        }
    }

    /**
     * ClientboundChunksBiomesPacket: decodes what vanilla writes of lists of chunks with their biome buffers (written by
     * biome containers, or two bytes), cut short or with more after, and byte arrays of 2 MiB and past it:
     * {@code [[[x, z, hex], …], unread, written]}.
     */
    static final class ChunksBiomesPacketSuite implements Suite {
        @Override
        public String name() {
            return ClientboundChunksBiomesPacket.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            Random random = new Random(0xb10e);
            List<BlockState> special = LevelChunkSectionSuite.specialStates();
            List<byte[]> buffers = new ArrayList<>();
            for (int i = 0; i < 40; i++) {
                List<ClientboundChunksBiomesPacket.ChunkBiomeData> chunks = new ArrayList<>();
                for (int chunk = random.nextInt(4); chunk > 0; chunk--) {
                    FriendlyByteBuf biomes = friendly();
                    for (int section = 0; section < SECTION_COUNT; section++) {
                        LevelChunkSectionSuite.randomSection(random, special, section).getBiomes().write(biomes);
                    }
                    byte[] bytes = random.nextInt(5) == 0 ? ChunkSuites.unhex("0001") : bytes(biomes);
                    chunks.add(new ClientboundChunksBiomesPacket.ChunkBiomeData(new ChunkPos(random.nextInt(65) - 32, random.nextInt(65) - 32), bytes));
                }
                ByteBuf out = Unpooled.buffer();
                ClientboundChunksBiomesPacket.STREAM_CODEC.encode(out, new ClientboundChunksBiomesPacket(chunks));
                buffers.addAll(i % 4 == 0 ? variants(random, bytes(out)) : List.of(bytes(out)));
            }
            for (int length : new int[]{2097152, 2097153}) {
                ByteBuf out = Unpooled.buffer();
                VarInt.write(out, 1);
                out.writeLong(0L);
                VarInt.write(out, length);
                buffers.add(bytes(out));
            }
            for (byte[] bytes : buffers) {
                decodeCase(writer, "STREAM_CODEC.decode", bytes, b -> Unpooled.wrappedBuffer(b), Unpooled::buffer, ClientboundChunksBiomesPacket.STREAM_CODEC, packet -> {
                    JsonArray chunks = new JsonArray();
                    packet.chunkBiomeData().forEach(data -> chunks.add(ChunkSuites.array(data.pos().x(), data.pos().z(), ChunkSuites.hex(data.buffer()))));
                    return chunks;
                });
            }
        }
    }
}
