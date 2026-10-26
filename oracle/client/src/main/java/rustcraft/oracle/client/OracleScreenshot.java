package rustcraft.oracle.client;

import com.mojang.blaze3d.platform.NativeImage;
import com.mojang.blaze3d.systems.RenderSystem;
import java.io.IOException;
import java.io.UncheckedIOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.TimeUnit;
import net.fabricmc.fabric.api.client.gametest.v1.context.ClientGameTestContext;
import net.minecraft.client.DeltaTracker;
import net.minecraft.client.Screenshot;

/**
 * Captures the current frame without advancing the game.
 *
 * <p>Fabric's {@code takeScreenshot} runs ticks until the GPU readback lands, which would move the player between
 * trace lines. This renders the frame the same way, then polls the readback fence on the client thread instead.
 */
public final class OracleScreenshot {
    private static final long READBACK_TIMEOUT_NANOS = TimeUnit.SECONDS.toNanos(30);

    private OracleScreenshot() {
    }

    public static void take(ClientGameTestContext context, Path file) {
        NativeImage image = context.computeOnClient(client -> {
            client.gameRenderer.update(DeltaTracker.ONE);
            client.gameRenderer.extract(DeltaTracker.ONE, true);
            client.gameRenderer.render();
            RenderSystem.getDevice().createCommandEncoder().submit();

            CompletableFuture<NativeImage> readback = new CompletableFuture<>();
            Screenshot.takeScreenshot(client.gameRenderer.mainRenderTarget(), readback::complete);
            // The readback fence belongs to the open submission, so close it before polling.
            RenderSystem.getDevice().createCommandEncoder().submit();

            // On Vulkan the copy's callback isn't a fenced task: it runs when the encoder's destruction queue comes round
            // to its submit again, two submits on (VulkanCommandEncoder.submit), so the loop submits (nothing) until then.
            boolean callbackRunsOnSubmit = RenderSystem.getDevice().getDeviceInfo().backendName().equals("Vulkan");
            long deadline = System.nanoTime() + READBACK_TIMEOUT_NANOS;
            while (!readback.isDone()) {
                if (System.nanoTime() > deadline) {
                    throw new AssertionError("Screenshot readback did not finish in 30s");
                }
                if (callbackRunsOnSubmit) {
                    RenderSystem.getDevice().createCommandEncoder().submit();
                }
                RenderSystem.executePendingTasks();
                Thread.onSpinWait();
            }
            return readback.join();
        });

        try (image) {
            Files.createDirectories(file.getParent());
            image.writeToFile(file);
        } catch (IOException exception) {
            throw new UncheckedIOException(exception);
        }
    }
}
