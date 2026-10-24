package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonNull;
import com.google.gson.JsonPrimitive;
import com.mojang.brigadier.StringReader;
import com.mojang.brigadier.exceptions.CommandSyntaxException;
import com.mojang.datafixers.util.Pair;
import com.mojang.serialization.DataResult;
import java.io.IOException;
import java.lang.reflect.Field;
import java.util.ArrayList;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Random;
import java.util.Set;
import java.util.UUID;
import net.minecraft.advancements.predicates.MinMaxBounds;
import net.minecraft.commands.arguments.selector.EntitySelector;
import net.minecraft.commands.arguments.selector.EntitySelectorParser;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.nbt.NbtOps;
import net.minecraft.nbt.StringTag;
import net.minecraft.nbt.Tag;
import net.minecraft.util.CompilableString;
import net.minecraft.world.entity.EntityType;
import net.minecraft.world.phys.AABB;

/** Entity selectors and NBT paths, which text components' selector, score and NBT contents hold. */
final class SelectorSuites {
    private SelectorSuites() {
    }

    static List<Suite> suites() {
        return List.of(new ParserSuite(), new NbtPathSuite());
    }

    /**
     * {@code NbtPathArgument.parse} of paths: hand-written ones of every node (names quoted and not, indexes, all
     * elements, patterns on the root, an element and an object), wrong ones, random paths and mutations. The result is
     * the path's text, its nodes ({@code [class, name or index or pattern…]}, read by reflection) and the cursor after,
     * or {@code ["error", message, cursor]}; a path ending in {@code [} throws (the reader peeks past the end).
     */
    private static final class NbtPathSuite implements Suite {
        private static final List<String> PIECES = List.of(
            "a", "b_c", "Item", "\"quoted key\"", "'single'", "\"\"", "[0]", "[-1]", "[]", "[{a:1b}]", "{a:1}", "{}", "[x]", "[{]",
            ".", "..", " ", "{", "}", "]", "[", "é", "😀", "a b"
        );

        @Override
        public String name() {
            return net.minecraft.commands.arguments.NbtPathArgument.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            Random random = new Random(71);
            Set<String> inputs = new LinkedHashSet<>(List.of(
                "", "a", "a.b", "a[0]", "a[]", "a[-1].b", "a[{b:1}]", "{a:1b}", "{a:1}.b", "a{b:\"c\"}", "a{b:1}.c", "\"q\".x", "'s'[0]",
                "a.", ".a", "a b", "a[x]", "a]{", "a..b", "a{", "a[{]", "x.{a:1}", "a[0", "\"unclosed", "a[2147483648]", "[0][1][]",
                "{a:1}{b:2}", "a[{b:1}]c", "a.\"b\"", "a.'b'.c", "a[ 0]"
            ));
            for (int i = 0; i < 1500; i++) {
                StringBuilder builder = new StringBuilder();
                int pieces = 1 + random.nextInt(4);
                for (int j = 0; j < pieces; j++) {
                    if (j > 0 && random.nextInt(3) != 0) {
                        builder.append('.');
                    }
                    builder.append(PIECES.get(random.nextInt(PIECES.size())));
                }
                inputs.add(builder.toString());
            }
            for (String input : inputs) {
                if (!GsonSuites.wellFormed(input)) {
                    continue;
                }
                JsonArray args = new JsonArray();
                args.add(input);
                StringReader reader = new StringReader(input);
                try {
                    net.minecraft.commands.arguments.NbtPathArgument.NbtPath path = new net.minecraft.commands.arguments.NbtPathArgument().parse(reader);
                    JsonElement result = RenderSuites.array(new JsonPrimitive(path.toString()), nodes(path), new JsonPrimitive(reader.getCursor()));
                    if (GsonSuites.wellFormed(result)) {
                        writer.result("parse", args, result, null);
                    }
                } catch (CommandSyntaxException e) {
                    JsonElement result = RenderSuites.array(new JsonPrimitive("error"), new JsonPrimitive(e.getMessage()), new JsonPrimitive(reader.getCursor()));
                    if (GsonSuites.wellFormed(result)) {
                        writer.result("parse", args, result, null);
                    }
                } catch (StringIndexOutOfBoundsException e) {
                    writer.threw("parse", args, e, null);
                }
            }
        }

