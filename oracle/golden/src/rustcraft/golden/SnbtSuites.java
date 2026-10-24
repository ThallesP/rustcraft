package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonPrimitive;
import com.mojang.brigadier.StringReader;
import com.mojang.brigadier.exceptions.CommandSyntaxException;
import com.mojang.serialization.Codec;
import com.mojang.serialization.DataResult;
import java.io.IOException;
import java.util.ArrayList;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Random;
import java.util.Set;
import net.minecraft.nbt.CompoundTag;
import net.minecraft.nbt.EndTag;
import net.minecraft.nbt.IntTag;
import net.minecraft.nbt.ListTag;
import net.minecraft.nbt.NbtOps;
import net.minecraft.nbt.StringTag;
import net.minecraft.nbt.Tag;
import net.minecraft.nbt.TagParser;

/** SNBT, the game's text form of NBT ({@code TagParser} over {@code SnbtGrammar}). */
final class SnbtSuites {
    private SnbtSuites() {
    }

    static List<Suite> suites() {
        return List.of(new TagParserSuite());
    }

    /**
     * {@code TagParser} over {@code NbtOps}. Inputs are hand-written SNBT of every kind (every number form at its edges:
     * suffixes, signedness, hex and binary, underscores, exponents, leading zeros, ranges; booleans; every string form
     * and escape, surrogate pairs made of two escapes; lists, arrays and their element types; compounds with every kind
     * of key, repeated keys and keys whose hashes collide; the operations; whitespace everywhere), hand-written input
     * hitting each error, random SNBT in the forms it can be written in, the SNBT {@code toString} writes of random tags,
     * and mutations of all of those (one to three characters inserted, deleted or replaced).
     *
     * <p>A result is {@code ["tag", tag]} ({@code NbtValues.encodeTag}) or {@code ["error", message]}, the
     * {@code CommandSyntaxException}'s message with its context; a reader's cases add the cursor after. Each method:
     * <ul>
     * <li>{@code parseFully(String)}: {@code [input]}.
     * <li>{@code parseFully(StringReader)}: {@code [input, start]}, a reader at {@code start} (after a prefix).
     * <li>{@code parseAsArgument}: {@code [input]}, most with trailing text after a value.
     * <li>{@code parseCompoundFully}: {@code [input]}.
     * <li>{@code parseCompoundAsArgument}: {@code [input]}.
     * <li>{@code FLATTENED_CODEC.decode} and {@code LENIENT_CODEC.decode}: {@code [tag]} → the {@code DataResult}
     * ({@code DfuValues.result}) of {@code [compound, rest]}; {@code .encode}: {@code [compound, prefix]} → the
     * {@code DataResult} of the tag.
     * </ul>
     *
     * <p>Left out: {@code \N{name}} escapes, which need {@code Character.codePointOf}'s table of Unicode's character names
     * (the port fails them), cases whose input, result or message would hold a lone surrogate, which the golden files'
     * UTF-8 can't (an escape of half a pair, a mutation splitting one, a context cutting one), and compounds of more
     * than eight keys of one hash, whose bin Java's {@code HashMap} makes a tree of, which the port's doesn't have.
     */
    private static final class TagParserSuite implements Suite {
        private static final TagParser<Tag> PARSER = TagParser.create(NbtOps.INSTANCE);
        private static final int RANDOM_DOCUMENTS = 1500;
        private static final int RANDOM_TAGS = 400;
        private static final int MUTATIONS = 3000;
        private static final List<String> MUTATION_PIECES = List.of(
            "{", "}", "[", "]", ":", ";", ",", "\"", "'", "\\", " ", "\t", "\n", ".", "-", "+", "_", "0", "1", "7", "9", "a",
            "b", "B", "e", "E", "f", "d", "i", "I", "l", "L", "s", "S", "u", "U", "x", "X", "n", "t", "(", ")", "é", "😀", "\u00a0",
            "\u3000", "\u0000"
        );
        private static final List<String> TRAILERS = List.of(
            "", " ", " x", "x", "]", "}", ",", ", 1", " 1", "abc", ":", ")", " {}", "\"", "'", ";", "0", ".", "e5", "b"
        );

