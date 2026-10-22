# Progress

The loop's memory: newest entries first. Each step adds an entry and rewrites "Next".

## Next

Milestone 3, part 1 continued: `Connection`'s pipeline, Netty's machinery around the ported frame handlers, so the port can later join the server through `packet-proxy`. `ByteToMessageDecoder`'s cumulation (`MERGE_CUMULATOR`: how the buffer it reads from grows and is discarded) and its decode loop (`callDecode`, which the golden `[in chunks]` cases already drive by hand), and `MessageToByteEncoder`'s `write` (the buffer it allocates for `encode` and when it releases the message), from `netty-codec-base`'s sources, which `tools/setup_vanilla.py` would fetch next to `netty-buffer`'s; then `PacketEncoder` and `PacketDecoder` (a protocol's codec on a frame, with their checks of what's left unread), `PacketBundlePacker` if configuration uses it, and `PacketListener`'s methods with the listeners' `flow()` and `protocol()` defaults. `ConfigurationProtocols.CLIENTBOUND` stays pending until `ClientboundDisconnectPacket`, `ClientboundResourcePackPushPacket` and `ClientboundServerLinksPacket` (text components: `ComponentSerialization`'s NBT codec, through DataFixerUpper's `Codec` and `NbtOps`), `ClientboundUpdateTagsPacket` and `ClientboundShowDialogPacket` (the registries) come.

`ByteBufCodecs.registryFriendlyLengthPrefixed` comes with `RegistryFriendlyByteBuf` (its decorator wraps the slice with the parent's registry access; `length_prefixed_decorated` takes it as it is). A slice's maximum capacity isn't checked by any golden, since the golden encoding of a buffer leaves it out and nothing writes to a slice; if something ever does, add it to the encoding.

Left from NBT: `java.util.HashMap`'s tree bins (`TreeNode`: `treeify`, `untreeify`, `putTreeVal`, `removeTreeNode`, `split`, the rotations and `moveRootToFront`, which decide a tree bin's linked order), which un-pend the `treeified programs` of `HashMapSuite` (112 of 600) and its `treeified sized programs` (57 of 306), and replace the port's panic; only 9 keys in one bin of a table of 64 or more get there, which a server would have to send on purpose. `toString` of every tag (`StringTagVisitor`'s SNBT), which needs Java's `Float.toString` and `Double.toString` (the JDK's `FloatToDecimal` and `DoubleToDecimal`: shortest digits, but a one-digit answer may become a closer two-digit one, and its own exponent format; the same un-pends `Vec3.toString` and `AABB.toString`), and `StringTag.quoteAndEscape` and `escapeWithoutQuotes` (`SnbtGrammar.escapeControlCharacters`). The visitors (`TagVisitor`, `StreamTagVisitor`, `TagType.parse`, `NbtIo.parse`), `CompoundTag`'s `CODEC`, `store` and `read` and `NbtOps` (DataFixerUpper, with `shallowCopy`), and `NbtIo`'s file methods, when their callers come.

`KnownPack.vanilla(String)` needs the game's version (`SharedConstants.getCurrentVersion()`, which `DetectedVersion` reads from the jar's `version.json`; vanilla's 524 golden cases are recorded now that the generator bootstraps the game), and comes with the client's pack repository, which answers the server's `select_known_packs`.

Login's clientbound protocol only waits for `ClientboundLoginDisconnectPacket`, which reads its reason as lenient JSON (`ByteBufCodecs.lenientJson(262144)`, Gson) through `ComponentSerialization.CODEC` with `JsonOps` (DataFixerUpper), so `LoginProtocols.CLIENTBOUND_TEMPLATE`, `CLIENTBOUND` and its codec stay pending until text components come; settle how much of DataFixerUpper's `Codec` gets ported when they do (configuration and play read components as NBT instead).

