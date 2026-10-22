package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonNull;
import com.google.gson.JsonObject;
import com.google.gson.JsonPrimitive;
import io.netty.buffer.Unpooled;
import java.io.ByteArrayOutputStream;
import java.io.IOException;
import java.lang.reflect.Method;
import java.util.ArrayList;
import java.util.HexFormat;
import java.util.Iterator;
import java.util.List;
import java.util.Map;
import java.util.Random;
import java.util.TreeMap;
import java.util.function.Supplier;
import net.minecraft.core.IdMap;
import net.minecraft.core.IdMapper;
import net.minecraft.network.FriendlyByteBuf;
import net.minecraft.util.BitStorage;
import net.minecraft.util.CrudeIncrementalIntIdentityHashBiMap;
import net.minecraft.util.SimpleBitStorage;
import net.minecraft.util.ZeroBitStorage;
import net.minecraft.world.level.block.Block;
import net.minecraft.world.level.block.state.BlockState;
import net.minecraft.world.level.chunk.Configuration;
import net.minecraft.world.level.chunk.PalettedContainer;
import net.minecraft.world.level.chunk.Strategy;

/**
 * Chunk data: the bit storages and palettes a chunk section keeps its blocks and biomes in. See TESTING.md ("Chunk
 * data") for the encodings. Programs run ops on one object and record each op's output; an op that throws ends its
 * program, recorded as {@code {"throws": class}}.
 */
final class ChunkSuites {
    private static final HexFormat HEX = HexFormat.of();

    private ChunkSuites() {
    }

    static List<Suite> suites() {
        return List.of(new SimpleBitStorageSuite(), new ZeroBitStorageSuite(), new BiMapSuite(), new PalettedContainerSuite());
    }

    static JsonArray array(final Object... elements) {
        JsonArray array = new JsonArray();
        for (Object element : elements) {
            array.add(switch (element) {
                case null -> JsonNull.INSTANCE;
                case JsonElement json -> json;
                case Boolean b -> new JsonPrimitive(b);
                case Number n -> new JsonPrimitive(n);
                case String s -> new JsonPrimitive(s);
                default -> throw new IllegalArgumentException("can't encode " + element);
            });
        }
        return array;
    }

    static JsonArray longs(final long[] values) {
        JsonArray array = new JsonArray();
        for (long value : values) {
            array.add(value);
        }
        return array;
    }

    static JsonArray ints(final int[] values) {
        JsonArray array = new JsonArray();
        for (int value : values) {
            array.add(value);
        }
        return array;
    }

    static JsonObject thrown(final Throwable thrown) {
        JsonObject object = new JsonObject();
        object.addProperty("throws", thrown.getClass().getName());
        return object;
    }

    static String hex(final byte[] bytes) {
        return HEX.formatHex(bytes);
    }

    static byte[] unhex(final String hex) {
        return HEX.parseHex(hex);
    }

    /** What the buffer holds from its reader index to its writer index. */
    static byte[] readable(final FriendlyByteBuf buffer) {
        byte[] bytes = new byte[buffer.readableBytes()];
        buffer.getBytes(buffer.readerIndex(), bytes);
        return bytes;
    }

    static long[] randomLongs(final Random random, final int length) {
        long[] values = new long[Math.max(0, length)];
        for (int i = 0; i < values.length; i++) {
            values[i] = random.nextLong();
        }
        return values;
    }

    static int valuesPerLong(final int bits) {
        return 64 / bits;
    }

    static int requiredLength(final int bits, final int size) {
        int valuesPerLong = valuesPerLong(bits);
        return (size + valuesPerLong - 1) / valuesPerLong;
    }

