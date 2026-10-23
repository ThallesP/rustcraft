package rustcraft.golden;

import com.google.gson.Gson;
import com.google.gson.GsonBuilder;
import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonNull;
import com.google.gson.JsonObject;
import com.google.gson.JsonParser;
import com.google.gson.JsonPrimitive;
import com.google.gson.Strictness;
import com.google.gson.internal.LazilyParsedNumber;
import com.google.gson.stream.JsonReader;
import com.google.gson.stream.JsonToken;
import java.io.IOException;
import java.io.StringReader;
import java.math.BigDecimal;
import java.util.ArrayList;
import java.util.LinkedHashMap;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Map;
import java.util.Random;
import java.util.Set;
import java.util.function.Supplier;
import net.minecraft.util.StrictJsonParser;

/**
 * Gson, as the game reads JSON from the network leniently ({@code JsonParser.parseString}, which {@code
 * LenientJsonParser} and {@code ByteBufCodecs.lenientJson} call), strictly ({@code StrictJsonParser}), and writes it
 * ({@code toJson} of a Gson with HTML escaping off, {@code JsonElement.toString()}).
 *
 * <p>An element is {@code ["null"]}, {@code ["boolean", b]}, {@code ["number", box, toString]} (the box's simple name),
 * {@code ["string", s]}, {@code ["array", [element, …]]} or {@code ["object", [[name, element], …]]} in the object's
 * order. A parsed element nested deeper than {@link #MAX_ENCODED_DEPTH} is {@code ["deep", depth, toString]}, since
 * serde_json's recursion limit is 128. A thrown exception is recorded as the case's class, and its class and message
 * again under the method's name and " message", so messages are checked too.
 *
 * <p>The golden files are UTF-8, which can't hold a lone surrogate, so cases whose input, result or message has one (a
 * {@code \ud800} escape, a mutation splitting a pair) are left out.
 */
final class GsonSuites {
    private static final int MAX_ENCODED_DEPTH = 40;
    private static final String PARSE_STRING = "parseString(Ljava/lang/String;)Lcom/google/gson/JsonElement;";
    private static final String STRICT_PARSE = "StrictJsonParser.parse(Ljava/lang/String;)Lcom/google/gson/JsonElement;";
    private static final String TO_JSON = "toJson(Lcom/google/gson/JsonElement;)Ljava/lang/String;";
    private static final String HTML_SAFE_TO_JSON = "new Gson().toJson(Lcom/google/gson/JsonElement;)Ljava/lang/String;";
    private static final String TO_STRING = "toString()Ljava/lang/String;";
    private static final String EQUALS = "equals(Ljava/lang/Object;)Z";
    private static final Gson LENIENT_JSON_GSON = new GsonBuilder().disableHtmlEscaping().create();
    private static final Gson DEFAULT_GSON = new Gson();
    private static final String STRICT_TO_JSON = "[STRICT]toJson(Lcom/google/gson/JsonElement;)Ljava/lang/String;";
    private static final Gson STRICT_GSON = new GsonBuilder().setStrictness(Strictness.STRICT).create();
    private static final String NULLS_TO_JSON = "[serializeNulls, generateNonExecutableJson]toJson(Lcom/google/gson/JsonElement;)Ljava/lang/String;";
    private static final Gson NULLS_GSON = new GsonBuilder().serializeNulls().generateNonExecutableJson().create();

    private GsonSuites() {
    }

    static List<Suite> suites() {
        return List.of(new ParserSuite(), new ReaderSuite(), new ElementSuite(), new GsonSuite(), new PrimitiveSuite());
    }

    // ---- Inputs ----

    // Valid JSON of every kind: literals, numbers at int's, long's, float's and double's edges and past them, big
    // integers, huge exponents, every escape, non-BMP characters, and nested and repeated members.
    private static final List<String> VALID = List.of(
        "true", "false", "null", " true ", "\ttrue\n", "\r\n null \r\n",
        "0", "-0", "1", "-1", "0.0", "-0.0", "1.5", "-1.5", "1e3", "1E3", "1e+3", "1e-3", "1E-3", "0e0", "0E-0", "0.0e+00", "-0e-0",
        "123", "1234567890", "2147483647", "2147483648", "-2147483648", "-2147483649", "4294967296",
        "9223372036854775807", "9223372036854775808", "-9223372036854775808", "-9223372036854775809", "922337203685477580",
        "-922337203685477580", "-922337203685477581", "-9223372036854775800", "-9223372036854775810",
        "18446744073709551616", "123456789012345678901234567890", "-123456789012345678901234567890",
        "0.1", "0.30000000000000004", "1.7976931348623157e308", "1.7976931348623158e308", "1.8e308", "4.9e-324", "2.4e-324",
        "2.5e-324", "5e-324", "1e-400", "2.2250738585072014E-308", "2.2250738585072011E-308", "3.4028235e38", "3.4028236e38",
        "1.4e-45", "1e309", "-1e309", "1e99999", "1e-99999", "1e10000", "1e9999", "1e-9999", "1e-10000", "1e2147483647",
        "1e2147483648", "1e-2147483648", "1e-2147483649", "1e00000000000000000001",
        "100000000000000000000000000000000000000000000000000000000000000000000000000000000",
        "0.000000000000000000000000000000000000000000000000000000000000000000000000000001",
        "9007199254740993", "9007199254740992.5", "1.0000000000000002", "123.456e-7", "12.5E-3", "100", "1.50", "-1.0e+10",
        "\"\"", "\"a\"", "\"abc def\"", "\"é\"", "\"日本語\"", "\"😀\"", "\"\\uD83D\\uDE00\"", "\"\\ud83d\\ude00\"", "\"\\u00e9\"",
        "\"\\u00E9\"", "\"\\u0000\"", "\"\\u001f\"", "\"\\u001F\"", "\"\\uFFFF\"", "\"\\u2028\\u2029\"", "\"\u2028\"", "\"\\uFEFF\"",
        "\"\\\"\"", "\"\\\\\"", "\"\\/\"", "\"\\b\\f\\n\\r\\t\"", "\"a\\nb\"", "\"tab\\there\"", "\"<>&='\"", "\"</script>\"",
        "\"\\u0041\\u0062\\u0063\"", "\"\\\\u0041\"", "\"a\\\\\"", "\"\\\"\\\"\"", "\"x😀y😀z\"",
        "[]", "[ ]", "[1]", "[1,2,3]", "[ 1 , 2 , 3 ]", "[true,false,null]", "[\"a\",1,true,null,[],{}]", "[[[]]]",
        "[[1],[2,[3]]]", "[-0,0.0,-0.0]", "[null]", "[null,null]", "[[],[]]", "[{},{}]",
        "{}", "{ }", "{\"a\":1}", "{\"a\":1,\"b\":2}", "{ \"a\" : 1 , \"b\" : [ 2 ] }", "{\"\":0}", "{\"a\":{\"b\":{\"c\":null}}}",
        "{\"a\":1,\"a\":2}", "{\"a\":1,\"b\":2,\"a\":3}", "{\"b\":1,\"a\":2}", "{\"\\u0061\":1,\"a\":2}", "{\"é\":\"😀\"}",
        "{\"a\\nb\":1}", "{\"a\":null}", "{\"a\":null,\"b\":[null,{\"c\":null}]}", "{\"a\":{},\"a\":[]}", "{\"a\":[1],\"b\":{\"a\":2},\"a\":3}",
        "{\"name\":\"Minecraft\",\"version\":{\"name\":\"26.3\",\"protocol\":777},\"players\":{\"max\":20,\"online\":0,"
            + "\"sample\":[{\"name\":\"Steve\",\"id\":\"8667ba71-b85a-4004-af54-457a9734eed7\"}]},"
            + "\"description\":{\"text\":\"A Minecraft Server\",\"color\":\"gold\",\"extra\":[\"§a§lHi\"]},\"enforcesSecureChat\":true}",
        "{\n  \"a\": [\n    1,\n    2\n  ],\n  \"b\": \"c\"\n}", "[\r\n1\r\n]", "\n\n\n[\n\n1\n\n]\n\n"
    );

