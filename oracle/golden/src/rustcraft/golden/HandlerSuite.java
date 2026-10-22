package rustcraft.golden;

import io.netty.buffer.ByteBuf;
import io.netty.buffer.Unpooled;
import io.netty.channel.ChannelHandlerContext;
import java.io.IOException;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.List;
import java.util.Random;
import net.minecraft.network.Varint21FrameDecoder;
import net.minecraft.network.Varint21LengthFieldPrepender;

/**
 * The Netty handlers that frame packets on the wire. Their decode and encode are protected, and take a context they never
 * use, so subclasses here call them with null. Each case gets a fresh handler, like each connection does. A decode
 * case reads the buffer in {@code args[0]} the way Netty's ByteToMessageDecoder asks, again while bytes are left and
 * the last call took a frame out. Its result is the frames found (none until a whole one has arrived), and
 * {@code after[0]} is the buffer afterwards. An encode case writes the message in {@code args[0]} into the buffer in
 * {@code args[1]}, which {@code after[1]} shows afterwards.
 */
final class HandlerSuite implements Suite {
    private static final int RANDOM_CASES = 500;
    // Chunks of a stream, which only chunkedArrivals makes.
    private static final Values.Generator CHUNKS = new Values.Simple(
        List.of(), random -> new byte[0][], chunks -> Values.encode(chunks, byte[][].class)
    );

    private final Class<?> type;
    private final Cases cases;

    @FunctionalInterface
    private interface Cases {
        void write(GoldenWriter writer) throws IOException;
    }

    private HandlerSuite(final Class<?> type, final Cases cases) {
        this.type = type;
        this.cases = cases;
    }

    /** Varint21FrameDecoder, which splits what arrives into frames. Its bandwidth monitor only counts the bytes. */
    static HandlerSuite frameDecoder() {
        return new HandlerSuite(Varint21FrameDecoder.class, writer -> {
            writer.skipped("<init>(Lnet/minecraft/network/BandwidthDebugMonitor;)V", "no generator for net.minecraft.network.BandwidthDebugMonitor");
            Values.Generator[] generators = {NetworkValues.BUFFER};
            String decode = descriptor(Varint21FrameDecoder.class, "decode", ChannelHandlerContext.class, ByteBuf.class, List.class);
            Codecs.writeCases(
                writer, decode, 0, leading -> new FrameDecoder(), generators, specs(generators, arrivals(), 1101), ByteBuf[].class,
                (made, rest) -> ((FrameDecoder)made).frames((ByteBuf)rest[0])
            );
            // The rest of the stream arrives in the chunks in args[1], each written to the buffer before the decoder is
            // asked again, which is how a length gets split. The result is every frame found.
            Codecs.writeCases(
                writer, decode + "[in chunks]", 0, leading -> new FrameDecoder(), new Values.Generator[]{NetworkValues.BUFFER, CHUNKS},
                chunkedArrivals(), ByteBuf[].class, (made, rest) -> {
                    FrameDecoder decoder = (FrameDecoder)made;
                    ByteBuf in = (ByteBuf)rest[0];
                    List<ByteBuf> frames = new ArrayList<>(List.of(decoder.frames(in)));
                    for (byte[] chunk : (byte[][])rest[1]) {
                        in.writeBytes(chunk);
                        frames.addAll(List.of(decoder.frames(in)));
                    }
                    return frames.toArray(ByteBuf[]::new);
                }
            );
        });
    }

    /** Varint21LengthFieldPrepender, which puts each frame's length in front of it. */
    static HandlerSuite lengthFieldPrepender() {
        return new HandlerSuite(Varint21LengthFieldPrepender.class, writer -> {
            StaticSuite.writeFields(Varint21LengthFieldPrepender.class, writer);
            Values.Generator[] generators = {NetworkValues.BUFFER, NetworkValues.BUFFER};
            Codecs.writeCases(
                writer, descriptor(Varint21LengthFieldPrepender.class, "encode", ChannelHandlerContext.class, ByteBuf.class, ByteBuf.class), 0,
                leading -> new LengthFieldPrepender(), generators, specs(generators, bodies(), 1102), void.class,
                (made, rest) -> {
                    ((LengthFieldPrepender)made).prepend((ByteBuf)rest[0], (ByteBuf)rest[1]);
                    return null;
                }
            );
        });
    }

    @Override
    public String name() {
        return this.type.getName();
    }

    @Override
    public void write(final GoldenWriter writer) throws IOException {
        this.cases.write(writer);
    }

    // The edges, the extra inputs, then random ones.
    private static List<Object[]> specs(final Values.Generator[] generators, final List<Object[]> extra, final long seed) {
        Random random = new Random(seed);
        List<Object[]> specs = new ArrayList<>(StaticSuite.edgeTuples(generators, random));
        specs.addAll(extra);
        for (int i = 0; i < RANDOM_CASES; i++) {
            specs.add(StaticSuite.randomTuple(generators, random));
        }
        return specs;
    }

