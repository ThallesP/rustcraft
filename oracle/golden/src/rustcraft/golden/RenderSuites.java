package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonNull;
import com.google.gson.JsonPrimitive;
import com.mojang.blaze3d.systems.RenderSystem;
import com.mojang.blaze3d.vertex.DefaultVertexFormat;
import com.mojang.renderpearl.api.device.DeviceFeatures;
import com.mojang.renderpearl.api.device.DeviceInfo;
import com.mojang.renderpearl.api.device.GpuDevice;
import com.mojang.renderpearl.api.device.HintsAndWorkarounds;
import com.mojang.renderpearl.api.pipeline.BindGroupLayout;
import com.mojang.renderpearl.api.pipeline.BlendEquation;
import com.mojang.renderpearl.api.pipeline.BlendFunction;
import com.mojang.renderpearl.api.pipeline.ColorTargetState;
import com.mojang.renderpearl.api.pipeline.DepthStencilState;
import com.mojang.renderpearl.api.pipeline.RenderPipeline;
import com.mojang.renderpearl.api.pipeline.ShaderSource;
import com.mojang.renderpearl.api.pipeline.ShaderType;
import com.mojang.renderpearl.api.vertex.VertexFormat;
import com.mojang.renderpearl.api.vertex.VertexFormatElement;
import com.mojang.renderpearl.backend.api.BackendRenderPipeline;
import com.mojang.renderpearl.backend.api.GpuDeviceBackend;
import com.mojang.renderpearl.backend.api.SpvModule;
import com.mojang.renderpearl.backend.opengl.GlPipelineRecompiler;
import com.mojang.renderpearl.frontend.shaders.GlslCompiler;
import com.mojang.renderpearl.frontend.shaders.PipelineBuilder;
import com.mojang.renderpearl.frontend.shaders.SpvUtil;
import it.unimi.dsi.fastutil.objects.Object2IntOpenHashMap;
import it.unimi.dsi.fastutil.objects.ReferenceArrayList;
import java.io.IOException;
import java.io.InputStream;
import java.io.UncheckedIOException;
import java.lang.reflect.Field;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.lang.reflect.Proxy;
import java.nio.ByteBuffer;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.HexFormat;
import java.util.LinkedHashMap;
import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.Set;
import java.util.TreeMap;
import java.util.jar.JarEntry;
import java.util.jar.JarFile;
import net.minecraft.client.renderer.BindGroupLayouts;
import net.minecraft.client.renderer.RenderPipelines;
import net.minecraft.client.renderer.ShaderDefines;
import net.minecraft.client.renderer.ShaderManager;
import net.minecraft.client.renderer.oit.OitPipelineSet;
import net.minecraft.resources.Identifier;
import net.minecraft.server.packs.resources.Resource;
import net.minecraft.server.packs.resources.ResourceManager;

/**
 * The renderer's pipelines and the shaders vanilla compiles for them (TESTING.md, "Pipelines and shaders").
 *
 * <p>Shaders take vanilla's path: the GLSL in the client jar's {@code assets/minecraft/shaders/}, loaded by
 * {@code ShaderManager.loadConfigs}, compiled to SPIR-V by {@code GlslCompiler} (shaderc), reflected by
 * {@code SPIRVModule} (SPIRV-Cross), its bindings remapped by {@code PipelineBuilder}, and translated to GLSL 330 by
 * {@code GlPipelineRecompiler}, whose text is what the OpenGL backend hands the driver. A device's flags change the
 * text, so each case says which: {@code [isZZeroToOne, shaderDrawParameters, isExplicitDepthRequired]}. Mesa's
 * llvmpipe, which the oracle client renders with, has {@code GL_ARB_clip_control} and
 * {@code GL_ARB_shader_draw_parameters} and isn't an Apple GPU, so it's {@code [true, true, false]}: every pipeline
 * gets that, and a few get the other seven combinations too.
 */
final class RenderSuites {
    private static final List<Boolean> LLVMPIPE = List.of(true, true, false);
    // Pipelines whose shaders read each flag: depth (every projection), the instance index (multidraw terrain), a
    // texel buffer (clouds), the OIT passes, the GUI.
    private static final List<String> ALL_FLAGS_PIPELINES = List.of(
        "minecraft:pipeline/solid_terrain", "minecraft:pipeline/solid_terrain_multidraw", "minecraft:pipeline/clouds",
        "minecraft:pipeline/oit_accumulate_entity", "minecraft:pipeline/gui_textured"
    );

