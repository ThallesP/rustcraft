package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonNull;
import com.google.gson.JsonPrimitive;
import java.io.IOException;
import java.lang.reflect.Field;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Random;
import java.util.stream.IntStream;

/**
 * Runs random programs against a java.util.HashMap of strings, the map a CompoundTag keeps its tags in. Its iteration
 * order, which is the order NBT writes a compound's tags in, goes by its table: bins in order, each bin's entries in the
 * order they came. A case runs the ops in {@code args[0]} on a new map; the result has {@code [output, state]} after each
 * op, the state being {@code [capacity, [[key, value], …]]}: the table's length (0 before it's made) and the entries in
 * iteration order. The ops are {@code put}, {@code remove}, {@code get}, {@code containsKey} and {@code copy}
 * ({@code new HashMap<>(map)}, which CompoundTag.shallowCopy does, and which sizes its table from the size). Keys come
 * from small pools so they repeat, from a family whose hashCodes are all equal ("Aa" and "BB" blocks), and from one whose
 * hashes only agree in their low bits, so bins grow long: a bin of 9 makes a table under 64 grow. A program whose map
 * turns a bin into a red-black tree (a bin of 9 in a table of 64 or more) is written under "treeified programs".
 *
 * <p>"sized programs" run on a map made with {@code new HashMap<>(initialCapacity)}, {@code [initialCapacity, program]},
 * the way codecs make their maps and sets (ByteBufCodecs.map and collection pass the count they read, up to 65536): the
 * table its first put makes is the smallest power of two at least the capacity, and a negative one throws. Those whose map
 * makes a tree bin are written under "treeified sized programs".
 *
 * <p>The suite also has what other suites need to encode and copy a HashMap or HashSet with its table.
 */
final class HashMapSuite implements Suite {
    private static final int PROGRAMS = 600;
    private static final int RANDOM_SIZED_PROGRAMS = 120;
    private static final Field TABLE = field(HashMap.class, "table");
    private static final Field SET_MAP = field(HashSet.class, "map");
    // What an empty copy with a table puts and takes out again, to make the table.
    private static final Object PLACEHOLDER = new Object();

    // Capacities around the powers of two and the load factor's thresholds, a random list's (up to 140), the most a
    // codec passes (65536), and negative ones, which throw.
    private static final List<Integer> INITIAL_CAPACITIES = List.of(
        0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 11, 12, 13, 15, 16, 17, 24, 25, 31, 32, 33, 63, 64, 65, 100, 128, 140, 65535, 65536, -1,
        Integer.MIN_VALUE
    );

    // A compound's keys, an item's and a block entity's, and the empty key a list's wrapper compound has.
    private static final List<String> COMMON_KEYS = List.of(
        "id", "Count", "count", "Slot", "x", "y", "z", "Name", "Items", "display", "tag", "components", "", "minecraft:custom_data", "Damage",
        "keepPacked", "Lore", "CustomName", "a", "b", "c", "text", "color", "extra", "bold", "italic"
    );
    // Strings whose hashCodes are all equal: every string of five "Aa" and "BB" blocks.
    private static final List<String> COLLIDING_KEYS = colliding(5);
    // Strings whose spread hashes agree in their low 6 bits, so they share a bin in tables up to 64.
    private static final List<String> LOW_BIT_KEYS = lowBitKeys(24);

    @Override
    public String name() {
        return HashMap.class.getName();
    }

    @Override
    public void write(final GoldenWriter writer) throws IOException {
        Random random = new Random(0x4a5b);
        for (int i = 0; i < PROGRAMS; i++) {
            List<Op> program = randomProgram(random, i % 6);
            JsonArray args = new JsonArray();
            args.add(encode(program));
            Run run = run(new HashMap<>(), program);
            writer.result(run.treeified() ? "treeified programs" : "programs", args, run.steps(), null);
        }

        // Each capacity with each kind of program, then random capacities.
        Random sized = new Random(0x5a4b);
        List<Integer> capacities = new ArrayList<>();
        INITIAL_CAPACITIES.forEach(capacity -> IntStream.range(0, 6).forEach(kind -> capacities.add(capacity)));
        IntStream.range(0, RANDOM_SIZED_PROGRAMS).forEach(i -> capacities.add(sized.nextInt(300)));
        for (int i = 0; i < capacities.size(); i++) {
            int initialCapacity = capacities.get(i);
            List<Op> program = randomProgram(sized, i % 6);
            JsonArray args = new JsonArray();
            args.add(initialCapacity);
            args.add(encode(program));
            HashMap<String, Integer> map;
            try {
                map = new HashMap<>(initialCapacity);
            } catch (IllegalArgumentException e) {
                writer.threw("sized programs", args, e, null);
                continue;
            }
            Run run = run(map, program);
            writer.result(run.treeified() ? "treeified sized programs" : "sized programs", args, run.steps(), null);
        }
    }