        private static JsonArray nodes(final net.minecraft.commands.arguments.NbtPathArgument.NbtPath path) {
            JsonArray nodes = new JsonArray();
            try {
                Field field = path.getClass().getDeclaredField("nodes");
                field.setAccessible(true);
                for (Object node : (Object[])field.get(path)) {
                    JsonArray spec = new JsonArray();
                    spec.add(node.getClass().getSimpleName());
                    for (String name : List.of("name", "index", "pattern")) {
                        try {
                            Field nodeField = node.getClass().getDeclaredField(name);
                            nodeField.setAccessible(true);
                            Object value = nodeField.get(node);
                            spec.add(value instanceof Tag tag ? NbtValues.encodeTag(tag) : value instanceof Integer index ? new JsonPrimitive(index) : new JsonPrimitive((String)value));
                        } catch (NoSuchFieldException ignored) {
                            // The node hasn't got this field.
                        }
                    }
                    nodes.add(spec);
                }
            } catch (ReflectiveOperationException e) {
                throw new IllegalStateException(e);
            }
            return nodes;
        }
    }

    /**
     * {@code EntitySelectorParser.parse} of selectors, names and UUIDs: hand-written ones hitting each option's value
     * and applicability rules and each error, a few thousand random selectors (a type, then options with values good and
     * bad, with whitespace and separators good and bad), and mutations of them. {@code parse} is with selectors allowed,
     * {@code parseNoSelectors} without; a result is the selector as the parse leaves it ({@code getMaxResults},
     * {@code includesEntities}, {@code isSelfSelector}, {@code isWorldLimited}, {@code usesSelector}, and through
     * reflection the player's name, the UUID, the type's key, the distance's bounds, the position's coordinates, the box
     * and the order: {@code arbitrary}, {@code nearest}, {@code furthest} or {@code random}) and the reader's cursor
     * after, or {@code ["error", message, cursor]}. {@code COMPILABLE_CODEC} decodes each as a string tag (the source,
     * or the error's message). Doubles are their bits.
     */
    private static final class ParserSuite implements Suite {
        private static final List<String> TYPES = List.of("p", "a", "r", "s", "e", "n", "x", "P", "", " ", "@", "[");
        private static final List<String[]> OPTIONS = List.of(
            new String[]{"name", "Steve", "!Steve", "\"quoted name\"", "'single'", "!", "", "\"unclosed", "!\"a b\""},
            new String[]{"distance", "1", "..5", "1..5", "5..1", "-1", "-1..2", ".5..1.5", "1...", "..", "", "abc", "1.5.5", "0..-0", "-0", "..-.5"},
            new String[]{"level", "0", "1..3", "-1", "3..1", "1.5", "..2147483648", "2147483647..", ""},
            new String[]{"x", "1", "-2.5", "~1", "abc", "", ".5", "-", "1e5"},
            new String[]{"y", "64", "-64.0", "1.2.3"},
            new String[]{"z", "0", "--1"},
            new String[]{"dx", "3", "-3", "0"},
            new String[]{"dy", "-1.5", "2"},
            new String[]{"dz", "10", "x"},
            new String[]{"x_rotation", "0..90", "-90", "180..-180", "x", "..", "1.5..2.5"},
            new String[]{"y_rotation", "-45..45", "360", ""},
            new String[]{"limit", "1", "0", "-5", "2147483648", "abc", "10", ""},
            new String[]{"sort", "nearest", "furthest", "random", "arbitrary", "bogus", "", "NEAREST"},
            new String[]{"gamemode", "survival", "!creative", "adventure", "spectator", "SURVIVAL", "hardcore", "!", ""},
            new String[]{"team", "red", "!red", "!", "", "blue.team"},
            new String[]{
                "type", "pig", "minecraft:zombie", "player", "!player", "#minecraft:skeletons", "!#raiders", "unknown_mob", "Pig",
                "minecraft:", ":", "a:b:c", "!pig", "#", "minecraft:player", "!#minecraft:skeletons"
            },
            new String[]{"tag", "foo", "!foo", "", "!", "a.b"},
            new String[]{"nbt", "{}", "{a:1}", "!{Health:20f}", "{a:[1,2]}", "{", "{a:}", "[1]", "\"str\"", "{a:b,c:{d:1b}}", ""},
            new String[]{"scores", "{}", "{obj=1}", "{a=1..5,b=..2}", "{a=}", "{a=5..1}", "{a=1, b=2 }", "{a=1 b=2}", "{", "", "{a 1}"},
            new String[]{
                "advancements", "{}", "{minecraft:story/root=true}", "{a={crit=false}}", "{a=maybe}", "{A=true}", "{a={b=true,c=false}}",
                "{a={b=1}}", "{a=true,}", "{"
            },
            new String[]{"predicate", "my:pred", "!x", "Bad:Pred", "", "a/b"},
            new String[]{"foo", "1"},
            new String[]{"", "1"},
            new String[]{"Name", "x"}
        );
        private static final List<String> HAND_MADE = List.of(
            "", "Steve", "a", "abcdefghijklmnop", "abcdefghijklmnopq", "\"quoted\"", "'q'", "\"\"", "\"unclosed", "with space", "é",
            "f81d4fae-7dec-11d0-a765-00a0c91e6bf6", "F81D4FAE-7DEC-11D0-A765-00A0C91E6BF6", "1-2-3-4-5", "1-2-3-4", "0-0-0-0-0",
            "f81d4fae-7dec-11d0-a765-00a0c91e6bf6-1", "123456789-1-1-1-1", "@", "@p", "@a", "@r", "@s", "@e", "@n", "@x", "@@", "@p[",
            "@p[]", "@p[ ]", "@e[,]", "@e[type=pig", "@e[type=pig,]", "@e[type=pig]x", "@e[type=pig] trailing", "@e[type]", "@e[type=]",
            "@e[ type = pig , limit = 2 ]", "@e[type=pig,type=cow]", "@e[type=!pig,type=!cow]", "@e[type=pig,type=!cow]",
            "@e[type=#skeletons,type=#skeletons]", "@e[type=#skeletons,type=#raiders]", "@e[type=!pig,type=#a]", "@s[limit=1]",
            "@s[sort=nearest]", "@p[limit=2,limit=3]", "@e[name=a,name=b]", "@e[name=!a,name=!b]", "@e[gamemode=!survival,gamemode=!creative]",
            "@e[gamemode=survival,gamemode=creative]", "@e[team=a,team=b]", "@e[team=!a,team=!b]", "@e[distance=1,distance=2]",
            "@e[x=1,x=2]", "@e[scores={},scores={}]", "@e[advancements={},advancements={}]", "@e[level=1,level=2]",
            "@a[advancements={story/root=true}]", "@e[dx=1,dy=-2]", "@e[distance=..10]", "@e[distance=5]", "@e[x=1,y=2,z=3]",
            "@e[\"type\"=pig]", "@e['limit'=1]", "@e[type=player]", "@e[type=!player]", "@e[nbt={a:1}]", "@e[nbt=!{}]",
            "@e[tag=]", "@e[tag=!]", "@e\t[type=pig]", "@e [type=pig]", "@e[type=pig] ", "@😀", "@p[type=pig]", "@a[type=!pig]"
        );
        private static final int RANDOM = 3000;
        private static final int MUTATIONS = 1500;