    /**
     * SimpleBitStorage, the packed array behind every palette with entries: its constructors (from a size, from packed
     * longs and from values), every {@code get} of storages read from random longs (which runs its divide-by-magic-number
     * cell index for every index), {@code unpack}, {@code getAll} and {@code fill}, and programs of {@code get},
     * {@code set}, {@code getAndSet}, {@code fill} and {@code copy}, for every bits per entry from 1 to 32.
     */
    static final class SimpleBitStorageSuite implements Suite {
        @Override
        public String name() {
            return SimpleBitStorage.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            Random random = new Random(0xb175);
            // Bits it refuses.
            for (int bits : new int[]{Integer.MIN_VALUE, -1, 0, 33, 64}) {
                construct(writer, bits, 4096);
            }

            for (int bitCount = 1; bitCount <= 32; bitCount++) {
                final int bits = bitCount;
                int valuesPerLong = valuesPerLong(bits);
                List<Integer> sizes = new ArrayList<>(List.of(4096, 64, 0, 1, valuesPerLong - 1, valuesPerLong, valuesPerLong + 1, 3 * valuesPerLong + 1));
                sizes.add(1 + random.nextInt(300));
                if (bits % 4 == 1) {
                    sizes.add(4095);
                    sizes.add(4097);
                }
                for (int size : sizes) {
                    construct(writer, bits, size);
                    int required = requiredLength(bits, size);

                    // Every index read from random longs, and longs of the wrong length.
                    long[] data = randomLongs(random, required);
                    JsonArray args = array(bits, size, longs(data));
                    writer.result("<init>(II[J)V", args, ints(readAll(new SimpleBitStorage(bits, size, data.clone()))), null);
                    for (int length : new int[]{required - 1, required + 1}) {
                        if (length >= 0) {
                            long[] wrong = randomLongs(random, length);
                            call(writer, "<init>(II[J)V", array(bits, size, longs(wrong)), () -> ints(readAll(new SimpleBitStorage(bits, size, wrong))));
                        }
                    }

                    // Packed from values: any int now and then, which the mask cuts down, and too few values.
                    int[] values = randomValues(random, bits, size);
                    writer.result("<init>(II[I)V", array(bits, size, ints(values)), longs(new SimpleBitStorage(bits, size, values).getRaw()), null);
                    if (size > 0 && size <= 300) {
                        int[] tooFew = java.util.Arrays.copyOf(values, size - 1);
                        call(writer, "<init>(II[I)V", array(bits, size, ints(tooFew)), () -> longs(new SimpleBitStorage(bits, size, tooFew).getRaw()));
                    }

                    if (size != 4096 && size != 4095 && size != 4097) {
                        long[] unpacked = randomLongs(random, required);
                        writer.result("unpack([I)V", array(bits, size, longs(unpacked)), ints(unpack(new SimpleBitStorage(bits, size, unpacked.clone()))), null);
                        long[] all = randomLongs(random, required);
                        writer.result("getAll(Ljava/util/function/IntConsumer;)V", array(bits, size, longs(all)), ints(getAll(new SimpleBitStorage(bits, size, all.clone()))), null);
                    }
                }

                // fill, which also writes the bits past the last whole value of each long.
                long mask = (1L << bits) - 1L;
                for (long value : new long[]{0, 1, mask, mask >> 1, random.nextLong() & mask, -1, mask + 1, Integer.MIN_VALUE}) {
                    int size = random.nextBoolean() ? 64 : 1 + random.nextInt(200);
                    long[] data = randomLongs(random, requiredLength(bits, size));
                    JsonArray args = array(bits, size, longs(data), (int)value);
                    call(writer, "fill(I)V", args, () -> {
                        SimpleBitStorage storage = new SimpleBitStorage(bits, size, data.clone());
                        storage.fill((int)value);
                        return longs(storage.getRaw());
                    });
                }

                for (int program = 0; program < 6; program++) {
                    int size = program < 1 ? 4096 : 1 + random.nextInt(300);
                    long[] data = randomLongs(random, requiredLength(bits, size));
                    JsonArray ops = randomProgram(random, bits, size);
                    writer.result("programs", array(bits, size, longs(data), ops), run(new SimpleBitStorage(bits, size, data.clone()), ops), null);
                }
            }
        }

        private static void construct(final GoldenWriter writer, final int bits, final int size) throws IOException {
            call(writer, "<init>(II)V", array(bits, size), () -> {
                SimpleBitStorage storage = new SimpleBitStorage(bits, size);
                return array(storage.getBits(), storage.getSize(), longs(storage.getRaw()));
            });
        }

        private static int[] randomValues(final Random random, final int bits, final int size) {
            long mask = (1L << bits) - 1L;
            int[] values = new int[size];
            for (int i = 0; i < size; i++) {
                values[i] = random.nextInt(8) == 0 ? random.nextInt() : (int)(random.nextLong() & mask);
            }
            return values;
        }

        static int[] readAll(final BitStorage storage) {
            int[] values = new int[storage.getSize()];
            for (int i = 0; i < values.length; i++) {
                values[i] = storage.get(i);
            }
            return values;
        }

        static int[] unpack(final BitStorage storage) {
            int[] output = new int[storage.getSize()];
            storage.unpack(output);
            return output;
        }

        static int[] getAll(final BitStorage storage) {
            List<Integer> values = new ArrayList<>();
            storage.getAll(values::add);
            return values.stream().mapToInt(Integer::intValue).toArray();
        }