    private RenderSuites() {
    }

    static List<Suite> suites() {
        List<Suite> suites = new ArrayList<>(List.of(
            new FieldsSuite(DefaultVertexFormat.class),
            new FieldsSuite(BindGroupLayouts.class),
            new RenderPipelinesSuite(),
            new CompileSuite(GlslCompiler.class.getName(), CompileSuite::writeCompileToSpv),
            new CompileSuite("com.mojang.renderpearl.frontend.shaders.SPIRVModule", CompileSuite::writeReflect),
            new CompileSuite(PipelineBuilder.class.getName(), CompileSuite::writeCreateInfo),
            new CompileSuite(GlPipelineRecompiler.class.getName(), CompileSuite::writeDecompiled)
        ));
        suites.addAll(TextureSuites.suites());
        suites.addAll(CameraSuites.suites());
        return suites;
    }

    // ---- Encodings ----

    static JsonElement encode(final Object value) {
        return switch (value) {
            case null -> JsonNull.INSTANCE;
            case RenderPipeline pipeline -> pipeline(pipeline);
            case RenderPipeline.Snippet snippet -> snippet(snippet);
            case OitPipelineSet set -> array(pipeline(set.depthBoundsPipeline()), pipeline(set.transmittancePipeline()), pipeline(set.accumulatePipeline()));
            case VertexFormat format -> vertexFormat(format);
            case BindGroupLayout layout -> bindGroupLayout(layout);
            case ColorTargetState state -> colorTargetState(state);
            case DepthStencilState state -> depthStencilState(state);
            case BlendFunction function -> array(blendEquation(function.color()), blendEquation(function.alpha()));
            case Enum<?> constant -> new JsonPrimitive(constant.name());
            case Identifier id -> new JsonPrimitive(id.toString());
            case Optional<?> optional -> encode(optional.orElse(null));
            case String string -> new JsonPrimitive(string);
            default -> throw new IllegalArgumentException("no encoding for " + value.getClass());
        };
    }

    // [location, shaders, defines, bindGroupLayouts, colorTargetStates, depthStencilState, polygonMode, cull,
    //  vertexFormats, primitiveTopology, pushConstantSize, sortKey]
    private static JsonArray pipeline(final RenderPipeline pipeline) {
        return array(
            encode(pipeline.getLocation()),
            shaders(pipeline.getShaders()),
            defines(pipeline.getShaderDefines()),
            bindGroupLayouts(pipeline.getBindGroupLayouts()),
            list(pipeline.getColorTargetStates()),
            encode(pipeline.getDepthStencilState()),
            encode(pipeline.getPolygonMode()),
            new JsonPrimitive(pipeline.isCull()),
            list(pipeline.getVertexFormatBindings()),
            encode(pipeline.getPrimitiveTopology()),
            new JsonPrimitive(pipeline.pushConstantSize()),
            new JsonPrimitive(pipeline.getSortKey())
        );
    }

    // [shaders, defines, bindGroupLayouts, colorTargetStates, activeColorTargetStateCount, depthStencilState,
    //  polygonMode, cull, vertexFormats, primitiveTopology, pushConstantSize], an empty Optional as null.
    private static JsonArray snippet(final RenderPipeline.Snippet snippet) {
        return array(
            shaders(snippet.shaders()),
            snippet.shaderDefines().<JsonElement>map(RenderSuites::defines).orElse(JsonNull.INSTANCE),
            snippet.bindGroupLayouts().<JsonElement>map(RenderSuites::bindGroupLayouts).orElse(JsonNull.INSTANCE),
            list(java.util.Arrays.asList(snippet.colorTargetStates())),
            new JsonPrimitive(snippet.activeColorTargetStateCount()),
            encode(snippet.depthStencilState()),
            encode(snippet.polygonMode()),
            snippet.cull().<JsonElement>map(JsonPrimitive::new).orElse(JsonNull.INSTANCE),
            list(java.util.Arrays.asList(snippet.vertexFormatPerBuffer())),
            encode(snippet.vertexFormatMode()),
            new JsonPrimitive(snippet.pushConstantSize())
        );
    }