    // Gson's lenient extensions: comments, unquoted and single-quoted names and values, = and => separators, ; between
    // members and elements, the non-execute prefix, NaN and infinities (which read as unquoted strings), missing array
    // values, keywords in any case, legacy escapes and raw control characters; and what's past them.
    private static final List<String> LENIENT = List.of(
        "// comment\n1", "# comment\n1", "/* comment */1", "/**/1", "/* a */ /* b */ [1, /* c */ 2]", "[1, // c\n2]",
        "[1 # c\n,2]", "{/*x*/\"a\"/*y*/:/*z*/1}", "/* multi\nline\ncomment */[1]", "/* * / ** */1", "/*/1", "/*", "/**",
        "1 // trailing", "1 /* trailing */", "1 # trailing", "/* unterminated", "/* unterminated *", "//", "#", "/", "/x",
        "[/]", "[1/2]", "//\r1", "// a\r\n1", "# a\r1", "[1//\n]", "[1/**/]", "{\"a\"://c\n1}", "[/*\n*/\nx y]",
        "{a:1}", "{a:b}", "{abc:def}", "{a b:1}", "{1:2}", "{true:false}", "{null:null}", "{-1:1}", "{a.b:c.d}", "{$:@}",
        "{a:1,b:2}", "{é:1}", "{😀:😀}", "{a\\b:1}", "{a#b:1}", "{a/b:1}",
        "[abc]", "[a b]", "[a,b]", "[NaN]", "[Infinity]", "[-Infinity]", "NaN", "Infinity", "-Infinity", "[nan]", "[+1]",
        "[.5]", "[5.]", "[1e]", "[1e+]", "[-]", "[--1]", "[1.2.3]", "[0x10]", "[1f]", "[01]", "[-01]", "[00]", "[0.]",
        "[1_000]", "[1e5x]", "[1-2]", "[1+2]", "[1ee5]", "[1e5.5]", "[-.5]", "[0e]", "[0.e1]", "abc", "a b", "x",
        "['a']", "'a'", "{'a':'b'}", "['a\\'b']", "['a\"b']", "[\"a\\'b\"]", "['\\u0041']", "['unterminated]", "''", "{'':''}",
        "{\"a\"=1}", "{\"a\"=>1}", "{\"a\"= >1}", "{\"a\":1;\"b\":2}", "[1;2]", "[1;;2]", "{\"a\"=1;\"b\"=>2}", "{a=b;c=>d}",
        "{\"a\"=", "{\"a\"=>", "{a=>}", "[1=2]",
        ")]}'\n[1]", ")]}'\n", ")]}'\n{\"a\":1}", " )]}'\n1", ")]}'[1]", ")]}' \n1", ")]}", ")]}'", "\n)]}'\n1", ")]}'\n)]}'\n1",
        ")]}'\r\n1",
        "1 2", "[] []", "{} {}", "true false", "null x", "null null", "null 1", "null [", "1 x", "\"a\" \"b\"", "[1]]", "{}}", "1,",
        "1;", "[1],", "[1] // c", "[1] /*", "1\n2",
        "[1,,2]", "[,]", "[,1]", "[1,]", "[,,]", "[;]", "[1,;2]", "[ , ]", "[\n,\n]", "{\"a\":[,]}", "{\"a\":1,}", "{,}",
        "{\"a\":1,,\"b\":2}", "[,,,,]", "[[,],[,]]",
        "TRUE", "True", "tRuE", "FALSE", "False", "NULL", "Null", "nUlL", "[TRUE,FALSE,NULL]", "truex", "nullx", "falsey", "tru",
        "nul", "t", "n", "f", "[true1]", "true1", "[true,]", "[null;]", "nulL", "TRUEx", "[falsE]",
        "\ufeff[1]", "\ufeff", "\ufeff\ufeff1", " \ufeff1", "\ufeff 1", "\ufeff\ufeff", "[\ufeff]",
        "\"a\\\nb\"", "\"a\\'b\"", "\"a\tb\"", "\"a\u0000b\"", "\"a\u001fb\"", "\"line\nbreak\"", "\"\r\"", "{\"a\nb\":1}",
        "'a\\\nb'", "'\u0001'", "{\"a\u0007\":1}",
        "[1\n,\n2\n,\nx y]", "{\n\"a\":\n1\n\"b\"}", "{\"a\":1}\n{\"b\":2}", "[1,\n2,\n3,\n4,\n5,\n]x"
    );

    // Malformed input of every kind: unterminated strings, arrays and objects, bad escapes, misplaced separators, values
    // where names go, and empty documents.
    private static final List<String> MALFORMED = List.of(
        "", " ", "\n", "\t\r\n ", "[", "[1", "[1,", "[1,2", "{", "{\"a\"", "{\"a\":", "{\"a\":1", "{\"a\":1,", "\"", "\"abc",
        "\"\\", "\"\\u", "\"\\u1", "\"\\u12", "\"\\u123", "\"\\u1234", "\"\\u12G4\"", "\"\\uXYZW\"", "\"\\x\"", "\"\\a\"",
        "\"\\U0041\"", "\"\\0\"", "\"\\u-123\"", "\"\\u 123\"", "\"\\u12😀\"", "\"\\uéééé\"", "'\\x'", "{\"\\x\":1}",
        "[1}", "{\"a\":1]", "]", "}", ":", ",", ";", "=", "{\"a\" 1}", "{\"a\"}", "{\"a\",1}", "[1 2]", "[1:2]",
        "{\"a\":1 \"b\":2}", "{\"a\":1 2}", "[\"a\" \"b\"]", "{[]:1}", "{{}:1}", "{\"a\"::1}", "{:1}", "[:]", "{,\"a\":1}",
        "{\"a\":}", "{\"a\":,}", "[}", "{]", "[{]}", "{\"a\":[}", "[\"a\":1]", "{\"a\":1}}", "[[]]]", "{}]", "{\"a\"=}",
        "\\", "@", "\u0000", "\u0001", "[\u0000]", "\u00e9", "😀", "[😀]", "{\"a\":😀}", "\u2028", "[\u2028]", "\u00a0",
        "[\u00a0]", "\f", "[\f1]", "\u000b1", "{\"a\":1\f}", "[1\u000b]"
    );