    // Frames as they arrive: a length of one, two or three bytes and that many bytes, whole, with more after them, or
    // cut short anywhere, half the time after bytes already read; the client's first frame, the handshake; runs of up to
    // eight frames, the last one whole or not; and lengths of zero (an overlong one too), cut short, and wider than 21
    // bits.
    private static List<Object[]> arrivals() {
        Random random = new Random(1103);
        List<byte[]> frames = new ArrayList<>();
        for (int length : new int[]{1, 2, 3, 127, 128, 255, 300, 16383, 16384, 20000}) {
            byte[] payload = new byte[length];
            random.nextBytes(payload);
            byte[] frame = NetworkValues.lengthPrefixed(payload);
            frames.addAll(Codecs.variants(frame, random));
            frames.add(NetworkValues.concat(frame, NetworkValues.lengthPrefixed(NetworkValues.randomField(random))));
        }
        frames.add(NetworkValues.lengthPrefixed(NetworkValues.handshake()));
        for (int i = 0; i < 40; i++) {
            byte[] run = new byte[0];
            for (int count = 2 + random.nextInt(7); count > 0; count--) {
                run = NetworkValues.concat(run, NetworkValues.lengthPrefixed(NetworkValues.randomField(random)));
            }
            frames.add(random.nextBoolean() ? run : Arrays.copyOf(run, random.nextInt(run.length)));
        }
        frames.addAll(List.of(
            NetworkValues.bytes(0x00), NetworkValues.bytes(0x00, 0x01), NetworkValues.bytes(0x80, 0x00), NetworkValues.bytes(0x81, 0x00, 0x2a),
            NetworkValues.bytes(0x80), NetworkValues.bytes(0x80, 0x80), NetworkValues.bytes(0x80, 0x80, 0x80), NetworkValues.bytes(0x80, 0x80, 0x80, 0x01),
            NetworkValues.bytes(0xff, 0xff, 0x7f), NetworkValues.bytes(0xff, 0xff, 0x7f, 0x01, 0x02), NetworkValues.bytes(0xff, 0xff, 0xff, 0x7f)
        ));

        List<Object[]> inputs = new ArrayList<>();
        for (byte[] frame : frames) {
            inputs.add(new Object[]{NetworkValues.buffer(random.nextInt(17), 0, frame)});
            byte[] read = new byte[1 + random.nextInt(4)];
            random.nextBytes(read);
            inputs.add(new Object[]{NetworkValues.buffer(random.nextInt(17), read.length, NetworkValues.concat(read, frame))});
        }
        return inputs;
    }

    // Runs of up to eight frames, a quarter of them long enough for a two-byte length, arriving in up to six chunks. Half
    // the cuts are one byte into a frame, inside its length when it takes two bytes, and the rest anywhere. The first
    // chunk is in the buffer, with up to 16 bytes of capacity to spare.
    private static List<Object[]> chunkedArrivals() {
        Random random = new Random(1105);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 300; i++) {
            byte[] run = new byte[0];
            List<Integer> starts = new ArrayList<>();
            for (int count = 1 + random.nextInt(8); count > 0; count--) {
                byte[] payload = NetworkValues.randomField(random);
                if (random.nextInt(4) == 0) {
                    payload = new byte[128 + random.nextInt(300)];
                    random.nextBytes(payload);
                }
                starts.add(run.length);
                run = NetworkValues.concat(run, NetworkValues.lengthPrefixed(payload));
            }
            int[] cuts = new int[random.nextInt(6)];
            for (int c = 0; c < cuts.length; c++) {
                cuts[c] = random.nextBoolean() ? starts.get(random.nextInt(starts.size())) + 1 : random.nextInt(run.length + 1);
            }
            Arrays.sort(cuts);
            byte[][] chunks = new byte[cuts.length][];
            for (int c = 0; c < cuts.length; c++) {
                chunks[c] = Arrays.copyOfRange(run, cuts[c], c + 1 < cuts.length ? cuts[c + 1] : run.length);
            }
            byte[] first = Arrays.copyOf(run, cuts.length > 0 ? cuts[0] : run.length);
            inputs.add(new Object[]{NetworkValues.buffer(random.nextInt(17), 0, first), chunks});
        }
        return inputs;
    }

    // Messages at each header length's bounds, and at 2^21, whose length would take four bytes: all readable or after
    // bytes already read, into the game's default buffer and into a full one, which has to grow. The largest only once.
    private static List<Object[]> bodies() {
        Random random = new Random(1104);
        List<Object[]> inputs = new ArrayList<>();
        for (int length : new int[]{0, 1, 127, 128, 16383, 16384, (1 << 21) - 1, 1 << 21}) {
            byte[] payload = new byte[length];
            random.nextBytes(payload);
            if (length >= (1 << 21) - 1) {
                inputs.add(new Object[]{NetworkValues.buffer(0, 0, payload), Unpooled.buffer()});
                continue;
            }
            for (ByteBuf message : List.of(NetworkValues.buffer(0, 0, payload), NetworkValues.buffer(3, 2, NetworkValues.concat(NetworkValues.bytes(1, 2), payload)))) {
                inputs.add(new Object[]{message, Unpooled.buffer()});
                inputs.add(new Object[]{message, NetworkValues.buffer(0, 1, NetworkValues.bytes(1, 2, 3))});
            }
        }
        return inputs;
    }

    private static String descriptor(final Class<?> type, final String name, final Class<?>... parameters) {
        try {
            return Descriptors.of(type.getDeclaredMethod(name, parameters));
        } catch (NoSuchMethodException e) {
            throw new IllegalStateException(e);
        }
    }

    private static final class FrameDecoder extends Varint21FrameDecoder {
        FrameDecoder() {
            super(null);
        }

        ByteBuf[] frames(final ByteBuf in) {
            List<Object> out = new ArrayList<>();
            int found;
            do {
                found = out.size();
                this.decode(null, in, out);
            } while (in.isReadable() && out.size() > found);
            return out.toArray(ByteBuf[]::new);
        }
    }

    private static final class LengthFieldPrepender extends Varint21LengthFieldPrepender {
        void prepend(final ByteBuf msg, final ByteBuf out) {
            this.encode(null, msg, out);
        }
    }
}
