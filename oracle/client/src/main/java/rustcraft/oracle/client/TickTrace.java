package rustcraft.oracle.client;

import net.minecraft.client.Minecraft;
import org.jspecify.annotations.Nullable;

/**
 * The trace line of the tick the test thread is waiting for, written on the client thread as {@code Minecraft.tick}
 * returns ({@code MinecraftTickMixin}), so that it's the tick's own result: the packets the client handles before its
 * next tick (at the start of the next frame, before Fabric hands the client to the test thread) come after it
 * (TESTING.md).
 */
public final class TickTrace {
    private static volatile int armedTick = -1;
    private static volatile @Nullable String line;

    private TickTrace() {
    }

    /** Before {@code waitTick}: the next tick's line is the scenario's tick {@code tick}. */
    public static void arm(final int tick) {
        line = null;
        armedTick = tick;
    }

    /** On the client thread, at the end of every tick. */
    public static void afterTick(final Minecraft client) {
        int tick = armedTick;
        if (tick >= 0 && client.player != null) {
            line = TraceWriter.line(tick, client.player);
            armedTick = -1;
        }
    }

    /** After {@code waitTick}: the armed tick's line. */
    public static String take() {
        String taken = line;
        if (taken == null) {
            throw new IllegalStateException("no tick ran since the trace was armed");
        }
        line = null;
        return taken;
    }
}
