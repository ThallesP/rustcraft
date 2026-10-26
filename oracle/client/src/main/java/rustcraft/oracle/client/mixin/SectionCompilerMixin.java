package rustcraft.oracle.client.mixin;

import com.mojang.blaze3d.vertex.VertexSorting;
import net.minecraft.client.renderer.SectionBufferBuilderPack;
import net.minecraft.client.renderer.chunk.RenderSectionRegion;
import net.minecraft.client.renderer.chunk.SectionCompiler;
import net.minecraft.core.SectionPos;
import org.spongepowered.asm.mixin.Mixin;
import org.spongepowered.asm.mixin.injection.At;
import org.spongepowered.asm.mixin.injection.Inject;
import org.spongepowered.asm.mixin.injection.callback.CallbackInfoReturnable;
import rustcraft.oracle.client.LightDump;

/** Dumps what a section's compile reads of the light dump's box (LightDump), when it's asked for. It only reads. */
@Mixin(SectionCompiler.class)
abstract class SectionCompilerMixin {
    @Inject(method = "compile", at = @At("HEAD"))
    private void dumpLight(
        final SectionPos sectionPos,
        final RenderSectionRegion region,
        final VertexSorting vertexSorting,
        final SectionBufferBuilderPack builders,
        final CallbackInfoReturnable<SectionCompiler.Results> cir
    ) {
        LightDump.compile(sectionPos, region);
    }

    @Inject(method = "compile", at = @At("RETURN"))
    private void dumpMesh(
        final SectionPos sectionPos,
        final RenderSectionRegion region,
        final VertexSorting vertexSorting,
        final SectionBufferBuilderPack builders,
        final CallbackInfoReturnable<SectionCompiler.Results> cir
    ) {
        LightDump.mesh(sectionPos, cir.getReturnValue());
    }
}