        // Indexes and values mostly in range, and out of it now and then (each such op ends the program).
        private static JsonArray randomProgram(final Random random, final int bits, final int size) {
            long mask = (1L << bits) - 1L;
            JsonArray ops = new JsonArray();
            int length = 20 + random.nextInt(150);
            for (int i = 0; i < length; i++) {
                int index = random.nextInt(60) == 0 ? new int[]{-1, size, Integer.MAX_VALUE, Integer.MIN_VALUE}[random.nextInt(4)] : random.nextInt(Math.max(1, size));
                int value = random.nextInt(60) == 0 ? (int)new long[]{-1, mask + 1, Integer.MIN_VALUE}[random.nextInt(3)] : (int)(random.nextLong() & mask);
                ops.add(switch (random.nextInt(20)) {
                    case 0, 1, 2, 3, 4, 5, 6 -> array("get", index);
                    case 7, 8, 9, 10, 11, 12 -> array("set", index, value);
                    case 13, 14, 15, 16, 17 -> array("getAndSet", index, value);
                    case 18 -> array("fill", value);
                    default -> array("copy");
                });
            }
            return ops;
        }

        // [outputs, raw]: each op's output (a throw ends them), and the storage's longs after.
        static JsonArray run(final BitStorage initial, final JsonArray ops) {
            BitStorage storage = initial;
            JsonArray outputs = new JsonArray();
            for (JsonElement element : ops) {
                JsonArray op = element.getAsJsonArray();
                try {
                    String name = op.get(0).getAsString();
                    outputs.add(switch (name) {
                        case "get" -> new JsonPrimitive(storage.get(op.get(1).getAsInt()));
                        case "set" -> {
                            storage.set(op.get(1).getAsInt(), op.get(2).getAsInt());
                            yield JsonNull.INSTANCE;
                        }
                        case "getAndSet" -> new JsonPrimitive(storage.getAndSet(op.get(1).getAsInt(), op.get(2).getAsInt()));
                        case "fill" -> {
                            storage.fill(op.get(1).getAsInt());
                            yield JsonNull.INSTANCE;
                        }
                        case "copy" -> {
                            storage = storage.copy();
                            yield JsonNull.INSTANCE;
                        }
                        default -> throw new IllegalArgumentException(name);
                    });
                } catch (RuntimeException e) {
                    outputs.add(thrown(e));
                    break;
                }
            }
            return array(outputs, longs(storage.getRaw()));
        }
    }

    /** ZeroBitStorage, the storage of a palette with a single value: every method, at sizes around a section's. */
    static final class ZeroBitStorageSuite implements Suite {
        @Override
        public String name() {
            return ZeroBitStorage.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            writer.field("RAW", longs(ZeroBitStorage.RAW));
            int[] interesting = {0, 1, -1, 63, 64, 4095, 4096, Integer.MIN_VALUE, Integer.MAX_VALUE};
            for (int size : new int[]{0, 1, 64, 4096}) {
                ZeroBitStorage storage = new ZeroBitStorage(size);
                JsonArray receiver = array(size);
                writer.result("getSize()I", receiver, new JsonPrimitive(storage.getSize()), null);
                writer.result("getBits()I", receiver, new JsonPrimitive(storage.getBits()), null);
                writer.result("getRaw()[J", receiver, longs(storage.getRaw()), null);
                writer.result("getAll(Ljava/util/function/IntConsumer;)V", receiver, ints(SimpleBitStorageSuite.getAll(storage)), null);
                BitStorage copy = storage.copy();
                writer.result("copy()Lnet/minecraft/util/BitStorage;", receiver, array(copy.getSize(), copy.getBits()), null);
                // unpack fills the first size entries of a longer array with zeros.
                call(writer, "unpack([I)V", receiver, () -> {
                    int[] output = new int[size + 2];
                    java.util.Arrays.fill(output, 7);
                    storage.unpack(output);
                    return ints(output);
                });
                for (int value : interesting) {
                    call(writer, "fill(I)V", array(size, value), () -> {
                        storage.fill(value);
                        return JsonNull.INSTANCE;
                    });
                    for (int index : interesting) {
                        call(writer, "set(II)V", array(size, index, value), () -> {
                            storage.set(index, value);
                            return JsonNull.INSTANCE;
                        });
                        call(writer, "getAndSet(II)I", array(size, index, value), () -> new JsonPrimitive(storage.getAndSet(index, value)));
                    }
                }
                for (int index : interesting) {
                    call(writer, "get(I)I", array(size, index), () -> new JsonPrimitive(storage.get(index)));
                }
            }
        }
    }