        @Override
        public String name() {
            return TagParser.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            Random random = new Random(1337);
            List<String> written = handWritten();
            List<String> generated = new ArrayList<>();
            for (int i = 0; i < RANDOM_DOCUMENTS; i++) {
                generated.add(randomDocument(random, 0));
            }
            for (int i = 0; i < RANDOM_TAGS; i++) {
                generated.add(NbtValues.randomTag(random, 0).toString());
            }
            List<String> sources = new ArrayList<>(written);
            sources.addAll(generated);
            List<String> mutated = new ArrayList<>();
            for (int i = 0; i < MUTATIONS; i++) {
                mutated.add(mutate(sources.get(random.nextInt(sources.size())), random));
            }
            Set<String> inputs = new LinkedHashSet<>();
            inputs.addAll(written);
            inputs.addAll(generated);
            inputs.addAll(mutated);

            for (String input : inputs) {
                if (kept(input)) {
                    record(writer, "parseFully(String)", args(input), parse(() -> PARSER.parseFully(input)));
                }
            }

            for (String input : inputs) {
                if (kept(input) && random.nextInt(3) == 0) {
                    String prefix = List.of("", " ", "a ", "{x:", "[1,", "😀 ").get(random.nextInt(6));
                    String whole = prefix + input;
                    StringReader reader = new StringReader(whole);
                    reader.setCursor(prefix.length());
                    JsonArray args = args(whole);
                    args.add(prefix.length());
                    record(writer, "parseFully(StringReader)", args, withCursor(parse(() -> PARSER.parseFully(reader)), reader));
                }
            }

            List<String> withTrailers = new ArrayList<>();
            for (String input : written) {
                withTrailers.add(input + TRAILERS.get(random.nextInt(TRAILERS.size())));
            }
            for (String input : generated) {
                if (random.nextInt(2) == 0) {
                    withTrailers.add(input + TRAILERS.get(random.nextInt(TRAILERS.size())));
                }
            }
            for (String input : mutated) {
                if (random.nextInt(4) == 0) {
                    withTrailers.add(input);
                }
            }
            for (String input : new LinkedHashSet<>(withTrailers)) {
                if (kept(input)) {
                    StringReader reader = new StringReader(input);
                    record(writer, "parseAsArgument", args(input), withCursor(parse(() -> PARSER.parseAsArgument(reader)), reader));
                }
            }

            for (String input : inputs) {
                if (kept(input) && (input.contains("{") || random.nextInt(4) == 0)) {
                    record(writer, "parseCompoundFully", args(input), parse(() -> TagParser.parseCompoundFully(input)));
                }
            }
            for (String input : new LinkedHashSet<>(withTrailers)) {
                if (kept(input) && (input.contains("{") || random.nextInt(4) == 0)) {
                    StringReader reader = new StringReader(input);
                    record(writer, "parseCompoundAsArgument", args(input), withCursor(parse(() -> TagParser.parseCompoundAsArgument(reader)), reader));
                }
            }

            codecs(writer, "FLATTENED_CODEC", TagParser.FLATTENED_CODEC, inputs, random);
            codecs(writer, "LENIENT_CODEC", TagParser.LENIENT_CODEC, inputs, random);
        }

        // Decodes of string tags of the inputs, compounds and other tags; encodes of compounds onto the empty prefix
        // mostly, and onto other tags.
        private static void codecs(final GoldenWriter writer, final String name, final Codec<CompoundTag> codec, final Set<String> inputs, final Random random) throws IOException {
            List<Tag> decoded = new ArrayList<>();
            List<CompoundTag> compounds = new ArrayList<>();
            for (String input : inputs) {
                if (!kept(input)) {
                    continue;
                }
                if (input.contains("{") || random.nextInt(8) == 0) {
                    decoded.add(StringTag.valueOf(input));
                }
                try {
                    if (PARSER.parseFully(input) instanceof CompoundTag compound && random.nextInt(2) == 0) {
                        compounds.add(compound);
                    }
                } catch (CommandSyntaxException e) {
                    // Only what parses is a compound to encode.
                }
            }
            for (int i = 0; i < 200; i++) {
                decoded.add(NbtValues.randomTag(random, 0));
                compounds.add(NbtValues.randomCompound(random, 0));
            }
            decoded.add(EndTag.INSTANCE);
            decoded.add(new CompoundTag());
            decoded.add(new ListTag());
            for (Tag tag : decoded) {
                JsonArray args = new JsonArray();
                args.add(NbtValues.encodeTag(tag));
                DataResult<JsonArray> result = codec.decode(NbtOps.INSTANCE, tag)
                    .map(pair -> DfuValues.pair(pair, NbtValues::encodeTag, NbtValues::encodeTag));
                JsonArray encoded = DfuValues.result(result, value -> value);
                if (GsonSuites.wellFormed(args) && GsonSuites.wellFormed(encoded)) {
                    writer.result(name + ".decode", args, encoded, null);
                }
            }
            for (CompoundTag compound : compounds) {
                Tag prefix = switch (random.nextInt(6)) {
                    case 0 -> new CompoundTag();
                    case 1 -> IntTag.valueOf(1);
                    case 2 -> NbtValues.randomTag(random, 0);
                    default -> EndTag.INSTANCE;
                };
                JsonArray args = new JsonArray();
                args.add(NbtValues.encodeTag(compound));
                args.add(NbtValues.encodeTag(prefix));
                JsonArray encoded = DfuValues.result(codec.encode(compound, NbtOps.INSTANCE, prefix), NbtValues::encodeTag);
                if (GsonSuites.wellFormed(args) && GsonSuites.wellFormed(encoded)) {
                    writer.result(name + ".encode", args, encoded, null);
                }
            }
        }