    // The EnumMap's entries, in ShaderType's order.
    private static JsonArray shaders(final Map<ShaderType, Identifier> shaders) {
        JsonArray array = new JsonArray();
        shaders.forEach((type, id) -> array.add(array(encode(type), encode(id))));
        return array;
    }

    // [[[key, value], …], [flag, …]], each in the order it was defined.
    static JsonArray defines(final ShaderDefines defines) {
        JsonArray values = new JsonArray();
        defines.values().forEach((key, value) -> values.add(array(new JsonPrimitive(key), new JsonPrimitive(value))));
        JsonArray flags = new JsonArray();
        defines.flags().forEach(flags::add);
        return array(values, flags);
    }

    // A pipeline's layouts are an ObjectOpenHashSet's, whose order changes from run to run: a layout's hash is its
    // record's, which hashes UniformType's constants by identity. So they're sorted by their uniforms' names.
    private static JsonArray bindGroupLayouts(final List<BindGroupLayout> layouts) {
        JsonArray array = new JsonArray();
        layouts.stream()
            .sorted(Comparator.comparing(layout -> String.join(",", layout.uniforms().stream().map(BindGroupLayout.UniformDescription::name).toList())))
            .forEach(layout -> array.add(bindGroupLayout(layout)));
        return array;
    }

    // [[name, type, gpuFormat], …]
    private static JsonArray bindGroupLayout(final BindGroupLayout layout) {
        JsonArray array = new JsonArray();
        for (BindGroupLayout.UniformDescription uniform : layout.uniforms()) {
            array.add(array(new JsonPrimitive(uniform.name()), encode(uniform.type()), encode(uniform.gpuFormat())));
        }
        return array;
    }

    // [stepRate, vertexSize, [[name, offset, format], …]]
    private static JsonArray vertexFormat(final VertexFormat format) {
        JsonArray elements = new JsonArray();
        for (VertexFormatElement element : format.getElements()) {
            elements.add(array(new JsonPrimitive(element.name()), new JsonPrimitive(element.offset()), encode(element.format())));
        }
        return array(new JsonPrimitive(format.getStepRate()), new JsonPrimitive(format.getVertexSize()), elements);
    }

    // [blendFunction, format, writeMask]
    private static JsonArray colorTargetState(final ColorTargetState state) {
        return array(encode(state.blendFunction()), encode(state.format()), new JsonPrimitive(state.writeMask()));
    }

    // [depthTest, writeDepth, depthBiasScaleFactor, depthBiasConstant]
    private static JsonArray depthStencilState(final DepthStencilState state) {
        return array(
            encode(state.depthTest()), new JsonPrimitive(state.writeDepth()),
            new JsonPrimitive(Values.floatBits(state.depthBiasScaleFactor())), new JsonPrimitive(Values.floatBits(state.depthBiasConstant()))
        );
    }

    private static JsonArray blendEquation(final BlendEquation equation) {
        return array(encode(equation.sourceFactor()), encode(equation.destFactor()), encode(equation.op()));
    }

    private static JsonArray list(final List<?> values) {
        JsonArray array = new JsonArray();
        values.forEach(value -> array.add(encode(value)));
        return array;
    }

    static JsonArray array(final JsonElement... elements) {
        JsonArray array = new JsonArray();
        for (JsonElement element : elements) {
            array.add(element);
        }
        return array;
    }

    private static JsonArray flags(final List<Boolean> flags) {
        JsonArray array = new JsonArray();
        flags.forEach(array::add);
        return array;
    }

    // ---- Suites ----

    /** Every public static field of a class, encoded as above. */
    private record FieldsSuite(Class<?> owner) implements Suite {
        @Override
        public String name() {
            return this.owner.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            for (Field field : this.owner.getDeclaredFields()) {
                if (Modifier.isPublic(field.getModifiers()) && Modifier.isStatic(field.getModifiers())) {
                    try {
                        Object value = field.get(null);
                        writer.field(field.getName(), encode(value));
                    } catch (IllegalAccessException e) {
                        throw new IllegalStateException(e);
                    }
                }
            }
        }
    }