        @Override
        public String name() {
            return EntitySelectorParser.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            Random random = new Random(67);
            Set<String> inputs = new LinkedHashSet<>(HAND_MADE);
            for (String[] option : OPTIONS) {
                for (int i = 1; i < option.length; i++) {
                    inputs.add("@e[" + option[0] + "=" + option[i] + "]");
                    inputs.add("@s[" + option[0] + "=" + option[i] + "]");
                }
            }
            List<String> generated = new ArrayList<>();
            for (int i = 0; i < RANDOM; i++) {
                generated.add(randomSelector(random));
            }
            inputs.addAll(generated);
            List<String> seeds = new ArrayList<>(inputs);
            for (int i = 0; i < MUTATIONS; i++) {
                inputs.add(mutate(seeds.get(random.nextInt(seeds.size())), random));
            }
            int i = 0;
            for (String input : inputs) {
                if (!GsonSuites.wellFormed(input)) {
                    continue;
                }
                JsonArray args = new JsonArray();
                args.add(input);
                JsonElement parsed = parse(input, true);
                if (GsonSuites.wellFormed(parsed)) {
                    writer.result("parse", args, parsed, null);
                }
                if (i++ % 4 == 0) {
                    JsonElement noSelectors = parse(input, false);
                    if (GsonSuites.wellFormed(noSelectors)) {
                        writer.result("parseNoSelectors", args, noSelectors, null);
                    }
                }
                DataResult<Pair<CompilableString<EntitySelector>, Tag>> decoded = EntitySelector.COMPILABLE_CODEC.decode(NbtOps.INSTANCE, StringTag.valueOf(input));
                JsonArray result = DfuValues.result(decoded.map(Pair::getFirst), compilable -> new JsonPrimitive(compilable.source()));
                if (GsonSuites.wellFormed(result)) {
                    writer.result("COMPILABLE_CODEC.decode", args, result, null);
                }
            }
        }