The protocol builder's error paths only fire on a programming mistake and no vanilla protocol reaches them, so no golden covers them yet: `ProtocolCodecBuilder.add`'s flow check (at `bind`, like Java's) and `IdDispatchCodec.Builder.build`'s duplicate check, and `CustomPacketPayload.codec_with_fallback`'s duplicate type. A hand-written suite building protocols with a packet of the wrong flow or twice would.

Codecs still pending, for when their callers come: `ByteBufCodecs.either` and `FriendlyByteBuf.readEither`/`writeEither` (DataFixerUpper's `Either`), `StreamCodec.dispatch` (particles, stats) and `recursive` (text components), and `BIT_SET`/`fixedBitSet` with `java.util.BitSet`. `TypeAndCodec` takes a codec of the dispatch codec's own buffer; Java's `? super B` (a `FriendlyByteBuf` codec in play's `RegistryFriendlyByteBuf` dispatch) needs an `AsMut` cast when play's payloads come.

`FriendlyByteBuf.readEnumSet` and `writeEnumSet` need a generator for `Class` arguments and `java.util.BitSet` (also `readBitSet`, `readFixedBitSet` and their writers). `readChunkPos` and `writeChunkPos` come with `ChunkPos` in milestone 4. `readGlobalPos` has golden cases now that `ResourceKey` has a generator (554), and comes with `GlobalPos` (a record of a dimension's key and a `BlockPos`, with DataFixerUpper and stream codecs); `writeGlobalPos` needs a generator for it. `ClientboundCustomPayloadPacket.GAMEPLAY_STREAM_CODEC` comes with `RegistryFriendlyByteBuf`.

Left from milestone 2, for when the renderer needs them: Minecraft's `Frustum` on top of `FrustumIntersection` (it needs `BoundingBox`, which also un-pends `AABB.of` and `BlockPos.betweenClosedStream(BoundingBox)`), `PoseStack.translate(Vec3)` (appended to `PoseStackSuite`'s programs), and `BlockMath` (`Quaternionf.rotateX/Y`, `Direction.getApproximateNearest`), which `BlockModelRotation` builds its face transformations with, along with `OctahedralGroup` (and its `SymmetricGroup3` and `FrontAndTop`), which also un-pends `Rotation.rotation()`. When block models and render states come, settle how they hold shared `Transformation`s: Java's `mulPose(Transformation)` and `BlockMath.getFaceTransformation` add properties to a shared transformation's own matrix (static ones like `TheEndPortalRenderer.TRANSFORMATION` included), so every holder sees them, and that changes which shortcuts later JOML calls take.

The golden files total 1,079 MB (1,053 before the HashSet and HashMap step, whose largest new file is `ClientboundCustomReportDetailsPacket`'s 16 MB). The largest are `ByteBuf` (90 MB), `BlockPos` (74 MB) and `NbtIo` (67 MB). In `ByteBuf`, 56 MB is `ensureWritable` growing each of 57 buffer edges by 65535 and 65536 bytes, and each of those results is a 64 or 128 KiB array in hex. `BlockPos`'s parsed cases take 650 MB in its test, because serde_json's `Value`s of small arrays cost about nine times their text. Neither file repeats one boundary the way the 32767-unit strings did. If CI memory gets tight again, `Golden` could keep each case as its line and parse it only when the case is checked.

`ProjectionType.applyLayeringTransform` and `VertexSorting` come with the renderer. `log10` (one call site, `Options`) and `cbrt` (one, `ClientExplosionTracker`) are small HotSpot stubs; port them when those classes come. The client never calls `Math.exp`. ARGB's `Vector3f`, `Vector4f` and `Vec3` methods have golden cases (its `pending_rest` covers them) and can be ported whenever ARGB is next touched. `Vec3.toString` and `AABB.toString` need Java's `Double.toString` (shortest digits, but a one-digit answer may become a closer two-digit one, and its own exponent format), which only logs and error messages print so far. `Identifier.withPath(UnaryOperator)` is one line (`with_path` of the operator's answer) once something calls it, and `resolveAgainst(Path)` comes with resource loading.

### render

Gate 2's first draw. The shaders' text is vanilla's; next the device runs it:
- `GlPipelineRecompiler.compileShader` and `compileProgram`, `GlProgram` (uniform blocks and samplers bound by the create info's bindings), `GlRenderPipeline` and `VertexArray` (the `ARB_vertex_attrib_binding` path llvmpipe takes), samplers, and buffers with `BufferStorage`'s immutable storage and mapping. Compiling every pipeline on llvmpipe, as `ShaderManager.reload` does at startup, is the first check.
- The frontend between the game and the backend: `FrontendGpuDevice`, `FrontendCommandEncoder` and `FrontendRenderPass` (their checks, uniforms bound by name, `draw`/`drawIndexed`), `RenderSystem`'s device and dynamic uniforms (`DynamicTransforms`, `Projection`, `Globals`), then a `pipeline/gui` fill drawn through it.
- The first pixel-for-pixel frame needs a vanilla screenshot of a state the port can draw that soon: add a scenario step to the oracle client that shoots a GUI-only frame (a screen of fills, or the loading overlay), extra output that doesn't change what vanilla draws.

Then this brief's third part, not started: a `ResourceManager` over `vanilla/<version>/resources/` (the vanilla pack's layers, `listResources`, which `ShaderManager::load_configs` would then use), `NativeImage.read` (stb's PNG decoding, checked against vanilla's pixels), and the block atlas: `SpriteLoader` and `Stitcher`, whose layout a golden suite running vanilla's `Stitcher` on the same sprites records.

## Needs Thalles

- **Licence of the code translated from OpenJDK, JOML, Netty and Guava.** `src/java/lang/math/hotspot_x86/` is translated from OpenJDK's HotSpot, which is GPLv2 only (no Classpath Exception), and keeps its notices. `src/java/lang/fd_libm.rs` comes from the JDK's `FdLibm.java`, GPLv2 with the Classpath Exception, `string::from_bytes_utf8` from the JDK's `String.java`, and `src/java/util/hash_map.rs`, `src/java/util/hash_set.rs` and `src/java/io/` from `HashMap.java`, `HashSet.java`, `DataInput.java`, `DataOutput.java`, `DataInputStream.java`, `DataOutputStream.java` and `ModifiedUtf.java`, the same, which keep their notices. `src/org/joml/` comes from JOML, MIT, whose notice a distribution has to carry. `src/io/netty/` comes from Netty, Apache 2.0, and keeps its notice, saying the files were translated and changed; a distribution has to carry the licence too. So does `src/com/google/`, from Guava, also Apache 2.0. That's fine while the repo stays private. Shipping a build of rustcraft would put it under GPLv2 too, and Apache 2.0 code isn't compatible with GPLv2 only. Not blocking. Decide before the repo or any build goes public.

## Log

### 2026-10-22 (render): Shaders through vanilla's shaderc and SPIRV-Cross, byte for byte, and a first OpenGL frame

- Ported and checked against vanilla:
  - LWJGL's natives, loaded the way LWJGL does (`org/lwjgl/system/SharedLibraryLoader`): `libshaderc.so`, `libspirv-cross.so` and `libSDL3.so` read out of vanilla's natives jars into `target/natives/` and opened with `libloading`, with bindings for the functions the game calls (`Shaderc`, `Spvc`, `SDLInit`, `SDLHints`, `SDLError`, `SDLVideo`, `SDLEvents`). New dependencies: `libloading`, `miniz_oxide` (the jars' deflate), `glow` (OpenGL) and `png`.
  - The pipelines as data: `GpuFormat`, `VertexFormat`, `DefaultVertexFormat` (all 26 constants), `BindGroupLayout` and `BindGroupLayouts` (26), the blend, depth and color target records, `ShaderDefines`, `RenderPipeline` with its builder and snippets, `OitPipelineSet`, and `RenderPipelines`: its 133 public pipelines and OIT sets, 7 public snippets, and `requiredPipelines()` (190) and `optionalPipelines()` (2) in their `HashMap`'s order, sort keys included.
  - The shader path: `ShaderType`, `FileToIdConverter`, `ShaderSource` and `ShaderManager.loadConfigs` (from the vanilla pack's directory), `GlslCompiler` (every shaderc option, the device's macros, includes through a C callback), `SPIRVModule` (reflection), `PipelineBuilder.generateBackendCreateInfo` (its checks and the binding remap) and `GlPipelineRecompiler.decompileShaders` (every SPIRV-Cross option and rename).
  - A window and a frame: `RenderSystem.initBackendSystem`, `GlBackend` (the context attributes), `GlDevice` (the hidden utility window, the 3.3 check, the extensions vanilla turns on in order, `GlHeuristics`' `DeviceInfo`, `glClipControl`), `GlCommandEncoder` (render passes that clear, clears, `copyTextureToBuffer`, presenting), `GlTexture`, `GlBuffer`, `GlSurface`, `GlDebug`, `Window`, `RenderTarget`, `MainTarget`, `NativeImage` (PNG writing) and `Screenshot.takeScreenshot`. `ARGB`'s float channels and `vector4fFromARGB32` (goldens).
  - `src/main.rs`: `--device-info`, `--screenshot PATH`, `--window`.
- Matched:
  - `GlslCompiler.compileToSpv` (309 cases), `SPIRVModule.reflect` (309), `PipelineBuilder.generateBackendCreateInfo` (227) and `GlPipelineRecompiler.decompileShaders` (227): the SPIR-V and the GLSL handed to the driver are byte-identical to vanilla's for all 192 pipelines with llvmpipe's flags, and for 5 pipelines with each of the other 7 flag combinations. `RenderPipelines` (142 entries), `DefaultVertexFormat`, `BindGroupLayouts` and `ARGB` match too.
  - Under Xvfb with llvmpipe, the port's device prints the three lines vanilla logs about its device, and they equal the oracle client's log: driver, device, and the ten extensions in the same `HashSet` order. Screenshots of a cleared main target are RGBA PNGs of the target's size, every pixel the clear color made opaque (transparent black, the loading overlay's `FFEF323D`), and a render pass clearing GL's lower left quarter lands at the image's lower left.
- Settled: llvmpipe makes vanilla pick `isZZeroToOne = true` (`GL_ARB_clip_control`), `shaderDrawParameters = true` (`GL_ARB_shader_draw_parameters`) and `isExplicitDepthRequired = false` (not Apple silicon), from the oracle client's log; the goldens record those, and `tests/opengl_frame.rs` checks the port's device gives the same.
- Found:
  - No vanilla shader reads `RENDERPEARL_INSTANCE_INDEX_INCLUDES_BASE_INSTANCE`, so the draw parameters flag only changes SPIRV-Cross's output (`gl_InstanceID` with the base instance).
  - `RenderPipelines` registers `pipeline/item_cutout` three times and `pipeline/item_translucent_glint` twice, so the map keeps only the last of each, and the game compiles 192 of the 195 pipelines it builds (sort keys 0 to 194).
  - A pipeline's bind group layouts are an `ObjectOpenHashSet` whose order changes between runs (the records hash `UniformType` by identity). No output depends on it: bindings come from the SPIR-V's descriptors, vertex stage first.
- Golden harness (TESTING.md, "Pipelines and shaders"): new `RenderSuites`. The shaders come from vanilla's own `ShaderManager.loadConfigs` over a `ResourceManager` listing the client jar's assets, the device flags from a stand-in device, and `generateBackendCreateInfo` runs by reflection. The new files come out byte-identical from two runs; they add 19 MB (the goldens total 1,099 MB, 1,079 before).
- Mutations (3, run once and not committed): flipping the depth macro fails 66 of 309 `compileToSpv` cases, and dropping SPIRV-Cross's zero-initialisation fails 78 of 227 `decompileShaders` cases. Flipping the instance index macro changes nothing, since no shader reads it (above).
- CI installs `xvfb`, `xauth` and Mesa's llvmpipe, and `tests/opengl_frame.rs` runs the binary under `xvfb-run -a`; it compares the device with the oracle's log where one exists, and its flags with its extensions elsewhere. On the runner's Xvfb (xserver 21.1.12, Mesa 25.2) SDL found no GLX visual for vanilla's attributes, which failed the first push's CI; a throwaway PR's diagnostics showed GLX working in `glxinfo` and SDL's EGL path giving the same device, so CI runs the tests with `SDL_VIDEO_FORCE_EGL=1` (TESTING.md). Locally they take GLX, like vanilla.
- Evidence: `cargo test --workspace` passes 194 tests, including 154 golden suites (181 and 147 before) and the 3 OpenGL tests (2.6 s). The render goldens take 4 s. fmt, clippy and the port check are clean.

### 2026-10-22: java.util.HashSet, HashMap(int), ByteBufCodecs.map, and the enabled features and report details packets

- Ported and checked against vanilla, milestone 3's eleventh slice:
  - `java.util.HashSet` (`src/java/util/hash_set.rs`): the ported `HashMap` with `()` for Java's `PRESENT`, with both constructors, `add`, `contains`, `size`, iterating, `AbstractSet`'s `equals` and `hashCode` and `AbstractCollection`'s `toString`. Checked through the features packet; it keeps `HashSet.java`'s notice.
  - `HashMap(int)` (`from_initial_capacity`: the threshold the first `put` makes the table from, a panic on a negative capacity), checked by 249 new `sized programs` (57 `treeified sized programs` pending with the tree bins), and `AbstractMap.toString`. The map's iterator knows its length, so codecs write `size()` first.
  - `ByteBufCodecs.map` (both overloads, `map` and `map_max`, a `Map` codec), un-pended: 2,655 cases on `HashMap::new` with `STRING_UTF8` and `VAR_INT`, and with the report details' codecs and any maximum.
  - `ClientboundUpdateEnabledFeaturesPacket` (5,975 cases) and `ClientboundCustomReportDetailsPacket` (6,155), with `ClientConfigurationPacketListener.handleEnabledFeatures` and `ClientCommonPacketListener.handleCustomReportDetails`. `Identifier` implements `HashCode` for the set.
- Settled (PORTING.md): a Java `HashSet` or `HashMap` in a codec is `java::util`'s port, since encoding writes in its table's order, and `HashSet::new` or `HashMap::new` as an `IntFunction` is `from_initial_capacity`. A map codec's collection is anything it can extend with `(key, value)` and iterate as `(&key, &value)`.
- Golden harness (TESTING.md): a `HashSet` is `[capacity, [element, …]]` and a `HashMap` `[capacity, [[key, value], …]]`, read by reflection like a compound's map (`HashMapSuite.encode`), and each argument is a copy with the same table and order (`HashMapSuite.copy`). `Codecs.hashSetOf` and `hashMapOf` make them like `listOf`, also as a codec makes them and with tables bigger than they need. The features are made the game's way (`FeatureFlags.REGISTRY.toNames`), plus identifiers whose hashCodes are equal (paths of "ak" and "c-" blocks) added either way round; the details have names around 128, values around 4096 (only in the maps' edges: as value edges, random maps drew them and the file was 83 MB instead of 16), 32 and 33 entries and colliding names. Both codecs also decode counts with nothing after them, entries twice, and (the details) what's written without the limits. `HashMapSuite` gained `sized programs` with their own seed, so its old programs are unchanged.
- Found:
  - A set decoded from a count with an identifier twice keeps one, so it writes a smaller count back; a map keeps the second value where the first was.
  - Filling `new HashMap<>(n)` with n distinct keys gives the same table as `new HashMap<>(n + 1)`, since n entries in a table of n always grow it; only duplicates or fewer entries tell them apart.
  - A negative count throws `IllegalArgumentException` making the set or map, before anything else is read.
- Mutations (20, run once and not committed): all 20 fail the goldens, among them the capacity off by one, the threshold not a power of two, a set's `equals` by size or in order, `add` moving an element to its bin's end, value before key, the limits at 129, 4095 and 33, the max ignored on read, and the sets and maps made with `new()`. Dropping the negative check outright makes the port's `from_initial_capacity(i32::MIN)` a table of 2^30 bins at the first put, which ran the machine out of memory, so that mutation reads a negative capacity as 0 instead.
- With NaNs normalized, every line of the other golden files is identical to before, apart from `PacketType`'s types (two more), `ByteBufCodecs`' `map` entries (skipped lines replaced) and `HashMap`'s new sized programs. The new files, and the changed ones, come out byte-identical from two runs.
- Evidence: `cargo test --workspace` passes 181 tests, including 147 golden suites (179 and 145 before). The library's test binary peaks at 2.0 GB of RSS and takes 10 s. The golden files total 1,079 MB (1,053 before). fmt, clippy and the port check are clean.

### 2026-10-22: The configuration protocol's clientbound packets, ResourceKey and Registries

- Ported and checked against vanilla, milestone 3's tenth slice:
  - `ClientboundFinishConfigurationPacket` (1,112 cases), `ClientboundResetChatPacket` (1,111), `ClientboundClearDialogPacket` (1,111), `ClientboundKeepAlivePacket` (3,583), `ClientboundPingPacket` (3,673), `ClientboundSelectKnownPacks` (5,459), `ClientboundResourcePackPopPacket` (5,016), `ClientboundTransferPacket` (7,100), `ClientboundCodeOfConductPacket` (5,482), `ClientboundPostEffectsPacket` (5,221) and `ClientboundCustomPayloadPacket`'s `CONFIG_STREAM_CODEC` (5,487; `GAMEPLAY_STREAM_CODEC` pending for `RegistryFriendlyByteBuf`).
  - `ClientboundRegistryDataPacket` (6,776; `hashCode` pending, an identity hash, and `toString` pending for SNBT) with `RegistrySynchronization.PackedRegistryEntry` (5,397; `toString` pending), `ResourceKey` (8,382; `codec` and `dependent(UnaryOperator)` pending) and `Registries` (all 157 constants, and its static methods, 2,580 cases).
  - `ClientCommonPacketListener`'s handlers for those packets, and `ClientConfigurationPacketListener` with all but `handleEnabledFeatures`.
  - Un-pended: `FriendlyByteBuf.readResourceKey`, `writeResourceKey` and `readRegistryKey` (3,282 cases). `readGlobalPos` got 554 cases with `ResourceKey`'s generator and stays pending for `GlobalPos`.
  - Left (Next): `ClientboundUpdateEnabledFeaturesPacket` and `ClientboundCustomReportDetailsPacket`, which need `java.util.HashSet` and `ByteBufCodecs.map` over the ported `HashMap`.
- Design (PORTING.md): Java interns `ResourceKey`s and compares them by reference, so a key is a value comparing its registry's name and identifier: with one key per pair, that's the same answer. Its type argument is dropped like `PacketType`'s.
- Golden harness (TESTING.md):
  - A `ResourceKey` is `[registry, identifier]`: the synchronized registries' keys, the dimensions', the root's own, elements' keys, keys with odd identifiers, and random ones (`CoreValues.RESOURCE_KEY`). `Codecs.codecOf` takes a parameterized value type as its raw class unless its arguments pick a generator, for `REGISTRY_STREAM_CODEC`'s `ResourceKey<? extends Registry<?>>` and `streamCodec`'s.
  - Generators for each packet, `Optional<UUID>`, `List<Identifier>` and `List<PackedRegistryEntry>` arguments, hosts and codes of conduct around 32767 (whose codecs also decode what's written without the limit), and the clientbound custom payload's 1 MiB discarded payloads and a byte more. `PORTED_TYPES` has the clientbound types.
  - With 68 type edges, `PacketType.equals`' pairs of edges went past `MAX_EDGE_PRODUCT` and fell to each edge against the first (2,264 cases to 636), so `equals` gets every pair of edges as extra inputs (5,260 cases, every old pair of edges among them).
  - The registry data's `equals` also compares packets of every pair of key edges, and an entry's codec decodes data at the default quota and past it: lists of 40,329 empty compounds (2,097,144 bytes accounted) and 40,330 (2,097,196, which throws `NbtAccounterException`), a byte each to read.
- Found:
  - `DIMENSION` and `LEVEL_STEM` are the same key (`minecraft:dimension`), so `levelStemToLevel` and `levelToLevelStem` give the same key for the same identifier.
  - The registry data codec writes only a key's identifier and reads it back as a registry's key, so a packet holding an element's key (`minecraft:dimension / minecraft:overworld`) decodes to a different one.
  - The transfer packet's port is a VarInt, so any int goes, negative ones too. The clientbound `select_known_packs` has no limit on its packs, where the client's answer takes 64.
- Mutations (29, run once and not committed): 27 failed the goldens at first. The two survivors led to the inputs above: the registry data's `equals` ignoring the keys' registries, and an entry's tag read with `TRUSTED_TAG`'s unlimited quota. Both fail now.
- With NaNs normalized, every line of the other golden files is identical to before, apart from `FriendlyByteBuf`'s four `ResourceKey` members, `PacketType`'s types and `equals`, and three skip reasons (`ByteBufCodecs`' `registry`, `holderRegistry` and `holderSet` now name `RegistryFriendlyByteBuf`, `holder` the codec it takes, and `ConfigurationProtocols.CLIENTBOUND.codec()` the `disconnect` packet). The new files, and the changed ones, come out byte-identical from two runs.
- Evidence: `cargo test --workspace` passes 179 tests, including 145 golden suites (164 and 130 before). The library's test binary peaks at 2.0 GB of RSS (1.9 before) and takes 8 s. The golden files total 1,053 MB. fmt, clippy and the port check are clean.

### 2026-10-22: ServerboundCustomClickActionPacket, lengthPrefixed, ByteBuf.slice and the configuration protocol's serverbound side

- Ported and checked against vanilla, milestone 3's ninth slice:
  - Netty's `ByteBuf.slice(int, int)` (1,654 cases): a copy of the range, which can't grow, checked against the array rather than the writer index. Java's slice shares the array, which only differs when something writes to it, and the game only reads one (PORTING.md).
  - `ByteBufCodecs.lengthPrefixed(int)` and `lengthPrefixed(int, BiFunction)` (`length_prefixed` and `length_prefixed_decorated`, a `LengthPrefixed` codec): 1,903 cases on the click action's `optionalTagCodec` and 1,596 on a discarded payload's codec in a `FriendlyByteBuf`. Decoding reads the VarInt size, checks the maximum, slices, moves the reader index past the slice, then decodes from the slice; encoding writes into a scratch buffer (`alloc().buffer()`, `unpooled::buffer()`), checks its size, and only then writes the size and the bytes. `registryFriendlyLengthPrefixed` stays pending for `RegistryFriendlyByteBuf`.
  - `ServerboundCustomClickActionPacket` (5,986 cases; `toString` pending for the tags' SNBT) and `ServerCommonPacketListener.handleCustomClickAction`.
  - `ConfigurationProtocols.SERVERBOUND_TEMPLATE`, `SERVERBOUND` and `SERVERBOUND.codec()` (1,694 cases over all ten packets). `CLIENTBOUND_TEMPLATE`, `CLIENTBOUND` and its codec are pending: text components and the registries (Next).
- Golden harness (TESTING.md):
  - The generator bootstraps the game first (`SharedConstants.tryDetectVersion()`, `Bootstrap.bootStrap()`), like the game's own tests: `ConfigurationProtocols` didn't load without it, since its clientbound template holds the dialog packet, whose codec needs the registries. Bootstrapping changed no golden case. It made `KnownPack.vanilla` (524 cases, pending in the port for the game's version) and `ClientInformation.createDefault` (1 case, pending: only the server calls it) recordable, so they're no longer written as skipped. The progress lines go to the stream from before, since bootstrapping sends `System.out` to the log. `python3 tools/golden.py` takes 41 s.
  - `CodecSuite` instances take extra decode buffers like codec constants (`extraInputs("descriptor[label].decode", …)`): prefixes right, a byte short, a byte long, zero, negative or an int's limit, against maximums of the payload's length, a byte less and 65536, inside and outside the array's spare bytes, with a byte read before them so a mark set on the buffer would show. The tags' values also hold a string too long for `writeUTF`, whose writer throws after the tag's id, which shows nothing reaches the buffer.
  - An `Optional<Tag>` argument gets tags or none (a parameterized type picks its generator), the click action's `hashCode` gets no packet holding an `EndTag` (whose hash is an identity hash), and its codec also decodes prefixes of 65536 and 65537, byte arrays of 32744 and 32745 bytes (the 32 KiB accounter takes the first and refuses the second) and 40000, and compounds 16 and 17 deep.
- Found:
  - A slice is checked against the buffer's capacity, so a prefix a byte past the writer index slices bytes past it and only throws when the reader index moves there. A negative size passes the maximum check and throws at the slice, before the reader index moves.
  - The codec's value reads no further than its slice: a discarded payload skips only the slice's bytes, and bytes of the slice the value doesn't read are skipped with it. Reading NBT marks the slice's reader index, not the buffer's.
  - The size is checked against the maximum before the reader index moves, so a prefix of 65537 leaves the reader after the VarInt, while a tag the accounter refuses leaves it past the slice.
- Mutations (22, run once and not committed): 20 fail the goldens. The first run left the click action's accounter at 32767 and 16 levels at 15 passing, which led to the byte arrays and compounds at its limits. Two survive: dropping `slice`'s own negative-length check is equivalent (the bounds check refuses a negative length too), and a slice that can grow isn't visible, since a buffer's golden encoding leaves out its maximum capacity and nothing writes to a slice (Next).
- With NaNs normalized, every line of the other golden files is identical to before, apart from `ByteBuf`'s and `ByteBufCodecs`' new cases (the latter replacing two skipped lines), `PacketType`'s, whose types include the click action's, and the two members bootstrapping un-skipped. The new files, and the changed ones, come out byte-identical from two runs.
- Evidence: `cargo test --workspace` passes 164 tests, including 130 golden suites (162 and 128 before). fmt, clippy and the port check are clean.

### 2026-10-22: Golden files from 2.6 GB to 912 MB

- Why they were big: a random value is one of its generator's edges one time in eight, and some edges hold a string at a codec's limit of 32767 units: a `KnownPack`'s parts, a brand, a property's value. A random list of up to 140 packs held about ten such strings, and every member of `ServerboundSelectKnownPacks` repeated them in its args, results and `after`s. That file alone was 1.49 GB.
- Fixed in the generator (TESTING.md): `Values.draw`, which both `Values.Simple` and `JomlValues.Mutable` now use, gives a fresh value instead of an edge whose encoding is over 16,384 characters (`MAX_DRAWN_EDGE`). The rule works through every layer that draws edges: packs, lists of packs, properties, multimaps, property maps and profiles. Random cases keep their counts and their lists keep their lengths, but no random value holds one of those strings anymore. Each string stays in the edges, which are unchanged: alone in a value with every other field at its first edge, in a list on its own and in the list of all the edges, against every other edge for `equals`, and in the extra decode inputs.
- Sizes: `ServerboundSelectKnownPacks` 1,495 → 50 MB, `KnownPack` 106 → 20, `ClientboundLoginFinishedPacket` 75 → 16, `GameProfile` 66 → 21, `ByteBufCodecs` 66 → 37, `PropertyMap` 63 → 17, `BrandPayload` 51 → 5, `ServerboundCustomPayloadPacket` 45 → 13, `Property` 30 → 8, `CustomPacketPayload` 12 → 4. No other file changed: with NaNs normalized, the other 118 are byte-identical to before.
- Boundaries still covered, counted per member before and after with a script that finds each string as JSON or as `writeUtf`'s bytes in a buffer's hex. Every member that held one still holds it:
  - `KnownPack`: namespaces and versions of 32767 and 32768 `a`s, and ids of 16383 emoji plus `a` (32767 units) and 16384 emoji. Each getter, `hashCode`, `toString` and `isVanilla` has 2 cases with each `a` string and 1 with each emoji string. `equals` has 56 and 29, the codec's `encode` 2 and 1, and `decode` reads the readable ones in 5 and 3 cases.
  - `ServerboundSelectKnownPacks`: each of those packs alone in a list, all 15 pack edges in one list, and lists of 64 and 65 packs. The all-edges list throws on its sixth pack after five are written. Each member has 2 to 3 cases with each string, and `equals` has 72 to 105.
  - `BrandPayload`, `ServerboundCustomPayloadPacket` and `CustomPacketPayload`'s codecs: brands of 32767 and 32768 `a`s and 16384 emoji, in 1 case per member (11 to 37 for `equals`). The extra decode inputs are unchanged.
  - `Property`, `PropertyMap`, `GameProfile`, `ClientboundLoginFinishedPacket` and `ByteBufCodecs`' `GAME_PROFILE_PROPERTIES` and `GAME_PROFILE`: values of 32767 and 32768 `a`s, in 1 to 6 cases per member (2 to 83 for `equals`). Signatures around 1024 and names around 64 are under the limit, so random values still draw them about as often as before.
  - `decode` cases went up (e.g. `ServerboundSelectKnownPacks` 893 → 1044). Random values that used to throw when encoded now encode, and their encodings get decoded.
- Mutations (38, run once against the trimmed goldens and not committed), aimed at what the dropped cases exercised. Each named the suites that should catch it:
  - Mutations: each string limit ±1 in `KnownPack`, `BrandPayload` and the profile codecs, `list(64)` at 63 and 65, property counts of 17, the custom payload maximum at 32766 and 32768, the fallback taken for every identifier, and the `hashCode`, `toString`, `equals` and `isVanilla` of each record and of the multimap.
  - All 38 fail the class's own suite. 36 also fail every other suite named. The other two still pass in one suite each, and pass there with the old goldens too, so I named those suites wrongly: no login-finished decode reads a count of 17 (only `ByteBufCodecs`' extra inputs do), and `GameProfile` never calls `size()`.
- `Golden::load` reads its file line by line, so the text and the parsed cases are never in memory together.
- Evidence:
  - `cargo test --workspace` passes 162 tests, including 128 golden suites. The library's test binary takes 8.4 s instead of 13.4 s, and peaks at 1.9 GB of RSS instead of 4.5 GB. `ServerboundSelectKnownPacks`' test alone peaks at 110 MB instead of 3.0 GB.
  - `python3 tools/golden.py` takes 34 s instead of 55 s. A second run's files are byte-identical, apart from NaN bits in the usual six matrix files.
  - fmt, clippy and the port check are clean.

### 2026-10-22: Golden generation without OSHI

- `python3 tools/golden.py` takes 55 s locally again (96.7 s user, 5.7 s sys, javac included), down from 452.5 s (177.8 s user, 285.4 s sys), both measured on this shared machine. Every golden case is the same.
- Where the time went: reading NBT wraps an `IOException` in a crash report (`NbtIo.readTagSafe`), and a run makes 16,942 of them. Each `CrashReport` makes a `SystemReport`, whose details make two OSHI `SystemInfo`s, for the hardware and the software: about a thousand files opened under `/proc` and `/sys`, and six processes (`lspci -vnnm`, `lshw -C display`, `lshw -class processor`, `dmidecode -t 4`, `dmidecode --type 17`, `cpuid -1r`), each a `jspawnhelper` that only found out the command wasn't on the empty `PATH`. That's 23 ms a report (19 ms of it the processor's details), and the three suites that read NBT from random bytes took 410 s of the 452 (`ByteBufCodecs` 171 s, `NbtIo` 135 s, `FriendlyByteBuf` 104 s). Found with JFR samples of `new SystemReport()` in a loop and `strace -f` counting its `execve`s.
- Fixed in the tools: `tools/golden.py` runs the generator without OSHI's jar on the classpath, so making a `SystemInfo` throws `NoClassDefFoundError`, which `SystemReport` catches like any detail it can't get (0.38 ms a report). The details only go into the exception's message, and a golden records the exception's class. Nothing spawns a process now (`strace -f` sees the JVM's own `execve` and no other), so the generator gets the normal `PATH` back. Its `main` also sets `SystemReport`'s logger to `ERROR`: its four warnings a report, each with a stack trace, were 1.27 million lines of output; 360 are left, vanilla's 22 `Failed to write NBT String` errors. TESTING.md says so.
- Evidence: against the goldens the last step generated, 121 of the 128 files are byte-identical, the NBT, `FriendlyByteBuf` and `ByteBufCodecs` ones among them, and 7 (`PoseStack`, `MatrixUtil`, `Transformation`, `Vec2`, `Matrix3f`, `Matrix4f`, `Quaternionf`) differ only in NaN bits, which differ between any two runs (TESTING.md): two runs of the old generator differ in 6 of those files the same way. `cargo test --workspace` passes 162 tests, including 128 golden suites. fmt, clippy and the port check are clean.
- What's slowest now is `ServerboundSelectKnownPacks` (12 s): its file is 1.5 GB of the 2.6 GB, lists of 64 and 65 packs whose strings are around 32767 characters.

### 2026-10-22: NBT, java.util.HashMap, and the streams NBT is read and written through

- Ported and checked against vanilla, milestone 3's eighth slice:
  - `java.util.HashMap` (`src/java/util/hash_map.rs`): its table, `put`, `get`, `containsKey`, `remove`, `resize`'s split, a bin of 9 growing a table under 64, the copy constructor's sizing (`ceil(size / 0.75)`), and `AbstractMap`'s `equals` and `hashCode`, checked by 488 programs. Tree bins aren't ported: the port panics there, and the 112 `treeified programs` are pending (Next).
  - The tags: `Tag` (11,084 cases) as an enum of the 13 classes, `NumericTag` (7,860) and `CollectionTag` (7,032) as traits, `EndTag` (3,061), `ByteTag` (14,423), `ShortTag` (9,704), `IntTag` (12,952), `LongTag` (10,684), `FloatTag` (11,334), `DoubleTag` (11,532), `ByteArrayTag` (10,699), `StringTag` (10,348), `ListTag` (33,107), `CompoundTag` (55,281), `IntArrayTag` (10,413) and `LongArrayTag` (10,413): every member but `toString`, the visitors, `CompoundTag`'s DataFixerUpper codecs and `forEach`, and `StringTag`'s SNBT escapes (Next).
  - `TagType` (4,585) and `TagTypes` (549), `NbtAccounter` (4,905) and `NbtIo`'s `readAnyTag`, `writeAnyTag`, `readUnnamedTag`, `writeUnnamedTag`, `writeUnnamedTagWithFallback`, `read` and `write` over a `DataInput` (10,151; the file ones and `parse` pending).
  - Netty's `ByteBufInputStream` (9,848) and `ByteBufOutputStream` (12,523), with `java.io`'s `DataInput` and `DataOutput`, `DataInputStream.readUTF` and `DataOutputStream.writeUTF` (modified UTF-8), `DelegateDataOutput` and `java.lang.Number`'s boxes.
  - Un-pended: `ByteBufCodecs`' `TAG`, `TRUSTED_TAG`, `COMPOUND_TAG`, `TRUSTED_COMPOUND_TAG`, `OPTIONAL_COMPOUND_TAG`, `tagCodec`, `optionalTagCodec` and `compoundTagCodec` (12,145 cases), and `FriendlyByteBuf`'s six `readNbt` and `writeNbt` (7,631).
- Design (PORTING.md): a sealed interface the game tells apart with `instanceof` is an enum of its classes' structs, and the interfaces between are traits with an `as_…` for their `instanceof`. `TagType`, which each class implements in its `TYPE` constant and `createInvalid` for other ids, is a `Copy` enum. Java's `finally` that must run when a load throws (the accounter's depth) is a guard whose `Drop` pops it. Tags are values: getters lend, `…OrEmpty` returns a `Cow`. `HashMap` and modified UTF-8 come from the JDK and keep its notices; a lone surrogate reads as U+FFFD, which can't be checked.
- Golden harness (TESTING.md): new `NbtValues`, `NbtSuites` and `HashMapSuite`. A tag is `[id, payload]`, a compound's payload its map's table length and entries in order (read by reflection, so `tools/golden.py` opens `java.util`), and each argument is a copy with the same table. Members taking a key or an index get the receiver's own; readers read what the writers write, and compounds with a name twice; strings around 65535 bytes go to the writers and readers only. Reads with more than the default quota are dropped unless the same read fits it (`readsLittle`), since a count read from random bytes would make vanilla allocate gigabytes; `CodecSuite.whereDecode` takes a codec constant's name for that. Buffers never hold an encoded surrogate without its pair, which the golden files' UTF-8 can't hold. `Codecs.codecOf` gives a codec of an `Optional` its values or empty.
- Fixed in the tools: reading NBT wraps an `IOException` in a crash report, whose system details run `lshw` for a second each time, so `tools/golden.py` runs the generator without a `PATH` (the details only go into the exception's message).
- Found:
  - `FloatTag.valueOf` and `DoubleTag.valueOf` make -0.0 their `ZERO`, 0.0, so a -0.0 read from NBT reads back as 0.0.
  - Making a `ByteBufInputStream` marks the buffer's reader index, so `readNbt` overwrites any mark, and its `skipBytes` skips what's left at most, so `skipString` with a length past the end doesn't throw.
  - A list written as compounds writes an `EndTag` element's wrapper as two zero bytes (an end has no name), which reads back as an empty compound with a byte left over. An `EndTag` doesn't count as a list's kind.
  - A compound read with a name twice keeps the second tag where the first was, and accounts the name's 36 bytes once. An accounter's usage near `Long.MAX_VALUE` wraps past the quota check.
  - `readUTF` reads overlong forms and a raw U+0000, and a four-byte lead is malformed even when continuations follow. A number type skips `size * count` bytes as an int, which wraps, `END` skips nothing, and an invalid type throws even for a count of 0.
  - `new HashMap<>(map)` sizes its table with `ceil(size / 0.75)` since JDK 19, so a shallow copy of 6 tags has 8 bins where a compound of them has 16.
- Mutations (42, run once and not committed): 40 fail the goldens. Four survived the first inputs, which led to a four-byte lead followed by continuations, continuations that are lead bytes, compounds with a name twice and merges into compounds under the same names. Two are equivalent: an `EndTag` skipped in `identifyRawElementType` (setting the kind to 0 when it's 0 already), and `getBooleanOr` through `getBoolean`.
- With NaNs normalized, every line of the other golden files is identical to before, apart from `FriendlyByteBuf`'s and `ByteBufCodecs`' NBT entries, whose cases replace their skipped lines. The new files, and the changed ones, come out byte-identical from two runs.
- Evidence: `cargo test --workspace` passes 162 tests, including 128 golden suites (139 and 105 before). fmt, clippy and the port check are clean.

### 2026-10-22: The configuration protocol's serverbound packets

- Ported and checked against vanilla, milestone 3's seventh slice:
  - `ConfigurationPacketTypes` (its 9 types), and `ServerCommonPacketListener` and `ServerConfigurationPacketListener` with their handlers (`handleCustomClickAction` comes with its packet, which waits for NBT).
  - `ServerboundClientInformationPacket` (6,708 cases) with `ClientInformation` (10,032) and the enums it holds, `ChatVisiblity` (1,850), `HumanoidArm` (1,849) and `ParticleStatus` (1,850), whose `STREAM_CODEC`s map ids with `ByIdMap` (their DataFixerUpper codecs and `caption()` pending). The records' `hashCode`s are pending, since they hash enum constants' identity hashes; so is `ClientInformation.createDefault` (below).
  - `ServerboundCustomPayloadPacket` (5,459) with `CustomPacketPayload`'s `codec`s and `createType` (4,584), `Type` (2,718), `TypeAndCodec`, `FallbackProvider`, `BrandPayload` (4,516) and `DiscardedPayload` (4,449, its `codec(Identifier, int)` included). The scenario-start marker is a `DiscardedPayload` of `rustcraft:scenario_start`.
  - `ServerboundSelectKnownPacks` (5,241) with `KnownPack` (5,846; `vanilla(String)` pending for the game's version).
  - `ServerboundFinishConfigurationPacket` (1,112), `ServerboundAcceptCodeOfConductPacket` (1,115), `ServerboundKeepAlivePacket` (3,583), `ServerboundPongPacket` (3,673), and `ServerboundResourcePackPacket` (5,256) with its `Action` (567).
  - `FriendlyByteBuf.readEnum` and `writeEnum` (1,844 cases), un-pended.
  - `ConfigurationProtocols` isn't ported yet: its serverbound template lists `ServerboundCustomClickActionPacket`, whose payload is an NBT tag (Next).
- Design (PORTING.md): a packet holds a payload as an `Arc<dyn CustomPacketPayload>`, Java's shared reference, so its getter and the codec's `map` hand out a clone of the `Arc`. `TypeAndCodec` holds its codec as one of any payload, which casts what it writes back to its payload (`Upcast`, like `ProtocolCodecBuilder`'s), and `codec(FallbackProvider, List)` is `codec_with_fallback`. `readEnum(Class)` is `read_enum::<T>()` for an enum implementing the new `java::lang::r#enum::Enum` trait (`ENUM_CONSTANTS`, `ordinal`).
- Golden harness:
  - Generators for client information (languages around 16 characters, view distances and skin parts past a byte), the enums, known packs and lists of them (a `List<KnownPack>` argument gets them, with 64 and 65 packs), brands around 32767, payload identifiers (the marker, the brand's, `minecraft:register`), and the seven packets. A payload declared as `CustomPacketPayload` is `[class, fields]`, since a `DiscardedPayload` can have the brand's identifier.
  - `InstanceSuite` gives a static method making a codec from generated arguments codec cases like `CodecSuite`'s (`DiscardedPayload.codec`, whose buffer is a type variable, read as its bound), and encodes a result declared as an `Enum` by its own class, for `readEnum`. `golden.rs` gained `check_codec_from2`.
  - Extra decode inputs: `ClientInformation`'s reader on what `write` writes, discarded payloads at their maximum and a byte either side, the custom payload packet's 32767- and 32768-byte payloads and brands past 32767, and `readEnum` of ordinals from -16 to 16. TESTING.md describes all of it.
  - Two members are written as skipped, since the harness doesn't bootstrap the game: `KnownPack.vanilla` needs the game's version, and `ClientInformation.createDefault` reads `Player.DEFAULT_MAIN_HAND`, whose class needs the registries, so it threw `ExceptionInInitializerError` in the first run. Only the server calls `createDefault` (`CommonListenerCookie`), so it isn't ported.
- Found:
  - A `DiscardedPayload` with the brand's identifier can't be written: the dispatch codec picks `BrandPayload`'s codec by the identifier, and that codec's writer throws `ClassCastException` after the identifier is out. Anything read under `minecraft:brand` is a `BrandPayload`.
  - `ClientInformation` writes the language with `writeUtf`'s limit of 32767 and reads it with 16, so a client can write what no server reads. The view distance and skin parts are written as a byte; the distance reads back signed (128 is -128) and the parts unsigned (-1 is 255).
  - Ids out of range read as `LEFT` for `HumanoidArm` (`ZERO`), but wrap for `ChatVisiblity` and `ParticleStatus`.
  - `readEnum` throws `ArrayIndexOutOfBoundsException` for a negative or too large ordinal, after the VarInt is read.
- Mutations (43, run once and not committed): all 43 fail the goldens, among them the dispatch and upcast codecs writing nothing instead of throwing on a payload of the wrong class, a maximum of 32768, `HumanoidArm` wrapping, the view distance read unsigned and the language written with 16.
- With NaNs normalized, every line of the other golden files is identical to before, apart from `FriendlyByteBuf`'s, which only lost the 2 skipped lines its `readEnum` and `writeEnum` cases replace, and `PacketType`'s, whose types now include the new packets'. The new files, and the changed ones, come out byte-identical from two runs.
- Evidence: `cargo test --workspace` passes 139 tests, including 105 golden suites (120 and 86 before). fmt, clippy and the port check are clean.

### 2026-10-22: GameProfile, PropertyMap, UUIDUtil and the login's last packet

- Ported and checked against vanilla, milestone 3's sixth slice:
  - authlib's `GameProfile` (6,127 cases), `Property` (5,623; `isSignatureValid` pending, the client never calls it) and `PropertyMap` (4,514, with the `Multimap` methods it forwards: `values`, `size`, `isEmpty`, `equals`, `hashCode`, `toString`). authlib publishes no sources, so `tools/setup_vanilla.py` decompiles its jar with Vineflower into `vanilla/<version>/src/com/mojang/authlib/`.
  - Guava's `ImmutableMultimap` (`src/com/google/common/collect/`), from the sources `setup_vanilla.py` now fetches next to JOML's and Netty's (Maven Central's jar is the one Mojang ships). It's what `builder()`, `of()` and `copyOf` make, an `ImmutableListMultimap`: a `Vec` of keys in the order they were first put, each with its values. Checked through `PropertyMap`. It keeps Guava's Apache notice.
  - `UUIDUtil`: `STREAM_CODEC`, `UUID_BYTES`, `uuidFromIntArray`, `uuidToIntArray`, `uuidToByteArray`, `createOfflinePlayerUUID` and `createOfflineProfile` (4,086 cases); the DataFixerUpper codecs and `readUUID(Dynamic)` pending. `java.util.UUID.nameUUIDFromBytes` (513), with the md-5 crate.
  - `ByteBufCodecs.GAME_PROFILE_PROPERTIES` (1,738) and `GAME_PROFILE` (1,752), un-pended.
  - `ClientboundLoginFinishedPacket` (6,436) and `ClientLoginPacketListener.handleLoginFinished`. `LoginProtocols.CLIENTBOUND` now only waits for `ClientboundLoginDisconnectPacket`.
- Design (PORTING.md): generic code that calls `hashCode` on any value (Guava's collections) takes `java::lang::object::HashCode`, which `String` and `Property` implement. A class forwarding to a Guava collection holds it and forwards what the game calls. Overloaded record constructors are `new` and `from_<what the longer one adds>` (`GameProfile::from_properties`, `Property::from_signature`), like `BlockHitResult`'s.
- Golden harness:
  - New `ProfileValues`: properties (the skin's textures, names the random ones share so keys repeat, strings around the codec's limits of 64, 32767 and 1024, no signature a quarter of the time), multimaps built with `ImmutableMultimap.builder()` (empty, one property, two under one name either way round, interleaved names either way round, a property twice, a key that isn't its property's name, 16 and 17 properties), property maps and profiles. A multimap or property map is its `asMap()`'s entries, `[[key, [property, …]], …]` (TESTING.md).
  - `CodecSuite.extraInputs("N.decode", …)`, like `InstanceSuite`'s: the profile codecs also decode what's written without their limits, and counts from `MIN_VALUE` to `MAX_VALUE` with nothing after them. `PLAYER_NAMES` moved to `ProfileValues`, unchanged.
- Found:
  - `GAME_PROFILE_PROPERTIES` reads into a multimap that groups properties by name, in the order each name first came, so properties named a, b, a are written back as a, a, b.
  - A property map's `equals` ignores the order of its names but not of the properties under one name, and its `hashCode` is a sum, so it ignores that order too.
  - A negative property count passes `readCount`'s check and reads as no properties. The count written is every property under every name, so 17 throws when written, not just when read.
  - An empty signature is a signature: `hasSignature` only checks for null.
- Mutations (33, run once and not committed): 31 fail the goldens. One is equivalent: clamping the read count to 0..16, which `readCount` and an empty range already do. One survived: a multimap's size counting its names, since only the codec's decoded maps came from the builder and nothing reads a decoded map's size. So the port no longer keeps Guava's size field and adds up the lists instead, and that mutation of `size()` fails 3 suites now.
- With NaNs normalized, every line of the other golden files is identical to before, apart from `ByteBufCodecs`' two profile codecs, whose cases replace their skipped lines, and `java.util.UUID`'s new member. The new files, and the changed ones, come out byte-identical from two runs.
- Evidence: `cargo test --workspace` passes 120 tests, including 86 golden suites (115 and 81 before). fmt, clippy and the port check are clean.

### 2026-10-22: Login's serverbound protocol, the custom queries and the cookie packets

- Ported and checked against vanilla, milestone 3's fifth slice:
  - `ServerboundKeyPacket` (2,556 cases; the constructor, `getSecretKey` and `isChallengeValid` pending for `Crypt`), `ServerboundCustomQueryAnswerPacket` (7,046) and `ClientboundCustomQueryPacket` (7,990), with `CustomQueryAnswerPayload`, `CustomQueryPayload`, `DiscardedQueryAnswerPayload` (559 and `INSTANCE`) and `DiscardedQueryPayload` (3,680).
  - The cookie packets: `ClientboundCookieRequestPacket` (5,178), `ServerboundCookieResponsePacket` (5,412) and `CookiePacketTypes`, and from `common` `ClientboundStoreCookiePacket` with its `PAYLOAD_STREAM_CODEC` (6,852), `CommonPacketTypes`' 20 types, and `ClientCommonPacketListener` with `handleStoreCookie`.
  - The listeners' handlers for them: `handleKey`, `handleCustomQueryPacket`, `handleCustomQuery`, `handleRequestCookie` and `handleCookieResponse`.
  - `LoginProtocols.SERVERBOUND_TEMPLATE` and `SERVERBOUND`, with `SERVERBOUND.codec()` (1,545 cases over all five packets). `CLIENTBOUND_TEMPLATE`, `CLIENTBOUND` and its codec are pending: they need `ClientboundLoginFinishedPacket` (`GameProfile`) and `ClientboundLoginDisconnectPacket` (`Component`).
  - `StreamEncoder` and `writeNullable` take an unsized value, so a trait object passes where Java passes an interface.
- Settled (PORTING.md): a record's array component is an `Arc<Vec<i8>>`, which `equals` compares with `Arc::ptr_eq` like Java's reference. Its `hashCode` and `toString` go by the array's identity hash, so vanilla writes them skipped (`InstanceSuite.exclude`) and the port leaves them pending, like `ClientIntentionPacket`'s `hashCode`. An interface implemented by records (the payloads) is a trait with `equals`, `hash_code` and `Display`.
- Golden harness:
  - Each argument is now its own copy of a packet or payload, made of copies of its fields, so no two arguments share an array. Before, a receiver and an argument from the same edge were the same object, so a cookie packet's `equals` would have answered true where two values decoded from JSON can't be one object. The check was ambiguous, not weakened: no existing file changed.
  - Generators for the new packets and payloads: transaction ids, identifiers, cookies around the 5120-byte limit, and key packets made by reading their arrays from a buffer. Extra decode buffers: cookie payloads over the limit, written without it, and custom query payloads of 1 MiB and a byte more.
  - `PacketType`'s types include the cookie ones.
- Found:
  - `ServerboundCustomQueryAnswerPacket` writes its payload as a nullable (a boolean first) but reads it by skipping every byte left, so an answer of none reads back as a `DiscardedQueryAnswerPayload`, and no answer ever reads as none.
  - Both custom query readers skip the rest of the buffer up to 1 MiB, and throw `IllegalArgumentException` past it.
  - `ClientboundLoginDisconnectPacket` reads its reason as JSON (`lenientJson` with `ComponentSerialization.CODEC`), not NBT like configuration and play; the previous Next had it wrong.
- Mutations (32, run once and not committed): 30 fail the goldens. Two are equivalent: dropping the answer reader's `length >= 0` check (`readableBytes` is never negative), and writing a query's payload before its identifier (a discarded payload writes nothing).
- With NaNs normalized, every line of the other golden files is identical to before, apart from `PacketType`'s, whose types now include the cookie ones. The new files, and `PacketType`'s, come out byte-identical from two runs.
- Evidence: `cargo test --workspace` passes 115 tests, including 81 golden suites (104 and 70 before). fmt, clippy and the port check are clean.

### 2026-10-22: The protocol machinery, the handshake and four login packets

- Ported and checked against vanilla, milestone 3's fourth slice:
  - `ConnectionProtocol` (552 cases and its 5 constants), `PacketFlow` (539), `PacketType` (2,980; `hashCode` pending, below), `ClientIntent` (1,075).
  - `Packet` (a trait for its listener, with `packet::codec`), the listener traits the slice's packets need (`PacketListener`, `ServerboundPacketListener`, `ClientboundPacketListener`, `ServerPacketListener`, the cookie listeners, `ServerHandshakePacketListener`, `ServerLoginPacketListener`, `ClientLoginPacketListener`, with the handlers of the ported packets), `IdDispatchCodec` and its `Builder`, `ProtocolCodecBuilder`, `ProtocolInfoBuilder` (`addPacket`, `buildUnbound(C)`, `serverboundProtocol`, `clientboundProtocol`), `ProtocolInfo` with `Details` and `DetailsProvider`, and `SimpleUnboundProtocol`, all checked through `HandshakeProtocols`: its template's details, its bound protocol's id and flow, and `SERVERBOUND.codec()` (1,537 cases, 376 decoding a handshake).
  - `StreamCodec.mapStream` (1,537 cases), un-pended: the wrapper takes the `ByteBuf` for the call and gives it back after, even on a panic.
  - `ClientIntentionPacket` (7,703 cases, `hashCode` pending), `HandshakePacketTypes`, `HandshakeProtocols`, and from login `ServerboundHelloPacket` (6,130), `ClientboundHelloPacket` (6,374; `getPublicKey` pending for `Crypt`), `ServerboundLoginAcknowledgedPacket` (1,112 and `INSTANCE`), `ClientboundLoginCompressionPacket` (3,620) and `LoginPacketTypes`' 9 types. Each packet's `STREAM_CODEC`, constructor, getters, `type()`, `handle`, `isTerminal`, and a record's `equals` and `toString` (`hashCode` too for `ServerboundHelloPacket`).
  - `java.util.UUID`'s `hashCode` and `toString` (1,008 cases), which `ServerboundHelloPacket`'s use.
- Design (PORTING.md's new Packets and protocols section): `Packet<T>` is a trait for its listener, a protocol's codec reads and writes `Box<dyn Packet<L>>`, and a packet implements `Packet<L>` for every listener of its kind (Java's `? super L`). `PacketType` drops its type argument, and `type()` hands out the `LazyLock` constant. `ProtocolCodecBuilder` boxes packets and casts them back (`Upcast`), the cast Java's generics make silently. The context wrapper is an `Arc<dyn Fn(ByteBuf) -> B>`, and `Unit` is `()`.
- Golden harness:
  - New `PacketValues`: encodings and generators for flows, protocols, intents, packet types, the five packets (fields around each reader's string limit, ports past an unsigned short, byte arrays of a key's length), values declared as a `Packet` (`[type, fields]`), templates (their details) and bound protocols. A packet listener argument is a `Proxy` that records the calls on it, so `handle` cases show which handler a packet calls; the port's `Recorder` implements each listener trait the same way.
  - New `ProtocolSuite`, for a class of protocols: its constants, and each bound protocol's codec with any of the packets its template lists, or skipped naming the first packet without a generator.
  - `InstanceSuite.exclude`, for a record's `hashCode` that hashes an enum constant's identity hash, which changes from run to run (`PacketType`, `ClientIntentionPacket`): vanilla writes it as skipped, and the port marks it pending. A codec constant can take more buffers to decode (`extraInputs("STREAM_CODEC.decode", …)`), which `ServerboundHelloPacket` needed (below). TESTING.md describes all of it.
  - `golden.rs` gained `encode_packets!`, which decodes and encodes a protocol's `Box<dyn Packet<L>>` by type.
- Found:
  - `ServerboundHelloPacket` writes names with a limit of 16 but the codec's own encodings never hold a longer one, so reading with a limit of 17 survived until the decode cases got names written without the limit.
  - `ClientIntentionPacket` writes the host name with the usual 32767 limit and reads it with 255, and `ClientboundHelloPacket` writes the server id with 32767 and reads it with 20: either side can write what the other won't read.
  - The handshake's port is an int written as a short and read unsigned, so 65536 reads back as 0 and -1 as 65535.
  - `ProtocolCodecBuilder.add`'s flow check runs at `bind`, not when the template is built, since `buildPacketCodec` only runs then.
- Mutations (27, run once and not committed): 24 fail the goldens. One is equivalent (`IdDispatchCodec`'s id range as `<= len - 1`), and two are error paths no vanilla protocol reaches (the flow check and the duplicate check, under Next). Reading a hello's name with a limit of 17 survived the first inputs, which led to the extra decode buffers.
- With NaNs normalized, every line of the other golden files is identical to before, apart from `StreamCodec`'s `mapStream`, whose skipped line its cases replace. The new files, and `StreamCodec`'s, come out byte-identical from two runs.
- Evidence: `cargo test --workspace` passes 104 tests, including 70 golden suites (91 and 57 before). fmt, clippy and the port check are clean.

### 2026-10-22: FriendlyByteBuf's readers and writers taking codecs, and the frame handlers

- Ported and checked against vanilla, milestone 3's third slice:
  - `FriendlyByteBuf`'s 9 readers and writers that take a decoder, an encoder or a function: `readWithCount`, `readOptional`, `writeOptional`, `readNullable` and `writeNullable` (instance and static), `readById` and `writeById`, 18,897 cases over 16 instances. Only `readEither` and `writeEither` (DataFixerUpper's `Either`) are left of that kind.
  - `Varint21FrameDecoder` (1,058 cases) and `Varint21LengthFieldPrepender` (634 cases and its constant), the handlers that split what arrives into frames and put each frame's length in front of it. Pending: the decoder's `BandwidthDebugMonitor`, which only counts bytes for the debug screen's chart.
- Golden harness:
  - `InstanceSuite.given` lists the instances a member is given, like `CodecSuite.codec`: the cases are named `descriptor[label]`, a generator stands for each generated argument, and the instance itself stays out of `args`. A result declared as an `Object` is encoded by its own class. The readers also read what the writers write of 100 values, whole, with a field after and cut short (`Codecs.variants`, which `Codecs`' own encodings now share). `Codecs.nullableOf` makes values that are null a quarter of the time.
  - New `HandlerSuite` for the Netty handlers' protected `decode` and `encode`, called through subclasses with a null context on a fresh handler per case. Decoding runs the way Netty's `ByteToMessageDecoder` asks, again while it takes frames out, and its `[in chunks]` cases write the rest of the stream into the buffer a chunk at a time, half the cuts inside a frame's length.
  - TESTING.md describes both, and PORTING.md how decoder-taking methods and Netty handlers become Rust.
- Found:
  - `Varint21FrameDecoder` marks the reader index on every call, so it overwrites any mark the buffer had, and three bytes that all have the continuation bit throw even when nothing follows them, rather than waiting for more.
  - `Varint21LengthFieldPrepender` writes the body without reading it (Netty releases it after), and its error says "is over 8" whatever the size.
- Mutations (22, run once and not committed): 21 fail the goldens. Dropping the helper buffer's `clear()` survived until the `[in chunks]` cases: only a length that arrives split leaves stale bytes in it, and the first chunked inputs split a length in 2 of 300 runs, so half the cuts now land one byte into a frame (61 runs). The other survivor is equivalent: the prepender's `ensureWritable` before writing, since `calculateNewCapacity` depends only on the size needed, so the writes that follow grow the buffer to the same capacity.
- With NaNs normalized, every line of the other golden files is identical to before. `FriendlyByteBuf`'s only lost the 9 `skipped` lines its new entries replace.
- Evidence: `cargo test --workspace` passes 91 tests, including 57 golden suites (89 and 55 before). fmt, clippy and the port check are clean.

### 2026-10-22: StreamCodec, ByteBufCodecs, ByIdMap, LpVec3 and the core types' stream codecs

- Ported and checked against vanilla, milestone 3's second slice:
  - `StreamCodec` with `StreamDecoder`, `StreamEncoder`, `StreamMemberEncoder` and `CodecOperation`: `of`, `ofMember`, `unit`, `apply`, `map`, `cast` and all 14 `composite`s (one macro), 31,813 cases. Pending: `mapStream` (comes with the protocols), `dispatch` (its first caller) and `recursive` (text components).
  - `ByteBufCodecs`: its 20 codecs that need nothing unported (`BOOL` to `RGB_COLOR`, `OPTIONAL_VAR_INT`, `ROTATION_BYTE`, `PLAYER_NAME`…), its 4 int constants, `byteArray`, `stringUtf8`, `readCount`, `writeCount`, `optional`, `collection` (all 4), `list` (both), `fixedSizeCollection` (both), `fixedSizeList` and `idMapper(IntFunction, ToIntFunction)`, 49,981 cases. Pending (33): BitSet, NBT, DataFixerUpper, Gson, authlib, `Instant`, `Crypt`, the registries, `Either`, `map` and `lengthPrefixed`.
  - `ByIdMap` (`continuous` with each strategy, `sparse`, and the maps they refuse to make, 2,205 cases) and `OutOfBoundsStrategy` (539), `LpVec3` (5,011), `Math.clamp(DDD)D` (722), and ARGB's `red`, `green`, `blue` and `color(III)` (2,198), and `java.util.ArrayList`'s capacity constructor, which throws on a negative capacity a count read from a buffer can be.
  - Un-pended: the `STREAM_CODEC`s of `Vec3i`, `BlockPos`, `Direction`, `Vec3` (and `LP_STREAM_CODEC`), `BlockHitResult`, `Identifier` and `Rotation` (about 1,500 cases each), and `Direction.BY_ID` and `Rotation.BY_ID` (366 each).
- Design (PORTING.md's new Codecs section): `StreamCodec<B, V>` is a trait with associated types, `StreamCodec<Buffer = B, Value = V>`. Java's `StreamCodec<? super B, T>` is a codec whose buffer the wanted one is `AsMut` of (`FriendlyByteBuf` is `AsMut<ByteBuf>`); generic buffers on every codec made `BYTE.map(…)` and nested generic codecs ambiguous, which two prototypes showed. Static codecs are `LazyLock`s of boxed codecs (`StaticStreamCodec<B, V>`) behind associated consts (`BlockPos::STREAM_CODEC`). `ByteBufCodecs` is a unit struct like `Mth`.
- Golden harness:
  - New `Codecs` and `CodecSuite`: a codec named N gets `N.decode` and `N.encode` cases (the buffer before and after), an `IntFunction` `N.apply` cases. Decoding reads buffer edges, random buffers and the codec's own encodings of 200 values, whole, cut short and with a field after. Codec and `IntFunction` fields get them wherever a suite writes fields, so the core types' codecs came with no change to their suites. `CodecSuite` lists instances of methods taking codecs and functions under the method's descriptor and a label (`optional(…)[VAR_INT]`), and every `composite` gets a `FunctionN` constructor made with `Proxy` (TESTING.md).
  - Boxed numbers get their primitives' generators, `OptionalInt` is its int or null, and `golden.rs` gained `check_codec`, `check_codec_from`, `check_function`, `check_function_from`, and tuples up to 14 that decode too. `Golden` takes suite names after the output directory, to write only those.
- Found:
  - `OPTIONAL_VAR_INT` writes `of(-1)` as empty's 0, and `of(Integer.MAX_VALUE)` wraps to `MIN_VALUE` and back.
  - `RGB_COLOR` masks each byte with `0xFF` before `ARGB.color` masks it again, so the codec's mask changes nothing.
  - `ByIdMap.continuous` rejects a duplicate id before the missing one it implies; either way it throws.
  - `LpVec3`'s VarInt holds the scale's bits above the second as an unsigned int: a scale near `ABS_MAX_VALUE` writes -1.
  - `Math.clamp(DDD)` throws for bounds 0.0 and -0.0 (`Double.compare`), but not -0.0 and 0.0.
- Mutations (47, run once and not committed): 40 fail the goldens. Two survived the first inputs, which led to `sparse` of no values and `clamp`'s signed-zero and NaN bounds. The other 7 are equivalent: `RGB_COLOR`'s mask (above); `LpVec3`'s scale at least 1 (it always is past the threshold); `ByIdMap`'s `ZERO` treating id 0 as out of range (it gives the first value either way), a -1 id allowed (indexing panics), and no duplicate check (the missing-index check throws); `clamp` as `max(min, min(value, max))` (the same for every valid bound); and `ArrayList::new` without its check (`Vec::with_capacity` of a negative capacity panics anyway, for element types that aren't zero-sized).
- With NaNs normalized, every line of the other golden files is identical to before, apart from the entries that are checked now: the 9 codec and `BY_ID` fields above and `java.lang.Math`'s new `clamp(DDD)D`.
- Evidence: `cargo test --workspace` passes 89 tests, including 55 golden suites (84 and 50 before). fmt, clippy and the port check are clean.

### 2026-10-22: Netty's ByteBuf, VarInt, VarLong, Utf8String and FriendlyByteBuf

- Ported and checked against vanilla, milestone 3's first slice:
  - Netty's `ByteBuf` (netty-buffer 4.2.16, whose sources `tools/setup_vanilla.py` now puts next to JOML's): the 48 methods the client calls on a buffer, for its indexes, marks and capacity, primitives, byte arrays, other buffers, and `toString` of a range as UTF-8 (68,682 cases). It's one struct, AbstractByteBuf's checks over UnpooledHeapByteBuf's array, and grows like `Unpooled.buffer()`'s (`calculateNewCapacity`), with `ByteBufUtil`'s UTF-8 helpers and `Unpooled.buffer` (PORTING.md). The translated files keep Netty's Apache notice.
  - `VarInt` (4 methods and its constant, 4,426 cases), `VarLong` (4 methods, 3,695) and `Utf8String` (both methods, 3,186).
  - `FriendlyByteBuf`: its constructor, both constants and 47 readers and writers, for VarInts, VarLongs, strings, identifiers, UUIDs, block positions, byte, VarInt and long arrays, container ids, `Vector3f`s and `Quaternionf`s, instance and static (49,682 cases). It owns its `ByteBuf` and derefs to it. Pending (41): the codecs' (5), the ones taking a stream codec or a function (11), `readEnum`, `writeEnum` and the `EnumSet` ones (4, no `Class` or `Enum` generator yet), the `BitSet` ones (6), `ChunkPos` (4), `ResourceKey` (5) and NBT (6).
  - The JDK's UTF-8 decoding (`String.decodeUTF8_UTF16`, which `new String(bytes, UTF_8)` and Netty's `toString` run) as `string::from_bytes_utf8`, checked in `java.lang.String`'s suite (129,456 cases).
- Golden harness:
  - New `NetworkValues`: `ByteBuf`s are `[array, readerIndex, writerIndex, markedReaderIndex, markedWriterIndex]`, the whole array as hex, so the capacity and the bytes past the writer index are checked too. They hold fields the way packets write them, of every length, cut short, too long, malformed and negative (TESTING.md). A `FriendlyByteBuf` is the buffer it wraps.
  - Generators for `byte[]`, `int[]` and `long[]` (copied for each call, so `after` shows what a method wrote into them), `UUID`, `CharSequence` and `Charset` (UTF-8 only). No member of the earlier suites takes one, so no other file changed.
  - `InstanceSuite.withoutOverridesOf(ByteBuf.class)` leaves out `FriendlyByteBuf`'s forwarding overrides, which `ByteBuf`'s own suite covers.
  - Extra inputs where answers change, and two `where` filters, since vanilla allocates up to gigabytes before it throws on some lengths: `ensureWritable` and `writeBytes` grow the buffer before checking the source, and `readByteArray(int)` and `readVarIntArray(int)` allocate the array before reading it. `tools/golden.py` turns Netty's leak detector off: the cases never release what `readBytes(int)` returns.
- Found:
  - The JDK's decoder replaces an encoded surrogate (`ED A0 80`) with one U+FFFD, and one cut short (`ED A0`) too. Rust's `from_utf8_lossy` gives three and two, and fails 323 cases.
  - `utf8MaxBytes(maxLength)` is `maxLength * 3` in ints, so it wraps: with a maximum of `1 << 30` every string is too long, even `""`, and from about 1.43 billion the limit wraps back to a small positive one, which a string can fit.
  - `ByteBuf.toString(index, length, charset)` checks the range against the array, not the writer index, and doesn't check an empty range at all.
  - `writeBytes(byte[], int, int)` grows the array before it checks the source's range, so a call that throws can still grow it.
  - A negative length prefix passes the `size > maxSize` checks of `readByteArray`, `readVarIntArray` and `readLongArray`, and throws `NegativeArraySizeException`.
  - `writeFloat` writes raw bits: a signaling NaN keeps its payload.
- Mutations (44, run once and not committed): 38 fail the goldens. The other 6 are equivalent:
  - `ensureWritable` growing a buffer that's exactly full: `maxFastWritableBytes` grows it to the same capacity;
  - netty-common's `isOutOfBounds` without its end check: Rust's slice bounds panic before anything changes;
  - `readBytes(0)` returning a growable empty buffer instead of `EMPTY_BUFFER`, which only differ in their maximum capacity;
  - `VarInt.read` shifting without Java's mask: only a sixth byte shifts past 31, and it throws anyway;
  - `Utf8String.read` checking for a negative length first: both checks throw `DecoderException` after the same VarInt;
  - no negative-size check before allocating an array: `vec!` panics at the same point.

  `readUtf()`'s limit one less survived the first inputs, which led to strings of exactly 32767 and 32768 units. That mutation and the same one in `writeUtf(String)` fail now.
- With NaNs normalized, every line of the other golden files is identical to before. `java.lang.String`'s only gained the decoding cases after its 1,600 old lines.
- Evidence: `cargo test --workspace` passes 84 tests, including 50 golden suites (79 and 45 before). fmt, clippy and the port check are clean.

### 2026-10-22: Identifier and String.compareTo

- Ported and checked against vanilla, which finishes milestone 2's core types:
  - `Identifier`: 26 methods and its 4 constants (21,929 cases). Pending: the codecs, `ERROR_INVALID` and `read(String)` (DataFixerUpper, the network buffers, brigadier), `read(StringReader)` and `readNonEmpty` (brigadier), `withPath(UnaryOperator)` (no generator) and `resolveAgainst(Path)` (resource loading). `IdentifierException` is a panic, whose message isn't escaped like commons-lang's `escapeJava` does: goldens only record the exception's class.
  - `String.compareTo` in `JavaString` (1,076 cases), which `Identifier.compareTo` uses: it returns the difference of the first UTF-16 units that differ, or of the lengths.
- Golden harness: an `Identifier` generator (the game's identifiers, every allowed character, empty parts, `..` as a path) and extra inputs that break each rule, split at other separators, and compare identifiers with parts in common (TESTING.md). `java.lang.String`'s suite gained `compareTo`.
- Found:
  - `bySeparator` takes a `char`, so it can be asked to split at a surrogate, which Java's `indexOf` finds inside a supplementary character. The port takes a `u16` and never finds a surrogate; the answer is the same, since a split there leaves a lone surrogate in a part, and the unsplit string isn't a valid path either (PORTING.md).
  - The empty namespace is valid, and so is `.`; `..` is the only namespace of allowed characters that isn't. A path may be `..` or `../..`.
  - `tryBySeparator` checks the path before the namespace, and `bySeparator` the namespace before the path. Either way an invalid identifier gives `null` or throws, so the order doesn't show.
- Mutations (21, run once and not committed): 20 fail the goldens. The other is equivalent: checking a path's chars instead of its UTF-16 units, since every valid character is ASCII.
- With NaNs normalized, every line of the other golden files is identical to before. `java.lang.String`'s only gained `compareTo`.
- Evidence: `cargo test --workspace` passes 79 tests, including 45 golden suites (78 and 44 before). fmt, clippy and the port check are clean.

### 2026-10-22: AABB, HitResult and BlockHitResult

- Ported and checked against vanilla, milestone 2's third slice:
  - `AABB`: 51 of its 53 members (57,262 cases). Pending: `of(BoundingBox)`, which waits for `BoundingBox`, and `toString`, which needs Java's `Double.toString`. `clip`'s `Iterable<AABB>` overload takes a slice (`clip_aabbs`), and `move(Vec3)` is `r#move`.
  - `AABB.Builder`: all 4 members (1,627 cases).
  - `HitResult`: `getLocation` and `getType` (1,010 cases, on `BlockHitResult`s); `distanceTo` waits for `Entity`. `HitResult.Type` (539). `BlockHitResult`: everything but `STREAM_CODEC` (7,464). `HitResult` is abstract, so `BlockHitResult` holds it and derefs to it (PORTING.md).
  - With `AABB` there, `BlockPos.betweenClosed(AABB)`, `betweenClosedStream(AABB)` and `betweenCornersInDirection(AABB, Vec3)` (3,270 cases) and `Mth.rayIntersectsAABB` (3,547) are un-pended.
  - `Math.nextUp(D)` and `nextDown(D)` (1,082 cases), for `nextDeflated`.
- Golden harness:
  - A parameter's generic type can pick its generator where the raw class can't: `List<AABB>` and `Iterable<AABB>` are lists of boxes. `InstanceSuite` keeps parameterized parameter types, and any other gets its raw class's generator, so no other file changed.
  - An `Optional` result is its value, encoded by its own class, or null.
  - New generators: boxes (whole blocks, blocks' shapes in sixteenths, entities' boxes with `EntityDimensions`' float half widths, boxes between two `Vec3`s, any doubles), a block's shape as a list of boxes, `AABB.Builder`s (their private fields read and copied by reflection), `BlockHitResult`s and `HitResult.Type`.
  - Extra inputs where answers flip, which TESTING.md lists: near-equal boxes, boxes whose sides meet or miss by an ulp, points on sides, rays aimed at sides, edges and corners (and along an axis past a side at `clip`'s 1.0E-7 margin, give or take two ulps), rays moving exactly 1.0E-7, collisions, blocks at the int limits, and signed-zero moves.
- Found:
  - `AABB(BlockPos)`, `encapsulatingFullBlocks` and `intersects(BlockPos)` add 1 to the block's coordinates as ints, so at `Integer.MAX_VALUE` the far side wraps to -2^31.
  - `clip` only tries the side a ray enters through, on axes it moves more than 1.0E-7 along, and the scale has to be strictly between 0 and the nearest so far. So a ray that starts or ends on a side misses it, and on a tie the first side (x, then y, then z) and the first box win.
  - A ray that moves exactly 1.0E-7 along an axis can only be built near 0: elsewhere `to - from` can't come out as 1.0E-7, whose low bits are below the coordinates' ulp.
  - `collidedAlongVector`'s `contains(to)` only matters when a move ends exactly on the min side of the inflated box, where `clip`'s scale is 1, which misses.
  - Edition 2024's `impl Trait` borrows every argument, so `BlockPos.between_closed` couldn't take `between_closed_aabb`'s temporary corners until it said `+ use<>` (PORTING.md).
- Mutations (63, run once and not committed): 61 fail the goldens. The other 2 are equivalent: `clip_aabbs` passing `None` and keeping the old direction when nothing hits (the same thing written differently), and `rayIntersectsAABB`'s `>= 0.0` as `> 0.0` (a ray parallel to a slab it's outside of fails the cross-product tests anyway, unless the product underflows). 10 more survived the first version of the inputs, which led to the blocks at the int limits, the signed-zero moves, the rays moving 1.0E-7 near 0 (and starting at -0.0, where the hit's sign comes from the end's), the entities against a box, and the moves onto the inflated box's side. Rerunning every mutation on the new inputs, the margin's order (`pb - EPSILON < max_b`) survived, since the reshuffled rays passed the margin less often; 3,000 rays at the margin from a seed of their own fixed that without changing any other input.
- With NaNs normalized, every line of the other golden files is identical to before, apart from the members that are checked now: `BlockPos`'s 3 `AABB` overloads, `Mth.rayIntersectsAABB` (both skipped before) and `Math`'s 2 new methods.
- Evidence: `cargo test --workspace` passes 78 tests, including 44 golden suites (73 and 39 before). fmt, clippy and the port check are clean.

### 2026-10-22: BlockPos, MutableBlockPos, Rotation and AxisCycle

- Ported and checked against vanilla, milestone 2's second slice. Each class is ported whole, apart from what waits for other types:
  - `BlockPos`: 53 members and its 4 constants (56,627 cases). It holds its `Vec3i` and derefs to it, and the methods it overrides return a `BlockPos`. Its iterables are iterators over the fields of Java's anonymous `AbstractIterator`s: `betweenClosed` and its streams, `withinManhattan`, `withinClippedManhattan`, `withinBoxByManhattanDistance`, `neighborColumn`, `spiralAround`, `betweenCornersInDirection`, `randomBetweenClosed` and `randomInCube`. Pending: the codecs, the `AABB` and `BoundingBox` overloads, and `breadthFirstTraversal`, which only the server's blocks call (`SpongeBlock`, `CreakingHeartBlockEntity`) and whose callbacks have no generator.
  - `BlockPos.MutableBlockPos`: all 26 members (21,373 cases), through `Vec3i`'s protected setters, now `pub(super)`. `move` is `r#move`.
  - `BlockPos.TraversalNodeStatus` (539 cases), `AxisCycle` (7 members and 5 constants, 1,794 cases) and `Rotation` (8 members, 2,273 cases). Pending: `Rotation`'s codecs and `BY_ID`, and `rotation()`, which waits for `OctahedralGroup`.
  - `Mth.log2` and the functions it calls are `const fn`s now, so `BlockPos`'s packing constants are worked out as in Java.
  - `toString` prints the receiver's class name (`BlockPos{x=1, y=2, z=3}`), so `Vec3i`'s `Display` goes through a helper that takes the name.
- Golden harness:
  - An `Iterable` result is encoded like an iterator, each element as it comes, since `BlockPos`'s hand out one `MutableBlockPos` they move. When working out an element throws, the call counts as throwing (`Values.ElementThrew`), so the port collects the elements inside the checked call.
  - `InstanceSuite.where`, like `StaticSuite`'s, and `InstanceSuite.inherited`, for an inherited method whose answer depends on the receiver's class (`Vec3i.toString` for a `BlockPos` and a `MutableBlockPos`).
  - New generators: `BlockPos` and `MutableBlockPos` (`Vec3i`'s blocks plus the largest and smallest coordinates `asLong` packs, and one past them), and the constants of `AxisCycle`, `Rotation` and `TraversalNodeStatus`.
  - Extra inputs where results change: every block at the packing limits for `asLong`, packed blocks for `of`, `getX(J)` and the rest, points on `clampLocationWithin`'s bounds and an ulp either side, Manhattan reaches from -2 to 4, small boxes with their corners either way round, columns, every pair of directions for `spiralAround`, walks along movements with zero, -0.0 and NaN components, draws from small and empty boxes, and every rotation of 4 and 16 steps.
- Found:
  - `clampLocationWithin`'s lower bound is a float sum (`getX() + 1.0E-5F`, widened afterwards), and the upper one a double minus `1.0E-5F` widened; the bytecode shows both. From 256 blocks out the 1.0E-5 rounds away, so the lower bound is the block's own face. Past 2^24 the coordinate itself rounds, and the lower bound can land a block below, or above the upper bound.
  - `manhattanOrdered` looks only at the origin at depth 0, whatever the reaches, since its fields start at 0, and its `for` loop moves `y` on after a find too. With a negative reach, later depths start outside it. With a negative reach and a large depth it never finds a block and runs until the depth wraps, so the suite passes such inputs only with a depth up to 64.
  - `betweenClosed` never divides by zero: a side of 0 makes the end 0, so it visits nothing.
  - `spiralAround` starts at the center: the cursor starts a step along the second direction and steps back first.
- Mutations (35, run once and not committed): 34 fail the goldens. Two of them make a Manhattan search's depth the sum of the reaches instead of the largest, or the reverse: they hang on inputs where vanilla's ends, which counts as failing. The other mutation is equivalent: `neighborColumn`'s `>` as `>=` only changes the direction when the column is one block high, where `index % 1` is always 0.
- With NaNs normalized, every line of the other golden files is identical to before.
- Evidence: `cargo test --workspace` passes 73 tests, including 39 golden suites (68 and 34 before). fmt, clippy and the port check are clean.

### 2026-10-22: Vec3i, Direction, Vec3 and Vec2

- Ported and checked against vanilla, milestone 2's first slice. The bytecode scan (PORTING.md) shows the client calling nearly every method of these classes, so each is ported whole, apart from what waits for other types:
  - `Vec3i`: 41 members, all but the codecs (28,808 cases). The protected setters come with `MutableBlockPos`.
  - `Direction` (42 members, 18,304 cases) with its nested `Axis` (17, 3,383), `AxisDirection` (6, 543) and `Plane` (9, 2,091). Pending: the codecs and `BY_ID`, and `orderedByNearest` and `getFacingAxis`, which take an `Entity`.
  - `Vec3`: 56 members (42,128 cases). Pending: the codecs, and `toString`, which needs Java's `Double.toString`.
  - `Vec2`: 13 members (10,809 cases).
  - `Mth.getSeed(Vec3i)` and `Mth.lerp(double, Vec3, Vec3)`, un-pended (1,087 cases).
  - JOML's `Vector3i`: the 4 methods the client calls (2,474 cases).
  - `java.lang.Float.floatToIntBits` and `hashCode`, `Double.doubleToLongBits` and `hashCode` (2,184 cases, other NaN payloads included), which `Matrix4f`'s `equals` and `hashCode` now use too.
  - `Util.getRandom`, `shuffledCopy` and `shuffle`, checked through `Direction`'s and `Plane`'s random cases.
- Golden harness:
  - `InstanceSuite.every(Class)` covers every public constructor, method and constant a class declares, and writes the ones without a generator or encoder as skipped, so the port's test has to mark them pending.
  - New `CoreValues`: enum constants by name, `EnumSet<Direction.Axis>`, `Vec3i`, `Vec3` (also for `Position`) and `Vec2`, generated like the game's values (TESTING.md). Collections, streams and iterators are encoded as arrays of their elements. When every argument is an enum, there are no random cases: the edges are every combination already.
  - Extra inputs where results flip: points exactly as far apart as a strict `<` distance check's distance (and an ulp either side), lengths at `normalize`'s threshold, ties for `getNearest` and `axisStepOrder`, subnormal and NaN dot products for `getApproximateNearest`, `fromYRot`'s multiples of 45 degrees, every degree of `isFacingAngle`, and `BlockModelRotation`'s 48 matrices for `Direction.rotate`.
  - `golden.rs`: `encode_names!` encodes an enum by its constants' names, a `Vec` decodes from an array, and a `LegacyRandomSource` argument decodes from its seed and encodes as its next `nextLong()`, so `check_mut` checks instance methods that draw from one (`Vec3.offsetRandom`, `Plane.getRandomDirection`).
- Found:
  - `Direction.getApproximateNearest` starts from `Float.MIN_VALUE`, the smallest positive float: a zero or NaN vector gives `NORTH`, and a subnormal dot product beats it.
  - `Vec3.normalize` compares with `1.0E-5F` widened to a double (`9.999999747378752E-6`), which the bytecode shows.
  - x86's own NaN (`0.0 / 0.0`) is negative, unlike Java's canonical one, so `Vec3` and `Vec2` edges include one: their `hashCode` must canonicalize it.
- Mutations (42, run once and not committed): 41 fail the goldens. The other one is equivalent: `>>` for `>>>` in `Double.hashCode`, whose cast to int keeps only the low half. `<=` for `<` in `normalize` survived the first inputs, since no length came out exactly at the threshold. Along an axis it does, and that mutation fails 6 cases now.
- With NaNs normalized, every line of the other golden files is identical to before, except where the new types made skipped entries checkable: `Mth`'s `getSeed(Vec3i)` and `lerp(double, Vec3, Vec3)`, `ARGB.color(Vec3)` (under ARGB's `pending_rest`), and `Mth.rayIntersectsAABB`'s skip reason, which names `AABB` now.
- Evidence: `cargo test --workspace` passes 68 tests, including 34 golden suites (58 and 24 before). fmt, clippy and the port check are clean.

### 2026-10-22: Transformation, GivensParameters, MatrixUtil's decompositions and fdlibm's hypot

- Ported and checked against vanilla, which finishes the client's math that doesn't need milestone 2's types:
  - `GivensParameters`, a record: its constructor, both factories, accessors, `inverse`, `cos`, `sin` and the six `around*` (11,277 cases). Records' `equals`, `hashCode` and `toString` aren't ported: nothing calls them.
  - `MatrixUtil.eigenvalueJacobi` (2,962 cases) and `svdDecompose` (3,585), with their private helpers and the `G` and `PI_4` constants. Java names four matrices in the SVD but they're two objects that the steps overwrite in turn; the port follows the objects. Its zero-column checks compare as doubles (the bytecode widens them).
  - `Transformation`: both constructors, `compose`, `inverse`, `getMatrix`, `getMatrixCopy`, the four decomposition getters, `equals`, `hashCode`, `slerp`, the static `compose(Matrix4fc, Optional)` and `IDENTITY` (11,088 cases). The codecs wait for DataFixerUpper. The decomposition is a `OnceLock`, so its getters take `&self`. `IDENTITY` carries a flag its clones keep, since the game compares with it by reference (`FaceBakery`, and `inverse` returns it as it is).
  - `PoseStack.mulPose(Transformation)`. Java hands `mulPose` the transformation's own matrix, and `MatrixUtil.checkProperty` adds properties to it, so the port takes `&mut Transformation` and goes through `get_matrix_mut()` (PORTING.md). 256 more programs (768) multiply by transformations: 1,536 such steps, 16 of which add properties to the transformation's matrix.
  - fdlibm's `Hypot` in `fd_libm.rs`, as `Math.hypot` (8,253 cases: every edge pair, 6,000 spread over its paths, and its thresholds). std's `f64::hypot` (glibc's) gets 249 of them wrong.
  - JOML: `Matrix3f.scale(F)`; `Matrix4f.invertAffine` (both overloads), `isFinite`, `translation(Vector3fc)`, `scale(Vector3fc)`, `getTranslation`, `equals` (as `PartialEq`, comparing `floatToIntBits` and not the properties) and `hashCode`; `Quaternionf.slerp` into a `dest`; `Vector3f.lerp` into a `dest` (11,734 cases between them). The client calls each of them, or `Transformation` does (`equals` and `hashCode`, through `Objects`).
- Golden harness:
  - `StaticSuite` covers `void` methods: the result is null, and `after` shows what they wrote. On the Rust side, `golden.check_mut` takes a function with every argument as `&mut` and compares all of them afterwards.
  - `InstanceSuite` takes static members too (a record's factories), and `equals(Object)` gets another object of the same class.
  - New `MathValues`: `GivensParameters`, `Transformation` (made the ways the game makes them: block entities', block model rotations, entities' parts, their matrices, any JOML matrix, and decomposed ones) and `Optional<Transformation>`. JOML's generator (`Joml`, now `Mutable`) passes a null spec as null, and `golden.rs` decodes null as `None`.
- Mutations (41, run once and not committed): 34 fail the goldens. The other 7 are equivalent, or hidden where the game rounds:
  - `hypot`'s scaling thresholds moved inside their safety margins (3), and its early return moved from a ratio of 2^60 to 2^61, where the full computation rounds to the same answer;
  - swapping the two terms of a commutative addition in `slerp`;
  - comparing the SVD's zero columns as floats, which differs on one float value only;
  - `hypot`'s `w > b` as `w >= b`, which agrees on all 500 inputs with a = 2b exactly, where the two formulas add the same exactly split terms;
  - glibc's `hypot` in `qrGivensQuat`, whose result is rounded to a float.

  2 survived the first version of the inputs: `G` one ulp off and `w >= b`. That led to the matrices on `approxGivensQuat`'s quarter-turn boundary, after which `G` one ulp lower fails 40 cases (one ulp higher gives the same float `G`), and to the a = 2b pairs.
- With NaNs normalized, every line of the other golden files is identical to before. The changed files only gained the new members, and `PoseStack` its 256 programs after the old 512.
- Evidence: `cargo test --workspace` passes 58 tests, including 24 golden suites (56 and 22 before). fmt, clippy and the port check are clean.

### 2026-10-22: Matrix3x2f, Matrix3x2fStack, Vector2f and FrustumIntersection

- Ported from JOML 1.10.9 the methods the client calls, found by scanning its bytecode for `org/joml/` calls (PORTING.md). Each is checked against vanilla:
  - `Vector2f`: 2 constructors and 6 methods (5,524 cases).
  - `Matrix3x2f`, the GUI's 2D pose: 2 constructors and 12 methods (8,831 cases). These are `set`, `translate`, both `scale`s, `rotate`, `transformPosition` and `invert` with a `dest`; each fluent one also has its `dest` overload.
  - `Matrix3x2fStack` (push and pop). New `Matrix3x2fStackSuite`: 256 programs of 48 ops, using the ops the client calls on `GuiGraphicsExtractor`'s stack (push, pop, translate, scale, rotate), 10,055 steps.
  - `Matrix4f.mul(Matrix3x2fc)` and its `dest` overload (1,350 cases). `GlyphRenderState` uses it to put a glyph's 2D pose into 3D.
  - `FrustumIntersection`: `<init>()`, both `set`s, `intersectAab`, `testPoint` and its 15 constants (20,239 cases). Minecraft's `Frustum` needs `AABB`, so it waits for milestone 2.
- Golden harness:
  - `InstanceSuite` takes extra inputs per member, and can write the class's constants (`withFields`).
  - `golden.rs` encodes fixed-size arrays.
  - New generators: GUI poses, and frustums. Most frustums are the camera's culling frustums (`Camera.createProjectionMatrixForCulling` times the view rotation) looking anywhere, with the walk scenario's two looks among the edges. The rest are frustums of any JOML matrix, plus one perspective that ignores x.
  - `intersectAab` and `testPoint` get extra inputs:
    - boxes like the game's: chunk sections, entities, the camera's 8-block cube, and points;
    - boxes and points on a frustum face, to rounding: solved from three planes, then nudged by up to two ulps;
    - every float edge in each coordinate of a box in front of each edge frustum.
  - Every `intersectAab` result occurs: `INSIDE` 1,128 times, `INTERSECT` 2,569, and each plane from 262 to 4,109.
- Found:
  - Only cases on a plane pin down the order a plane's terms are added in. Before they existed, reassociating `testPoint`'s or `intersectAab`'s sums still passed the goldens.
  - A camera frustum's side planes pass through the camera (w = 0), so adding w first is exact there. Only the near and far planes, and frustums that aren't the camera's, can tell the order apart.
  - Moving `w` across `intersectAab`'s `>=` can't change a comparison near equality: `s + w` is exact when `s` is close to `-w` (Sterbenz). Only infinities can tell, and the frustum of an infinite translation covers those.
- Mutations (37, run once and not committed): 35 fail the goldens. The other 2 are equivalent:
  - a stack size of 0 panics anyway, on its allocation of -1, as Java throws `NegativeArraySizeException`;
  - the far plane's `w` moved across the `>=` (Sterbenz, above).

  4 mutations survived the first version of the frustum inputs, which led to the on-plane cases, the perspective that ignores x (a zero normal component times an infinite coordinate) and the float-edge sweep.
- With NaNs normalized, every line of the other golden files is identical to before. `Matrix4f`'s only gained the 1,350 new cases.
- Evidence: `cargo test --workspace` passes 56 tests, including 22 golden suites (52 and 18 before). fmt, clippy and the port check are clean.

### 2026-10-22: Axis, MatrixUtil, PoseStack, Projection, Matrix4fStack and Vector4f

- Ported Mojang's math on top of JOML, each checked against vanilla:
  - `com.mojang.math.Axis`: all 6 methods (3,716 cases). It's an interface, but only its six constants implement it, so it's an enum. `Axis.of` makes another implementation, but nothing calls it, so it isn't ported.
  - `MatrixUtil`: `mulComponentWise`, `checkPropertyRaw`, `checkProperty`, `isIdentity` and `isPureTranslation` (4,747 cases). `eigenvalueJacobi` and `svdDecompose` are pending: only `Transformation` calls them, and they return void, so the static suite skips them.
  - `PoseStack` and its `Pose`, except `translate(Vec3)` and `mulPose(Transformation)`. New `PoseStackSuite`: 512 programs of 48 ops (push, pop, both translates, uniform, mirrored and other scales, the rotations, `mulPose`, `transformNormal`, `setIdentity`, `isEmpty`), recording `[output, pose, normal, trustedNormals]` after each op (23,396 steps). One program in four takes any float, NaN included.
  - `Projection` and `ProjectionType`'s constants. `getMatrix` asks `RenderSystem`'s device whether depth runs from 0 to 1, so the port takes that as a parameter (PORTING.md). New `ProjectionSuite`, which installs a stand-in device (a `GpuDevice` proxy that only answers `getDeviceInfo`): 256 programs, half with each depth range, recording `getMatrix`'s matrix and every getter after each op (6,144 steps).
- JOML:
  - `Vector4f`: 5 constructors and 15 methods (13,264 cases).
  - `Matrix4f`: `assume`, and `transform`/`transformTranspose` of a `Vector4f` with their `dest` overloads (3,925 cases), which the frustum needs.
  - `Matrix4fStack` (`pushMatrix`, `popMatrix`, `clear`), which holds its `Matrix4f` and derefs to it. New `Matrix4fStackSuite`: 256 programs over stack sizes 1 to 20 and three the constructor rejects, recording the matrix after each op (10,426 steps).
  - With `Vector4f` values, ARGB's 12 `Vector4f` methods have cases now (8,514), under its `pending_rest`.
- Golden harness:
  - A slot vanilla leaves empty in `after` means the argument didn't change, or is the returned object. The port's argument must then equal what it was or the result. Before, `check_method` couldn't check a static method writing to its argument (`checkProperty`), since vanilla only records a changed argument.
  - `Encode::matches` never panics now: a value of another shape doesn't match. The rule above needs that, since it compares an argument with the result, like a matrix with `checkProperty`'s boolean.
  - Rust: `Encode` for `Box<dyn Encode>` and tuples, and `Thrown` for an op that panicked, so a program's steps keep their types (any NaN still matches any NaN). Java: an `Op` record shared by the program suites.
- Found:
  - NaN bits in the golden files differ between two runs of the generator. Two runs in a row gave 39 differing lines in `Matrix3f` and a few in four other files, all of them NaN signs. With NaNs normalized, every line of the old golden files is identical to before.
  - `PoseStack.Pose.mulPose` changes the matrix it's given: `MatrixUtil.checkProperty` adds the properties it works out to the matrix's. So `mul_pose` takes `&mut Matrix4f`.
- Mutations (31, run once and not committed): 29 fail the goldens, and the other 2 are equivalent (an index past the stack's end panics anyway, and JOML's `fma` is `a * b + c`). One found a gap: no program changed only the FOV between two setups, so dropping the FOV comparison from `setupPerspective` passed. Setups now also repeat the previous one with one argument changed (578 times), and that mutation fails 40 of 256 programs.
- Evidence: `cargo test --workspace` passes 52 tests, including 18 golden suites (12 before). fmt, clippy and the port check are clean.

### 2026-10-22: JOML's Math, Vector3f, Quaternionf, Matrix3f and Matrix4f

- The golden harness covers instance methods now. `InstanceSuite` (oracle/golden/) calls the constructors and instance methods it lists by reflection. The receiver is `args[0]`, `after[0]` is its state afterwards, and any other argument the method wrote to gets a slot unless it's the returned object. On the Rust side, `golden.check_method` compares all of that and fails a case when vanilla recorded a slot the check can't see. `encode_fields!` turns a struct into its JSON. TESTING.md has the format.
  - JOML's receivers and arguments come from JOML itself (`JomlValues`): identities (one with -0.0 in it, which still counts as the identity), translations, rotations, scales, the camera's reversed-depth perspective, the GUI's ortho, products of those, matrices built from components, NaN and infinities, and random transforms composed of one to four steps.
- Ported from JOML 1.10.9's sources into `src/org/joml/` (PORTING.md has a new section on libraries). `tools/setup_vanilla.py` fetches the sources jar from Maven Central, checks that Maven's binary jar is the one Mojang ships (same SHA-1), and unpacks it next to the decompiled game.
  - `Math`: 74 of its 76 methods, and its constants. `exp` is pending (HotSpot's stub isn't ported, and the client never calls it); `random` is clock-seeded.
  - `Matrix4fc`'s 20 constants.
  - The methods `Camera`, `PoseStack`, `Axis`, `Projection` and `GameRenderer` call, with what they call: 37 on `Vector3f`, 25 on `Quaternionf`, 31 on `Matrix3f`, 59 on `Matrix4f` (104,118 cases between them).
- Found:
  - JOML's fast trig (`-Djoml.fastmath`) and fused `fma` (`-Djoml.useMathFma`) are off unless the JVM is told otherwise, and the game doesn't. So JOML's trig goes through HotSpot's stubs, and its `fma` is `a * b + c`, rounded twice.
  - `Matrix4f.rotateAround(q, x, y, z, dest)` on an identity matrix writes and returns `this`, and leaves `dest` alone. Ported as is.
  - `Matrix4f`'s property shortcuts change bits: `identity()` keeps an identity's -0.0. Dropping that shortcut fails `identity` and `invert`.
  - `Math.roundLongUsing`'s half modes round to an int, so they saturate at the int range.
- `java.lang.Math` gained `acos`, `atan2`, `rint`, `round(F)`, `round(D)`, `signum(F)`, `signum(D)`, `toRadians` and `toDegrees`, all checked. With the libm crate, `atan2` got 76 of its 2,181 cases wrong: the crate ports musl, which returns ±pi/2 for |y/x| > 2^64 where fdlibm adds half of pi's low part. Ported the JDK's `FdLibm` `Asin`, `Acos`, `Atan` and `Atan2` into `src/java/lang/fd_libm.rs` instead, and dropped the libm dependency.
- `Mth`: un-pended and checked `rotationAroundAxis` and `X_AXIS`/`Y_AXIS`/`Z_AXIS`.
- In the other golden files, only the JOML entries vanilla used to skip changed (in `Mth` and `ARGB`). Every other line is byte-identical to before.
- Extra evidence, run once and not committed:
  - 12.3 million random `asin`/`acos`/`atan2` inputs from vanilla's JDK (10M with C2, 2M C1 only, 300k `-Xint`), compared with a release build of the port: 0 mismatches. The libm crate's `atan2` gets 12.6% of them wrong.
  - Mutations that each fail the goldens: `identity()` without its shortcut, `translate` keeping `PROPERTY_PERSPECTIVE`, `rotateAround` writing `dest` on an identity, `mulProject` without its identity shortcut, and a `normalize()` that leaves the receiver alone.
- Evidence: `cargo test --workspace` passes 46 tests, including 12 golden suites (6 before). fmt, clippy and the port check are clean.

### 2026-10-22: HotSpot's tan and pow

- Ported HotSpot's x86-64 stubs for `Math.tan` and `Math.pow` (from the same OpenJDK 25 source as sin, cos and log) into `src/java/lang/math/hotspot_x86/`. `Math::tan` and `Math::pow` run them.
  - tan's jumps nest, so it's `if`s like sin. pow jumps back and forth between 56 labels, so it's a loop with one `match` arm per label, in the C++'s order, and a `jmp!` macro (PORTING.md).
  - The register model gained `andpd`, `cvttpd2dq`, `cvtdq2pd`, `mulss`, `paddq`, `sqrtsd`, `movapd`, `movw`, `notl` and `bsrl`.
- Added and checked in the `java.lang.Math` suite: `tan(D)D` (20,541 cases) and `pow(DD)D` (22,181: every pair of the 41 double edge values, 20,000 extra inputs, 500 random). Nothing was pending.
  - tan's extra inputs come from sin's generator, with tan's thresholds (2^-29, 0.1 and 12800).
  - pow's cover game-like inputs, small integer exponents of any base, bases near 1 with exponents up to 2^64, results across the whole range (subnormal, overflowing, underflowing), negative bases with integer and non-integer exponents, tiny exponents, the special exponents (±0, ±1, 0.5, 2, 3, ±inf, NaN) and random bits.
  - The input helpers were factored out without changing a draw: every other line of the golden files is byte-identical to before.
  - Against these inputs, glibc gets 96 tan cases wrong, and 21 pow cases: 13 finite results one ulp off and 8 special cases (`pow(±1, ±inf)` and `pow(1, NaN)` are NaN in Java). fdlibm (the libm crate) gets 462 and 714 wrong. The port matches every case.
- Extra evidence, run once and not committed:
  - 10 million random inputs per function, streamed from vanilla's JDK into a release build of the port: 0 mismatches. The same with `-Xint` (300k each) and C1 only (2M each): 0 mismatches.
  - The same with garbage in every register lane the stubs don't set (the goldens, plus 5M streamed each): 0 mismatches.
  - All 29 tables equal the C++ arrays.
- `tools/hotspot_port_check.py`: all of tan's 448 and pow's 753 instructions match, and so do all 22 and 91 straight-line runs, in order. It now also reads `cmp64`/`jccb`, 16-bit stack loads (`movw`), indexed addresses (`Address(r11, rdx, Address::times_8, 0)`) and `ExternalAddress(e_coeff + 0)`. A swapped pair in pow, a changed displacement in pow's `L_tbl` address and a changed constant in tan each make it fail.
- pow picks a table entry with `rcpps` like log does, so its last bit can differ between CPUs too (PORTING.md).
- Evidence: `cargo test --workspace` passes 40 tests, including 6 golden suites. fmt, clippy and the port check are clean.

### 2026-10-22: HotSpot's sin, cos and log

- Ported HotSpot's x86-64 stubs for `Math.sin`, `cos` and `log` from OpenJDK 25 (adoptium/jdk25u at 520406d8, the source of the Temurin 25.0.4.1 the oracle runs) into `src/java/lang/math/hotspot_x86/`. Each `stubGenerator_x86_64_*.cpp` becomes one `.rs` file, with one Rust call per assembler instruction (`__ mulsd(xmm1, xmm0);` → `xmm1.mulsd(xmm0);`) on a small register model in `assembler.rs`. `Math::sin/cos/log` run them now, so `Mth`'s tables and `nextGaussian` do too.
- Un-pended and checked: `cos(D)D` and `log(D)D` in `java.lang.Math`, `atan2(DD)D` in `Mth`, and the four `<source>+nextGaussian` RandomSource programs. No pending entry that depends on `Math` is left.
- More golden inputs:
  - `Math.sin/cos/log`: 20,000 extra each (541 → 20,541 cases each), spread over every path through the stubs.
  - `Mth.atan2`: a 4096-angle sweep that reads every table entry (2,181 → 6,277 cases).
  - Gaussian programs: 16 → 64 per source.
  - Against these inputs, glibc's versions get 17 sin, 30 cos, 2 log and 6 atan2 cases and one gaussian program wrong. The 541 old inputs had let glibc's sin pass. The port matches every case.
- Extra evidence, run once and not committed:
  - 10 million random inputs per function, streamed from vanilla's JDK into a release build of the port: 0 mismatches.
  - The same with `-Xint` (300k) and C1 only (2M): 0 mismatches. So every JVM tier uses these stubs.
  - The same with garbage in every register lane the stubs don't set (5M): 0 mismatches. What a caller leaves in the registers never reaches the result.
  - All 22 tables match the C++ values.
- New: `tools/hotspot_port_check.py` reads a port back into assembler calls and compares it with the C++ of the installed JDK. All of sin's 359, cos's 357 and log's 105 instructions match, and so do all 53 straight-line runs, in order. A swapped pair and a changed constant both make it fail. CI runs it.
- Found: HotSpot's `log` picks its table entry with `rcpps`, an approximate reciprocal that Intel and AMD implement differently. With a correctly rounded reciprocal instead, 17 in 10 million log results on this Ryzen 5 5600G come out one ulp off. So the port runs the real instruction, and vanilla's outputs only compare with the port's from the same kind of CPU (PORTING.md). Golden files are regenerated on whichever machine runs the tests, CI included, so they stay consistent.
- Evidence: `cargo test --workspace` passes 40 tests, including 6 golden suites. fmt and clippy are clean.

### 2026-10-22: setup

- Vanilla 26.3 is decompiled into `vanilla/` (gitignored). The oracle covers four layers: function goldens, tick traces, packet captures and screenshots. See TESTING.md.
- Two vanilla runs of `walk` through a vanilla server match exactly (`check-determinism walk --capture`).
- Ported, and matching vanilla bit for bit: `Mth`, `RandomSource` and its four implementations, `RandomSupport`, `Xoroshiro128PlusPlus`, `MarsagliaPolarGaussian`, plus the JDK pieces they use (`Math`, `String.hashCode`, `UUID`).
- Pending, because HotSpot's x86-64 `cos`/`log` aren't ported: `Math.cos`, `Math.log`, `Mth.atan2`, and gaussian draws.
- Decided: a multiplayer-only client first. Entities will live in an arena addressed by ID when they come.
