package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonNull;
import com.google.gson.JsonPrimitive;
import java.io.IOException;
import java.lang.reflect.Field;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Random;

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
 */
final class HashMapSuite implements Suite {
    private static final int PROGRAMS = 600;
    private static final Field TABLE = tableField();

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
            Run run = run(program);
            writer.result(run.treeified() ? "treeified programs" : "programs", args, run.steps(), null);
        }
    }

    private record Op(String name, String key, Integer value) {
    }

    private record Run(JsonArray steps, boolean treeified) {
    }

    private static Run run(final List<Op> program) {
        HashMap<String, Integer> map = new HashMap<>();
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

    private static Field tableField() {
        try {
            Field field = HashMap.class.getDeclaredField("table");
            field.setAccessible(true);
            return field;
        } catch (NoSuchFieldException e) {
            throw new IllegalStateException(e);
        }
    }
}