        private static String randomSelector(final Random random) {
            StringBuilder builder = new StringBuilder("@");
            builder.append(TYPES.get(random.nextInt(TYPES.size())));
            if (random.nextInt(8) == 0) {
                return builder.toString();
            }
            builder.append('[');
            int count = random.nextInt(4);
            for (int i = 0; i < count; i++) {
                String[] option = OPTIONS.get(random.nextInt(OPTIONS.size()));
                if (random.nextInt(6) == 0) {
                    builder.append(' ');
                }
                builder.append(option[0]);
                builder.append(random.nextInt(10) == 0 ? " = " : "=");
                builder.append(option[1 + random.nextInt(option.length - 1)]);
                if (i < count - 1) {
                    builder.append(random.nextInt(12) == 0 ? " " : ",");
                }
            }
            if (random.nextInt(10) != 0) {
                builder.append(']');
            }
            return builder.toString();
        }

        private static String mutate(final String input, final Random random) {
            StringBuilder builder = new StringBuilder(input);
            String alphabet = "@[]=,!#{}:.-0123456789 \"'abcpe";
            int edits = 1 + random.nextInt(3);
            for (int i = 0; i < edits; i++) {
                int at = builder.length() == 0 ? 0 : random.nextInt(builder.length() + 1);
                char c = alphabet.charAt(random.nextInt(alphabet.length()));
                switch (random.nextInt(3)) {
                    case 0 -> builder.insert(at, c);
                    case 1 -> {
                        if (at < builder.length()) {
                            builder.deleteCharAt(at);
                        }
                    }
                    default -> {
                        if (at < builder.length()) {
                            builder.setCharAt(at, c);
                        }
                    }
                }
            }
            return builder.toString();
        }