    /** RenderPipelines' public pipelines, sets and snippets, and the lists ShaderManager compiles, in their order. */
    private static final class RenderPipelinesSuite implements Suite {
        @Override
        public String name() {
            return RenderPipelines.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            new FieldsSuite(RenderPipelines.class).write(writer);
            writer.result("requiredPipelines()Ljava/util/List;", new JsonArray(), locations(RenderPipelines.requiredPipelines()), null);
            writer.result("optionalPipelines()Ljava/util/List;", new JsonArray(), locations(RenderPipelines.optionalPipelines()), null);
        }

        private static JsonArray locations(final List<RenderPipeline> pipelines) {
            JsonArray array = new JsonArray();
            pipelines.forEach(pipeline -> array.add(pipeline.getLocation().toString()));
            return array;
        }
    }

    /**
     * Compiles every pipeline ShaderManager compiles, the way PipelineBuilder.compilePipeline does, for a device with
     * each case's flags. The four suites write parts of the same runs, which are made once.
     */
    private record CompileSuite(String name, Part part) implements Suite {
        private static Map<List<Boolean>, Map<String, Run>> runs;

        interface Part {
            void write(GoldenWriter writer, List<Boolean> flags, Run run, Set<JsonElement> written) throws IOException;
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            Set<JsonElement> written = new java.util.HashSet<>();
            for (Map.Entry<List<Boolean>, Map<String, Run>> entry : runs().entrySet()) {
                for (Run run : entry.getValue().values()) {
                    this.part.write(writer, entry.getKey(), run, written);
                }
            }
        }

        // compileToSpv's arguments, without the source text, which is the shader's in the jar, and the ShaderSource,
        // which is ShaderManager's over the jar: [flags, name, type, defines] → the SPIR-V's bytes. Pipelines share
        // shaders, so each is written once.
        static void writeCompileToSpv(final GoldenWriter writer, final List<Boolean> flags, final Run run, final Set<JsonElement> written) throws IOException {
            for (Compiled compiled : run.compiled) {
                JsonArray args = array(flags(flags), new JsonPrimitive(compiled.name), encode(compiled.type), defines(run.pipeline.getShaderDefines()));
                if (written.add(args)) {
                    writer.result("compileToSpv", args, new JsonPrimitive(compiled.spirv), null);
                }
            }
        }

        // reflect() of each module compileToSpv made, with the same arguments: [inputs, outputs, descriptors, pushConstants].
        static void writeReflect(final GoldenWriter writer, final List<Boolean> flags, final Run run, final Set<JsonElement> written) throws IOException {
            for (Compiled compiled : run.compiled) {
                JsonArray args = array(flags(flags), new JsonPrimitive(compiled.name), encode(compiled.type), defines(run.pipeline.getShaderDefines()));
                if (written.add(args)) {
                    writer.result("reflect", args, compiled.reflection, null);
                }
            }
        }

        // generateBackendCreateInfo for the pipeline: [flags, location] → the CreateInfo and the uniform bindings.
        static void writeCreateInfo(final GoldenWriter writer, final List<Boolean> flags, final Run run, final Set<JsonElement> written) throws IOException {
            JsonArray args = array(flags(flags), encode(run.pipeline.getLocation()));
            writer.result("generateBackendCreateInfo", args, run.createInfo, null);
        }

        // decompileShaders: [flags, location] → [[shader name, GLSL], …], each the text handed to glShaderSource.
        static void writeDecompiled(final GoldenWriter writer, final List<Boolean> flags, final Run run, final Set<JsonElement> written) throws IOException {
            JsonArray args = array(flags(flags), encode(run.pipeline.getLocation()));
            writer.result("decompileShaders", args, run.decompiled, null);
        }

        private static synchronized Map<List<Boolean>, Map<String, Run>> runs() {
            if (runs == null) {
                runs = new LinkedHashMap<>();
                ShaderSource configs = loadConfigs();
                List<RenderPipeline> pipelines = new ArrayList<>(RenderPipelines.requiredPipelines());
                pipelines.addAll(RenderPipelines.optionalPipelines());
                pipelines.sort(Comparator.comparing(pipeline -> pipeline.getLocation().toString()));
                for (int combination = 0; combination < 8; combination++) {
                    List<Boolean> flags = List.of((combination & 4) == 0, (combination & 2) == 0, (combination & 1) != 0);
                    Map<String, Run> byLocation = new LinkedHashMap<>();
                    for (RenderPipeline pipeline : pipelines) {
                        if (flags.equals(LLVMPIPE) || ALL_FLAGS_PIPELINES.contains(pipeline.getLocation().toString())) {
                            byLocation.put(pipeline.getLocation().toString(), Run.of(pipeline, flags, configs));
                        }
                    }
                    runs.put(flags, byLocation);
                }
            }
            return runs;
        }
    }