    /**
     * CrudeIncrementalIntIdentityHashBiMap, the two-way map a hash map palette numbers its values with. Programs run on a
     * map made with {@code create(args[0])}: {@code add} (of keys it doesn't have, as a palette adds them), {@code addMapping}
     * with ids around its capacity (one past what a grow makes throws, after the key is in), the lookups both ways,
     * {@code contains} of keys and of ids, {@code size}, {@code clear}, {@code copy} and iterating, each step
     * {@code [output, size]}. Keys are block states, by id. Where a key lands in its table goes by its identity hash,
     * which changes from run to run, so no output may depend on it: a key is never added twice, since which of two
     * copies {@code getId} finds first changes once the table grows.
     */
    static final class BiMapSuite implements Suite {
        @Override
        public String name() {
            return CrudeIncrementalIntIdentityHashBiMap.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            Random random = new Random(0xb1a9);
            int[] capacities = {0, 1, 2, 3, 4, 16, 32, 64, 128, 256};
            for (int program = 0; program < 160; program++) {
                int capacity = program < capacities.length ? capacities[program] : capacities[random.nextInt(capacities.length)];
                JsonArray ops = randomProgram(random, capacity);
                writer.result("programs", array(capacity, ops), run(CrudeIncrementalIntIdentityHashBiMap.create(capacity), ops), null);
            }
        }

        // A shadow map runs the ops as they're made, so addMapping only gets ids no key has (or ones that throw): a key
        // mapped over another's id leaves both in the table, and which one byId keeps once the table grows goes by
        // where their identity hashes put them.
        private static JsonArray randomProgram(final Random random, final int capacity) {
            CrudeIncrementalIntIdentityHashBiMap<BlockState> shadow = CrudeIncrementalIntIdentityHashBiMap.create(capacity);
            int registrySize = Block.BLOCK_STATE_REGISTRY.size();
            List<Integer> keys = new ArrayList<>();
            for (int i = 0; i < 40 + random.nextInt(400); i++) {
                keys.add(random.nextInt(registrySize));
            }
            List<Integer> added = new ArrayList<>();
            JsonArray ops = new JsonArray();
            int length = 20 + random.nextInt(400);
            for (int i = 0; i < length; i++) {
                int kind = random.nextInt(24);
                Integer fresh = keys.stream().filter(key -> !added.contains(key)).findFirst().orElse(null);
                Integer known = added.isEmpty() || random.nextInt(4) == 0 ? keys.get(random.nextInt(keys.size())) : added.get(random.nextInt(added.size()));
                int id = random.nextInt(8) == 0 ? new int[]{-1, capacity, 2 * capacity, (int)(capacity / 0.8F), 5 * capacity + 3}[random.nextInt(5)] : random.nextInt(Math.max(1, added.size() + 4));
                if (kind < 10 && fresh != null) {
                    added.add(fresh);
                    ops.add(array("add", fresh));
                } else if (kind == 10 && fresh != null && capacity > 0 && (id < 0 || shadow.byId(id) == null)) {
                    // An empty table doubles to an empty one forever when it has to hold an id past 0.
                    added.add(fresh);
                    ops.add(array("addMapping", fresh, id));
                } else {
                    ops.add(switch (kind % 8) {
                        case 0, 1 -> array("getId", known);
                        case 2 -> array("byId", id);
                        case 3 -> array("containsKey", known);
                        case 4 -> array("containsId", id);
                        case 5 -> array(random.nextInt(8) == 0 ? "clear" : "size");
                        case 6 -> array("copy");
                        default -> array("iterator");
                    });
                    if (ops.get(ops.size() - 1).getAsJsonArray().get(0).getAsString().equals("clear")) {
                        added.clear();
                    }
                }
                JsonArray single = new JsonArray();
                single.add(ops.get(ops.size() - 1));
                JsonArray steps = run(shadow, single);
                if (steps.get(0).isJsonObject()) {
                    // It threw, which ends the program.
                    break;
                }
            }
            return ops;
        }

        private static JsonArray run(final CrudeIncrementalIntIdentityHashBiMap<BlockState> initial, final JsonArray ops) {
            CrudeIncrementalIntIdentityHashBiMap<BlockState> map = initial;
            JsonArray steps = new JsonArray();
            for (JsonElement element : ops) {
                JsonArray op = element.getAsJsonArray();
                try {
                    String name = op.get(0).getAsString();
                    JsonElement output = switch (name) {
                        case "add" -> new JsonPrimitive(map.add(state(op.get(1))));
                        case "addMapping" -> {
                            map.addMapping(state(op.get(1)), op.get(2).getAsInt());
                            yield JsonNull.INSTANCE;
                        }
                        case "getId" -> new JsonPrimitive(map.getId(state(op.get(1))));
                        case "byId" -> stateId(map.byId(op.get(1).getAsInt()));
                        case "containsKey" -> new JsonPrimitive(map.contains(state(op.get(1))));
                        case "containsId" -> new JsonPrimitive(map.contains(op.get(1).getAsInt()));
                        case "size" -> new JsonPrimitive(map.size());
                        case "clear" -> {
                            map.clear();
                            yield JsonNull.INSTANCE;
                        }
                        case "copy" -> {
                            map = map.copy();
                            yield JsonNull.INSTANCE;
                        }
                        case "iterator" -> {
                            JsonArray keys = new JsonArray();
                            map.iterator().forEachRemaining(key -> keys.add(stateId(key)));
                            yield keys;
                        }
                        default -> throw new IllegalArgumentException(name);
                    };
                    steps.add(array(output, map.size()));
                } catch (RuntimeException e) {
                    steps.add(thrown(e));
                    break;
                }
            }
            return steps;
        }