    // The names of random documents' members.
    private static final List<String> NAMES = List.of("a", "b", "text", "color", "", "é", "extra", "with space", "\\\"q\\\"", "😀");

    static List<String> parserInputs() {
        Set<String> inputs = new LinkedHashSet<>();
        inputs.addAll(VALID);
        inputs.addAll(LENIENT);
        inputs.addAll(MALFORMED);
        inputs.addAll(deepInputs());
        inputs.addAll(bufferInputs());
        inputs.addAll(lineInputs());
        Random random = new Random(47);
        List<String> documents = new ArrayList<>();
        for (int i = 0; i < 300; i++) {
            documents.add(randomDocument(random, 0));
        }
        inputs.addAll(documents);
        List<String> seeds = new ArrayList<>(VALID);
        seeds.addAll(LENIENT);
        seeds.addAll(documents);
        for (int i = 0; i < 2000; i++) {
            inputs.add(mutate(seeds.get(random.nextInt(seeds.size())), random));
        }
        inputs.removeIf(input -> !wellFormed(input));
        return new ArrayList<>(inputs);
    }

    // Nesting around the reader's limit of 255, closed and not, arrays and objects and both.
    private static List<String> deepInputs() {
        List<String> inputs = new ArrayList<>();
        for (int depth : new int[]{50, 100, 254, 255, 256, 300}) {
            inputs.add("[".repeat(depth) + "]".repeat(depth));
            inputs.add("[".repeat(depth));
            inputs.add("[".repeat(depth) + "1" + "]".repeat(depth));
            inputs.add("{\"a\":".repeat(depth - 1) + "{}" + "}".repeat(depth - 1));
            inputs.add("[{\"a\":".repeat(depth / 2) + "1" + "}]".repeat(depth / 2));
            inputs.add("{a:[".repeat(depth / 2) + "]}".repeat(depth / 2));
        }
        return inputs;
    }

    // Tokens that straddle the reader's 1024-char buffer, after whitespace that fills it, and literals, strings and
    // numbers around its length: a number of 1024 chars or more is an unquoted string to the reader.
    private static List<String> bufferInputs() {
        List<String> inputs = new ArrayList<>();
        List<String> tokens = List.of(
            "true", "null", "false", "TRUE", "1234567", "-0.5e-3", "\"abc\"", "\"\\u00e9\\n\"", "\"a😀b\"", "/*c*/1", "//c\n1",
            "#c\n1", ")]}'\n1", "[1,2]", "{\"a\"=>1}", "'x'", "abc", "[,]", "\ufeff1", "x y", "\"\\u12", "1 2", "[1]/*", "nul"
        );
        for (int pad = 1016; pad <= 1028; pad++) {
            String spaces = " ".repeat(pad);
            for (String token : tokens) {
                inputs.add(spaces + token);
            }
            inputs.add("[" + "1,".repeat(pad / 2) + "x y]");
        }
        for (int length = 1018; length <= 1030; length++) {
            inputs.add("\"" + "a".repeat(length) + "\"");
            inputs.add("\"" + "a".repeat(length) + "\\nb\"");
            inputs.add("\"" + "a".repeat(length) + "\\u00e9\"");
            inputs.add("1".repeat(length));
            inputs.add("[" + "1".repeat(length) + "]");
            inputs.add("-" + "1".repeat(length));
            inputs.add("0." + "1".repeat(length));
            inputs.add("a".repeat(length));
            inputs.add("[" + "a".repeat(length) + "]");
            inputs.add("{" + "a".repeat(length) + ":1}");
            inputs.add("/*" + "c".repeat(length) + "*/1");
        }
        inputs.add("1".repeat(2050));
        inputs.add("a".repeat(2050));
        inputs.add("[" + "a".repeat(3000) + ",1]");
        inputs.add("\"" + "é".repeat(2500) + "\"");
        inputs.add("\"" + "😀".repeat(1500) + "\"");
        inputs.add("[" + "\"x\",".repeat(600) + "1]");
        inputs.add("[" + "\"x\",".repeat(600) + "x y]");
        return inputs;
    }

    // Errors after newlines, carriage returns and non-BMP characters, whose line and column the messages give.
    private static List<String> lineInputs() {
        return List.of(
            "{\n  \"a\": 1,\n  \"b\": x y\n}", "[\n\n\n\nx y]", "[\r\n\r\nx y]", "[\r\rx y]", "[\"😀\", x y]", "[\"😀😀\",\nx y]",
            "{\n\"a\":\"b\nc\" d}", "// c\n// d\n[x y]", "/* \n \n */ [x y]", "[\"a\\\nb\", x y]", "\n\n\n", "[\n1,\n2\n",
            "{\"😀\":[1,\n{\"é\":x y}]}", "[\"abc\",\n\t\"def\" \"ghi\"]"
        );
    }

    // A random document as text: values nested a few levels, with whitespace between tokens now and then.
    private static String randomDocument(final Random random, final int depth) {
        int kind = depth > 3 ? random.nextInt(5) : random.nextInt(7);
        String space = random.nextInt(4) == 0 ? List.of(" ", "\n", "\t", "\r\n", "  ").get(random.nextInt(5)) : "";
        return space + switch (kind) {
            case 0 -> List.of("true", "false", "null").get(random.nextInt(3));
            case 1, 2 -> randomNumber(random);
            case 3, 4 -> randomString(random);
            case 5 -> {
                StringBuilder builder = new StringBuilder("[");
                for (int i = random.nextInt(5); i > 0; i--) {
                    builder.append(randomDocument(random, depth + 1));
                    if (i > 1) {
                        builder.append(',');
                    }
                }
                yield builder.append(space).append(']').toString();
            }
            default -> {
                StringBuilder builder = new StringBuilder("{");
                for (int i = random.nextInt(5); i > 0; i--) {
                    builder.append(space).append('"').append(NAMES.get(random.nextInt(NAMES.size()))).append("\":");
                    builder.append(randomDocument(random, depth + 1));
                    if (i > 1) {
                        builder.append(',');
                    }
                }
                yield builder.append(space).append('}').toString();
            }
        } + space;
    }