    private record Compiled(String name, ShaderType type, String spirv, JsonArray reflection) {
    }

    private record Run(RenderPipeline pipeline, List<Compiled> compiled, JsonElement createInfo, JsonElement decompiled) {
        static Run of(final RenderPipeline pipeline, final List<Boolean> flags, final ShaderSource configs) {
            DeviceInfo info = installDevice(flags);
            try (GlslCompiler compiler = new GlslCompiler(info.isZZeroToOne(), info.features().shaderDrawParameters())) {
                List<Compiled> compiled = new ArrayList<>();
                for (Map.Entry<ShaderType, Identifier> shader : pipeline.getShaders().entrySet()) {
                    String name = shader.getValue().toString();
                    String source = configs.getShader(shader.getValue(), shader.getKey());
                    try (SpvModule module = compiler.compileToSpv(name, source, shader.getKey(), pipeline.getShaderDefines(), configs)) {
                        String spirv = hex(module.spv());
                        compiled.add(new Compiled(name, shader.getKey(), spirv, reflection(module.reflect())));
                    }
                }

                PipelineBuilder builder = new PipelineBuilder(backendDevice(info));
                ReferenceArrayList<BackendRenderPipeline.CreateInfo.Shader> shaders = new ReferenceArrayList<>();
                Object2IntOpenHashMap<String> uniformBindings = new Object2IntOpenHashMap<>();
                Method generate = PipelineBuilder.class.getDeclaredMethod(
                    "generateBackendCreateInfo", RenderPipeline.class, ShaderSource.class, ReferenceArrayList.class, Object2IntOpenHashMap.class
                );
                generate.setAccessible(true);
                BackendRenderPipeline.CreateInfo createInfo = (BackendRenderPipeline.CreateInfo)generate.invoke(builder, pipeline, configs, shaders, uniformBindings);
                if (createInfo == null) {
                    throw new IllegalStateException("vanilla couldn't compile " + pipeline.getLocation());
                }
                Map<BackendRenderPipeline.CreateInfo.Shader, String> decompiled = new GlPipelineRecompiler(null, info.features().shaderDrawParameters())
                    .decompileShaders(createInfo);
                JsonArray decompiledArray = new JsonArray();
                for (BackendRenderPipeline.CreateInfo.Shader shader : createInfo.shaders()) {
                    decompiledArray.add(array(new JsonPrimitive(shader.name()), new JsonPrimitive(decompiled.get(shader))));
                }
                JsonElement createInfoJson = encodeCreateInfo(createInfo, uniformBindings);
                createInfo.shaders().forEach(shader -> shader.module().close());
                builder.close();
                return new Run(pipeline, compiled, createInfoJson, decompiledArray);
            } catch (ReflectiveOperationException | com.mojang.renderpearl.util.ShaderCompileException e) {
                throw new IllegalStateException(e);
            }
        }
    }