        private static BlockState state(final JsonElement id) {
            return Block.BLOCK_STATE_REGISTRY.byIdOrThrow(id.getAsInt());
        }

        private static JsonElement stateId(final BlockState state) {
            return state == null ? JsonNull.INSTANCE : new JsonPrimitive(Block.BLOCK_STATE_REGISTRY.getId(state));
        }
    }

    /**
     * A strategy and the values it numbers: the block states' over Block.BLOCK_STATE_REGISTRY, or the biomes' over a
     * registry of {@code size} stand-ins (the client's comes from the server's registry data), named {@code biomes:size}.
     */
    record Registry(String name, Strategy<Object> strategy, IdMap<Object> map, int size) {
        @SuppressWarnings({"unchecked", "rawtypes"})
        static Registry blocks() {
            IdMap<Object> map = (IdMap)Block.BLOCK_STATE_REGISTRY;
            return new Registry("blocks", Strategy.createForBlockStates(map), map, map.size());
        }

        static Registry biomes(final int size) {
            IdMapper<Object> map = new IdMapper<>();
            for (int i = 0; i < size; i++) {
                map.add(new BiomeStandIn(i));
            }
            return new Registry("biomes:" + size, Strategy.createForBiomes(map), map, size);
        }

        Object value(final JsonElement id) {
            return this.map.byIdOrThrow(id.getAsInt());
        }

        JsonElement id(final Object value) {
            return value == null ? JsonNull.INSTANCE : new JsonPrimitive(this.map.getId(value));
        }

        int sideLength() {
            return this.strategy.entryCount() == 4096 ? 16 : 4;
        }

        Configuration configuration(final int bits) {
            try {
                Method method = Strategy.class.getDeclaredMethod("getConfigurationForBitCount", int.class);
                method.setAccessible(true);
                return (Configuration)method.invoke(this.strategy, bits);
            } catch (ReflectiveOperationException e) {
                throw new IllegalStateException(e);
            }
        }
    }

    record BiomeStandIn(int id) {
    }

    /**
     * PalettedContainer, for the block states' strategy and the biomes' over registries of several sizes (the bits a
     * global palette takes follow from the size). Programs (TESTING.md) run ops on a container made with its initial
     * value, each step {@code [output, bitsPerEntry, getSerializedSize]}: setting values from a pool, so palettes grow
     * from a single value through linear and hash map palettes to the global one, getting them, reading buffers (vanilla's
     * own writes, and ones made by hand for every bits per entry), writing, counting, copying and recreating.
     */
    static final class PalettedContainerSuite implements Suite {
        // A palette's size at each of its resizes, and one either side: single value, linear (1 to 4 bits), hash map
        // (5 to 8), global.
        private static final int[] POOL_SIZES = {1, 2, 3, 4, 5, 8, 9, 16, 17, 32, 33, 64, 65, 128, 129, 256, 257, 300, 700};
        private static final int[] BIOME_REGISTRY_SIZES = {1, 2, 3, 4, 5, 8, 9, 16, 17, 64, 65, 67, 128, 129};