    private static String randomNumber(final Random random) {
        return switch (random.nextInt(6)) {
            case 0 -> Integer.toString(random.nextInt());
            case 1 -> Long.toString(random.nextLong());
            case 2 -> Double.toString(random.nextDouble() * Math.pow(10, random.nextInt(40) - 20)).replace("E", random.nextBoolean() ? "e" : "E");
            case 3 -> Integer.toString(random.nextInt(100));
            case 4 -> (random.nextBoolean() ? "-" : "") + random.nextInt(1000) + "." + random.nextInt(1000) + "e" + (random.nextInt(800) - 400);
            default -> new BigDecimal(random.nextLong()).multiply(new BigDecimal(random.nextLong())).toString();
        };
    }

    private static String randomString(final Random random) {
        StringBuilder builder = new StringBuilder("\"");
        for (int i = random.nextInt(8); i > 0; i--) {
            builder.append(switch (random.nextInt(12)) {
                case 0 -> "\\n";
                case 1 -> "\\\"";
                case 2 -> "\\\\";
                case 3 -> String.format("\\u%04x", random.nextInt(0xd800));
                case 4 -> "é";
                case 5 -> "😀";
                case 6 -> "\\/";
                case 7 -> "\\t";
                default -> Character.toString('a' + random.nextInt(26));
            });
        }
        return builder.append('"').toString();
    }

    // One to three characters inserted, deleted or replaced, from the characters JSON gives meaning to and a few others.
    private static final List<String> MUTATIONS = List.of(
        "{", "}", "[", "]", ":", ",", "\"", "'", "\\", "/", "*", "#", ";", "=", ">", " ", "\n", "\t", "\r", "0", "1", "9", "e",
        "E", "+", "-", ".", "a", "t", "f", "n", "u", "l", "r", "N", "I", "x", "é", "😀", "\ufeff", "\u2028", "\u0000", ")", "u"
    );

    private static String mutate(final String document, final Random random) {
        StringBuilder builder = new StringBuilder(document);
        for (int i = 1 + random.nextInt(3); i > 0; i--) {
            int at = random.nextInt(builder.length() + 1);
            String mutation = MUTATIONS.get(random.nextInt(MUTATIONS.size()));
            switch (builder.isEmpty() ? 0 : random.nextInt(3)) {
                case 0 -> builder.insert(at, mutation);
                case 1 -> builder.deleteCharAt(Math.min(at, builder.length() - 1));
                default -> builder.replace(Math.min(at, builder.length() - 1), Math.min(at, builder.length() - 1) + 1, mutation);
            }
        }
        return builder.toString();
    }

    // ---- Encodings ----

    static JsonArray encode(final JsonElement element) {
        JsonArray array = new JsonArray();
        if (element.isJsonNull()) {
            array.add("null");
        } else if (element.isJsonPrimitive()) {
            JsonPrimitive primitive = element.getAsJsonPrimitive();
            if (primitive.isBoolean()) {
                array.add("boolean");
                array.add(primitive.getAsBoolean());
            } else if (primitive.isNumber()) {
                Number number = primitive.getAsNumber();
                array.add("number");
                array.add(number.getClass().getSimpleName());
                array.add(number.toString());
            } else {
                array.add("string");
                array.add(primitive.getAsString());
            }
        } else if (element.isJsonArray()) {
            JsonArray elements = new JsonArray();
            element.getAsJsonArray().forEach(child -> elements.add(encode(child)));
            array.add("array");
            array.add(elements);
        } else {
            JsonArray members = new JsonArray();
            for (Map.Entry<String, JsonElement> entry : element.getAsJsonObject().entrySet()) {
                members.add(RenderSuites.array(new JsonPrimitive(entry.getKey()), encode(entry.getValue())));
            }
            array.add("object");
            array.add(members);
        }
        return array;
    }

    // A parsed element, shallow enough for serde_json, or its depth and text.
    private static JsonArray encodeParsed(final JsonElement element) {
        int depth = depth(element);
        if (depth <= MAX_ENCODED_DEPTH) {
            return encode(element);
        }
        return RenderSuites.array(new JsonPrimitive("deep"), new JsonPrimitive(depth), new JsonPrimitive(element.toString()));
    }

    private static int depth(final JsonElement element) {
        int depth = 0;
        if (element.isJsonArray()) {
            for (JsonElement child : element.getAsJsonArray()) {
                depth = Math.max(depth, depth(child));
            }
            return depth + 1;
        }
        if (element.isJsonObject()) {
            for (JsonElement child : element.getAsJsonObject().asMap().values()) {
                depth = Math.max(depth, depth(child));
            }
            return depth + 1;
        }
        return 0;
    }

    static boolean wellFormed(final String string) {
        for (int i = 0; i < string.length(); i++) {
            char c = string.charAt(i);
            if (Character.isHighSurrogate(c) && i + 1 < string.length() && Character.isLowSurrogate(string.charAt(i + 1))) {
                i++;
            } else if (Character.isSurrogate(c)) {
                return false;
            }
        }
        return true;
    }

    // Every string in an encoding (names, strings, number texts, messages) without a lone surrogate.
    static boolean wellFormed(final JsonElement encoded) {
        if (encoded.isJsonArray()) {
            for (JsonElement child : encoded.getAsJsonArray()) {
                if (!wellFormed(child)) {
                    return false;
                }
            }
            return true;
        }
        if (encoded.isJsonObject()) {
            for (JsonElement child : encoded.getAsJsonObject().asMap().values()) {
                if (!wellFormed(child)) {
                    return false;
                }
            }
            return true;
        }
        return !encoded.isJsonPrimitive() || !encoded.getAsJsonPrimitive().isString() || wellFormed(encoded.getAsString());
    }

    private static JsonArray thrown(final Throwable thrown) {
        return RenderSuites.array(new JsonPrimitive(thrown.getClass().getName()), new JsonPrimitive(String.valueOf(thrown.getMessage())));
    }

    // A case's result, or its exception and the exception's class and message under "<method> message".
    private static void record(
        final GoldenWriter writer, final String method, final JsonArray args, final Supplier<JsonElement> call
    ) throws IOException {
        JsonElement result;
        try {
            result = call.get();
        } catch (RuntimeException e) {
            JsonArray message = thrown(e);
            if (wellFormed(message)) {
                writer.threw(method, args, e, null);
                writer.result(method + " message", args, message, null);
            }
            return;
        }
        if (wellFormed(result)) {
            writer.result(method, args, result, null);
        }
    }

    // ---- Suites ----

