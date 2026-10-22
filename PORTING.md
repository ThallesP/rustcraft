# Porting rules

How Java becomes Rust in this repo. The port should read side by side with `vanilla/<version>/src/` and match vanilla bit for bit, which TESTING.md checks.

## Layout and names

- One Java file becomes one Rust file at the same path, in snake_case: `net/minecraft/util/Mth.java` → `src/net/minecraft/util/mth.rs`. Each file starts with `//! net/minecraft/util/Mth.java`.
- Class names stay as they are, acronyms included (`Mth`, `ARGB`, `UUID`). Methods and fields become snake_case (`wrapDegrees` → `wrap_degrees`).
- Static methods and constants become associated functions and consts (`Mth::sin`, `Mth::PI`). Static methods of an interface become free functions in its module (`random_source::create`), since a trait has to stay object safe.
- Nested classes live in their outer class's module: `RandomSupport.Seed128bit` → `random_support::Seed128bit`.
- Rust has no overloading, so overloads get a suffix that tells them apart:
  - The Rust type, when they differ by type: `clamp_i32`, `clamp_f32`.
  - What they add, when they differ by arity: `next_int_bounded`, `length3_f64`.
  - The golden test is the mapping table: each `golden.check` line pairs a JVM descriptor with its Rust function.
- Constructors are `new`. Overloaded constructors follow the same rule (`from_seed128bit`).
- A Rust module named like its parent (`net/minecraft/util/Util.java` → `util::util`) keeps the name, with `#[allow(clippy::module_inception)]` on its `mod`.
- A Java name that's a Rust keyword stays, as a raw identifier: `MutableBlockPos.move` → `r#move`. Enum constants keep Java's names, underscores included (`Rotation::CLOCKWISE_90`).
- JDK behaviour the game depends on lives under `src/java/`, mirroring the JDK's packages (`java::lang::math::Math`, `java::util::uuid::UUID`).
- HotSpot code the JDK runs instead of Java follows the same rule under the class that calls it: `src/hotspot/cpu/x86/stubGenerator_x86_64_sin.cpp` → `src/java/lang/math/hotspot_x86/stub_generator_x86_64_sin.rs`. Assembly ports one call per instruction (`__ mulsd(xmm1, xmm0);` → `xmm1.mulsd(xmm0);`) on the register model in `hotspot_x86/assembler.rs`, and keeps OpenJDK's GPLv2 notices. A stub whose jumps nest becomes `if`s (sin); one that jumps back and forth between its blocks becomes a loop with one `match` arm per label, in the C++'s order, and a `jmp!` macro for its jumps (pow). `python3 tools/hotspot_port_check.py` checks that each port has its stub's instructions in the same order; CI runs it.
- Anything not ported yet is named in the file's header comment and marked pending in its golden test.

## Libraries

Libraries whose classes the game's math and packets run through are ported like the game, from their published sources, which `tools/setup_vanilla.py` puts next to the decompiled game (`vanilla/<version>/src/org/joml/Matrix4f.java` → `src/org/joml/matrix4f.rs`). So far that's JOML, the vector and matrix library, and Netty's buffers (`netty-buffer`). Their classes have hundreds of methods, so only the ones the client calls are ported, and the golden suite lists them. Translated files keep the library's licence notice when it asks for one (Netty's Apache notice, like HotSpot's GPL ones).

Which ones the client calls comes from its bytecode: `javap -c -p` over every class in `client.jar`, keeping the `org/joml/` invocations. A call through a subclass or an interface (`Matrix3x2fStack.translate`, `Matrix3x2fc.transformPosition`) runs the class's own method unless the subclass overrides it.

- JOML's read-only interfaces (`Matrix4fc`, `Vector3fc`, …) are only implemented by its own classes, so they become shared references (`&Matrix4f`), and their constants live in their module (`matrix4fc::PROPERTY_IDENTITY`).
- Constructors: the no-argument one is `new()`, the one taking the fields is `from_xyz`/`from_xyzw`/`from_values`, and copies are `from_<type>` (`Matrix3f::from_matrix4f`).
- Overloads taking components instead of a vector get `_xyz` (`add_xyz`), a scalar `_scalar`, an axis and angle `_axis`/`_axis_xyz` (`rotate_axis(angle, &axis)`), another type its name (`mul_matrix3f`).
- A method that writes to a `dest` argument gets `_into` and returns `dest` (`invert_into(&mut dest)`). Its fluent overload passes `this` as `dest` and returns `&mut Self`. Rust can't alias the two, so the fluent one writes into `self` from a copy of it, which is only the same as JOML when JOML reads all of `this` before writing `dest`. Check that it does. Where JOML gives the fluent overload a body of its own (`Matrix4f.translate`), or a method reads back what it just wrote (`rotationAround`), port it on `&mut self` as it is. The golden suite checks both overloads.
- `Matrix4f` keeps JOML's `properties`, which methods take shortcuts on. The shortcuts can give different bits than the full computation (`identity()` leaves an identity with -0.0 in it alone), so they're part of the port, and of the encoded state.
- JOML's `Math.fma` is `a * b + c`, rounded twice: the fused `java.lang.Math.fma` needs `-Djoml.useMathFma`, which the game doesn't set. Never use `mul_add`.