        @Override
        public String name() {
            return PalettedContainer.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            Random random = new Random(0x9a1e);
            List<Registry> registries = new ArrayList<>();
            registries.add(Registry.blocks());
            for (int size : BIOME_REGISTRY_SIZES) {
                registries.add(Registry.biomes(size));
            }

            for (Registry registry : registries) {
                // Buffers to read: every bits per entry made by hand, and what vanilla writes for each pool size.
                List<byte[]> buffers = new ArrayList<>(craftedBuffers(random, registry));
                for (int poolSize : POOL_SIZES) {
                    buffers.add(writtenBuffer(random, registry, Math.min(poolSize, registry.size())));
                }

                for (byte[] buffer : buffers) {
                    Object initial = registry.value(new JsonPrimitive(random.nextInt(registry.size())));
                    JsonArray ops = array(array("read", hex(buffer)), array("values"), array("write"));
                    writer.result("programs", array(registry.name(), registry.id(initial), ops), run(registry, initial, ops), null);
                }

                int programs = registry.name().equals("blocks") ? 120 : 24;
                for (int program = 0; program < programs; program++) {
                    int poolSize = Math.min(POOL_SIZES[program % POOL_SIZES.length], registry.size());
                    List<Object> pool = pool(random, registry, poolSize);
                    Object initial = random.nextInt(3) == 0 ? registry.value(new JsonPrimitive(random.nextInt(registry.size()))) : pool.get(0);
                    JsonArray ops = randomProgram(random, registry, pool, buffers);
                    writer.result("programs", array(registry.name(), registry.id(initial), ops), run(registry, initial, ops), null);
                }
            }
        }

        private static List<Object> pool(final Random random, final Registry registry, final int size) {
            List<Object> pool = new ArrayList<>();
            while (pool.size() < size) {
                Object value = registry.value(new JsonPrimitive(random.nextInt(registry.size())));
                if (!pool.contains(value)) {
                    pool.add(value);
                }
            }
            return pool;
        }

        // A container filled from a pool of that many values, as vanilla writes it.
        private static byte[] writtenBuffer(final Random random, final Registry registry, final int poolSize) {
            List<Object> pool = pool(random, registry, poolSize);
            PalettedContainer<Object> container = new PalettedContainer<>(pool.get(0), registry.strategy());
            int entries = registry.strategy().entryCount();
            for (int i = 0; i < entries; i++) {
                int side = registry.sideLength();
                int x = i % side;
                int z = i / side % side;
                int y = i / side / side;
                container.set(x, y, z, i < pool.size() ? pool.get(i) : pool.get(random.nextInt(pool.size())));
            }
            FriendlyByteBuf buffer = new FriendlyByteBuf(Unpooled.buffer());
            container.write(buffer);
            return readable(buffer);
        }

        // For every bits per entry a buffer could say (a signed byte): the palette its configuration reads and as many longs
        // as its storage holds, with entries in the palette's range mostly; and variants with an id the registry doesn't
        // have, a linear palette longer than it holds, a hash map palette longer than its bits, entries past the palette,
        // a byte short, and a byte more.
        private static List<byte[]> craftedBuffers(final Random random, final Registry registry) {
            List<byte[]> buffers = new ArrayList<>();
            List<Integer> bitCounts = new ArrayList<>();
            for (int bits = 0; bits <= 33; bits++) {
                bitCounts.add(bits);
            }
            bitCounts.addAll(List.of(-1, -128, 64, 127));
            int globalBits = registry.configuration(Integer.MAX_VALUE).bitsInMemory();
            for (int bits : bitCounts) {
                Configuration configuration = registry.configuration(bits);
                // A global palette reads nothing, so the palette's variants are the same buffer; and the bits past a
                // hash map's all read the same global storage, so most of them get a valid buffer only.
                boolean global = configuration instanceof Configuration.Global;
                boolean nearGlobal = Math.abs(bits - globalBits) <= 1 || bits == 9 || bits == 5 || bits == 32 || bits == 33 || bits == -1;
                for (int variant = 0; variant < 7; variant++) {
                    if (!global || variant == 0 || nearGlobal && (variant == 3 || variant == 5 || variant == 6)) {
                        buffers.add(crafted(random, registry, bits, configuration, variant));
                    }
                }
            }
            return buffers;
        }