    /**
     * {@code JsonParser.parseString} of every input: valid JSON, the lenient extensions, malformed input, nesting past
     * the limit, tokens across the reader's buffer, and random documents and mutations of them. {@code
     * StrictJsonParser.parse} reads the same inputs with the reader set to {@code STRICT}.
     */
    private static final class ParserSuite implements Suite {
        @Override
        public String name() {
            return JsonParser.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            for (String input : parserInputs()) {
                JsonArray args = RenderSuites.array(new JsonPrimitive(input));
                record(writer, PARSE_STRING, args, () -> encodeParsed(JsonParser.parseString(input)));
                record(writer, STRICT_PARSE, args, () -> encodeParsed(StrictJsonParser.parse(input)));
            }
        }
    }

    /**
     * {@code JsonReader} itself, in each strictness: programs that peek each token and read it the way it asks (or skip
     * it, or read it another way: a number as a string, an int, a long or a double, a string as a number), now and then
     * the wrong way, which throws. {@code args} are the strictness, the nesting limit, the input and the ops; the result
     * is each op's output with the path and the previous path afterwards, the last one {@code {"exception": class,
     * "message": message}} if it threw.
     */
    private static final class ReaderSuite implements Suite {
        private static final int MAX_OPS = 1200;
        private static final List<String> WRONG_OPS = List.of(
            "beginArray", "endArray", "beginObject", "endObject", "nextName", "nextString", "nextBoolean", "nextNull",
            "nextDouble", "nextLong", "nextInt"
        );

        @Override
        public String name() {
            return JsonReader.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            Random random = new Random(53);
            List<String> inputs = new ArrayList<>();
            inputs.addAll(VALID);
            inputs.addAll(LENIENT);
            inputs.addAll(MALFORMED);
            inputs.addAll(lineInputs());
            List<String> deep = deepInputs();
            for (int i = 0; i < deep.size(); i += 3) {
                inputs.add(deep.get(i));
            }
            List<String> buffer = bufferInputs();
            for (int i = 0; i < buffer.size(); i += 4) {
                inputs.add(buffer.get(i));
            }
            List<String> seeds = new ArrayList<>(VALID);
            seeds.addAll(LENIENT);
            for (int i = 0; i < 400; i++) {
                inputs.add(mutate(seeds.get(random.nextInt(seeds.size())), random));
            }
            for (String input : inputs) {
                if (!wellFormed(input)) {
                    continue;
                }
                for (Strictness strictness : Strictness.values()) {
                    int nestingLimit = random.nextInt(10) == 0 ? random.nextInt(4) : 255;
                    JsonArray ops = new JsonArray();
                    JsonArray outputs = run(strictness, nestingLimit, input, ops, random);
                    if (wellFormed(outputs)) {
                        JsonArray args = RenderSuites.array(
                            new JsonPrimitive(strictness.name()), new JsonPrimitive(nestingLimit), new JsonPrimitive(input), ops
                        );
                        writer.result("programs", args, outputs, null);
                    }
                }
            }
        }

        private static JsonArray run(
            final Strictness strictness, final int nestingLimit, final String input, final JsonArray ops, final Random random
        ) {
            JsonReader reader = new JsonReader(new StringReader(input));
            reader.setStrictness(strictness);
            reader.setNestingLimit(nestingLimit);
            JsonArray outputs = new JsonArray();
            JsonToken token = null;
            while (ops.size() < MAX_OPS) {
                String op = token == null ? "peek" : choose(token, random);
                ops.add(op);
                try {
                    JsonElement output = apply(reader, op);
                    outputs.add(RenderSuites.array(output, new JsonPrimitive(reader.getPath()), new JsonPrimitive(reader.getPreviousPath())));
                    if (op.equals("peek")) {
                        token = JsonToken.valueOf(output.getAsString());
                        if (token == JsonToken.END_DOCUMENT) {
                            // Past the end, the reader answers the same: once more, then stop.
                            String last = List.of("peek", "hasNext", "skipValue", "nextString").get(random.nextInt(4));
                            ops.add(last);
                            outputs.add(RenderSuites.array(apply(reader, last), new JsonPrimitive(reader.getPath()), new JsonPrimitive(reader.getPreviousPath())));
                            break;
                        }
                    } else {
                        token = null;
                    }
                } catch (IOException | RuntimeException e) {
                    JsonObject exception = new JsonObject();
                    exception.addProperty("exception", e.getClass().getName());
                    exception.addProperty("message", e.getMessage());
                    outputs.add(exception);
                    break;
                }
            }
            return outputs;
        }

        // How to read the token peeked: its own way mostly, skipValue a fifth of the time, numbers and strings any of
        // the four ways, now and then a wrong one, and hasNext or toString in between.
        private static String choose(final JsonToken token, final Random random) {
            int roll = random.nextInt(20);
            if (roll == 0) {
                return WRONG_OPS.get(random.nextInt(WRONG_OPS.size()));
            }
            if (roll == 1) {
                return random.nextBoolean() ? "hasNext" : "toString";
            }
            if (roll < 5) {
                return "skipValue";
            }
            return switch (token) {
                case BEGIN_ARRAY -> "beginArray";
                case END_ARRAY -> "endArray";
                case BEGIN_OBJECT -> "beginObject";
                case END_OBJECT -> "endObject";
                case NAME -> "nextName";
                case STRING, NUMBER -> List.of("nextString", "nextString", "nextInt", "nextLong", "nextDouble").get(random.nextInt(5));
                case BOOLEAN -> "nextBoolean";
                case NULL -> "nextNull";
                case END_DOCUMENT -> "peek";
            };
        }

        private static JsonElement apply(final JsonReader reader, final String op) throws IOException {
            switch (op) {
                case "peek" -> {
                    return new JsonPrimitive(reader.peek().name());
                }
                case "hasNext" -> {
                    return new JsonPrimitive(reader.hasNext());
                }
                case "toString" -> {
                    return new JsonPrimitive(reader.toString());
                }
                case "beginArray" -> reader.beginArray();
                case "endArray" -> reader.endArray();
                case "beginObject" -> reader.beginObject();
                case "endObject" -> reader.endObject();
                case "skipValue" -> reader.skipValue();
                case "nextNull" -> reader.nextNull();
                case "nextName" -> {
                    return new JsonPrimitive(reader.nextName());
                }
                case "nextString" -> {
                    return new JsonPrimitive(reader.nextString());
                }
                case "nextBoolean" -> {
                    return new JsonPrimitive(reader.nextBoolean());
                }
                case "nextDouble" -> {
                    return new JsonPrimitive(Values.doubleBits(reader.nextDouble()));
                }
                case "nextLong" -> {
                    return new JsonPrimitive(reader.nextLong());
                }
                case "nextInt" -> {
                    return new JsonPrimitive(reader.nextInt());
                }
                default -> throw new IllegalArgumentException(op);
            }
            return JsonNull.INSTANCE;
        }
    }

    // ---- Elements to write ----

    private static final List<String> STRINGS = strings();