    // [name, shaders [[name, entryPoint, type, spirv], …], vertexBuffers [[slot, stride, stepRate], …],
    //  attribBindings [[slot, location, offset, format], …], uniforms [[name, type, format], …], pushConstantsSize,
    //  depthStencilState, polygonMode, cull, colorTargetStates, primitiveTopology, uniformBindings [[name, binding], …]
    //  by binding]
    private static JsonArray encodeCreateInfo(final BackendRenderPipeline.CreateInfo info, final Object2IntOpenHashMap<String> uniformBindings) {
        JsonArray shaders = new JsonArray();
        for (BackendRenderPipeline.CreateInfo.Shader shader : info.shaders()) {
            shaders.add(array(new JsonPrimitive(shader.name()), new JsonPrimitive(shader.entryPoint()), encode(shader.module().type()), new JsonPrimitive(hex(shader.module().spv()))));
        }
        JsonArray vertexBuffers = new JsonArray();
        for (BackendRenderPipeline.CreateInfo.VertexBuffer buffer : info.vertexBuffers()) {
            vertexBuffers.add(array(new JsonPrimitive(buffer.bufferSlot()), new JsonPrimitive(buffer.stride()), new JsonPrimitive(buffer.stepRate())));
        }
        JsonArray attribBindings = new JsonArray();
        for (BackendRenderPipeline.CreateInfo.AttribBinding binding : info.attribBindings()) {
            attribBindings.add(array(new JsonPrimitive(binding.bufferSlot()), new JsonPrimitive(binding.location()), new JsonPrimitive(binding.offset()), encode(binding.format())));
        }
        JsonArray uniforms = new JsonArray();
        for (BindGroupLayout.UniformDescription uniform : info.uniforms()) {
            uniforms.add(array(new JsonPrimitive(uniform.name()), encode(uniform.type()), encode(uniform.gpuFormat())));
        }
        JsonArray bindings = new JsonArray();
        uniformBindings.object2IntEntrySet().stream()
            .sorted(Comparator.comparingInt(Object2IntOpenHashMap.Entry::getIntValue))
            .forEach(entry -> bindings.add(array(new JsonPrimitive(entry.getKey()), new JsonPrimitive(entry.getIntValue()))));
        return array(
            new JsonPrimitive(info.name()), shaders, vertexBuffers, attribBindings, uniforms, new JsonPrimitive(info.pushConstantsSize()),
            encode(info.depthStencilState()), encode(info.polygonMode()), new JsonPrimitive(info.cull()), list(info.colorTargetStates()),
            encode(info.primitiveTopology()), bindings
        );
    }

    // [inputs, outputs, descriptors, pushConstants]: an interface variable is [name, location, flat, component, type],
    // a descriptor [name, resourceType, set, binding, type] (in SpvUtil.DESCRIPTOR_TYPES' order, as descriptors() lists
    // them), a push constant block its size, and a type [baseType, dimensions, vectorSize, [arrayLength, …]].
    private static JsonArray reflection(final SpvModule.Reflection reflection) {
        JsonArray inputs = new JsonArray();
        reflection.inputs().forEach(variable -> inputs.add(interfaceVariable(variable)));
        JsonArray outputs = new JsonArray();
        reflection.outputs().forEach(variable -> outputs.add(interfaceVariable(variable)));
        JsonArray descriptors = new JsonArray();
        for (SpvModule.Reflection.Descriptor descriptor : reflection.descriptors()) {
            descriptors.add(array(
                new JsonPrimitive(descriptor.name()), new JsonPrimitive(descriptor.resourceType()), new JsonPrimitive(descriptor.descriptorSetIndex()),
                new JsonPrimitive(descriptor.binding()), type(descriptor.type())
            ));
        }
        for (int resourceType : SpvUtil.DESCRIPTOR_TYPES) {
            if (reflection.descriptors(resourceType).stream().anyMatch(descriptor -> descriptor.resourceType() != resourceType)) {
                throw new IllegalStateException("descriptors listed under the wrong type");
            }
        }
        JsonArray pushConstants = new JsonArray();
        reflection.pushConstants().forEach(pushConstant -> pushConstants.add(pushConstant.size()));
        return array(inputs, outputs, descriptors, pushConstants);
    }

    private static JsonArray interfaceVariable(final SpvModule.Reflection.InterfaceVariable variable) {
        return array(
            new JsonPrimitive(variable.name()), new JsonPrimitive(variable.location()), new JsonPrimitive(variable.hasDecoration(14)),
            new JsonPrimitive(variable.decoration(31)), type(variable.type())
        );
    }

    private static JsonArray type(final SpvModule.Reflection.Type type) {
        JsonArray lengths = new JsonArray();
        for (int i = 0; i < type.arrayDimensions(); i++) {
            lengths.add(type.arrayLength(i));
        }
        return array(new JsonPrimitive(type.baseType()), new JsonPrimitive(type.dimensions()), new JsonPrimitive(type.vectorSize()), lengths);
    }

    private static String hex(final ByteBuffer buffer) {
        byte[] bytes = new byte[buffer.remaining()];
        buffer.duplicate().get(bytes);
        return HexFormat.of().formatHex(bytes);
    }