        // Whether an input goes in the goldens: no lone surrogate, and no \N{…}, whose names aren't ported.
        private static boolean kept(final String input) {
            return GsonSuites.wellFormed(input) && !input.contains("\\N");
        }

        private interface Parse {
            Tag run() throws CommandSyntaxException;
        }

        private static JsonArray parse(final Parse parse) {
            JsonArray result = new JsonArray();
            try {
                Tag tag = parse.run();
                result.add("tag");
                result.add(NbtValues.encodeTag(tag));
            } catch (CommandSyntaxException e) {
                result.add("error");
                result.add(e.getMessage());
            }
            return result;
        }

        private static JsonArray withCursor(final JsonArray result, final StringReader reader) {
            result.add(reader.getCursor());
            return result;
        }

        private static JsonArray args(final String input) {
            JsonArray args = new JsonArray();
            args.add(input);
            return args;
        }

        private static void record(final GoldenWriter writer, final String method, final JsonArray args, final JsonElement result) throws IOException {
            if (GsonSuites.wellFormed(args) && GsonSuites.wellFormed(result)) {
                writer.result(method, args, result, null);
            }
        }

        private static String mutate(final String input, final Random random) {
            StringBuilder builder = new StringBuilder(input);
            for (int i = 1 + random.nextInt(3); i > 0; i--) {
                int at = random.nextInt(builder.length() + 1);
                String mutation = MUTATION_PIECES.get(random.nextInt(MUTATION_PIECES.size()));
                switch (builder.isEmpty() ? 0 : random.nextInt(3)) {
                    case 0 -> builder.insert(at, mutation);
                    case 1 -> builder.deleteCharAt(Math.min(at, builder.length() - 1));
                    default -> builder.replace(Math.min(at, builder.length() - 1), Math.min(at, builder.length() - 1) + 1, mutation);
                }
            }
            return builder.toString();
        }

        // ---- Random SNBT ----

        private static String pick(final Random random, final String... options) {
            return options[random.nextInt(options.length)];
        }

        private static String space(final Random random) {
            return switch (random.nextInt(12)) {
                case 0 -> " ";
                case 1 -> "  ";
                case 2 -> "\n";
                case 3 -> "\t";
                default -> "";
            };
        }

        private static String randomDocument(final Random random, final int depth) {
            int kind = depth >= 4 ? random.nextInt(5) : random.nextInt(9);
            String value = switch (kind) {
                case 0 -> randomInteger(random);
                case 1 -> randomFloat(random);
                case 2 -> randomQuoted(random);
                case 3 -> randomUnquoted(random);
                case 4 -> random.nextInt(3) == 0 ? randomOperation(random) : randomInteger(random);
                case 5, 6 -> randomCompoundText(random, depth);
                case 7 -> randomListText(random, depth);
                default -> randomArrayText(random);
            };
            return space(random) + value + space(random);
        }

        private static String digits(final Random random, final String alphabet, final int max) {
            StringBuilder builder = new StringBuilder();
            int length = 1 + random.nextInt(max);
            for (int i = 0; i < length; i++) {
                builder.append(alphabet.charAt(random.nextInt(alphabet.length())));
                if (random.nextInt(12) == 0 && i + 1 < length) {
                    builder.append('_');
                }
            }
            return builder.toString();
        }

        private static String randomInteger(final Random random) {
            String sign = pick(random, "", "", "", "-", "+");
            String body = switch (random.nextInt(8)) {
                case 0 -> "0x" + digits(random, "0123456789abcdefABCDEF", 17);
                case 1 -> "0b" + digits(random, "01", 34);
                case 2 -> "0";
                case 3 -> String.valueOf(random.nextLong());
                case 4 -> String.valueOf(random.nextInt());
                case 5 -> String.valueOf(random.nextInt(300) - 150);
                default -> digits(random, "0123456789", 20);
            };
            String suffix = random.nextInt(2) == 0 ? "" : pick(random, "b", "B", "s", "S", "i", "I", "l", "L") ;
            if (!suffix.isEmpty() && random.nextInt(3) == 0) {
                suffix = pick(random, "u", "U", "s", "S") + suffix;
            }
            return sign + body + suffix;
        }

        private static String randomFloat(final Random random) {
            String sign = pick(random, "", "", "-", "+");
            String body = switch (random.nextInt(6)) {
                case 0 -> String.valueOf(random.nextDouble() * Math.pow(10, random.nextInt(40) - 20));
                case 1 -> String.valueOf(Float.intBitsToFloat(random.nextInt()));
                case 2 -> String.valueOf(Double.longBitsToDouble(random.nextLong()));
                case 3 -> digits(random, "0123456789", 6) + "." + digits(random, "0123456789", 6);
                case 4 -> "." + digits(random, "0123456789", 6);
                default -> digits(random, "0123456789", 4) + pick(random, "e", "E") + pick(random, "", "-", "+") + digits(random, "0123456789", 3);
            };
            String suffix = pick(random, "", "", "f", "F", "d", "D");
            return sign + body + suffix;
        }