    private record Op(String name, String key, Integer value) {
    }

    private record Run(JsonArray steps, boolean treeified) {
    }

    private static Run run(final HashMap<String, Integer> initial, final List<Op> program) {
        HashMap<String, Integer> map = initial;
        JsonArray steps = new JsonArray();
        boolean treeified = false;
        for (Op op : program) {
            JsonElement output = switch (op.name()) {
                case "put" -> integer(map.put(op.key(), op.value()));
                case "remove" -> integer(map.remove(op.key()));
                case "get" -> integer(map.get(op.key()));
                case "containsKey" -> new JsonPrimitive(map.containsKey(op.key()));
                case "copy" -> {
                    map = new HashMap<>(map);
                    yield JsonNull.INSTANCE;
                }
                default -> throw new IllegalArgumentException(op.name());
            };
            treeified |= hasTreeBins(map);
            JsonArray step = new JsonArray();
            step.add(output);
            step.add(state(map));
            steps.add(step);
        }
        return new Run(steps, treeified);
    }

    /** {@code [capacity, [[key, value], …]]}: the table's length, and the entries in iteration order. */
    static JsonArray state(final HashMap<String, Integer> map) {
        JsonArray entries = new JsonArray();
        map.forEach((key, value) -> {
            JsonArray entry = new JsonArray();
            entry.add(key);
            entry.add(value);
            entries.add(entry);
        });
        JsonArray state = new JsonArray();
        state.add(capacity(map));
        state.add(entries);
        return state;
    }

    /** The length of the map's table, or 0 before it's made. */
    static int capacity(final HashMap<?, ?> map) {
        Object[] table = table(map);
        return table == null ? 0 : table.length;
    }

    /** The length of the set's map's table, or 0 before it's made. */
    static int capacity(final HashSet<?> set) {
        return capacity(map(set));
    }

    /**
     * A HashMap is {@code [capacity, [[key, value], …]]} and a HashSet {@code [capacity, [element, …]]}, like a compound's
     * map: its table's length and what it holds in its order, which is the order an encoder writes it in. Keys, values
     * and elements are encoded by their own class.
     */
    static JsonArray encode(final HashMap<?, ?> map) {
        JsonArray entries = new JsonArray();
        // Holders hash by identity, so a map of them iterates in another order every run: its entries sorted by key.
        Iterable<? extends Map.Entry<?, ?>> ordered = RegistryValues.keyedByHolders(map) ? RegistryValues.sortedByHolder(map) : map.entrySet();
        for (Map.Entry<?, ?> mapping : ordered) {
            JsonArray entry = new JsonArray();
            entry.add(Values.encode(mapping.getKey(), mapping.getKey().getClass()));
            entry.add(Values.encode(mapping.getValue(), mapping.getValue().getClass()));
            entries.add(entry);
        }
        JsonArray array = new JsonArray();
        array.add(capacity(map));
        array.add(entries);
        return array;
    }

    static JsonArray encode(final HashSet<?> set) {
        JsonArray elements = new JsonArray();
        set.forEach(element -> elements.add(Values.encode(element, element.getClass())));
        JsonArray array = new JsonArray();
        array.add(capacity(set));
        array.add(elements);
        return array;
    }

    /**
     * A copy with the same table length and order: made with the capacity (no table until the first put), a placeholder
     * put and taken out again when the map is empty but has a table, then its entries put in its order, which puts each in
     * the same bin after the ones before it. The copy never grows, since the map had no bin of 9 and no more entries than
     * its threshold, and the generators never make a tree bin.
     */
    @SuppressWarnings("unchecked")
    static <K, V> HashMap<K, V> copy(final HashMap<K, V> map) {
        int capacity = capacity(map);
        HashMap<K, V> copy = capacity == 0 ? new HashMap<>() : new HashMap<>(capacity);
        if (capacity > 0 && map.isEmpty()) {
            ((Map<Object, Object>)(Map<?, ?>)copy).put(PLACEHOLDER, PLACEHOLDER);
            copy.remove(PLACEHOLDER);
        }
        map.forEach(copy::put);
        return copy;
    }

    @SuppressWarnings("unchecked")
    static <E> HashSet<E> copy(final HashSet<E> set) {
        int capacity = capacity(set);
        HashSet<E> copy = capacity == 0 ? new HashSet<>() : new HashSet<>(capacity);
        if (capacity > 0 && set.isEmpty()) {
            ((HashSet<Object>)(HashSet<?>)copy).add(PLACEHOLDER);
            copy.remove(PLACEHOLDER);
        }
        set.forEach(copy::add);
        return copy;
    }

