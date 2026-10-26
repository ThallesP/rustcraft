package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonPrimitive;
import com.mojang.renderpearl.api.GpuFormat;
import com.mojang.renderpearl.api.device.GpuSurface;
import com.mojang.renderpearl.api.pipeline.BlendFactor;
import com.mojang.renderpearl.api.pipeline.BlendOp;
import com.mojang.renderpearl.api.pipeline.ColorTargetState;
import com.mojang.renderpearl.api.pipeline.CompareOp;
import com.mojang.renderpearl.api.pipeline.PolygonMode;
import com.mojang.renderpearl.api.pipeline.PrimitiveTopology;
import com.mojang.renderpearl.api.pipeline.ShaderType;
import com.mojang.renderpearl.api.textures.AddressMode;
import com.mojang.renderpearl.api.textures.FilterMode;
import com.mojang.renderpearl.backend.vulkan.VulkanConst;
import java.io.IOException;
import java.util.List;
import java.util.Optional;
import java.util.function.ToIntFunction;

/**
 * The Vulkan backend's tables (VulkanConst): every enum constant's Vulkan value, every texture usage (the five bits) with
 * every format, every buffer usage (the ten bits), every format's aspects, and every color write mask. An enum argument
 * is its constant's name, a color target state its write mask (its blending and format aren't read).
 */
final class VulkanSuites {
    private static final String P = "Lcom/mojang/renderpearl/api/";

    private VulkanSuites() {
    }

    static List<Suite> suites() {
        return List.of(new VulkanConstSuite());
    }

    private static final class VulkanConstSuite implements Suite {
        @Override
        public String name() {
            return VulkanConst.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            for (GpuFormat format : GpuFormat.values()) {
                writer.result("toVk(" + P + "GpuFormat;)I", args(name(format)), new JsonPrimitive(VulkanConst.toVk(format)), null);
                writer.result(
                    "formatAspectMask(" + P + "GpuFormat;)I", args(name(format)), new JsonPrimitive(VulkanConst.formatAspectMask(format)), null
                );
                for (int usage = 0; usage < 32; usage++) {
                    writer.result(
                        "textureUsageToVk(I" + P + "GpuFormat;)I",
                        args(new JsonPrimitive(usage), name(format)),
                        new JsonPrimitive(VulkanConst.textureUsageToVk(usage, format)),
                        null
                    );
                }
            }
            for (int usage = 0; usage < 1024; usage++) {
                writer.result("bufferUsageToVk(I)I", args(new JsonPrimitive(usage)), new JsonPrimitive(VulkanConst.bufferUsageToVk(usage)), null);
            }
            constants(writer, "toVk(" + P + "textures/AddressMode;)I", AddressMode.values(), VulkanConst::toVk);
            constants(writer, "toVk(" + P + "textures/FilterMode;)I", FilterMode.values(), VulkanConst::toVk);
            constants(writer, "toVk(" + P + "pipeline/BlendFactor;)I", BlendFactor.values(), VulkanConst::toVk);
            constants(writer, "toVk(" + P + "pipeline/BlendOp;)I", BlendOp.values(), VulkanConst::toVk);
            constants(writer, "toVk(" + P + "pipeline/CompareOp;)I", CompareOp.values(), VulkanConst::toVk);
            constants(writer, "toVk(" + P + "pipeline/PolygonMode;)I", PolygonMode.values(), VulkanConst::toVk);
            constants(writer, "toVk(" + P + "pipeline/PrimitiveTopology;)I", PrimitiveTopology.values(), VulkanConst::toVk);
            constants(writer, "toVk(" + P + "device/GpuSurface$PresentMode;)I", GpuSurface.PresentMode.values(), VulkanConst::toVk);
            constants(writer, "toVk(" + P + "pipeline/ShaderType;)I", ShaderType.values(), VulkanConst::toVk);
            for (int writeMask = 0; writeMask < 16; writeMask++) {
                ColorTargetState state = new ColorTargetState(Optional.empty(), GpuFormat.RGBA8_UNORM, writeMask);
                writer.result(
                    "toVk(" + P + "pipeline/ColorTargetState;)I",
                    args(new JsonPrimitive(writeMask)),
                    new JsonPrimitive(VulkanConst.toVk(state)),
                    null
                );
            }
        }

        private static <E extends Enum<E>> void constants(
            final GoldenWriter writer, final String method, final E[] constants, final ToIntFunction<E> toVk
        ) throws IOException {
            for (E constant : constants) {
                writer.result(method, args(name(constant)), new JsonPrimitive(toVk.applyAsInt(constant)), null);
            }
        }

        private static JsonPrimitive name(final Enum<?> constant) {
            return new JsonPrimitive(constant.name());
        }

        private static JsonArray args(final JsonElement... args) {
            JsonArray array = new JsonArray();
            for (JsonElement arg : args) {
                array.add(arg);
            }
            return array;
        }
    }
}