        private static String randomQuoted(final Random random) {
            String quote = pick(random, "\"", "'");
            StringBuilder builder = new StringBuilder(quote);
            int length = random.nextInt(8);
            for (int i = 0; i < length; i++) {
                builder.append(switch (random.nextInt(20)) {
                    case 0 -> pick(random, "\\b", "\\s", "\\t", "\\n", "\\f", "\\r", "\\\\", "\\'", "\\\"");
                    case 1 -> "\\x" + String.format("%02x", random.nextInt(256));
                    case 2 -> "\\u" + String.format("%04X", random.nextInt(0xd800));
                    case 3 -> "\\U" + String.format("%08x", random.nextInt(0x110000));
                    case 4 -> quote.equals("\"") ? "'" : "\"";
                    case 5 -> " ";
                    case 6 -> "é";
                    case 7 -> "😀";
                    case 8 -> pick(random, "{", "}", "[", "]", ":", ",", ";");
                    default -> Character.toString('a' + random.nextInt(26));
                });
            }
            return builder.append(quote).toString();
        }

        private static String randomUnquoted(final Random random) {
            return switch (random.nextInt(6)) {
                case 0 -> pick(random, "true", "false", "TRUE", "False", "tRUE");
                case 1 -> pick(random, "minecraft", "stone", "a.b", "a_b", "A-B", "x+y", "NaN", "Infinity", "e");
                default -> {
                    StringBuilder builder = new StringBuilder();
                    builder.append("abcxyzABCXYZ_".charAt(random.nextInt(13)));
                    for (int i = random.nextInt(6); i > 0; i--) {
                        builder.append("abcxyzABCXYZ0123456789_-.+".charAt(random.nextInt(26)));
                    }
                    yield builder.toString();
                }
            };
        }

        private static String randomOperation(final Random random) {
            return switch (random.nextInt(7)) {
                case 0 -> "bool(" + space(random) + pick(random, "1", "0", "true", "false", "0.5", "-0.0", "256", "\"x\"", "1b", "[]") + space(random) + ")";
                case 1 -> "uuid(" + space(random) + "\"" + java.util.UUID.nameUUIDFromBytes(new byte[]{(byte)random.nextInt()}) + "\"" + ")";
                case 2 -> "uuid(" + pick(random, "\"1-2-3-4-5\"", "'0-0-0-0-0'", "\"x\"", "1", "abc", "\"\"") + ")";
                case 3 -> pick(random, "bool", "uuid", "foo", "Bool") + "(" + pick(random, "", "1", "1,2", "1,", "1,2,") + ")";
                case 4 -> "bool(1" ;
                default -> "bool(" + pick(random, "1", "0") + ")";
            };
        }

        private static String randomKeyText(final Random random) {
            return switch (random.nextInt(10)) {
                case 0 -> randomQuoted(random);
                case 1 -> pick(random, "Aa", "BB", "AaAa", "BBBB", "AaBB", "BBAa");
                case 2 -> pick(random, "\"\"", "''");
                case 3 -> pick(random, "1", "-a", "0x1", "true", "a.b", "+");
                default -> pick(random, "id", "Count", "tag", "x", "y", "z", "name", "Items", "a", "b", "c", "display", "Damage");
            };
        }

        private static String randomCompoundText(final Random random, final int depth) {
            StringBuilder builder = new StringBuilder("{");
            int size = random.nextInt(8) == 0 ? 10 + random.nextInt(30) : random.nextInt(5);
            for (int i = 0; i < size; i++) {
                if (i > 0) {
                    builder.append(',');
                }
                builder.append(space(random)).append(randomKeyText(random)).append(space(random)).append(':').append(randomDocument(random, depth + 1));
            }
            if (size > 0 && random.nextInt(8) == 0) {
                builder.append(',');
            }
            return builder.append(space(random)).append('}').toString();
        }

        private static String randomListText(final Random random, final int depth) {
            StringBuilder builder = new StringBuilder("[");
            int size = random.nextInt(6);
            boolean sameKind = random.nextInt(2) == 0;
            int kind = random.nextInt(3);
            for (int i = 0; i < size; i++) {
                if (i > 0) {
                    builder.append(',');
                }
                if (sameKind) {
                    builder.append(space(random)).append(switch (kind) {
                        case 0 -> randomInteger(random);
                        case 1 -> randomFloat(random);
                        default -> randomQuoted(random);
                    }).append(space(random));
                } else {
                    builder.append(randomDocument(random, depth + 1));
                }
            }
            if (size > 0 && random.nextInt(8) == 0) {
                builder.append(',');
            }
            return builder.append(']').toString();
        }