        private static byte[] crafted(final Random random, final Registry registry, final int bits, final Configuration configuration, final int variant) {
            ByteArrayOutputStream out = new ByteArrayOutputStream();
            FriendlyByteBuf buffer = new FriendlyByteBuf(Unpooled.buffer());
            buffer.writeByte(bits);
            int bitsInMemory = configuration.bitsInMemory();
            int paletteSize;
            boolean global = configuration instanceof Configuration.Global;
            if (global) {
                paletteSize = registry.size();
            } else if (bitsInMemory == 0) {
                paletteSize = 1;
                buffer.writeVarInt(variant == 1 ? registry.size() : random.nextInt(registry.size()));
            } else {
                boolean linear = paletteFactoryName(configuration).equals("linear");
                int capacity = 1 << bitsInMemory;
                paletteSize = switch (variant) {
                    case 2 -> linear ? capacity + 1 : capacity + 3;
                    case 3 -> Math.max(1, capacity / 2);
                    default -> Math.min(capacity, registry.size());
                };
                paletteSize = Math.min(paletteSize, registry.size() + (variant == 1 ? 1 : 0));
                buffer.writeVarInt(paletteSize);
                // A hash map palette's entries never repeat (see BiMapSuite); a linear one's may.
                List<Integer> ids = new ArrayList<>();
                for (int i = 0; i < registry.size(); i++) {
                    ids.add(i);
                }
                java.util.Collections.shuffle(ids, random);
                for (int i = 0; i < paletteSize; i++) {
                    int id = i < ids.size() ? ids.get(i) : random.nextInt(registry.size());
                    buffer.writeVarInt(variant == 1 && i == paletteSize - 1 ? registry.size() : linear && variant == 4 ? ids.get(0) : id);
                }
            }
            if (bitsInMemory > 0) {
                int entries = registry.strategy().entryCount();
                int required = requiredLength(bitsInMemory, entries);
                int valuesPerLong = valuesPerLong(bitsInMemory);
                long mask = (1L << bitsInMemory) - 1L;
                // Entries within the palette mostly (variant 3: anywhere in the bits), and the bits past the last whole
                // entry of each long random.
                for (int cell = 0; cell < required; cell++) {
                    long packed = random.nextLong();
                    for (int index = 0; index < valuesPerLong; index++) {
                        long value = variant == 3 || random.nextInt(4000) == 0 ? random.nextLong() & mask : random.nextInt(Math.max(1, Math.min(paletteSize, (int)Math.min(mask + 1, Integer.MAX_VALUE))));
                        packed &= ~(mask << index * bitsInMemory);
                        packed |= value << index * bitsInMemory;
                    }
                    buffer.writeLong(packed);
                }
            }
            byte[] bytes = readable(buffer);
            out.writeBytes(bytes);
            if (variant == 5 && bytes.length > 0) {
                return java.util.Arrays.copyOf(bytes, bytes.length - 1);
            }
            if (variant == 6) {
                out.write(0x2a);
            }
            return out.toByteArray();
        }

        private static String paletteFactoryName(final Configuration configuration) {
            // Strategy's three Simple configurations differ in their factory: SingleValuePalette::create,
            // LinearPalette::create or HashMapPalette::create, whose palettes say which they are.
            Object palette = configuration.createPalette(Registry.biomes(1).strategy(), List.of());
            return switch (palette.getClass().getSimpleName()) {
                case "SingleValuePalette" -> "single";
                case "LinearPalette" -> "linear";
                case "HashMapPalette" -> "hashmap";
                default -> throw new IllegalStateException(palette.getClass().getName());
            };
        }

        private static JsonArray randomProgram(final Random random, final Registry registry, final List<Object> pool, final List<byte[]> buffers) {
            JsonArray ops = new JsonArray();
            int side = registry.sideLength();
            int length = 30 + random.nextInt(registry.name().equals("blocks") ? 900 : 200);
            for (int i = 0; i < length; i++) {
                int x = coordinate(random, side);
                int y = coordinate(random, side);
                int z = coordinate(random, side);
                JsonElement value = registry.id(pool.get(random.nextInt(pool.size())));
                int kind = random.nextInt(200);
                ops.add(
                    kind < 90 ? array("set", x, y, z, value)
                        : kind < 120 ? array("getAndSet", x, y, z, value)
                        : kind < 130 ? array("getAndSetUnchecked", x, y, z, value)
                        : kind < 170 ? array("get", x, y, z)
                        : kind < 174 ? array("maybeHas", value)
                        : kind < 178 ? array("count")
                        : kind < 182 ? array("getAll")
                        : kind < 184 ? array("forEachInPalette")
                        : kind < 187 ? array("values")
                        : kind < 189 ? array("write")
                        : kind < 191 ? array("copy")
                        : kind < 192 ? array("recreate")
                        : kind < 193 ? array("read", hex(buffers.get(random.nextInt(buffers.size()))))
                        : kind < 196 ? array("bitsPerEntry")
                        : array("getSerializedSize")
                );
            }
            ops.add(array("values"));
            ops.add(array("write"));
            return ops;
        }

        // In range, and out of it now and then, where an index can land past the storage or on another entry.
        private static int coordinate(final Random random, final int side) {
            return random.nextInt(400) == 0 ? new int[]{-1, side, side + 1, 2 * side}[random.nextInt(4)] : random.nextInt(side);
        }

