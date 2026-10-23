package rustcraft.oracle.client;

import java.io.BufferedWriter;
import java.io.IOException;
import java.io.UncheckedIOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.Locale;
import net.minecraft.client.multiplayer.ClientLevel;
import net.minecraft.client.player.LocalPlayer;
import net.minecraft.world.entity.Entity;
import net.minecraft.world.phys.Vec3;

/** Writes trace.jsonl: one line per client tick, floats as raw IEEE bits (TESTING.md). */
public final class TraceWriter implements AutoCloseable {
    private final BufferedWriter writer;

    public TraceWriter(Path file) {
        try {
            Files.createDirectories(file.getParent());
            this.writer = Files.newBufferedWriter(file, StandardCharsets.UTF_8);
        } catch (IOException exception) {
            throw new UncheckedIOException(exception);
        }
    }

    /** Runs on the client thread, right after the tick ran. */
    public static String line(int tick, LocalPlayer player) {
        Vec3 delta = player.getDeltaMovement();
        return "{\"tick\":" + tick
            + ",\"pos\":[" + bits(player.getX()) + "," + bits(player.getY()) + "," + bits(player.getZ()) + "]"
            + ",\"delta\":[" + bits(delta.x) + "," + bits(delta.y) + "," + bits(delta.z) + "]"
            + ",\"y_rot\":" + bits(player.getYRot())
            + ",\"x_rot\":" + bits(player.getXRot())
            + ",\"on_ground\":" + player.onGround()
            + ",\"horizontal_collision\":" + player.horizontalCollision
            + ",\"vertical_collision\":" + player.verticalCollision
            + ",\"sprinting\":" + player.isSprinting()
            + ",\"crouching\":" + player.isCrouching()
            + ",\"pose\":\"" + player.getPose().name() + "\""
            + entities(player)
            + "}";
    }

    /**
     * The other entities the client has, in the order it added them ({@code entitiesForRendering}), each its type and
     * where it is after the tick; nothing when there are none, so a scenario without them has the lines it always had.
     */
    private static String entities(LocalPlayer player) {
        StringBuilder entities = new StringBuilder();
        for (Entity entity : ((ClientLevel)player.level()).entitiesForRendering()) {
            if (entity == player) {
                continue;
            }
            if (!entities.isEmpty()) {
                entities.append(',');
            }
            entities.append("{\"type\":\"").append(entity.typeHolder().getRegisteredName()).append('"')
                .append(",\"pos\":[").append(bits(entity.getX())).append(',').append(bits(entity.getY())).append(',')
                .append(bits(entity.getZ())).append("]}");
        }
        return entities.isEmpty() ? "" : ",\"entities\":[" + entities + "]";
    }

    public void write(String line) {
        try {
            this.writer.write(line);
            this.writer.write('\n');
        } catch (IOException exception) {
            throw new UncheckedIOException(exception);
        }
    }

    @Override
    public void close() {
        try {
            this.writer.close();
        } catch (IOException exception) {
            throw new UncheckedIOException(exception);
        }
    }

    private static String bits(double value) {
        return String.format(Locale.ROOT, "\"%016x\"", Double.doubleToRawLongBits(value));
    }

    private static String bits(float value) {
        return String.format(Locale.ROOT, "\"%08x\"", Float.floatToRawIntBits(value));
    }
}
