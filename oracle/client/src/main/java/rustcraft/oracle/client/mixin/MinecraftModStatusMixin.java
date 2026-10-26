package rustcraft.oracle.client.mixin;

import net.minecraft.client.Minecraft;
import net.minecraft.util.ModCheck;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/**
 * Undoes what running under Fabric does to the game's own check of whether it's modified, so the oracle reports what
 * an unmodded client does.
 *
 * <p>{@code Minecraft.checkModStatus} is "very likely" modified when {@code Minecraft.class} has no signers, and under
 * Fabric's development launch the game's classes come from a remapped, unsigned jar through Knot's class loader; the
 * title screen then shows its version as "Minecraft 26.3 (Modded)". The released jar is signed, and with the brand
 * {@code ClientBrandRetrieverMixin} restores, an unmodded client finds its "jar signature and brand untouched".
 */
@Mixin(Minecraft.class)
abstract class MinecraftModStatusMixin {
    @Inject(method = "checkModStatus", at = @At("HEAD"), cancellable = true)
    private static void reportUnmodified(CallbackInfoReturnable<ModCheck> cir) {
        cir.setReturnValue(new ModCheck(ModCheck.Confidence.PROBABLY_NOT, "Client jar signature and brand is untouched"));
    }
}