    private static HashMap<?, ?> map(final HashSet<?> set) {
        try {
            return (HashMap<?, ?>)SET_MAP.get(set);
        } catch (IllegalAccessException e) {
            throw new IllegalStateException(e);
        }
    }

    /** Whether a bin of the map's table is a red-black tree. */
    static boolean hasTreeBins(final HashMap<?, ?> map) {
        Object[] table = table(map);
        if (table == null) {
            return false;
        }
        for (Object bin : table) {
            if (bin != null && bin.getClass().getName().equals("java.util.HashMap$TreeNode")) {
                return true;
            }
        }
        return false;
    }

    private static Object[] table(final HashMap<?, ?> map) {
        try {
            return (Object[])TABLE.get(map);
        } catch (IllegalAccessException e) {
            throw new IllegalStateException(e);
        }
    }

    // Kinds of programs: mostly puts of common keys, puts and removes of them (so tables are bigger than their maps),
    // copies among them, colliding keys up to 10 (a bin of 9 grows tables under 64), colliding keys past that (whose bin
    // becomes a tree once the table is 64), and keys that share bins only in small tables.
    private static List<Op> randomProgram(final Random random, final int kind) {
        List<Op> program = new ArrayList<>();
        int length = 1 + random.nextInt(random.nextInt(4) == 0 ? 160 : 40);
        List<String> keys = switch (kind) {
            case 0, 1, 2 -> pool(random, COMMON_KEYS, 3 + random.nextInt(40));
            case 3 -> mixed(random, COLLIDING_KEYS.subList(0, 3 + random.nextInt(8)), 20);
            case 4 -> mixed(random, COLLIDING_KEYS, 10);
            default -> mixed(random, LOW_BIT_KEYS, 20);
        };
        for (int i = 0; i < length; i++) {
            String key = keys.get(random.nextInt(keys.size()));
            int roll = random.nextInt(20);
            if ((kind == 1 && roll < 6) || roll == 0) {
                program.add(new Op("remove", key, null));
            } else if ((kind == 2 && roll >= 6 && roll < 8) || roll == 1) {
                program.add(new Op("copy", null, null));
            } else if (roll == 2) {
                program.add(new Op("get", key, null));
            } else if (roll == 3) {
                program.add(new Op("containsKey", key, null));
            } else {
                program.add(new Op("put", key, random.nextInt(1000)));
            }
        }
        return program;
    }

    // Some of the pool's keys, and random strings.
    private static List<String> pool(final Random random, final List<String> keys, final int size) {
        List<String> pool = new ArrayList<>();
        for (int i = 0; i < size; i++) {
            pool.add(random.nextInt(3) == 0 ? (String)Values.STRINGS.random(random) : keys.get(random.nextInt(keys.size())));
        }
        return pool;
    }

    // The keys, and as many common ones.
    private static List<String> mixed(final Random random, final List<String> keys, final int common) {
        List<String> mixed = new ArrayList<>(keys);
        for (int i = 0; i < common; i++) {
            mixed.add(COMMON_KEYS.get(random.nextInt(COMMON_KEYS.size())));
        }
        return mixed;
    }

    private static JsonArray encode(final List<Op> program) {
        JsonArray ops = new JsonArray();
        for (Op op : program) {
            JsonArray encoded = new JsonArray();
            encoded.add(op.name());
            if (op.key() != null) {
                encoded.add(op.key());
            }
            if (op.value() != null) {
                encoded.add(op.value());
            }
            ops.add(encoded);
        }
        return ops;
    }

    private static JsonElement integer(final Integer value) {
        return value == null ? JsonNull.INSTANCE : new JsonPrimitive(value);
    }

    /** Every string of {@code blocks} "Aa" and "BB" blocks, whose hashCodes are all equal. */
    static List<String> colliding(final int blocks) {
        List<String> strings = new ArrayList<>(List.of(""));
        for (int i = 0; i < blocks; i++) {
            List<String> longer = new ArrayList<>();
            for (String string : strings) {
                longer.add(string + "Aa");
                longer.add(string + "BB");
            }
            strings = longer;
        }
        return strings;
    }

    // "key0", "key1", … whose spread hashes (HashMap.hash) are 7 in their low 6 bits.
    private static List<String> lowBitKeys(final int count) {
        List<String> keys = new ArrayList<>();
        for (int i = 0; keys.size() < count; i++) {
            String key = "key" + i;
            int hash = key.hashCode() ^ key.hashCode() >>> 16;
            if ((hash & 63) == 7) {
                keys.add(key);
            }
        }
        return keys;
    }

    private static Field field(final Class<?> type, final String name) {
        try {
            Field field = type.getDeclaredField(name);
            field.setAccessible(true);
            return field;
        } catch (NoSuchFieldException e) {
            throw new IllegalStateException(e);
        }
    }
}
