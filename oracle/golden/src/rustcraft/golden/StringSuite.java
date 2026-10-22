package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonPrimitive;
import java.io.IOException;
import java.nio.charset.StandardCharsets;
import java.util.ArrayList;
import java.util.List;
import java.util.Random;

/**
 * java.lang.String behaviour the game depends on: hashCode runs over UTF-16 units, not UTF-8 bytes or chars, compareTo
 * returns the difference of the first UTF-16 units that differ, or of the lengths, and decoding UTF-8 (which Netty's
 * ByteBuf.toString does) replaces malformed input its own way.
 */
final class StringSuite implements Suite {
    private static final int RANDOM_CASES = 500;

    @Override
    public String name() {
        return String.class.getName();
    }

    @Override
    public void write(final GoldenWriter writer) throws IOException {
        Random random = new Random(31);
        List<Object> inputs = new ArrayList<>(Values.STRINGS.edges());
        for (int i = 0; i < RANDOM_CASES; i++) {
            inputs.add(Values.STRINGS.random(random));
        }

        for (Object input : inputs) {
            JsonArray args = new JsonArray();
            args.add((String)input);
            writer.result("hashCode()I", args, new JsonPrimitive(input.hashCode()), null);
        }

        // Every pair of edges, random pairs, and strings against their prefixes and one-unit changes.
        List<String[]> pairs = new ArrayList<>();
        for (Object a : Values.STRINGS.edges()) {
            for (Object b : Values.STRINGS.edges()) {
                pairs.add(new String[]{(String)a, (String)b});
            }
        }
        for (int i = 0; i < RANDOM_CASES; i++) {
            String a = (String)Values.STRINGS.random(random);
            String b = switch (random.nextInt(3)) {
                case 0 -> (String)Values.STRINGS.random(random);
                // Whole code points: a lone surrogate can't be written as UTF-8.
                case 1 -> a.substring(0, a.offsetByCodePoints(0, random.nextInt(a.codePointCount(0, a.length()) + 1)));
                default -> a + (char)('a' + random.nextInt(26));
            };
            pairs.add(random.nextBoolean() ? new String[]{a, b} : new String[]{b, a});
        }
        for (String[] pair : pairs) {
            JsonArray args = new JsonArray();
            args.add(pair[0]);
            args.add(pair[1]);
            writer.result("compareTo(Ljava/lang/String;)I", args, new JsonPrimitive(pair[0].compareTo(pair[1])), null);
        }

        for (byte[] bytes : utf8Inputs(random)) {
            JsonArray args = new JsonArray();
            args.add(Values.encode(bytes, byte[].class));
            args.add(StandardCharsets.UTF_8.name());
            writer.result("<init>([BLjava/nio/charset/Charset;)V", args, new JsonPrimitive(new String(bytes, StandardCharsets.UTF_8)), null);
        }
    }

    // Every byte and every pair of bytes; every 3-byte lead with every second byte and a third on either side of the
    // continuation range; every 4-byte lead (and the leads past U+10FFFF) with every second byte, cut short after the
    // third or not; then random bytes that are mostly UTF-8's, and the edge and random strings encoded.
    private static List<byte[]> utf8Inputs(final Random random) {
        List<byte[]> inputs = new ArrayList<>();
        for (int b1 = 0; b1 < 256; b1++) {
            inputs.add(new byte[]{(byte)b1});
            for (int b2 = 0; b2 < 256; b2++) {
                inputs.add(new byte[]{(byte)b1, (byte)b2});
            }
        }
        for (int b1 = 0xe0; b1 < 0xf0; b1++) {
            for (int b2 = 0; b2 < 256; b2++) {
                for (int b3 : new int[]{0x00, 0x41, 0x7f, 0x80, 0xbf, 0xc0, 0xff}) {
                    inputs.add(new byte[]{(byte)b1, (byte)b2, (byte)b3});
                }
            }
        }
        for (int b1 = 0xf0; b1 < 0xf8; b1++) {
            for (int b2 = 0; b2 < 256; b2++) {
                for (int b3 : new int[]{0x41, 0x80, 0xbf, 0xc0}) {
                    inputs.add(new byte[]{(byte)b1, (byte)b2, (byte)b3});
                    for (int b4 : new int[]{0x41, 0x80, 0xbf}) {
                        inputs.add(new byte[]{(byte)b1, (byte)b2, (byte)b3, (byte)b4});
                    }
                }
            }
        }
        for (int i = 0; i < 2000; i++) {
            inputs.add(NetworkValues.randomUtf8(random));
        }
        for (Object string : Values.STRINGS.edges()) {
            inputs.add(((String)string).getBytes(StandardCharsets.UTF_8));
        }
        for (int i = 0; i < 200; i++) {
            inputs.add(((String)Values.STRINGS.random(random)).getBytes(StandardCharsets.UTF_8));
        }
        return inputs;
    }
}
