package rustcraft.oracle.client.mixin;

import net.minecraft.network.PacketListener;
import net.minecraft.network.PacketProcessor;
import net.minecraft.network.protocol.Packet;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfo;
import rustcraft.oracle.client.PacketArrivals;

/**
 * Counts the packets the client queues for its game thread, by type, for {@code wait_for} (TESTING.md). It only
 * counts: what's queued and when it's handled are vanilla's.
 */
@Mixin(PacketProcessor.class)
abstract class PacketProcessorMixin {
    @Inject(method = "scheduleIfPossible", at = @At("HEAD"))
    private <T extends PacketListener> void countArrival(final T listener, final Packet<T> packet, final CallbackInfo ci) {
        PacketArrivals.arrived(packet.type().id().toString());
    }
}