        private static String randomArrayText(final Random random) {
            StringBuilder builder = new StringBuilder("[").append(space(random)).append(pick(random, "B", "I", "L", "B", "I", "L", "b", "X")).append(space(random)).append(';');
            int size = random.nextInt(6);
            for (int i = 0; i < size; i++) {
                if (i > 0) {
                    builder.append(',');
                }
                builder.append(space(random)).append(random.nextInt(10) == 0 ? randomFloat(random) : randomInteger(random)).append(space(random));
            }
            return builder.append(']').toString();
        }

        // ---- Hand-written SNBT ----

        private static List<String> handWritten() {
            List<String> inputs = new ArrayList<>();
            // Integers: each type at its edges, in each base, with each suffix and signedness.
            inputs.addAll(List.of(
                "0", "1", "-1", "+1", "42", "-42", "2147483647", "2147483648", "-2147483648", "-2147483649", "4294967296",
                "1b", "127b", "128b", "-128b", "-129b", "1B", "255b", "1s", "32767s", "32768s", "-32768s", "-32769s", "1S",
                "1i", "1I", "2147483648i", "1l", "1L", "9223372036854775807l", "9223372036854775808l", "-9223372036854775808L",
                "-9223372036854775809L", "99999999999999999999l", "255ub", "256ub", "-1ub", "0ub", "0UB", "1uB", "1Ub", "65535us",
                "65536us", "-1us", "4294967295ui", "4294967296ui", "-1ui", "18446744073709551615ul", "18446744073709551616ul",
                "-1ul", "1sb", "1SB", "-1sb", "-128sb", "-129sb", "127sb", "128sb", "1ss", "32768ss", "1si", "1sl", "1us", "1ui",
                "1ul", "1u", "1su", "1uu", "1ubb", "1bs", "1bu", "1lb", "1f", "1d", "1D", "1F", "1e", "1x", "1g", "1 b", "1\tl",
                "1 u b", "1 s b", "- 1", "+ 1", "-+1", "+-1", "--1", "++1", "-", "+", "-b", "1-", "1+1"
            ));
            inputs.addAll(List.of(
                "0x0", "0x1", "0xff", "0xFF", "0X10", "0xA", "0xa", "0x7fffffff", "0x80000000", "0xffffffff", "0x100000000",
                "0xffffffffl", "0xffffffffffffffffL", "0x10000000000000000L", "0x7fffffffffffffffsl", "0x8000000000000000sl",
                "-0x8000000000000000sl", "-0x8000000000000001sl", "0x1_0", "0x_1", "0x1_", "0x1__2", "0x", "0xg", "0x 1", "0 x1",
                "0xffub", "0x100ub", "0xffsb", "0x7fsb", "0x80sb", "-0x80sb", "-0x81sb", "-0x1", "-0x1sb", "-0x1b", "+0x1",
                "0xFFFFus", "0x10000us", "0x8000ss", "0x7fffss", "-0x8000ss", "0xFFFFFFFFsi", "0x7FFFFFFFsi", "-0x80000000si",
                "0x1b", "0x1B", "0x1sb", "0xbbb", "0xbub", "0x1L", "0x1ul", "0x1s", "0xfs", "0xes", "0x1e5", "0x1.5", "0xu",
                "0b0", "0b1", "0B1", "0b101", "0b11111111ub", "0b100000000ub", "0b1sb", "0b1111111sb", "0b10000000sb", "-0b1sb",
                "-0b10000000sb", "-0b1", "0b", "0b2", "0b_1", "0b1_", "0b1_0", "0b1b", "0b1s", "0b1l", "0b1i", "0bl", "0bb",
                "0b11111111111111111111111111111111", "0b100000000000000000000000000000000",
                "0b1111111111111111111111111111111111111111111111111111111111111111L", "0b 1", "0 b1",
                "00", "01", "007", "0_1", "0_", "0__0", "-0", "+0", "-00", "0l", "0s", "0b", "0B", "0i", "0u", "0ub", "0sb", "00b",
                "1_000", "1__000", "_1", "1_", "_", "1_000_000l", "1_0b", "-1_0", "-_1", "1_b"
            ));
            // Floats: every form, at the types' edges and past them.
            inputs.addAll(List.of(
                "1.0", "1.", ".5", "-.5", "+.5", "-1.5", "1.5f", "1.5F", "1.5d", "1.5D", "1e5", "1E5", "1e+5", "1e-5", "1.5e10",
                "1.5e-10f", "1e5f", "1e5d", "1f", "1d", "-1d", "1e", "1e+", "1e-", "1e5.5", ".", "-.", "+.", ".e1", ".5e", ".5f",
                "1.e5", "1.f", "1..5", "1.5.5", "3.4028235E38f", "3.4028236E38f", "3.4028237E38f", "-3.4028237E38f", "1e38f",
                "1e39f", "1.4e-45f", "1.4E-45F", "7e-46f", "1e-46f", "1.7976931348623157e308", "1.7976931348623158e308",
                "1.7976931348623159e308", "1e308", "1e309", "-1e309", "4.9e-324", "2.5e-324", "1e-400", "0.0", "-0.0", "-0.0f",
                "0e0", "0.0e0", "-0e0", "1e99999999999999999999", "1e-99999999999999999999", "1.0000000000000000000000000001",
                "0.1f", "0.1", "0.30000000000000004", "123456789012345678901234567890.0", "123456789012345678901234567890.0f",
                "1" + "0".repeat(400) + ".0", "0." + "0".repeat(400) + "1", "1e0001", "1_0.0_1", "1_.5", "1._5", "1.5_", "1e1_0",
                "1e_10", "1e10_", "1_e10", "1.0e1_0f", "01.5", "00.5", "01e1", "0e0", "0f", "0d", "00f", "00d", "-0f", "0.f",
                "1 . 5", "1. 5", "1 .5", "1 e 5", "1e 5", "1e -5", "1.5 f", ". 5", "- .5", "1.5ff", "1.5fd", "1.5b", "1.5l",
                "1.5u", "1.5x", "1e5b", "1e5x", "1.0E+38", "NaN", "Infinity", "-Infinity", "nan", "inf", "1e5e5", "0x1p3", "1d5"
            ));
            // Booleans and unquoted strings.
            inputs.addAll(List.of(
                "true", "false", "TRUE", "True", "FALSE", "False", "tRuE", "truex", "true1", "true b", "falsey", "t", "f",
                "abc", "a_b-c.d+e", "A", "Z", "_", "_abc", "minecraft:stone", "-abc", "+abc", ".abc", "1abc", "é", "ab cd",
                "a b", "a\tb", "x", "e", "E", "b", "l", "abc123", "a.b.c", "a+", "a-", "a.", "a_", "abc\"", "abc'", "@e", "#tag",
                "a;b", "a(b", "a)b", "a=b", "a~b", "a!b", "a?b", "a&b", "a|b", "a/b", "a*b", "a%b", "a$b", "a^b", "a<b", "a>b"
            ));
            // Quoted strings and every escape.
            inputs.addAll(List.of(
                "\"\"", "''", "\"abc\"", "'abc'", "\"a'b\"", "'a\"b'", "\"a\\\"b\"", "'a\\'b'", "\"a\\\\b\"", "'a\\\\b'",
                "\"\\b\\s\\t\\n\\f\\r\"", "'\\b\\s\\t\\n\\f\\r'", "\"\\x41\"", "\"\\x4\"", "\"\\x4g\"", "\"\\x\"", "\"\\xff\"",
                "\"\\xFF\"", "\"\\x00\"", "\"\\x1f\"", "\"\\x7f\"", "\"\\x411\"", "\"\\u0041\"", "\"\\u004\"", "\"\\u00e9\"",
                "\"\\u00E9\"", "\"\\uffff\"", "\"\\uFFFE\"", "\"\\u0000\"", "\"\\u004g\"", "\"\\u\"", "\"\\U0001F600\"",
                "\"\\U0001f600\"", "\"\\U00110000\"", "\"\\U0010FFFF\"", "\"\\U0010ffff\"", "\"\\UFFFFFFFF\"", "\"\\U80000000\"",
                "\"\\U7fffffff\"", "\"\\U00000041\"", "\"\\U0000041\"", "\"\\U\"", "\"\\ud83d\\ude00\"", "\"\\uD83D\\uDE00\"",
                "'\\ud83d\\ude00'", "\"\\U0000d83d\\U0000de00\"", "\"\\ud83d\\ude00\\ud83d\\ude00\"", "\"\\xff\\u00ff\"",
                "\"\\q\"", "\"\\a\"", "\"\\0\"", "\"\\/\"", "\"\\ n\"", "\"\\  t\"", "\"\\\tn\"", "\"\\ x41\"", "\"\\x 41\"",
                "\"\\\"", "\"\\", "\"abc", "'abc", "\"abc'", "'abc\"", "\"日本語\"", "\"😀\"", "'😀'", "\"a😀b\"", "\"a\nb\"",
                "\"a\tb\"", "\"\\\\\"", "'\\\"'", "\"\\'\"", "\"''\"", "'\"\"'", "\" \"", "\"  a  \"", "\"\\\\\\\"\"",
                "\"a\" \"b\"", "\"a\"b", "'a'b", "\"\\N\"", "\"\\U0001F600x\"", "\"\\u0041\\u0042\\u0043\"", "\"\\x41\\x42\""
            ));
            // Operations.
            inputs.addAll(List.of(
                "bool(1)", "bool(0)", "bool(true)", "bool(false)", "bool(0.5)", "bool(-0.0)", "bool(0.0f)", "bool(256)",
                "bool(1b)", "bool(0L)", "bool(\"x\")", "bool([])", "bool({})", "bool(1,2)", "bool()", "bool(1,)", "bool",
                "bool(", "bool(1", "bool(1))", "Bool(1)", "BOOL(1)", "bool ( 1 )", "bool(\n1\n)", "bool(bool(1))",
                "bool(uuid(\"0-0-0-0-0\"))", "uuid(\"f81d4fae-7dec-11d0-a765-00a0c91e6bf6\")",
                "uuid(\"F81D4FAE-7DEC-11D0-A765-00A0C91E6BF6\")", "uuid('0-0-0-0-0')", "uuid(\"1-2-3-4-5\")", "uuid(\"x\")",
                "uuid(1)", "uuid(abc)", "uuid(\"+1-+2-+3-+4-+5\")", "uuid(\"-1--2-3-4-5\")", "uuid(\"ffffffff-ffff-ffff-ffff-ffffffffffff\")",
                "uuid(\"00000000000000000-0-0-0-0\")", "uuid(\"1-2-3-4-5-6\")", "uuid(\"1-2-3-4\")",
                "uuid(\"f81d4fae-7dec-11d0-a765-00a0c91e6bf6x\")", "uuid(\"f81d4fae-7dec-11d0-a765-00a0c91e6bfg\")",
                "uuid(\"123456789-1-1-1-1\")", "uuid(\"1-12345-1-1-1\")", "uuid(\"1-1-1-1-1234567890123\")", "uuid(\"\")",
                "uuid()", "uuid(\"0-0-0-0-0\",1)", "foo(1)", "foo()", "foo(1,2,3)", "foo(,)", "true(1)", "false()", "a.b(1)",
                "x(y(z(1)))", "uuid(\"\\x31-2-3-4-5\")", "uuid(\"١-2-3-4-5\")", "uuid(\"Ａ-2-3-4-5\")", "bool(1)x", "bool(1) "
            ));
            // Lists and arrays.
            inputs.addAll(List.of(
                "[]", "[ ]", "[\n]", "[1]", "[1,2,3]", "[1,]", "[1, ]", "[,]", "[,1]", "[1,,2]", "[1 2]", "[1b,2s]", "[1,\"a\"]",
                "[1,1.5,true,\"s\",[],{}]", "[[]]", "[[1],[2]]", "[[1],[2b]]", "[{}]", "[{a:1},{b:2}]", "[{a:1},2]", "[1",
                "[", "]", "[1]]", "[[1]", "[B]", "[I]", "[L]", "[B;]", "[B;1b,2b]", "[B;1,2]", "[B;1s]", "[B;1i]", "[B;1l]",
                "[B;128]", "[B;127]", "[B;-128]", "[B;-129]", "[B;255ub]", "[B;256ub]", "[B;-1]", "[B;0xff]", "[B;0x7f]",
                "[B;0x80sb]", "[B;0b11111111]", "[B;true]", "[B;1.0]", "[B;1f]", "[B;a]", "[B;\"a\"]", "[B;[]]", "[I;]",
                "[I;1,2,3]", "[I;1b,1s,1i]", "[I;1l]", "[I;0xffffffff]", "[I;0x100000000]", "[I;4294967295ui]", "[I;-1ui]",
                "[I;9999999999]", "[I;2147483648]", "[I;-2147483648]", "[I;255ub]", "[I;-1sb]", "[I;65535us]", "[I;a]",
                "[I;1,]", "[I;,]", "[I;,1]", "[I;1 2]", "[I;1.5]", "[I; 0b101, 0x1F]", "[I;-0x1]", "[I;+1]", "[L;]",
                "[L;1,2]", "[L;1b,1s,1i,1l]", "[L;255ub]", "[L;-1sb]", "[L;1.0]", "[L;9223372036854775808]",
                "[L;9223372036854775807]", "[L;0xffffffffffffffff]", "[L;18446744073709551615ul]", "[L;1f]", "[b;1]", "[i;1]",
                "[l;1]", "[X;1]", "[B ; 1b]", "[ B;1b ]", "[ L;1 ]", "[B;1b", "[B;", "[B", "[B;1b]]", "[I;[1]]", "[B;1b,]",
                "[B:1]", "[B,1]", "[BB;1]", "[B;;1]", "[B;1;2]", "[I;1,2,]", "[\"B\";1]", "['a','b']", "[a,b,c]", "[true,false]",
                "[I;1b,2s,3i,4l]", "[L;1,2.0]", "[1.0f,2.0d]", "[[],[[]],[[[]]]]", "[[I;1],[B;1b],[L;1l]]", "[B;-0]", "[I;00]",
                "[I;01]", "[I;0_1]", "[I;1_]"
            ));
            // Compounds.
            inputs.addAll(List.of(
                "{}", "{ }", "{\n}", "{a:1}", "{a:1,}", "{a:1, }", "{a:1,b:2}", "{\"a\":1}", "{'a':1}", "{\"\":1}", "{'':1}",
                "{a:1,a:2}", "{a:1,b:2,a:3}", "{a:1,b:2,c:3,b:4}", "{a:}", "{:1}", "{a 1}", "{a=1}", "{a:1 b:2}", "{a:1",
                "{a", "{a:", "{", "}", "{a:{b:{c:1}}}", "{1:2}", "{-a:1}", "{+a:1}", "{.a:1}", "{a.b:1}", "{a+b:1}", "{a-b:1}",
                "{é:1}", "{\"é\":1}", "{\"a b\":1}", "{a b:1}", "{a:[1,2],b:[B;1b]}", "{a:1,,b:2}", "{,}", "{,a:1}",
                "{true:false}", "{a:bool(1)}", "{a:uuid(\"0-0-0-0-0\")}", "{\"\\u0041\":1}", "{'\\x41':1}", "{\"\\ud83d\\ude00\":1}",
                "{a:1}}", "{a:1}x", "{a:1} ", " {a:1}", "{ a : 1 , b : 2 }", "{\ta\t:\t1\t}", "{a:'b'}", "{a:\"b\",c:'d'}",
                "{Aa:1,BB:2}", "{BB:1,Aa:2}", "{Aa:1,BB:2,Aa:3}", "{AaAa:1,BBBB:2,AaBB:3,BBAa:4}", "{BBAa:1,AaBB:2,BBBB:3,AaAa:4}",
                "{id:\"minecraft:stone\",Count:1b,tag:{display:{Name:'{\"text\":\"x\"}'}}}", "{\"a\":1,'a':2}", "{0x1:1}",
                "{1b:1}", "{1.5:1}", "{a:-}", "{a:+}", "{a:.}", "{a:1.5.5}", "{a:[}", "{a:]}", "{a:{}", "{a:{}}}", "{[]:1}",
                "{{}:1}", "{a:1;b:2}", "{a::1}", "{a:1:2}", "{a\"b\":1}", "{\"a\"b:1}", "{a:b c}", "{a:\"b\" c}"
            ));
            // Compounds whose tables grow, with keys put in orders the bins keep.
            StringBuilder big = new StringBuilder("{");
            StringBuilder bigReversed = new StringBuilder("{");
            for (int i = 0; i < 40; i++) {
                big.append(i == 0 ? "" : ",").append("k").append(i).append(':').append(i);
                bigReversed.append(i == 0 ? "" : ",").append("k").append(39 - i).append(':').append(i);
            }
            inputs.add(big.append('}').toString());
            inputs.add(bigReversed.append('}').toString());
            // Eight keys of one hash: a ninth would make a tree of their bin, which the port's HashMap doesn't have.
            StringBuilder colliding = new StringBuilder("{");
            for (int i = 0; i < 8; i++) {
                colliding.append(i == 0 ? "" : ",").append(Integer.toBinaryString(i | 16).substring(1).replace("0", "Aa").replace("1", "BB")).append(':').append(i);
            }
            inputs.add(colliding.append('}').toString());
            // Nesting, as deep as the encoded tags stay within the JSON reader's limit of 128, and deeper where the
            // parse fails.
            for (int depth : new int[]{2, 5, 10, 20}) {
                inputs.add("[".repeat(depth) + "1" + "]".repeat(depth));
                inputs.add("{a:".repeat(depth) + "1" + "}".repeat(depth));
                inputs.add("[".repeat(depth) + "]".repeat(depth - 1));
                inputs.add("[{a:".repeat(depth / 2) + "1" + "}]".repeat(depth / 2));
            }
            for (int depth : new int[]{40, 100}) {
                inputs.add("[".repeat(depth) + "]".repeat(depth - 1) + "}");
                inputs.add("{a:".repeat(depth) + "1" + "}".repeat(depth - 1) + "]");
                inputs.add("[{a:".repeat(depth) + "1" + "}]".repeat(depth - 1) + "]");
            }
            // Whitespace, of every kind Character.isWhitespace takes and some it doesn't.
            inputs.addAll(List.of(
                "", " ", "  ", "\t", "\n", " 1", "1 ", " 1 ", "\t1\n", "\r\n1\r\n", "\u000b1", "\u000c1", "\u001c1", "\u001f1",
                "\u00a01", "\u20071", "\u202f1", "\u20281", "\u20291", "\u30001", "\u20001", "\u200a1", "\u16801", "\u205f1",
                "\u00851", "\ufeff1", "1\u3000", "[\u30001\u3000]", "{\u2028a\u2028:\u20281\u2028}", "\"\\\u3000n\""
            ));
            // Other characters where values go.
            inputs.addAll(List.of(
                "\"", "'", ",", ":", ";", "(", ")", "@", "#", "\\", "=", "~", "^", "!", "?", "*", "&", "%", "$", "/", "<", ">",
                "|", "`", "😀", "é", "日", "\u0000", "\u007f", "a\u0000", "1\u0000", "\"\u0000\""
            ));
            return inputs;
        }
    }
}