    private static List<String> strings() {
        List<String> strings = new ArrayList<>(List.of(
            "", "a", "abc", "true", "1", "é", "日本語", "😀", "a😀b", "\u2028", "\u2029", "a\u2028b\u2029c", "\u0085", "\u00a0",
            "\ufeff", "\uffff", "<", ">", "&", "=", "'", "<>&='", "</script>", "\"quoted\"", "back\\slash", "/", "\u007f", "\u0080"
        ));
        for (char c = 0; c < 128; c++) {
            strings.add(String.valueOf(c));
            strings.add("a" + c + "b");
        }
        return strings;
    }

    // Primitives of every box and value the writer and the getters tell apart.
    static List<JsonPrimitive> primitives() {
        List<JsonPrimitive> primitives = new ArrayList<>();
        primitives.add(new JsonPrimitive(true));
        primitives.add(new JsonPrimitive(false));
        for (int value : new int[]{0, 1, -1, 127, 128, -128, -129, 255, 256, 32767, 32768, -32768, -32769, 65535, 1000000, Integer.MAX_VALUE, Integer.MIN_VALUE}) {
            primitives.add(new JsonPrimitive(value));
        }
        for (long value : new long[]{0, 1, -1, 2147483647L, 2147483648L, -2147483649L, 4294967296L, 9007199254740993L, 1L << 53, Long.MAX_VALUE, Long.MIN_VALUE}) {
            primitives.add(new JsonPrimitive(value));
        }
        for (byte value : new byte[]{0, 1, -1, 127, -128}) {
            primitives.add(new JsonPrimitive(value));
        }
        for (short value : new short[]{0, 1, -1, 255, 256, 32767, -32768}) {
            primitives.add(new JsonPrimitive(value));
        }
        for (float value : new float[]{
            0.0F, -0.0F, 1.0F, -1.0F, 1.5F, -1.5F, 0.1F, 3.4028235E38F, 1.4E-45F, 1.17549435E-38F, Float.NaN, Float.POSITIVE_INFINITY,
            Float.NEGATIVE_INFINITY, 2.14748365E9F, -2.14748365E9F, 9.223372E18F, 1.0E10F, 128.7F, -128.9F, 16777217.0F, 1.0E7F, 0.001F, 1.0E-4F,
            32768.5F, -0.5F
        }) {
            primitives.add(new JsonPrimitive(value));
        }
        for (double value : new double[]{
            0.0, -0.0, 1.0, -1.0, 1.5, -1.5, 0.1, 1.0E300, Double.MAX_VALUE, Double.MIN_VALUE, Double.MIN_NORMAL, Double.NaN,
            Double.POSITIVE_INFINITY, Double.NEGATIVE_INFINITY, 2147483648.5, -2147483649.5, 9.2233720368547758E18, 1.0E19, 4.9E-324,
            1.0000000000000002, 255.9, -0.5, 1.0E7, 1.0E-3, 9.999999999999999E-4, 3.4028235677973366E38, 0.1F, 123456789.125
        }) {
            primitives.add(new JsonPrimitive(value));
        }
        for (String text : List.of(
            "0", "-0", "1", "-1", "1.5", "-1.5", "1e3", "1E-3", "2147483647", "2147483648", "-2147483649", "9223372036854775807",
            "9223372036854775808", "-9223372036854775809", "18446744073709551617", "123456789012345678901234567890", "1.9", "-1.9",
            "0.5", "1e10", "1e19", "1e20", "1.5e300", "1e309", "-1e309", "4.9e-324", "1e-400", "1e9999", "1e10000", "1e-9999",
            "1e-10000", "12345678901234567890.5", "-0.0", "0e5", "1.0", "128", "-129", "32768", "65536", "4294967296",
            "4294967297.75", "-4294967297.75", "3.4028235e38", "3.4028236e38", "1e-46", "0.1", "1.00", "100", "1E2"
        )) {
            primitives.add(new JsonPrimitive(new LazilyParsedNumber(text)));
        }
        for (String text : List.of(
            "abc", "", " 1", "1 ", "1f", "1d", "0x10", "0x1p3", "NaN", "Infinity", "-Infinity", "+1", "1_000", "١٢٣", "１２", "1e",
            ".5", "5.", "01", "1,5", "--1", "true", "1".repeat(10001)
        )) {
            primitives.add(new JsonPrimitive(new LazilyParsedNumber(text)));
        }
        for (String text : List.of(
            "0", "0.00", "1", "1.0", "1.00", "-1.5", "1E+400", "1E-400", "123456789012345678901234567890.123", "0.000001", "0.0000001",
            "-9223372036854775808", "9223372036854775808", "1E+3", "12.5E-3", "-0", "2147483648", "-1E+10", "0E+5", "100", "1.5E+300",
            "18446744073709551615"
        )) {
            primitives.add(new JsonPrimitive(new BigDecimal(text)));
        }
        for (String text : List.of(
            "", "a", "abc", "true", "TRUE", "tRuE", "false", "yes", "1", "-1", "+1", "1.5", "1e3", " 1", "1 ", "\t1\n", "0x10", "0X1P3",
            "0x1.8p1", "0x.8p1", "0x1p-1074", "0x1p-1075", "0x1.fffffffffffff8p1023", "0xp1", "0x1", "1f", "1F", "1d", "1.5D", "NaN", "-NaN",
            "Infinity", "-Infinity", "+Infinity", "infinity", "Infinityx", "128", "-129", "127", "-128", "32767", "32768", "-32769",
            "2147483647", "2147483648", "9223372036854775807", "9223372036854775808", "١٢٣", "１２", "1e10000", "0.1e-9999", "1E2147483648",
            "1e2147483647", "1e-2147483649", ".5", "5.", "-.5e-3", "00012", "1__2", "1,5", "1.5.5", "e5", "1e", "1e+", "--1", "+-1", "-", "+",
            ".", "\u0000 1", "1\u0000", "5".repeat(400), "0." + "0".repeat(330) + "1", "1e-46", "3.4028235e38",
            "3.40282357e38", "1.17549435E-38", "7.006492321624085e-46", "2.5e-324", "4.9e-324d"
        )) {
            primitives.add(new JsonPrimitive(text));
        }
        // Random numbers as parsed numbers (JSON's grammar) and as strings (Java's, with hex,
        // suffixes, whitespace and stray characters), where the parsers' paths and limits are.
        Random random = new Random(71);
        for (int i = 0; i < 120; i++) {
            primitives.add(new JsonPrimitive(new LazilyParsedNumber(randomJsonNumber(random))));
        }
        for (int i = 0; i < 200; i++) {
            primitives.add(new JsonPrimitive(randomJavaNumber(random)));
        }
        return primitives;
    }

    private static String digits(final Random random, final int min, final int max) {
        StringBuilder builder = new StringBuilder();
        for (int i = min + random.nextInt(max - min + 1); i > 0; i--) {
            builder.append((char)('0' + random.nextInt(10)));
        }
        return builder.toString();
    }