    // ---- Stand-ins ----

    // The device's answers that the shader path reads: isZZeroToOne, features().shaderDrawParameters() and
    // hintsAndWorkarounds().isExplicitDepthRequired(). The rest are llvmpipe's from the oracle client's log.
    static DeviceInfo deviceInfo(final List<Boolean> flags) {
        return new DeviceInfo(
            "llvmpipe (LLVM 21.1.8, 256 bits)", "Mesa", "4.5 (Core Profile) Mesa 26.0.8-1ubuntu0.3", flags.get(0), "OpenGL", 1.0F, null,
            new DeviceFeatures(true, flags.get(1), false, true, true, true, true, true), Set.of(),
            new HintsAndWorkarounds(false, false, flags.get(2), false), null
        );
    }

    // RenderSystem's device, which GlslCompiler asks for its hints: a stand-in that only answers getDeviceInfo.
    private static DeviceInfo installDevice(final List<Boolean> flags) {
        DeviceInfo info = deviceInfo(flags);
        GpuDevice device = (GpuDevice)Proxy.newProxyInstance(
            GpuDevice.class.getClassLoader(), new Class<?>[]{GpuDevice.class}, (proxy, method, args) -> {
                if (method.getName().equals("getDeviceInfo")) {
                    return info;
                }
                throw new UnsupportedOperationException(method.getName());
            }
        );
        try {
            Field field = RenderSystem.class.getDeclaredField("DEVICE");
            field.setAccessible(true);
            field.set(null, device);
        } catch (ReflectiveOperationException e) {
            throw new IllegalStateException(e);
        }
        return info;
    }

    // The backend device PipelineBuilder is made with, which it only asks for its DeviceInfo.
    private static GpuDeviceBackend backendDevice(final DeviceInfo info) {
        return (GpuDeviceBackend)Proxy.newProxyInstance(
            GpuDeviceBackend.class.getClassLoader(), new Class<?>[]{GpuDeviceBackend.class}, (proxy, method, args) -> {
                if (method.getName().equals("getDeviceInfo")) {
                    return info;
                }
                throw new UnsupportedOperationException(method.getName());
            }
        );
    }

    // ShaderManager.loadConfigs over a resource manager listing the client jar's assets, which is the vanilla pack's
    // shaders: the pack's other layer, the asset index, has none.
    private static ShaderSource loadConfigs() {
        try {
            JarFile jar = new JarFile(Path.of(ShaderManager.class.getProtectionDomain().getCodeSource().getLocation().toURI()).toFile());
            ResourceManager manager = (ResourceManager)Proxy.newProxyInstance(
                ResourceManager.class.getClassLoader(), new Class<?>[]{ResourceManager.class}, (proxy, method, args) -> {
                    if (method.getName().equals("listResources")) {
                        return listResources(jar, (String)args[0], (ResourceManager.Selector)args[1]);
                    }
                    throw new UnsupportedOperationException(method.getName());
                }
            );
            Method load = ShaderManager.class.getDeclaredMethod("loadConfigs", ResourceManager.class);
            load.setAccessible(true);
            return (ShaderSource)load.invoke(null, manager);
        } catch (ReflectiveOperationException | IOException | java.net.URISyntaxException e) {
            throw new IllegalStateException(e);
        }
    }

    private static Map<Identifier, Resource> listResources(final JarFile jar, final String directory, final ResourceManager.Selector filter) {
        Map<Identifier, Resource> resources = new TreeMap<>();
        jar.stream().filter(entry -> !entry.isDirectory()).forEach(entry -> {
            String[] parts = entry.getName().split("/", 3);
            if (parts.length == 3 && parts[0].equals("assets") && parts[2].startsWith(directory + "/")) {
                Identifier id = Identifier.tryBuild(parts[1], parts[2]);
                if (id != null && filter.isIncluded(id)) {
                    resources.put(id, new Resource(null, () -> open(jar, entry)));
                }
            }
        });
        return resources;
    }

    private static InputStream open(final JarFile jar, final JarEntry entry) {
        try {
            return jar.getInputStream(entry);
        } catch (IOException e) {
            throw new UncheckedIOException(e);
        }
    }
}
