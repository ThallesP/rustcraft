package rustcraft.oracle.client.mixin;

import net.minecraft.client.Minecraft;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;
import rustcraft.oracle.client.ServerTicks;

/**
 * Keeps the client's tick timer at 20 ticks a second in a paced scenario, where the server is slowed down (ServerTicks):
 * the test releases each tick at its slot, and a timer at the server's rate would only have the client ready for one
 * on its own grid of the server's tick, which a slow frame puts behind the slots for good. The rate only times the
 * client's frames (DeltaTracker); the game's ticks don't read it. The port's timer keeps 20 too.
 */
@Mixin(Minecraft.class)
abstract class MinecraftTickTargetMixin {
    @Inject(method = "getTickTargetMillis", at = @At("HEAD"), cancellable = true)
    private void keepDefaultTickTarget(final float defaultTickTargetMillis, final CallbackInfoReturnable<Float> cir) {
        if (ServerTicks.isPaced()) {
            cir.setReturnValue(defaultTickTargetMillis);
        }
    }
}