    private static String randomJsonNumber(final Random random) {
        StringBuilder builder = new StringBuilder(random.nextInt(3) == 0 ? "-" : "");
        builder.append(random.nextInt(4) == 0 ? "0" : (char)('1' + random.nextInt(9)) + digits(random, 0, random.nextBoolean() ? 25 : 8));
        if (random.nextBoolean()) {
            builder.append('.').append(digits(random, 1, 20));
        }
        if (random.nextBoolean()) {
            builder.append(random.nextBoolean() ? 'e' : 'E').append(List.of("", "+", "-").get(random.nextInt(3)));
            builder.append(random.nextInt(5) == 0 ? digits(random, 1, 6) : Integer.toString(random.nextInt(400)));
        }
        return builder.toString();
    }

    private static String randomJavaNumber(final Random random) {
        StringBuilder builder = new StringBuilder();
        if (random.nextInt(6) == 0) {
            builder.append(List.of(" ", "\t", "\n", "\u0000", "  ").get(random.nextInt(5)));
        }
        builder.append(List.of("", "", "-", "+").get(random.nextInt(4)));
        if (random.nextInt(4) == 0) {
            builder.append(random.nextBoolean() ? "0x" : "0X");
            String hex = "0123456789abcdefABCDEF";
            for (int i = random.nextInt(18); i > 0; i--) {
                builder.append(hex.charAt(random.nextInt(hex.length())));
            }
            if (random.nextBoolean()) {
                builder.append('.');
                for (int i = random.nextInt(6); i > 0; i--) {
                    builder.append(hex.charAt(random.nextInt(hex.length())));
                }
            }
            if (random.nextInt(5) != 0) {
                builder.append(random.nextBoolean() ? 'p' : 'P').append(List.of("", "+", "-").get(random.nextInt(3)));
                builder.append(random.nextInt(1100));
            }
        } else {
            builder.append(digits(random, 0, random.nextInt(4) == 0 ? 30 : 6));
            if (random.nextInt(3) == 0) {
                builder.append('.').append(digits(random, 0, 12));
            }
            if (random.nextInt(3) == 0) {
                builder.append(random.nextBoolean() ? 'e' : 'E').append(List.of("", "+", "-").get(random.nextInt(3)));
                builder.append(random.nextInt(6) == 0 ? digits(random, 0, 12) : Integer.toString(random.nextInt(340)));
            }
        }
        if (random.nextInt(5) == 0) {
            builder.append("fFdD".charAt(random.nextInt(4)));
        }
        if (random.nextInt(8) == 0) {
            builder.insert(random.nextInt(builder.length() + 1), "x.e+-_ ١".charAt(random.nextInt(8)));
        }
        if (random.nextInt(6) == 0) {
            builder.append(List.of(" ", "\t", "\r\n", "\u0001").get(random.nextInt(4)));
        }
        return builder.toString();
    }

    // The elements written: what the parser read, every primitive, strings needing each escape, nulls among members and
    // elements, and random trees of those.
    static List<JsonElement> elements() {
        Map<String, JsonElement> elements = new LinkedHashMap<>();
        for (String input : parserInputs()) {
            try {
                JsonElement element = JsonParser.parseString(input);
                if (depth(element) <= MAX_ENCODED_DEPTH && wellFormed(encode(element))) {
                    elements.putIfAbsent(encode(element).toString(), element);
                }
            } catch (RuntimeException e) {
                // Only what parses is written.
            }
        }
        List<JsonElement> built = new ArrayList<>(primitives());
        for (String string : STRINGS) {
            built.add(new JsonPrimitive(string));
        }
        built.add(JsonNull.INSTANCE);
        JsonObject nullMember = new JsonObject();
        nullMember.add("a", JsonNull.INSTANCE);
        built.add(nullMember);
        JsonObject nullMembers = new JsonObject();
        nullMembers.add("a", JsonNull.INSTANCE);
        nullMembers.addProperty("b", 1);
        nullMembers.add("c", JsonNull.INSTANCE);
        JsonArray nulls = new JsonArray();
        nulls.add(JsonNull.INSTANCE);
        nulls.add(nullMember);
        nulls.add(JsonNull.INSTANCE);
        nullMembers.add("d", nulls);
        built.add(nullMembers);
        built.add(nulls);
        Random random = new Random(59);
        List<JsonPrimitive> primitives = primitives();
        for (int i = 0; i < 300; i++) {
            built.add(randomElement(random, primitives, 0));
        }
        for (JsonElement element : built) {
            if (wellFormed(encode(element))) {
                elements.putIfAbsent(encode(element).toString(), element);
            }
        }
        return new ArrayList<>(elements.values());
    }

    private static JsonElement randomElement(final Random random, final List<JsonPrimitive> primitives, final int depth) {
        switch (depth > 3 ? random.nextInt(3) : random.nextInt(6)) {
            case 0 -> {
                return JsonNull.INSTANCE;
            }
            case 1 -> {
                return primitives.get(random.nextInt(primitives.size()));
            }
            case 2 -> {
                return new JsonPrimitive(STRINGS.get(random.nextInt(STRINGS.size())));
            }
            case 3, 4 -> {
                JsonObject object = new JsonObject();
                for (int i = random.nextInt(5); i > 0; i--) {
                    object.add(STRINGS.get(random.nextInt(STRINGS.size())), randomElement(random, primitives, depth + 1));
                }
                return object;
            }
            default -> {
                JsonArray array = new JsonArray();
                for (int i = random.nextInt(5); i > 0; i--) {
                    array.add(randomElement(random, primitives, depth + 1));
                }
                return array;
            }
        }
    }

