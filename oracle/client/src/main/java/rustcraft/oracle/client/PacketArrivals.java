package rustcraft.oracle.client;

import java.util.Map;
import java.util.concurrent.ConcurrentHashMap;
import java.util.concurrent.atomic.AtomicInteger;

/**
 * How many clientbound packets of each type the client has queued for its game thread ({@code PacketProcessor
 * .scheduleIfPossible}, counted by {@code PacketProcessorMixin} on the Netty thread), which a scenario step's
 * {@code wait_for} waits on (TESTING.md), and when the last of each came ({@code System.nanoTime()}), which a paced
 * scenario takes the server's ticks from ({@code ServerTicks}). A bundle is queued whole, so it counts as one
 * {@code minecraft:bundle}.
 */
public final class PacketArrivals {
    private static final Map<String, AtomicInteger> COUNTS = new ConcurrentHashMap<>();
    private static final Map<String, Long> LAST_NANOS = new ConcurrentHashMap<>();

    private PacketArrivals() {
    }

    public static void arrived(final String type) {
        LAST_NANOS.put(type, System.nanoTime());
        COUNTS.computeIfAbsent(type, ignored -> new AtomicInteger()).incrementAndGet();
    }

    public static int count(final String type) {
        AtomicInteger count = COUNTS.get(type);
        return count == null ? 0 : count.get();
    }

    /** When the last packet of the type came, in {@code System.nanoTime()}'s clock. */
    public static long lastNanos(final String type) {
        Long nanos = LAST_NANOS.get(type);
        if (nanos == null) {
            throw new IllegalStateException("no " + type + " has come");
        }
        return nanos;
    }
}
