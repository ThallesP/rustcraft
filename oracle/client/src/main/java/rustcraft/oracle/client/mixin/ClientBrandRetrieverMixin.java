package rustcraft.oracle.client.mixin;

import net.minecraft.client.ClientBrandRetriever;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/**
 * Undoes Fabric Loader's branding of the client, so the oracle reports vanilla's brand.
 *
 * <p>Fabric Loader's {@code BrandingPatch} makes {@code getClientModName} return {@code "fabric"}, which the client sends
 * as its brand payload while it joins a server and shows in F3. An unmodded client returns {@code "vanilla"}.
 */
@Mixin(ClientBrandRetriever.class)
abstract class ClientBrandRetrieverMixin {
    @Inject(method = "getClientModName", at = @At("HEAD"), cancellable = true)
    private static void returnVanillaName(CallbackInfoReturnable<String> cir) {
        cir.setReturnValue(ClientBrandRetriever.VANILLA_NAME);
    }
}
