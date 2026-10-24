package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonNull;
import com.google.gson.JsonPrimitive;
import com.mojang.brigadier.StringReader;
import com.mojang.brigadier.exceptions.CommandSyntaxException;
import java.io.IOException;
import java.util.ArrayList;
import java.util.List;
import java.util.Random;

/** Brigadier, with the game's wording of its errors ({@code BrigadierExceptions}, which bootstrapping sets). */
final class BrigadierSuites {
    private BrigadierSuites() {
    }

    static List<Suite> suites() {
        return List.of(new StringReaderSuite());
    }

    /**
     * {@code StringReader}: programs of reads on strings of what commands and selectors hold (numbers at the types'
     * edges, quoted strings with escapes good and bad, booleans, symbols, whitespace of every kind, non-BMP
     * characters). {@code args} are the input and the ops, each a name or {@code [name, argument]}; the result is
     * each op's output with the cursor after it, {@code ["exception", message, cursor]} last if one threw a
     * {@code CommandSyntaxException} (its message with the input's context). Doubles and floats are their bits. The
     * golden files can't hold a lone surrogate, so programs whose outputs split a pair are left out.
     */
    private static final class StringReaderSuite implements Suite {
        private static final int PROGRAMS = 3000;
        private static final int MAX_OPS = 24;
        private static final List<String> PIECES = List.of(
            "0", "1", "-1", "42", "2147483647", "2147483648", "-2147483648", "-2147483649", "9223372036854775807", "9223372036854775808",
            "1.5", "-0.25", ".5", "5.", "1.2.3", "--1", "-", ".", "..", "1e5", "0x10", "+3", "007", "3.4028235E38", "1" + "0".repeat(40),
            "true", "false", "TRUE", "truex", "\"quoted\"", "\"esc \\\" \\\\ q\"", "\"bad \\n escape\"", "\"unclosed", "'single'",
            "'mixed\"'", "\"\"", "''", "\\", "abc", "a_b-c.d+e", "@e", "[", "]", "{", "}", "=", ",", "!", "~", "^", ":", "#", "/",
            " ", "  ", "\t", "\n", "\u000b", "\u001c", " ", " ", " ", "　", "é", "😀", "日本"
        );
        private static final String SYMBOLS = "\"'[]{}=,!~^:#@ a0.";

        @Override
        public String name() {
            return StringReader.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            Random random = new Random(61);
            for (int i = 0; i < PROGRAMS; i++) {
                StringBuilder input = new StringBuilder();
                int pieces = random.nextInt(6);
                for (int j = 0; j < pieces; j++) {
                    input.append(PIECES.get(random.nextInt(PIECES.size())));
                }
                String string = input.toString();
                JsonArray ops = new JsonArray();
                JsonArray outputs = run(string, ops, random);
                if (!GsonSuites.wellFormed(outputs)) {
                    continue;
                }
                JsonArray args = new JsonArray();
                args.add(string);
                args.add(ops);
                writer.result("programs", args, outputs, null);
            }
        }

        private static JsonArray run(final String input, final JsonArray ops, final Random random) {
            StringReader reader = new StringReader(input);
            JsonArray outputs = new JsonArray();
            for (int i = 0; i < MAX_OPS; i++) {
                JsonElement op = choose(reader, random);
                ops.add(op);
                try {
                    JsonElement output = apply(reader, op);
                    outputs.add(RenderSuites.array(output, new JsonPrimitive(reader.getCursor())));
                } catch (CommandSyntaxException e) {
                    outputs.add(RenderSuites.array(new JsonPrimitive("exception"), new JsonPrimitive(e.getMessage()), new JsonPrimitive(reader.getCursor())));
                    break;
                }
            }
            return outputs;
        }

        // An op that doesn't throw anything but a CommandSyntaxException: peeks and reads of a character only where
        // there is one.
        private static JsonElement choose(final StringReader reader, final Random random) {
            List<JsonElement> ops = new ArrayList<>();
            for (String op : List.of(
                "readInt", "readLong", "readDouble", "readFloat", "readUnquotedString", "readQuotedString", "readString", "readBoolean",
                "skipWhitespace", "canRead", "getRead", "getRemaining", "getRemainingLength", "getTotalLength"
            )) {
                ops.add(new JsonPrimitive(op));
            }
            ops.add(RenderSuites.array(new JsonPrimitive("canRead"), new JsonPrimitive(random.nextInt(4))));
            ops.add(RenderSuites.array(new JsonPrimitive("expect"), new JsonPrimitive((int)SYMBOLS.charAt(random.nextInt(SYMBOLS.length())))));
            ops.add(RenderSuites.array(new JsonPrimitive("setCursor"), new JsonPrimitive(random.nextInt(reader.getTotalLength() + 1))));
            if (reader.canRead()) {
                ops.add(new JsonPrimitive("peek"));
                ops.add(new JsonPrimitive("read"));
                ops.add(new JsonPrimitive("skip"));
                ops.add(RenderSuites.array(new JsonPrimitive("readStringUntil"), new JsonPrimitive((int)SYMBOLS.charAt(random.nextInt(SYMBOLS.length())))));
                int offset = random.nextInt(reader.getRemainingLength());
                ops.add(RenderSuites.array(new JsonPrimitive("peek"), new JsonPrimitive(offset)));
            }
            return ops.get(random.nextInt(ops.size()));
        }

        private static JsonElement apply(final StringReader reader, final JsonElement op) throws CommandSyntaxException {
            String name = op.isJsonArray() ? op.getAsJsonArray().get(0).getAsString() : op.getAsString();
            int argument = op.isJsonArray() ? op.getAsJsonArray().get(1).getAsInt() : 0;
            return switch (name) {
                case "readInt" -> new JsonPrimitive(reader.readInt());
                case "readLong" -> new JsonPrimitive(reader.readLong());
                case "readDouble" -> new JsonPrimitive(Values.doubleBits(reader.readDouble()));
                case "readFloat" -> new JsonPrimitive(Values.floatBits(reader.readFloat()));
                case "readUnquotedString" -> new JsonPrimitive(reader.readUnquotedString());
                case "readQuotedString" -> new JsonPrimitive(reader.readQuotedString());
                case "readString" -> new JsonPrimitive(reader.readString());
                case "readStringUntil" -> new JsonPrimitive(reader.readStringUntil((char)argument));
                case "readBoolean" -> new JsonPrimitive(reader.readBoolean());
                case "skipWhitespace" -> {
                    reader.skipWhitespace();
                    yield JsonNull.INSTANCE;
                }
                case "skip" -> {
                    reader.skip();
                    yield JsonNull.INSTANCE;
                }
                case "setCursor" -> {
                    reader.setCursor(argument);
                    yield JsonNull.INSTANCE;
                }
                case "expect" -> {
                    reader.expect((char)argument);
                    yield JsonNull.INSTANCE;
                }
                case "canRead" -> new JsonPrimitive(op.isJsonArray() ? reader.canRead(argument) : reader.canRead());
                case "peek" -> new JsonPrimitive((int)(op.isJsonArray() ? reader.peek(argument) : reader.peek()));
                case "read" -> new JsonPrimitive((int)reader.read());
                case "getRead" -> new JsonPrimitive(reader.getRead());
                case "getRemaining" -> new JsonPrimitive(reader.getRemaining());
                case "getRemainingLength" -> new JsonPrimitive(reader.getRemainingLength());
                case "getTotalLength" -> new JsonPrimitive(reader.getTotalLength());
                default -> throw new IllegalArgumentException(name);
            };
        }
    }
}