Netty:

- `ByteBuf` is one struct: AbstractByteBuf's indexes and bounds checks over UnpooledHeapByteBuf's array, read and written big-endian (`src/io/netty/buffer/byte_buf.rs`). The game only sees `ByteBuf`'s API, and Netty's other buffers (pooled, direct, composite, slices) only differ in where the bytes live and how they grow. Its capacity is the array's length, and grows like `Unpooled.buffer()`'s (`AbstractByteBufAllocator.calculateNewCapacity`), which the golden suite checks. `retain` and `release` are Rust's ownership, and `alloc()` is the one allocator.
- A getter and its setter overload (`readerIndex()` and `readerIndex(int)`, `capacity()` and `capacity(int)`) become `reader_index` and `set_reader_index`. Overloads by argument type get the type: `readBytes(int)` is `read_bytes`, `readBytes(byte[])` `read_bytes_array`, `readBytes(byte[], int, int)` `read_bytes_array_range`, and `writeBytes(ByteBuf)` `write_bytes_buf`.
- A call that throws halfway leaves the buffer as Java's does: `writeBytes(byte[], int, int)` grows the array before it checks the source's range, and `VarInt.read` has consumed the bytes it read when it throws. The golden cases record the buffer after a throw too.

## Object model

- Interfaces become traits. An interface-typed value becomes `Box<dyn Trait>` when owned and `&mut dyn Trait` (or `&dyn Trait`, like a `Position`) when borrowed.
- An enum becomes a Rust enum. When its constants take constructor arguments (`Direction`'s data values, name, axis and normal), those live in a static table indexed by the constant, one line per constant like Java's. Its nested enums live in its module (`direction::Axis`). `values()` is a `VALUES` const and `values()`, and `valueOf` is `value_of`, which panics where Java throws.
- `toString` becomes `Display`, so `to_string()` works. `equals` becomes `PartialEq`: derived when Rust compares the fields the way Java does (ints, or floats with `==` in `Vec2`), written out when it doesn't (`Vec3` compares with `Double.compare`, so -0.0 differs from 0.0 and NaN equals NaN). `hashCode` is `hash_code()`, with `java::lang::float::Float` and `java::lang::double::Double` for floats' bits. A Java `equals` that returns true for the same object first only differs for a NaN component, where Rust has no identity to check.
- An argument Java lets be null is an `Option` (`Direction.getNearest`'s fallback, `Direction.Axis.test`).
- An interface implemented only by its own constants becomes an enum (`Axis::XP`; `Axis.of` would make another, but nothing calls it). Its overloads that differ by type get the type (`rotate_matrix4f`).
- A class that extends another to add state holds its parent as a field and derefs to it: `Matrix4fStack` is the `Matrix4f` it derefs to, plus the saved copies. So does one that adds no state: `BlockPos` holds the `Vec3i` it derefs to, and `MutableBlockPos` the `BlockPos`. The methods a subclass overrides are its own and return its type (`BlockPos.offset` returns a `BlockPos`). A parent's method that calls an override in Java (`BlockPos.above(int)` reaching `MutableBlockPos.relative`) calls the parent's own in Rust, which is fine while both give the same values: `MutableBlockPos`'s overrides only make sure a copy comes back instead of the cursor, and every Rust result is a copy anyway.
- A class that wraps another and forwards its API to it (`FriendlyByteBuf` around a `ByteBuf`) owns it and derefs to it. Its static methods that take the wrapped type, next to instance methods of the same name, get `_from` for readers and `_to` for writers: `readUUID()` is `read_uuid` and `readUUID(ByteBuf)` `read_uuid_from`, `writeUUID(ByteBuf, UUID)` `write_uuid_to`. An overload that adds a maximum gets `_max` (`readUtf(int)` is `read_utf_max`).
- An abstract class is the same: `BlockHitResult` holds its `HitResult` and derefs to it, and the abstract `getType` is `BlockHitResult`'s own. How the game holds either kind of hit (`Minecraft.hitResult`) gets settled with `EntityHitResult`.
- Protected members become `pub(super)`, visible to their package: `Vec3i`'s setters, which only `MutableBlockPos` calls. A `toString` that prints the receiver's class name (`MoreObjects.toStringHelper(this)`) becomes a helper taking the name, which each subclass's `Display` passes (`BlockPos{x=1, y=2, z=3}`).
- An `Iterable` or `List` argument becomes a slice (`AABB.clip`'s boxes, `&[AABB]`), and an `Optional` result an `Option`.
- Iterables and streams become iterators, built with `iter::from_fn` on the anonymous `AbstractIterator`'s fields. One that only copies its arguments says so with `+ use<>`: edition 2024's `impl Trait` otherwise borrows every argument for as long as the iterator lives, so a caller couldn't pass temporaries (`BlockPos.between_closed`, which `between_closed_aabb` calls with the corners it works out). Java's that hand out one cursor over and over (`BlockPos.betweenClosed`'s `MutableBlockPos`) yield copies of it, which is what a caller that uses each element at once, or copies it with `immutable()`, sees. A caller that keeps the cursor itself needs care when it's ported.
- Immutable value classes (`Vec3i`, `Vec3`) are `Copy` structs whose methods take `&self` and return new values. Where Java returns `this` (`offset(0, 0, 0)`), the port returns a copy, which is the same value. A getter handing out a shared read-only object (`Direction.getUnitVec3f`'s `Vector3fc`) returns a `&'static` reference.
- A getter that hands out a mutable object (`PoseStack.last()`) becomes `last()` returning `&` and `last_mut()` returning `&mut`. So does one whose read-only view a caller casts back and writes to: `MatrixUtil.checkProperty` adds properties to the `Matrix4fc` that `Transformation.getMatrix()` hands out, so `PoseStack.mulPose(Transformation)` takes `&mut Transformation` and goes through `get_matrix_mut()`.
- State a getter works out on first use (`Transformation`'s decomposition) lives in a `OnceLock`, so the getter takes `&self`. Cloning such an object stands for copying a Java reference to it when nothing else about it changes: the lazy state comes out the same whenever it's worked out.
- A constant the game compares by reference (`rotation != Transformation.IDENTITY`) carries a private flag that only the constant sets and its clones keep.
- Global state that isn't ported yet becomes a parameter, like a back-reference: `Projection.getMatrix` asks `RenderSystem`'s device whether depth runs from 0 to 1, so `Projection::get_matrix` takes `z_zero_to_one`.
- An interface that fills in another interface's methods with defaults (`BitRandomSource`) becomes functions that implementors forward to. A Rust trait can't provide another trait's methods.
- A back-reference, i.e. an object pointing at its owner, becomes a parameter. `MarsagliaPolarGaussian` holds its `RandomSource` in Java; in Rust, the owner passes itself in.
- Concurrency guards that `&mut self` already rules out, such as an AtomicLong checked by a ThreadingDetector, become plain fields.
- Deep class hierarchies (`Entity → LivingEntity → Mob → …`) get settled when the first entity is ported. The plan: entities live in an arena and are addressed by ID, the parent class is embedded as a field, and overridden methods become trait methods.

## Codecs

The network codecs are objects Java builds out of other objects and lambdas, so they get rules of their own, and so do the methods taking them.

- `StreamCodec<B, V>` is a trait with two associated types, `StreamCodec<Buffer = B, Value = V>`. A codec has one buffer, Java's `B`. Java passes a codec of a buffer's parent class where a codec of the buffer is wanted (`StreamCodec<? super B, T>`); here that's a codec whose buffer the wanted one is `AsMut` of. Every buffer is `AsMut` of itself and of the buffers it wraps (`FriendlyByteBuf` is `AsMut<ByteBuf>` and `AsMut<FriendlyByteBuf>`), and a direct call derefs a wrapper (`BlockPos::STREAM_CODEC.decode(friendly_buf)`). A codec Java makes generic in its buffer takes the buffer of the codec it wraps (`ByteBufCodecs::optional`). A `composite` takes the buffer it's used on, which the static's type pins; a nested one may need it spelled out.
- A static codec field is a module `static` of type `StaticStreamCodec<B, V>` (a `LazyLock` of a boxed codec, made on first use), which an associated const refers to: `BlockPos::STREAM_CODEC`. An `IntFunction` field is the same with a boxed `Fn(i32) -> T` (`Direction::BY_ID`), passed on as `Direction::BY_ID.as_ref()`. Codecs share other codecs by reference, since `&C` is a codec.
- Java's anonymous classes implementing a codec are `stream_codec::of(encoder, decoder)`, or a struct inside the function that makes them. A trait method's codec (`map`, `cast`) and one a `CodecOperation` makes (`ByteBufCodecs::list()`'s `Collection`) are named structs: `impl Trait` in a trait method hides that the codec is `Send` and `Sync`, which a static needs, and a closure can't return `impl Trait`.
- The functional interfaces (`StreamDecoder`, `StreamEncoder`, `StreamMemberEncoder`, `CodecOperation`) are traits that every closure of the right shape implements, so lambdas and method references pass as they are (`stream_codec::of(LpVec3::write, LpVec3::read)`). A codec isn't a `StreamDecoder` too, since Rust can't have both blanket impls: where Java passes a codec as a decoder, pass `|input| codec.decode(input)`. Other functional interfaces (`IntFunction`, `ToIntFunction`, `Function`) are closures.
- `ByteBufCodecs`, an interface Java only uses for its constants and static methods, is a unit struct like `Mth`.
- Overloads: `composite`'s get their number of fields (`composite3`), ones returning a `CodecOperation` instead of a codec get `_operation` (`collection(IntFunction)` is `collection_operation`), and ones adding a maximum `_max` (`collection_max`, `list_max`).
- A codec's collection is any Rust collection it can extend and iterate over by reference: a `Vec` for an `ArrayList`. Java's collection constructors throw on a negative capacity, which a count read from a buffer can be, so codecs making lists take `ArrayList::new` from `java::util::array_list`, which panics there too. Which Rust type stands for a Java hash set or map gets settled with its first caller, since encoding writes in Java's iteration order.
- `OptionalInt` is `Option<i32>`, and a codec's `encode` takes the value by reference, while getters and `map`'s functions return owned values (a getter clones where Java would share).
- A method taking a `StreamDecoder<? super FriendlyByteBuf, T>` (`FriendlyByteBuf.readOptional`) is generic in the decoder's buffer, `impl StreamDecoder<I, T>` where `Self: AsMut<I>`, so a decoder of a `ByteBuf` or of a `FriendlyByteBuf` passes. A static one generic in its buffer (`readNullable(B, StreamDecoder<? super B, T>)` with `B extends ByteBuf`) takes `B: AsMut<ByteBuf> + AsMut<I>`, and reaches each with `AsMut::<ByteBuf>::as_mut(input)`. A value they write that Java lets be null, or passes as an `Optional`, is an `Option<&T>`, and `T` may be a trait object, as Java's may be an interface (`writeNullable` of a `&dyn CustomQueryAnswerPayload`). A `Consumer` is an `FnMut`, an `IntFunction` an `Fn(i32) -> T`, and a `ToIntFunction` an `Fn(&T) -> i32`.

- `mapStream(Function)`, which wraps a connection's `ByteBuf` in the buffer a codec reads (`FriendlyByteBuf::new`) and shares its bytes, takes the buffer for the call and hands it back after (`From<FriendlyByteBuf> for ByteBuf` is Java's `unwrap()`), even when the call panics, so a throwing read leaves the buffer where Java's does.

## Packets and protocols

- `Packet<T>` is a trait for the listener `T` that handles the packet, and a protocol's codec reads and writes `Box<dyn Packet<L>>` for its listener `L`. Java's protocols take packets of the listener's parents too (`Packet<? super L>`, like the cookie packets in login), so a packet implements `Packet<L>` for every `L` of its kind: `impl<L: ClientLoginPacketListener + ?Sized> Packet<L> for ClientboundHelloPacket`. Calling a `Packet` method on a concrete packet then names the listener: `Packet::<dyn ServerHandshakePacketListener>::r#type(packet)`.
- Listener interfaces are traits with their handlers (`handle_intention(&mut self, &ClientIntentionPacket)`), and `handle(&self, listener: &mut T)` calls one. A listener's default `flow()` and `protocol()` would fill in its parent's methods, which a Rust trait can't, so they come with `PacketListener`'s methods and `Connection`.
- `PacketType<T>`'s type argument only tells the compiler which packet class has the type, so it's a plain `PacketType`. The `*PacketTypes` constants are `LazyLock<PacketType>` statics, and `type()` (`r#type`) returns the shared one, `&'static PacketType`.
- A packet's private constructor reading a buffer is `from_friendly_byte_buf`, and its private `write` is `write`; `packet::codec(write, from_friendly_byte_buf)` makes its `STREAM_CODEC` like Java's `Packet.codec`. A record's `toString` is `Display` (`ClientIntentionPacket[protocolVersion=777, …]`), and its `hashCode` adds each component's to 31 times the ones before.
- A record's array component (a cookie's payload) is an `Arc<Vec<i8>>`, a reference to the array like Java's. The record's `equals` compares it by reference, which is `Arc::ptr_eq`: a clone of the packet shares the array and equals it, and another packet with the same bytes doesn't. Its `hashCode` and `toString` hash and print the array's identity hash, which changes from run to run, so they aren't ported.
- An interface whose implementations are records (a custom query's payload) is a trait with `Display` as a supertrait and Java's `equals` and `hashCode` as methods, `equals(&self, other: &dyn Trait)` and `hash_code`, since `PartialEq` can't be a supertrait of a trait a `dyn` holds. `dyn Trait` is `PartialEq` through `equals`. A packet holding one as `Box<dyn Trait>` writes its own `PartialEq` out, since `derive` tries to move a boxed trait object to compare it.
- `IdDispatchCodec`'s `add` takes a codec of its values themselves. `ProtocolCodecBuilder` boxes each packet its codec reads and casts what it's given to write back to the packet's class (`Upcast`), the cast Java's generics make without saying.
- `ProtocolInfo`, `ProtocolInfo.Details`, `SimpleUnboundProtocol` and the context wrapper are traits and types made once each by `ProtocolInfoBuilder`, held in statics as `StaticProtocolInfo<L>` and `StaticSimpleUnboundProtocol<L, B>`. The context wrapper, Java's `Function<ByteBuf, B>`, is an `Arc<dyn Fn(ByteBuf) -> B>`, since each packet's codec holds it. A protocol without a context has Java's `Unit` as its context, which is `()`. `build_unbound` moves the builder in, where Java copies its list.
- Netty's `DecoderException` and `EncoderException` that `IdDispatchCodec` wraps a packet's exception in are panics, which carry no class, so the packet's panic goes on as it is.

## Netty handlers

- A handler in the game's pipeline (`Varint21FrameDecoder`, a `ByteToMessageDecoder`; `Varint21LengthFieldPrepender`, a `MessageToByteEncoder`) is a struct whose `decode` or `encode` leaves out the `ChannelHandlerContext` it doesn't use. Its output list is a `Vec` of what it makes (`Vec<ByteBuf>` for frames), and releasing buffers in `handlerRemoved` is Rust's drop.
- What Netty does around a handler (cumulating what arrives, calling `decode` again while it makes progress, releasing the message after `encode`) belongs to the pipeline, and comes with `Connection`. The golden cases drive a decoder the way Netty does (TESTING.md).

## Where Java and Rust disagree on numbers

The golden tests caught each of these, or exist to catch them.

- **Overflow.** Java ints and longs wrap; Rust panics in debug builds, so use `wrapping_*`. The port's tests run unoptimized with overflow checks, so a wrap you missed panics instead of passing.
- **Widen where Java widens.** In `Mth.getSeed`, `x * 3129871` is an int multiply that wraps *before* the result widens to long.
- **`>>>`** is a cast to unsigned, a shift and a cast back. `>>` is an arithmetic shift in both languages.
- **`Math.min` / `Math.max` on floats:** a NaN wins and -0.0 is less than 0.0. Rust's `f32::min` returns the operand that isn't NaN, so use `Math::min_f32`.
- **`Math.floorDiv` / `Math.floorMod`** round toward negative infinity, so use `Math::floor_div_i32`. `div_euclid` gives different answers for negative divisors.
- **`Math.abs(Integer.MIN_VALUE)`** wraps, so use `Math::abs_i32`.
- **`clamp`:** Rust's `i32::clamp` panics when min > max and Java's doesn't. Write `max(min).min(max)`.
- **NaN bits:** `Float.floatToIntBits` turns every NaN into the canonical one. `floatToRawIntBits` is `to_bits`.
- **Strings** are UTF-16. `String.hashCode` runs over UTF-16 units (`JavaString::hash_code`), and lengths and indices count UTF-16 units too. `String.compareTo` returns the difference of the first units that differ, or of the lengths (`JavaString::compare_to`), not just its sign, and beyond the BMP its order isn't code point order. A `char` is a `u16`, since a Rust `char` can't hold half of a surrogate pair (`Identifier.bySeparator` can be asked to split at one).
- **Decoding UTF-8** goes through `string::from_bytes_utf8`, the JDK's own decoder (`new String(bytes, UTF_8)`, which Netty's `ByteBuf.toString` runs). It replaces malformed input with U+FFFD differently from `String::from_utf8_lossy`: an encoded surrogate (`ED A0 80`) is one replacement, not three, and so is one cut short at the end (`ED A0`). Encoding a Rust string is its own bytes: Java writes a lone surrogate as `?`, and a Rust string can't hold one.
- **`byte[]`** is `Vec<i8>` or `&[i8]`, so reading an element sign-extends like Java's.
- **`new T[size]`** throws `NegativeArraySizeException` for a negative size. Check it before `vec![…; size as usize]`, which would try to allocate.
- **Shift amounts.** Java masks an int's shift to 5 bits and a long's to 6, and Rust's `<<` panics past the width, so a shift that can reach it is `wrapping_shl` (`VarInt.read`'s sixth byte shifts by 35, which is 3).
- **Same in both:** `(int)` casts from floating point (they saturate, and NaN becomes 0, like `as`), and float `%` (fmod).
- **Transcendental functions.**
  - On x86-64, HotSpot computes `Math.sin/cos/tan/log/log10/exp/pow` (and `tanh`, `cbrt`) with its own assembly, based on Intel's libm. The interpreter and both JITs call the same stubs. Functions without that (`asin`, `acos`, `atan2`, `hypot`, …) fall back to `StrictMath`, which runs the JDK's Java port of fdlibm, ported in `src/java/lang/fd_libm.rs`. The `libm` crate is no substitute: it ports musl, whose `atan2` differs from fdlibm's on one in eight random inputs. Nor is std's `f64::hypot` (glibc's), which gets 249 of the 8,253 `Math.hypot` goldens wrong.
  - glibc and fdlibm differ from the HotSpot routines in the last bit on some inputs. Vanilla itself gives different results on ARM.
  - Every call goes through `java::lang::Math`. `sin`, `cos`, `tan`, `log` and `pow` run ports of HotSpot's stubs (`src/java/lang/math/hotspot_x86/`). Port the others the same way when the game first calls them.
  - HotSpot's `log` and `pow` pick a table entry with `rcpps`, an approximate reciprocal that Intel and AMD implement differently, so `Math.log` and `Math.pow` can differ in the last bit between CPUs. The port runs the same instruction and matches vanilla on the machine both run on; a correctly rounded reciprocal instead puts 17 in 10 million results one ulp off. Only compare vanilla's outputs with the port's from the same kind of CPU.
- **Vineflower and float literals.** Vineflower prints a double constant that fits in a float with an `F` suffix, so `long * 1.110223E-16F` reads as float math when the bytecode multiplies doubles. When an expression's type matters, read the bytecode: `vanilla/toolchain/jdk-25/bin/javap -c -p -cp vanilla/26.3/client.jar net.minecraft.util.Mth`.
- **Methods that never return.** Some vanilla methods never return on extreme inputs (`Mth.rotLerpRad` with a huge difference). Port them as they are; the golden suite keeps its inputs in range with `where()`.

## Optimizing

The port starts literal. An optimization is welcome as long as every golden test, trace, packet capture and screenshot still matches. The comparisons are the contract, not the shape of the code. These look free but change results:

- Fused multiply-add (`mul_add`), or reordering float additions and multiplications. Java rounds every operation in source order.
- Narrowing `f64` math to `f32`, or the reverse.
- Replacing a lookup table with the real function (`Mth.sin` is a 65536-entry table, not `sin`).
- SIMD reductions that sum in a different order.
- Iterating a `HashMap` where Java iterated a `LinkedHashMap`, or relying on any hash order. Java's order is deterministic, Rust's isn't.

## Porting a class

1. Read the Java in `vanilla/<version>/src/`.
2. Add the class to `oracle/golden/src/rustcraft/golden/Golden.java`: a `StaticSuite` for static methods, or a hand-written suite for stateful classes like `RandomSourceSuite`. Then run `python3 tools/golden.py`.
3. Port it, and end the file with its golden test: `golden.check(descriptor, function)` for each method, `golden.pending(name, reason)` for the rest, then `golden.finish()`.
4. Run `cargo test`. A mismatch prints the arguments, vanilla's answer and yours, with floats shown as bits and decimals.
