package rustcraft.oracle.client.mixin;

import net.minecraft.client.Minecraft;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import rustcraft.oracle.client.TickTrace;

/** Writes the scenario's trace line as each client tick ends (TickTrace). It only reads the player. */
@Mixin(Minecraft.class)
abstract class MinecraftTickMixin {
    @Inject(method = "tick", at = @At("RETURN"))
    private void traceTick(final CallbackInfo ci) {
        TickTrace.afterTick((Minecraft)(Object)this);
    }
}