        private static JsonElement parse(final String input, final boolean allowSelectors) {
            StringReader reader = new StringReader(input);
            try {
                EntitySelector selector = new EntitySelectorParser(reader, allowSelectors).parse();
                return RenderSuites.array(describe(selector), new JsonPrimitive(reader.getCursor()));
            } catch (CommandSyntaxException e) {
                return RenderSuites.array(new JsonPrimitive("error"), new JsonPrimitive(e.getMessage()), new JsonPrimitive(reader.getCursor()));
            }
        }

        // [maxResults, includesEntities, isSelfSelector, isWorldLimited, usesSelector, playerName, entityUUID, type,
        // [distance min, max], [x, y, z], aabb, order].
        private static JsonArray describe(final EntitySelector selector) {
            JsonArray array = new JsonArray();
            array.add(selector.getMaxResults());
            array.add(selector.includesEntities());
            array.add(selector.isSelfSelector());
            array.add(selector.isWorldLimited());
            array.add(selector.usesSelector());
            String playerName = (String)field(selector, "playerName");
            array.add(playerName == null ? JsonNull.INSTANCE : new JsonPrimitive(playerName));
            UUID uuid = (UUID)field(selector, "entityUUID");
            array.add(uuid == null ? JsonNull.INSTANCE : new JsonPrimitive(uuid.toString()));
            Object type = field(selector, "type");
            array.add(type instanceof EntityType<?> entityType ? new JsonPrimitive(BuiltInRegistries.ENTITY_TYPE.getKey(entityType).toString()) : JsonNull.INSTANCE);
            MinMaxBounds.Doubles range = (MinMaxBounds.Doubles)field(selector, "range");
            array.add(range == null ? JsonNull.INSTANCE : RenderSuites.array(bound(range.min()), bound(range.max())));
            @SuppressWarnings("unchecked")
            java.util.function.Function<net.minecraft.world.phys.Vec3, net.minecraft.world.phys.Vec3> position =
                (java.util.function.Function<net.minecraft.world.phys.Vec3, net.minecraft.world.phys.Vec3>)field(selector, "position");
            // The source's position here is NaN, which a coordinate the selector sets replaces.
            net.minecraft.world.phys.Vec3 at = position.apply(new net.minecraft.world.phys.Vec3(Double.NaN, Double.NaN, Double.NaN));
            array.add(RenderSuites.array(coordinate(at.x), coordinate(at.y), coordinate(at.z)));
            AABB aabb = (AABB)field(selector, "aabb");
            array.add(aabb == null ? JsonNull.INSTANCE : RenderSuites.array(
                new JsonPrimitive(Values.doubleBits(aabb.minX)), new JsonPrimitive(Values.doubleBits(aabb.minY)), new JsonPrimitive(Values.doubleBits(aabb.minZ)),
                new JsonPrimitive(Values.doubleBits(aabb.maxX)), new JsonPrimitive(Values.doubleBits(aabb.maxY)), new JsonPrimitive(Values.doubleBits(aabb.maxZ))
            ));
            Object order = field(selector, "order");
            array.add(order == EntitySelector.ORDER_ARBITRARY ? "arbitrary" : order == EntitySelectorParser.ORDER_NEAREST ? "nearest"
                : order == EntitySelectorParser.ORDER_FURTHEST ? "furthest" : order == EntitySelectorParser.ORDER_RANDOM ? "random" : "?");
            return array;
        }

        private static JsonElement bound(final java.util.Optional<Double> value) {
            return value.<JsonElement>map(v -> new JsonPrimitive(Values.doubleBits(v))).orElse(JsonNull.INSTANCE);
        }

        private static JsonElement coordinate(final double value) {
            return Double.isNaN(value) ? JsonNull.INSTANCE : new JsonPrimitive(Values.doubleBits(value));
        }

        private static Object field(final EntitySelector selector, final String name) {
            try {
                Field field = EntitySelector.class.getDeclaredField(name);
                field.setAccessible(true);
                return field.get(selector);
            } catch (ReflectiveOperationException e) {
                throw new IllegalStateException(e);
            }
        }
    }
}
