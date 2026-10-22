package rustcraft.oracle.client.mixin;

import java.util.Collection;
import net.fabricmc.fabric.impl.networking.AbstractChanneledNetworkAddon;
import net.fabricmc.fabric.impl.networking.RegistrationPayload;
import net.fabricmc.fabric.impl.networking.client.ClientConfigurationNetworkAddon;
import net.fabricmc.fabric.impl.networking.client.ClientPlayNetworkAddon;
import net.minecraft.network.protocol.common.custom.CustomPacketPayload;
import net.minecraft.resources.Identifier;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;

/**
 * Keeps Fabric API's networking from announcing its channels to the server, which an unmodded client never does.
 *
 * <p>Fabric API (which the client game test API depends on) sends a {@code minecraft:register} custom payload naming
 * the channels it receives once the game protocol starts, after the server registers channels during configuration,
 * and whenever a receiver is added later. Each of those goes through {@code createRegistrationPayload}, and nothing is
 * sent when it returns null, as for no channels. Only the client's addons are stopped: the integrated server's are
 * left as they are.
 */
@Mixin(value = AbstractChanneledNetworkAddon.class, remap = false)
abstract class ChanneledNetworkAddonMixin {
    @Inject(method = "createRegistrationPayload", at = @At("HEAD"), cancellable = true)
    private void announceNoChannels(
        CustomPacketPayload.Type<RegistrationPayload> type,
        Collection<Identifier> channels,
        CallbackInfoReturnable<RegistrationPayload> cir
    ) {
        Object addon = this;
        if (addon instanceof ClientPlayNetworkAddon || addon instanceof ClientConfigurationNetworkAddon) {
            cir.setReturnValue(null);
        }
    }
}