        static JsonArray run(final Registry registry, final Object initial, final JsonArray ops) {
            PalettedContainer<Object> container = new PalettedContainer<>(initial, registry.strategy());
            JsonArray steps = new JsonArray();
            for (JsonElement element : ops) {
                JsonArray op = element.getAsJsonArray();
                String name = op.get(0).getAsString();
                try {
                    JsonElement output = switch (name) {
                        case "set" -> {
                            container.set(op.get(1).getAsInt(), op.get(2).getAsInt(), op.get(3).getAsInt(), registry.value(op.get(4)));
                            yield JsonNull.INSTANCE;
                        }
                        case "getAndSet" -> registry.id(container.getAndSet(op.get(1).getAsInt(), op.get(2).getAsInt(), op.get(3).getAsInt(), registry.value(op.get(4))));
                        case "getAndSetUnchecked" -> registry.id(container.getAndSetUnchecked(op.get(1).getAsInt(), op.get(2).getAsInt(), op.get(3).getAsInt(), registry.value(op.get(4))));
                        case "get" -> registry.id(container.get(op.get(1).getAsInt(), op.get(2).getAsInt(), op.get(3).getAsInt()));
                        case "maybeHas" -> {
                            Object value = registry.value(op.get(1));
                            yield new JsonPrimitive(container.maybeHas(candidate -> candidate == value));
                        }
                        case "count" -> {
                            // Sorted by id: Int2IntOpenHashMap's order, which the only caller (recalcBlockCounts) sums over.
                            Map<Integer, Integer> counts = new TreeMap<>();
                            container.count((value, count) -> counts.merge(registry.map().getId(value), count, Integer::sum));
                            JsonArray array = new JsonArray();
                            counts.forEach((id, count) -> array.add(array(id, count)));
                            yield array;
                        }
                        case "getAll" -> {
                            JsonArray values = new JsonArray();
                            container.getAll(value -> values.add(registry.id(value)));
                            yield values;
                        }
                        case "forEachInPalette" -> {
                            // The first 512, like any stream: a global palette is every value in the registry.
                            JsonArray values = new JsonArray();
                            container.forEachInPalette(value -> {
                                if (values.size() < 512) {
                                    values.add(registry.id(value));
                                }
                            });
                            yield values;
                        }
                        case "values" -> values(registry, container);
                        case "write" -> {
                            FriendlyByteBuf buffer = new FriendlyByteBuf(Unpooled.buffer());
                            container.write(buffer);
                            yield new JsonPrimitive(hex(readable(buffer)));
                        }
                        case "copy" -> {
                            container = container.copy();
                            yield JsonNull.INSTANCE;
                        }
                        case "recreate" -> {
                            container = container.recreate();
                            yield JsonNull.INSTANCE;
                        }
                        case "read" -> {
                            FriendlyByteBuf buffer = new FriendlyByteBuf(Unpooled.wrappedBuffer(unhex(op.get(1).getAsString())));
                            container.read(buffer);
                            yield new JsonPrimitive(buffer.readableBytes());
                        }
                        case "bitsPerEntry" -> new JsonPrimitive(container.bitsPerEntry());
                        case "getSerializedSize" -> new JsonPrimitive(container.getSerializedSize());
                        default -> throw new IllegalArgumentException(name);
                    };
                    steps.add(array(output, container.bitsPerEntry(), container.getSerializedSize()));
                } catch (RuntimeException e) {
                    steps.add(thrown(e));
                    break;
                }
            }
            return steps;
        }

        /** Every value in index order (y, then z, then x), as runs: {@code [[id, count], …]}. */
        static JsonArray values(final Registry registry, final PalettedContainer<Object> container) {
            int side = registry.sideLength();
            List<Object> values = new ArrayList<>();
            for (int y = 0; y < side; y++) {
                for (int z = 0; z < side; z++) {
                    for (int x = 0; x < side; x++) {
                        values.add(container.get(x, y, z));
                    }
                }
            }
            return runs(values.iterator(), registry::id);
        }
    }

    /** Values as runs of equal ones, {@code [[value, count], …]}, each value encoded by {@code encode}. */
    static <T> JsonArray runs(final Iterator<T> values, final java.util.function.Function<T, JsonElement> encode) {
        JsonArray runs = new JsonArray();
        JsonElement last = null;
        int count = 0;
        while (values.hasNext()) {
            JsonElement value = encode.apply(values.next());
            if (last != null && last.equals(value)) {
                count++;
            } else {
                if (last != null) {
                    runs.add(array(last, count));
                }
                last = value;
                count = 1;
            }
        }
        if (last != null) {
            runs.add(array(last, count));
        }
        return runs;
    }

    /** Writes the call's result, or what it threw. */
    static void call(final GoldenWriter writer, final String method, final JsonArray args, final Supplier<JsonElement> call) throws IOException {
        JsonElement result;
        try {
            result = call.get();
        } catch (RuntimeException e) {
            writer.threw(method, args, e, null);
            return;
        }
        writer.result(method, args, result, null);
    }
}