    /**
     * {@code JsonElement}: {@code toString()} of every element (a lenient writer that writes nulls), and {@code equals}
     * of pairs: each element and a copy of it, its members in another order, a value changed, and random pairs.
     */
    private static final class ElementSuite implements Suite {
        @Override
        public String name() {
            return JsonElement.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            List<JsonElement> elements = elements();
            for (JsonElement element : elements) {
                record(writer, TO_STRING, RenderSuites.array(encode(element)), () -> new JsonPrimitive(element.toString()));
            }
            Random random = new Random(61);
            for (JsonElement element : elements) {
                equals(writer, element, element.deepCopy());
                if (element.isJsonObject() && element.getAsJsonObject().size() > 1) {
                    List<Map.Entry<String, JsonElement>> entries = new ArrayList<>(element.getAsJsonObject().entrySet());
                    java.util.Collections.shuffle(entries, random);
                    JsonObject shuffled = new JsonObject();
                    entries.forEach(entry -> shuffled.add(entry.getKey(), entry.getValue()));
                    equals(writer, element, shuffled);
                    JsonObject changed = element.getAsJsonObject().deepCopy();
                    changed.add(entries.get(0).getKey(), new JsonPrimitive(1));
                    equals(writer, element, changed);
                    JsonObject removed = element.getAsJsonObject().deepCopy();
                    removed.remove(entries.get(0).getKey());
                    equals(writer, element, removed);
                }
                if (element.isJsonArray() && element.getAsJsonArray().size() > 1) {
                    JsonArray reversed = new JsonArray();
                    for (int i = element.getAsJsonArray().size() - 1; i >= 0; i--) {
                        reversed.add(element.getAsJsonArray().get(i));
                    }
                    equals(writer, element, reversed);
                }
                equals(writer, element, elements.get(random.nextInt(elements.size())));
            }
            List<JsonPrimitive> primitives = primitives();
            for (int i = 0; i < 400; i++) {
                JsonArray first = new JsonArray();
                first.add(primitives.get(random.nextInt(primitives.size())));
                JsonArray second = new JsonArray();
                second.add(primitives.get(random.nextInt(primitives.size())));
                equals(writer, first, second);
            }
        }

        private static void equals(final GoldenWriter writer, final JsonElement first, final JsonElement second) throws IOException {
            JsonArray args = RenderSuites.array(encode(first), encode(second));
            record(writer, EQUALS, args, () -> new JsonPrimitive(first.equals(second)));
        }
    }

    /**
     * {@code toJson(JsonElement)} of a Gson with HTML escaping off, as {@code ByteBufCodecs.lenientJson} writes, and of a
     * default Gson, which escapes HTML's characters: both lenient writers that drop null members. A third of the elements
     * also by a Gson that writes nulls after the non-execute prefix, and every primitive by a strict one.
     */
    private static final class GsonSuite implements Suite {
        @Override
        public String name() {
            return Gson.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            List<JsonElement> elements = elements();
            for (int i = 0; i < elements.size(); i++) {
                JsonElement element = elements.get(i);
                JsonArray args = RenderSuites.array(encode(element));
                record(writer, TO_JSON, args, () -> new JsonPrimitive(LENIENT_JSON_GSON.toJson(element)));
                record(writer, HTML_SAFE_TO_JSON, args, () -> new JsonPrimitive(DEFAULT_GSON.toJson(element)));
                if (i % 3 == 0) {
                    record(writer, NULLS_TO_JSON, args, () -> new JsonPrimitive(NULLS_GSON.toJson(element)));
                }
            }
            // A strict writer refuses NaN and infinities.
            for (JsonPrimitive primitive : primitives()) {
                JsonArray args = RenderSuites.array(encode(primitive));
                record(writer, STRICT_TO_JSON, args, () -> new JsonPrimitive(STRICT_GSON.toJson(primitive)));
            }
        }
    }

    /**
     * {@code JsonPrimitive}'s getters, {@code hashCode} and {@code equals} on primitives of every box (parsed numbers
     * among them, valid and not) and numeric and other strings; and what {@code getAsBigDecimal()} hands out, which
     * {@code JsonOps.convertTo} reads.
     */
    private static final class PrimitiveSuite implements Suite {
        @Override
        public String name() {
            return JsonPrimitive.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            List<JsonPrimitive> primitives = primitives();
            for (JsonPrimitive primitive : primitives) {
                JsonArray args = RenderSuites.array(encode(primitive));
                record(writer, "isBoolean()Z", args, () -> new JsonPrimitive(primitive.isBoolean()));
                record(writer, "isNumber()Z", args, () -> new JsonPrimitive(primitive.isNumber()));
                record(writer, "isString()Z", args, () -> new JsonPrimitive(primitive.isString()));
                record(writer, "getAsBoolean()Z", args, () -> new JsonPrimitive(primitive.getAsBoolean()));
                record(writer, "getAsNumber()Ljava/lang/Number;", args, () -> {
                    Number number = primitive.getAsNumber();
                    return RenderSuites.array(new JsonPrimitive(number.getClass().getSimpleName()), new JsonPrimitive(number.toString()));
                });
                record(writer, "getAsString()Ljava/lang/String;", args, () -> new JsonPrimitive(primitive.getAsString()));
                record(writer, "getAsDouble()D", args, () -> new JsonPrimitive(Values.doubleBits(primitive.getAsDouble())));
                record(writer, "getAsFloat()F", args, () -> new JsonPrimitive(Values.floatBits(primitive.getAsFloat())));
                record(writer, "getAsLong()J", args, () -> new JsonPrimitive(primitive.getAsLong()));
                record(writer, "getAsInt()I", args, () -> new JsonPrimitive(primitive.getAsInt()));
                record(writer, "getAsShort()S", args, () -> new JsonPrimitive(primitive.getAsShort()));
                record(writer, "getAsByte()B", args, () -> new JsonPrimitive(primitive.getAsByte()));
                record(writer, "getAsBigDecimal()Ljava/math/BigDecimal;", args, () -> new JsonPrimitive(primitive.getAsBigDecimal().toString()));
                record(writer, "getAsBigDecimal().intValue()I", args, () -> new JsonPrimitive(primitive.getAsBigDecimal().intValue()));
                record(writer, "getAsBigDecimal().longValue()J", args, () -> new JsonPrimitive(primitive.getAsBigDecimal().longValue()));
                record(writer, "getAsBigDecimal().longValueExact()J", args, () -> new JsonPrimitive(primitive.getAsBigDecimal().longValueExact()));
                record(writer, "getAsBigDecimal().doubleValue()D", args, () -> new JsonPrimitive(Values.doubleBits(primitive.getAsBigDecimal().doubleValue())));
                record(writer, "getAsBigDecimal().floatValue()F", args, () -> new JsonPrimitive(Values.floatBits(primitive.getAsBigDecimal().floatValue())));
                record(writer, "getAsBigDecimal().scale()I", args, () -> new JsonPrimitive(primitive.getAsBigDecimal().scale()));
                record(writer, "hashCode()I", args, () -> new JsonPrimitive(primitive.hashCode()));
            }
            // Each primitive against those of the same value in other boxes (by its text), and random others.
            Random random = new Random(67);
            for (JsonPrimitive primitive : primitives) {
                List<JsonPrimitive> others = new ArrayList<>();
                others.add(primitive);
                for (JsonPrimitive other : primitives) {
                    if (other != primitive && other.getAsString().equals(primitive.getAsString())) {
                        others.add(other);
                    }
                }
                for (int i = 0; i < 6; i++) {
                    others.add(primitives.get(random.nextInt(primitives.size())));
                }
                for (JsonPrimitive other : others) {
                    JsonArray args = RenderSuites.array(encode(primitive), encode(other));
                    record(writer, EQUALS, args, () -> new JsonPrimitive(primitive.equals(other)));
                }
            }
        }
    }
}
