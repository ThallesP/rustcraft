package rustcraft.oracle.client;

import java.io.IOException;
import java.io.UncheckedIOException;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardOpenOption;
import net.minecraft.core.BlockPos;
import net.minecraft.core.SectionPos;
import net.minecraft.world.level.BlockAndLightGetter;
import net.minecraft.world.level.LightLayer;
import net.minecraft.world.level.block.Block;

/**
 * An extra dump for finding what the level's meshes were compiled from (TESTING.md): with
 * {@code RUSTCRAFT_ORACLE_LIGHT_DUMP} naming a file and {@code RUSTCRAFT_ORACLE_LIGHT_BOX} a box of blocks
 * ({@code x0,y0,z0,x1,y1,z1}), each section compile that reads the box (SectionCompilerMixin) and each screenshot
 * write a line with the block states and the sky and block light the box had for it. It only reads.
 */
public final class LightDump {
    private static final String FILE = System.getenv("RUSTCRAFT_ORACLE_LIGHT_DUMP");
    private static final int[] BOX = parseBox(System.getenv("RUSTCRAFT_ORACLE_LIGHT_BOX"));

    private LightDump() {
    }

    private static int[] parseBox(final String box) {
        if (box == null || box.isEmpty()) {
            return null;
        }
        String[] parts = box.split(",");
        int[] values = new int[6];
        for (int i = 0; i < 6; i++) {
            values[i] = Integer.parseInt(parts[i].trim());
        }
        return values;
    }

    public static boolean enabled() {
        return FILE != null && !FILE.isEmpty() && BOX != null;
    }

    /** A section's compile, from its region, if the region (the section and its neighbours) reaches the box. */
    public static void compile(final SectionPos section, final BlockAndLightGetter region) {
        if (!enabled()) {
            return;
        }
        int minX = Math.max(BOX[0], section.minBlockX() - 16);
        int minY = Math.max(BOX[1], section.minBlockY() - 16);
        int minZ = Math.max(BOX[2], section.minBlockZ() - 16);
        int maxX = Math.min(BOX[3], section.maxBlockX() + 16);
        int maxY = Math.min(BOX[4], section.maxBlockY() + 16);
        int maxZ = Math.min(BOX[5], section.maxBlockZ() + 16);
        if (minX > maxX || minY > maxY || minZ > maxZ) {
            return;
        }
        String head = "\"compile\":[" + section.x() + "," + section.y() + "," + section.z() + "],\"thread\":\"" + Thread.currentThread().getName() + "\"";
        write(head, region, minX, minY, minZ, maxX, maxY, maxZ);
    }

    /**
     * A compile's meshes, when the light is dumped: each layer's vertex bytes, into the directory {@code <dump>.meshes}
     * as {@code <n>-<x>_<y>_<z>-<layer>.bin}, by the compiles' order.
     */
    public static synchronized void mesh(final SectionPos section, final net.minecraft.client.renderer.chunk.SectionCompiler.Results results) {
        if (!enabled()) {
            return;
        }
        String dir = FILE + ".meshes";
        int n = meshes++;
        results.renderedLayers.forEach((layer, mesh) -> {
            java.nio.ByteBuffer vertices = mesh.vertexBuffer().duplicate();
            byte[] bytes = new byte[vertices.remaining()];
            vertices.get(bytes);
            try {
                Files.createDirectories(Path.of(dir));
                Files.write(Path.of(dir, n + "-" + section.x() + "_" + section.y() + "_" + section.z() + "-" + layer.name() + ".bin"), bytes);
            } catch (IOException exception) {
                throw new UncheckedIOException(exception);
            }
        });
    }

    private static int meshes;

    /** The level as a screenshot sees it. */
    public static void screenshot(final String name, final BlockAndLightGetter level) {
        if (enabled()) {
            write("\"screenshot\":\"" + name + "\"", level, BOX[0], BOX[1], BOX[2], BOX[3], BOX[4], BOX[5]);
        }
    }

    private static synchronized void write(
        final String head, final BlockAndLightGetter level, final int minX, final int minY, final int minZ, final int maxX, final int maxY, final int maxZ
    ) {
        StringBuilder line = new StringBuilder("{").append(head).append(",\"nanos\":").append(System.nanoTime()).append(",\"blocks\":[");
        boolean first = true;
        BlockPos.MutableBlockPos pos = new BlockPos.MutableBlockPos();
        for (int y = minY; y <= maxY; y++) {
            for (int z = minZ; z <= maxZ; z++) {
                for (int x = minX; x <= maxX; x++) {
                    pos.set(x, y, z);
                    line.append(first ? "" : ",").append('[').append(x).append(',').append(y).append(',').append(z).append(',')
                        .append(Block.getId(level.getBlockState(pos))).append(',')
                        .append(level.getBrightness(LightLayer.SKY, pos)).append(',')
                        .append(level.getBrightness(LightLayer.BLOCK, pos)).append(']');
                    first = false;
                }
            }
        }
        line.append("]}\n");
        try {
            Files.writeString(Path.of(FILE), line, StandardCharsets.UTF_8, StandardOpenOption.CREATE, StandardOpenOption.APPEND);
        } catch (IOException exception) {
            throw new UncheckedIOException(exception);
        }
    }
}
