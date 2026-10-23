package rustcraft.oracle.client;

import java.util.Locale;
import java.util.function.Predicate;
import net.fabricmc.fabric.api.client.gametest.v1.context.ClientGameTestContext;
import net.minecraft.client.Minecraft;

/**
 * Ticks the client in step with the server for a scenario with a {@code tick_rate}: one client tick between each two
 * of the server's ticks (TESTING.md, "Paced scenarios").
 *
 * <p>The server ticks on its own clock, at fixed times ({@code MinecraftServer.runServer} adds a tick's nanoseconds to
 * the next tick's time), and before each tick handles the packets that came since the last. Left to their own clocks,
 * the client's ticks fall anywhere between the server's, and now and then two of them fall between two of the
 * server's, or none, which the server sees: a player under water loses air every server tick, and a falling player's
 * fall grows every server tick. A paced scenario slows the server down ({@code tick rate}), so that a server tick
 * lasts far longer than the client's frames, and releases each of the client's ticks at a slot halfway between two of
 * the server's. The server's ticks are found from a packet it sends as a tick ends ({@code set_time}, every 20 ticks),
 * once its new rate holds; the slots are that packet's arrival plus half a tick, plus whole ticks.
 *
 * <p>At a slot, the client first handles what it has queued, so that whatever the server sent before it is handled
 * before the tick, then the test acts (sends commands, the marker) and ticks. Every tick of the run from then on, up
 * to the disconnection, takes its own slot, so the server ticks exactly once between two client ticks. A tick that
 * ends more than a third of a tick after its slot may have come to the server after its next tick; one of the
 * scenario's ticks that does fails the run ({@code finish}), since the server's answers then aren't the scenario's.
 */
public final class ServerTicks {
    private static final int LOAD_TIMEOUT_TICKS = 60 * 20;
    /** The frame rate a paced client is held to, the lowest {@code maxFps} takes. */
    private static final int FRAMES_PER_SECOND = 10;
    /** What a run that fell behind fails with, which tools/oracle_client.py runs again. */
    public static final String FELL_BEHIND = "fell behind the server's ticks";
    /** Whether the run is paced, from the {@code tick rate} command on (MinecraftTickTargetMixin). */
    private static volatile boolean paced;
    /** When the client's last tick ended ({@code MinecraftTickMixin}). */
    private static volatile long lastTickEndNanos;

    private final ClientGameTestContext context;
    /** The server's tick, 0 when the scenario isn't paced. */
    private final long periodNanos;
    private long nextSlotNanos;
    /** The slot the test is at, while its tick hasn't run yet, or -1. */
    private long slotNanos = -1;
    /** Whether the ticks are the scenario's, from the marker on, and how many of them missed their slot. */
    private boolean replaying;
    private int missed;

    private ServerTicks(final ClientGameTestContext context, final long periodNanos, final long nextSlotNanos) {
        this.context = context;
        this.periodNanos = periodNanos;
        this.nextSlotNanos = nextSlotNanos;
    }

    /** The client ticking on its own clock, as unpaced scenarios do. */
    static ServerTicks unpaced(final ClientGameTestContext context) {
        return new ServerTicks(context, 0, 0);
    }

    /**
     * Sets the server's tick rate ({@code tick rate}), waits for the client to have it ({@code ticking_state}) and for
     * the next {@code set_time} the server sends as a tick ends, and paces the client from the slot after it.
     */
    static ServerTicks pace(final ClientGameTestContext context, final float tickRate) {
        paced = true;
        int states = PacketArrivals.count("minecraft:ticking_state");
        context.runOnClient(client -> {
            // The client only ticks at the slots, so it doesn't need more frames than this, and the frames llvmpipe
            // draws at 120 a second take the CPU the server's ticks and the client's own need to keep to the slots.
            client.options.framerateLimit().set(FRAMES_PER_SECOND);
            client.player.connection.sendCommand(tickRateCommand(tickRate));
        });
        context.waitFor(client -> PacketArrivals.count("minecraft:ticking_state") > states, LOAD_TIMEOUT_TICKS);
        // The server synchronizes the time every 20 ticks, as the tick ends (MinecraftServer.tickChildren), and no
        // command has made it send one out of turn since the rate changed.
        int times = PacketArrivals.count("minecraft:set_time");
        context.waitFor(client -> PacketArrivals.count("minecraft:set_time") > times, LOAD_TIMEOUT_TICKS);
        // TickRateManager.setTickRate's nanoseconds per tick.
        long periodNanos = (long)(1.0E9 / tickRate);
        long slot = PacketArrivals.lastNanos("minecraft:set_time") + periodNanos / 2;
        while (slot < System.nanoTime()) {
            slot += periodNanos;
        }
        return new ServerTicks(context, periodNanos, slot);
    }

    public static boolean isPaced() {
        return paced;
    }

    /** On the client thread, at the end of every tick. */
    public static void tickEnded() {
        lastTickEndNanos = System.nanoTime();
    }

    /** {@code tick rate <rate>}, the rate as {@code Float.toString} prints it. */
    static String tickRateCommand(final float tickRate) {
        return String.format(Locale.ROOT, "tick rate %s", tickRate);
    }

    /** From the scenario's first tick on, a tick that misses its slot fails the run. */
    void startScenario() {
        this.replaying = true;
    }

    /**
     * Waits for the next slot, if the test isn't at one already, and has the client handle what it has queued. Paced,
     * whatever the test does to the client between ticks is done at a slot, so the server handles it in the same tick
     * as the client's next tick. The client is only there once it's parked for its next tick (handling its queue
     * waits for that): one that gets there more than a third of a tick after the slot waits for the next slot instead,
     * and the server ticks without it.
     */
    void awaitSlot() {
        if (this.periodNanos == 0 || this.slotNanos >= 0) {
            return;
        }

        while (true) {
            long early = this.nextSlotNanos - System.nanoTime();
            if (early > 0) {
                sleepNanos(early);
            }
            this.context.runOnClient(client -> client.packetProcessor().processQueuedPackets());
            long slot = this.nextSlotNanos;
            long late = System.nanoTime() - slot;
            this.nextSlotNanos += this.periodNanos;
            if (late <= this.periodNanos / 3) {
                this.slotNanos = slot;
                return;
            }
            this.miss("got to its slot", late);
        }
    }

    /** One client tick, at its slot when paced. */
    void tick() {
        this.awaitSlot();
        long slot = this.slotNanos;
        this.slotNanos = -1;
        this.context.waitTick();
        if (slot >= 0 && lastTickEndNanos - slot > this.periodNanos / 3) {
            this.miss("ended its tick", lastTickEndNanos - slot);
        }
    }

    void waitTicks(final int ticks) {
        if (this.periodNanos == 0) {
            this.context.waitTicks(ticks);
            return;
        }

        for (int i = 0; i < ticks; i++) {
            this.tick();
        }
    }

    /**
     * {@code ClientGameTestContext.waitFor}: ticks until the condition holds, checked before each tick. Paced, it's
     * checked at the tick's slot, and the test is left at the slot where it held.
     */
    void waitFor(final Predicate<Minecraft> condition, final int timeoutTicks) {
        if (this.periodNanos == 0) {
            this.context.waitFor(condition, timeoutTicks);
            return;
        }

        for (int ticks = 0; ; ticks++) {
            this.awaitSlot();
            if (this.context.computeOnClient(condition::test)) {
                return;
            }
            if (ticks >= timeoutTicks) {
                throw new IllegalStateException("timed out after " + ticks + " ticks");
            }
            this.tick();
        }
    }

    /** Fails the run if one of the scenario's ticks missed its slot. */
    void finish() {
        if (this.missed > 0) {
            throw new IllegalStateException("the client " + FELL_BEHIND + " " + this.missed + " times in the scenario");
        }
    }

    private void miss(final String what, final long lateNanos) {
        if (this.replaying) {
            this.missed++;
        }
        System.out.printf(
            "rustcraft oracle: %s %d ms after its slot, %s%n",
            what,
            lateNanos / 1_000_000,
            this.replaying ? "in the scenario" : "before the scenario"
        );
    }

    private static void sleepNanos(final long nanos) {
        try {
            Thread.sleep(nanos / 1_000_000, (int)(nanos % 1_000_000));
        } catch (InterruptedException exception) {
            Thread.currentThread().interrupt();
            throw new IllegalStateException(exception);
        }
    }
}
