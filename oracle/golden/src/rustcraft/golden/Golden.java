package rustcraft.golden;

import com.google.gson.JsonElement;
import com.google.gson.JsonPrimitive;
import com.mojang.authlib.GameProfile;
import com.mojang.authlib.properties.Property;
import com.mojang.authlib.properties.PropertyMap;
import com.mojang.math.Axis;
import com.mojang.math.GivensParameters;
import com.mojang.math.MatrixUtil;
import com.mojang.math.OctahedralGroup;
import com.mojang.math.Transformation;
import io.netty.buffer.ByteBuf;
import io.netty.buffer.Unpooled;
import java.io.IOException;
import java.io.PrintStream;
import java.lang.reflect.Method;
import java.lang.reflect.Proxy;
import java.nio.charset.StandardCharsets;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.BitSet;
import java.util.HashMap;
import java.util.HexFormat;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.Optional;
import java.util.Random;
import java.util.UUID;
import java.util.function.BiConsumer;
import java.util.function.Consumer;
import java.util.function.Function;
import java.util.function.ToIntFunction;
import java.util.stream.IntStream;
import java.util.stream.Stream;
import net.minecraft.SharedConstants;
import net.minecraft.SystemReport;
import net.minecraft.core.AxisCycle;
import net.minecraft.core.BlockPos;
import net.minecraft.core.Direction;
import net.minecraft.core.Registry;
import net.minecraft.core.RegistrySynchronization;
import net.minecraft.core.SectionPos;
import net.minecraft.core.UUIDUtil;
import net.minecraft.core.Vec3i;
import net.minecraft.core.registries.Registries;
import net.minecraft.nbt.ByteArrayTag;
import net.minecraft.nbt.ByteTag;
import net.minecraft.nbt.CompoundTag;
import net.minecraft.nbt.EndTag;
import net.minecraft.nbt.NbtAccounter;
import net.minecraft.nbt.NbtIo;
import net.minecraft.nbt.StringTag;
import net.minecraft.nbt.Tag;
import net.minecraft.network.ConnectionProtocol;
import net.minecraft.network.FriendlyByteBuf;
import net.minecraft.network.LpVec3;
import net.minecraft.network.Utf8String;
import net.minecraft.network.VarInt;
import net.minecraft.network.VarLong;
import net.minecraft.network.codec.ByteBufCodecs;
import net.minecraft.network.codec.StreamCodec;
import net.minecraft.network.codec.StreamDecoder;
import net.minecraft.network.codec.StreamEncoder;
import net.minecraft.network.protocol.PacketFlow;
import net.minecraft.network.protocol.PacketType;
import net.minecraft.network.protocol.common.ClientboundClearDialogPacket;
import net.minecraft.network.protocol.common.ClientboundCustomPayloadPacket;
import net.minecraft.network.protocol.common.ClientboundCustomReportDetailsPacket;
import net.minecraft.network.protocol.common.ClientboundKeepAlivePacket;
import net.minecraft.network.protocol.common.ClientboundPingPacket;
import net.minecraft.network.protocol.common.ClientboundPostEffectsPacket;
import net.minecraft.network.protocol.common.ClientboundResourcePackPopPacket;
import net.minecraft.network.protocol.common.ClientboundStoreCookiePacket;
import net.minecraft.network.protocol.common.ClientboundTransferPacket;
import net.minecraft.network.protocol.common.CommonPacketTypes;
import net.minecraft.network.protocol.common.ServerboundClientInformationPacket;
import net.minecraft.network.protocol.common.ServerboundCustomClickActionPacket;
import net.minecraft.network.protocol.common.ServerboundCustomPayloadPacket;
import net.minecraft.network.protocol.common.ServerboundKeepAlivePacket;
import net.minecraft.network.protocol.common.ServerboundPongPacket;
import net.minecraft.network.protocol.common.ServerboundResourcePackPacket;
import net.minecraft.network.protocol.common.custom.BrandPayload;
import net.minecraft.network.protocol.common.custom.CustomPacketPayload;
import net.minecraft.network.protocol.common.custom.DiscardedPayload;
import net.minecraft.network.protocol.configuration.ClientboundCodeOfConductPacket;
import net.minecraft.network.protocol.configuration.ClientboundFinishConfigurationPacket;
import net.minecraft.network.protocol.configuration.ClientboundRegistryDataPacket;
import net.minecraft.network.protocol.configuration.ClientboundResetChatPacket;
import net.minecraft.network.protocol.configuration.ClientboundSelectKnownPacks;
import net.minecraft.network.protocol.configuration.ClientboundUpdateEnabledFeaturesPacket;
import net.minecraft.network.protocol.configuration.ConfigurationPacketTypes;
import net.minecraft.network.protocol.configuration.ConfigurationProtocols;
import net.minecraft.network.protocol.configuration.ServerboundAcceptCodeOfConductPacket;
import net.minecraft.network.protocol.configuration.ServerboundFinishConfigurationPacket;
import net.minecraft.network.protocol.configuration.ServerboundSelectKnownPacks;
import net.minecraft.network.protocol.cookie.ClientboundCookieRequestPacket;
import net.minecraft.network.protocol.cookie.CookiePacketTypes;
import net.minecraft.network.protocol.cookie.ServerboundCookieResponsePacket;
import net.minecraft.network.protocol.handshake.ClientIntent;
import net.minecraft.network.protocol.game.ClientboundChunkBatchFinishedPacket;
import net.minecraft.network.protocol.game.ClientboundChangeDifficultyPacket;
import net.minecraft.network.protocol.game.ClientboundInitializeBorderPacket;
import net.minecraft.network.protocol.game.ClientboundPlayerAbilitiesPacket;
import net.minecraft.network.protocol.game.ClientboundSetChunkCacheCenterPacket;
import net.minecraft.network.protocol.game.ClientboundSetChunkCacheRadiusPacket;
import net.minecraft.network.protocol.game.ClientboundSetExperiencePacket;
import net.minecraft.network.protocol.game.ClientboundSetHealthPacket;
import net.minecraft.network.protocol.game.ClientboundSetHeldSlotPacket;
import net.minecraft.network.protocol.game.ClientboundSetSimulationDistancePacket;
import net.minecraft.network.protocol.game.ClientboundTickingStatePacket;
import net.minecraft.network.protocol.game.ClientboundTickingStepPacket;
import net.minecraft.world.Difficulty;
import net.minecraft.network.protocol.game.ClientboundRecipeBookSettingsPacket;
import net.minecraft.stats.RecipeBookSettings;
import net.minecraft.world.inventory.RecipeBookType;
import net.minecraft.core.GlobalPos;
import net.minecraft.core.Rotations;
import net.minecraft.network.protocol.game.ClientboundSetDefaultSpawnPositionPacket;
import net.minecraft.network.protocol.game.ClientboundSetEntityDataPacket;
import net.minecraft.network.protocol.game.ClientboundSetTimePacket;
import net.minecraft.network.protocol.game.ClientboundUpdateAttributesPacket;
import net.minecraft.network.syncher.SynchedEntityData;
import net.minecraft.world.clock.ClockNetworkState;
import net.minecraft.world.clock.WorldClock;
import net.minecraft.world.entity.Pose;
import net.minecraft.world.entity.ai.attributes.AttributeModifier;
import net.minecraft.world.entity.ai.attributes.Attributes;
import net.minecraft.world.level.storage.LevelData;
import net.minecraft.network.protocol.game.ClientboundChunkBatchStartPacket;
import net.minecraft.network.protocol.game.ClientboundGameEventPacket;
import net.minecraft.network.protocol.game.ClientboundPlayerPositionPacket;
import net.minecraft.network.protocol.game.ClientboundStartConfigurationPacket;
import net.minecraft.network.protocol.game.GamePacketTypes;
import net.minecraft.network.protocol.game.GameProtocols;
import net.minecraft.network.protocol.game.ServerboundAcceptTeleportationPacket;
import net.minecraft.network.protocol.game.ServerboundChatCommandPacket;
import net.minecraft.network.protocol.game.ServerboundChunkBatchReceivedPacket;
import net.minecraft.network.protocol.game.ServerboundClientTickEndPacket;
import net.minecraft.network.protocol.game.ServerboundConfigurationAcknowledgedPacket;
import net.minecraft.network.protocol.game.ServerboundMovePlayerPacket;
import net.minecraft.network.protocol.game.ServerboundPlayerCommandPacket;
import net.minecraft.network.protocol.game.ServerboundPlayerInputPacket;
import net.minecraft.network.protocol.game.ServerboundPlayerLoadedPacket;
import net.minecraft.network.protocol.handshake.ClientIntentionPacket;
import net.minecraft.network.protocol.handshake.HandshakePacketTypes;
import net.minecraft.network.protocol.handshake.HandshakeProtocols;
import net.minecraft.network.protocol.login.ClientboundCustomQueryPacket;
import net.minecraft.network.protocol.login.ClientboundHelloPacket;
import net.minecraft.network.protocol.login.ClientboundLoginCompressionPacket;
import net.minecraft.network.protocol.login.ClientboundLoginFinishedPacket;
import net.minecraft.network.protocol.login.LoginPacketTypes;
import net.minecraft.network.protocol.login.LoginProtocols;
import net.minecraft.network.protocol.login.ServerboundCustomQueryAnswerPacket;
import net.minecraft.network.protocol.login.ServerboundHelloPacket;
import net.minecraft.network.protocol.login.ServerboundKeyPacket;
import net.minecraft.network.protocol.login.ServerboundLoginAcknowledgedPacket;
import net.minecraft.network.protocol.ping.ClientboundPongResponsePacket;
import net.minecraft.network.protocol.ping.PingPacketTypes;
import net.minecraft.network.protocol.ping.ServerboundPingRequestPacket;
import net.minecraft.network.protocol.status.ServerboundStatusRequestPacket;
import net.minecraft.network.protocol.status.StatusPacketTypes;
import net.minecraft.network.protocol.status.StatusProtocols;
import net.minecraft.network.protocol.login.custom.DiscardedQueryAnswerPayload;
import net.minecraft.network.protocol.login.custom.DiscardedQueryPayload;
import net.minecraft.resources.Identifier;
import net.minecraft.resources.ResourceKey;
import net.minecraft.server.Bootstrap;
import net.minecraft.server.level.ClientInformation;
import net.minecraft.server.level.ParticleStatus;
import net.minecraft.server.packs.repository.KnownPack;
import net.minecraft.util.ARGB;
import net.minecraft.util.ByIdMap;
import net.minecraft.util.Mth;
import net.minecraft.util.StringRepresentable;
import net.minecraft.world.entity.HumanoidArm;
import net.minecraft.world.entity.PositionMoveRotation;
import net.minecraft.world.entity.player.ChatVisiblity;
import net.minecraft.world.entity.player.Input;
import net.minecraft.world.level.block.Rotation;
import net.minecraft.world.level.levelgen.RandomSupport;
import net.minecraft.world.phys.AABB;
import net.minecraft.world.phys.BlockHitResult;
import net.minecraft.world.phys.HitResult;
import net.minecraft.world.phys.Vec2;
import net.minecraft.world.phys.Vec3;
import org.apache.logging.log4j.Level;
import org.apache.logging.log4j.core.config.Configurator;
import org.joml.FrustumIntersection;
import org.joml.Matrix3f;
import org.joml.Matrix3x2f;
import org.joml.Matrix4f;
import org.joml.Matrix4fc;
import org.joml.Quaternionf;
import org.joml.Vector2f;
import org.joml.Vector3f;
import org.joml.Vector3i;
import org.joml.Vector4f;

/** Writes vanilla's answers for the Rust golden tests, one JSON Lines file per class. See TESTING.md. */
public final class Golden {
    private static final double SIN_SCALE = 10430.378350470453;
    // Last-bit disagreements between libms are rare, so the methods HotSpot computes with its own
    // assembly get this many extra inputs each, spread over every path through the assembly.
    private static final int INTRINSIC_INPUTS = 20000;

    private static final Values.Generator INTS = Values.generator(int.class);
    private static final Values.Generator FLOATS = Values.generator(float.class);
    private static final Values.Generator BOOLEANS = Values.generator(boolean.class);
    private static final Values.Generator STRINGS = Values.generator(String.class);
    private static final Values.Generator DIRECTIONS = Values.generator(Direction.class);
    // Sizes of fixed-size lists, mostly ones a random list has. Larger ones can't be read from a random buffer, and
    // ArrayList's constructor rejects a negative one.
    private static final Values.Generator SIZES = new Values.Simple(
        List.of(-1, 0, 1, 2, 3, 4, 7, 140, 65536), random -> random.nextInt(7), value -> new JsonPrimitive((Integer)value)
    );
    private static final Values.Generator BIT_SET_SIZES = new Values.Simple(
        List.of(-9, -1, 0, 1, 7, 8, 9, 26, 63, 64, 65, 128, 256, 257, 65536), random -> random.nextInt(300), value -> new JsonPrimitive((Integer)value)
    );
    private static final Values.Generator VEC3S = Values.generator(Vec3.class);
    private static final Values.Generator IDENTIFIERS = Values.generator(Identifier.class);
    // Ids around a continuous map's, and any ints.
    private static final Values.Generator IDS = new Values.Simple(
        IntStream.rangeClosed(-16, 16).mapToObj(id -> (Object)id).toList(), INTS::random, INTS::encode
    );
    // Strings, and ones around a signature's limit of 1024 units, in characters of one, two and three bytes and in
    // supplementary ones.
    private static final Values.Generator SIGNATURES = new Values.Simple(
        Stream.concat(
            STRINGS.edges().stream(),
            Stream.of("a".repeat(1024), "a".repeat(1025), "é".repeat(1024), "€".repeat(1025), "😀".repeat(512), "😀".repeat(513))
        ).toList(),
        STRINGS::random,
        STRINGS::encode
    );
    private static final String OPTIONAL = "optional(Lnet/minecraft/network/codec/StreamCodec;)Lnet/minecraft/network/codec/StreamCodec;";
    private static final String READ_OPTIONAL = "readOptional(Lnet/minecraft/network/codec/StreamDecoder;)Ljava/util/Optional;";
    private static final String WRITE_OPTIONAL = "writeOptional(Ljava/util/Optional;Lnet/minecraft/network/codec/StreamEncoder;)V";
    private static final String READ_NULLABLE = "readNullable(Lnet/minecraft/network/codec/StreamDecoder;)Ljava/lang/Object;";
    private static final String READ_NULLABLE_FROM = "readNullable(Lio/netty/buffer/ByteBuf;Lnet/minecraft/network/codec/StreamDecoder;)Ljava/lang/Object;";
    private static final String WRITE_NULLABLE = "writeNullable(Ljava/lang/Object;Lnet/minecraft/network/codec/StreamEncoder;)V";
    private static final String WRITE_NULLABLE_TO = "writeNullable(Lio/netty/buffer/ByteBuf;Ljava/lang/Object;Lnet/minecraft/network/codec/StreamEncoder;)V";
    private static final String READ_BY_ID = "readById(Ljava/util/function/IntFunction;)Ljava/lang/Object;";
    private static final String WRITE_BY_ID = "writeById(Ljava/util/function/ToIntFunction;Ljava/lang/Object;)Lnet/minecraft/network/FriendlyByteBuf;";
    private static final String READ_WITH_COUNT = "readWithCount(Ljava/util/function/Consumer;)V";
    private static final String READ_ENUM = "readEnum(Ljava/lang/Class;)Ljava/lang/Enum;";
    private static final String WRITE_ENUM = "writeEnum(Ljava/lang/Enum;)Lnet/minecraft/network/FriendlyByteBuf;";
    private static final Values.Generator TAGS = Values.generator(Tag.class);
    private static final String TAG_CODEC = "tagCodec(Ljava/util/function/Supplier;)Lnet/minecraft/network/codec/StreamCodec;";
    private static final String DISCARDED_CODEC = "codec(Lnet/minecraft/resources/Identifier;I)Lnet/minecraft/network/codec/StreamCodec;";
    private static final String LENGTH_PREFIXED = "lengthPrefixed(I)Lnet/minecraft/network/codec/StreamCodec$CodecOperation;";
    private static final String LENGTH_PREFIXED_DECORATED =
        "lengthPrefixed(ILjava/util/function/BiFunction;)Lnet/minecraft/network/codec/StreamCodec$CodecOperation;";
    private static final String CLICK_ACTION_TAGS = "optionalTagCodec(new NbtAccounter(32768L, 16))";
    private static final String DISCARDED_MARKERS = "FriendlyByteBuf::new, DiscardedPayload.codec(rustcraft:scenario_start, 32767)";
    private static final Identifier MARKER = Identifier.fromNamespaceAndPath("rustcraft", "scenario_start");
    // The most a length prefix may say: the click action's 65536, small ones that some random tags' encodings fit and
    // others don't, a discarded payload's 32767 and one more, and the int limits.
    private static final Values.Generator PREFIX_SIZES = new Values.Simple(
        List.of(65536, 0, 1, 2, 3, 4, 16, 32767, 32768, Integer.MAX_VALUE, -1, Integer.MIN_VALUE),
        random -> random.nextBoolean() ? 65536 : random.nextInt(64),
        value -> new JsonPrimitive((Integer)value)
    );
    private static final String CONTINUOUS =
        "continuous(Ljava/util/function/ToIntFunction;[Ljava/lang/Object;Lnet/minecraft/util/ByIdMap$OutOfBoundsStrategy;)Ljava/util/function/IntFunction;";
    // A record's hashCode adds up its components', and an enum constant's is its identity hash.
    private static final String ENUM_IN_RECORD = "hashes an enum constant's identity hash, which changes from run to run";
    // An array's hashCode and toString are Object's, so a record holding one hashes and prints its identity hash.
    private static final String ARRAY_IN_RECORD = "goes by an array component's identity hash, which changes from run to run";
    // A ResourceKey doesn't override hashCode, since keys are interned, so a record holding one hashes its identity hash.
    private static final String KEY_IN_RECORD = "hashes a ResourceKey's identity hash, which changes from run to run";
    // Holder.Reference doesn't override hashCode either.
    private static final String HOLDER_IN_RECORD = "hashes a Holder's identity hash, which changes from run to run";
    // An entity data serializer is a lambda, whose hashCode and toString are its identity's.
    private static final String SERIALIZER_IN_RECORD = "hashes and prints a serializer's identity, which changes from run to run";
    private static final String SETTINGS_IN_RECORD = "compares, hashes or prints RecipeBookSettings by identity";
    private static final String DATA_VALUE_READ =
        "read(Lnet/minecraft/network/RegistryFriendlyByteBuf;I)Lnet/minecraft/network/syncher/SynchedEntityData$DataValue;";
    // The walk's set_time packets: the first, with both clocks (the end's, then the overworld's at 6000 and stopped), and
    // one with none.
    private static final List<byte[]> SET_TIMES = List.of(
        HexFormat.of().parseHex("0000000000000031020101000000000000000000f02e0000000000000000"), HexFormat.of().parseHex("000000000000003b00")
    );
    // The walk's update_attributes packets: the player's movement speed, block and entity interaction ranges, and its
    // movement speed with the sprinting modifier.
    private static final List<byte[]> UPDATE_ATTRIBUTES = List.of(
        HexFormat.of().parseHex("01031a3fb99999a000000000084012000000000000000d400800000000000000"),
        HexFormat.of().parseHex("01011a3fb99999a000000001136d696e6563726166743a737072696e74696e673fd333334000000002"),
        HexFormat.of().parseHex("01011a3fb99999a000000000")
    );
    private static final String SPARSE = "sparse(Ljava/util/function/ToIntFunction;[Ljava/lang/Object;Ljava/lang/Object;)Ljava/util/function/IntFunction;";

    private static final List<Suite> SUITES = List.of(
        new StaticSuite(Mth.class)
            .extraInputs("sin(D)F", sinTableInputs(0))
            .extraInputs("cos(D)F", sinTableInputs(-16384))
            .extraInputs("atan2(DD)D", atan2Inputs())
            .extraInputs("rayIntersectsAABB(Lnet/minecraft/world/phys/Vec3;Lnet/minecraft/world/phys/Vec3;Lnet/minecraft/world/phys/AABB;)Z", clipRays(3))
            // Steps by 2π until the difference is in range, which never ends once 2π is below the float's precision.
            .where("rotLerpRad(FFF)F", args -> !(Math.abs((Float)args[2] - (Float)args[1]) >= 0x1p26F)),
        new StaticSuite(ARGB.class),
        new StaticSuite(RandomSupport.class).exclude("generateUniqueSeed()J", "seeded from System.nanoTime"),
        new RandomSourceSuite(),
        new StaticSuite(Math.class)
            .only(
                "abs(I)I", "abs(J)J", "abs(F)F", "abs(D)D",
                "min(FF)F", "max(FF)F", "min(DD)D", "max(DD)D",
                "floorDiv(II)I", "floorDiv(JJ)J", "floorMod(II)I", "floorMod(JJ)J",
                "sin(D)D", "cos(D)D", "tan(D)D", "asin(D)D", "log(D)D", "pow(DD)D", "sqrt(D)D",
                "acos(D)D", "atan2(DD)D", "rint(D)D", "round(F)I", "round(D)J", "signum(F)F", "signum(D)D",
                "toRadians(D)D", "toDegrees(D)D", "hypot(DD)D", "nextUp(D)D", "nextDown(D)D", "clamp(DDD)D", "clamp(FFF)F"
            )
            .extraInputs("sin(D)D", trigInputs(1, -253, 16))
            .extraInputs("cos(D)D", trigInputs(2, -253, 16))
            .extraInputs("tan(D)D", trigInputs(5, -40, 13))
            .extraInputs("log(D)D", logInputs())
            .extraInputs("pow(DD)D", powInputs())
            .extraInputs("hypot(DD)D", hypotInputs())
            .extraInputs("clamp(DDD)D", clampInputs())
            .extraInputs("clamp(FFF)F", clampInputs().stream().map(Golden::asFloats).toList()),
        new StringSuite(),
        // JOML's own fast paths are off (no -Djoml.fastmath), so its trig goes through java.lang.Math.
        new StaticSuite(org.joml.Math.class).exclude("random()D", "seeded from the clock"),
        new StaticSuite(Matrix4fc.class),
        new InstanceSuite(
            Vector3f.class,
            "<init>()V", "<init>(FFF)V", "<init>(Lorg/joml/Vector3fc;)V",
            "set(FFF)Lorg/joml/Vector3f;", "set(Lorg/joml/Vector3fc;)Lorg/joml/Vector3f;", "zero()Lorg/joml/Vector3f;",
            "add(FFF)Lorg/joml/Vector3f;", "add(Lorg/joml/Vector3fc;)Lorg/joml/Vector3f;",
            "sub(FFF)Lorg/joml/Vector3f;", "sub(Lorg/joml/Vector3fc;)Lorg/joml/Vector3f;",
            "mul(F)Lorg/joml/Vector3f;", "mul(FFF)Lorg/joml/Vector3f;", "mul(Lorg/joml/Vector3fc;)Lorg/joml/Vector3f;",
            "div(F)Lorg/joml/Vector3f;", "get(I)F", "isFinite()Z",
            "mul(Lorg/joml/Matrix3fc;)Lorg/joml/Vector3f;", "mul(Lorg/joml/Matrix3fc;Lorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            "negate()Lorg/joml/Vector3f;", "cross(FFF)Lorg/joml/Vector3f;", "cross(Lorg/joml/Vector3fc;)Lorg/joml/Vector3f;",
            "lerp(Lorg/joml/Vector3fc;F)Lorg/joml/Vector3f;", "lerp(Lorg/joml/Vector3fc;FLorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            "dot(FFF)F", "dot(Lorg/joml/Vector3fc;)F", "length()F", "lengthSquared()F",
            "distance(FFF)F", "distance(Lorg/joml/Vector3fc;)F", "distanceSquared(FFF)F", "distanceSquared(Lorg/joml/Vector3fc;)F",
            "normalize()Lorg/joml/Vector3f;", "normalize(Lorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            "rotate(Lorg/joml/Quaternionfc;)Lorg/joml/Vector3f;", "rotate(Lorg/joml/Quaternionfc;Lorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            "mulProject(Lorg/joml/Matrix4fc;)Lorg/joml/Vector3f;", "mulProject(Lorg/joml/Matrix4fc;Lorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            "mulPosition(Lorg/joml/Matrix4fc;)Lorg/joml/Vector3f;", "mulPosition(Lorg/joml/Matrix4fc;Lorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            "mulDirection(Lorg/joml/Matrix4fc;)Lorg/joml/Vector3f;", "mulDirection(Lorg/joml/Matrix4fc;Lorg/joml/Vector3f;)Lorg/joml/Vector3f;"
        ),
        new InstanceSuite(
            Quaternionf.class,
            "<init>()V", "<init>(FFFF)V", "<init>(Lorg/joml/Quaternionfc;)V",
            "set(FFFF)Lorg/joml/Quaternionf;", "set(Lorg/joml/Quaternionfc;)Lorg/joml/Quaternionf;", "identity()Lorg/joml/Quaternionf;",
            "rotationX(F)Lorg/joml/Quaternionf;", "rotationY(F)Lorg/joml/Quaternionf;", "rotationZ(F)Lorg/joml/Quaternionf;",
            "rotateX(F)Lorg/joml/Quaternionf;", "rotateX(FLorg/joml/Quaternionf;)Lorg/joml/Quaternionf;",
            "rotateY(F)Lorg/joml/Quaternionf;", "rotateY(FLorg/joml/Quaternionf;)Lorg/joml/Quaternionf;",
            "rotationXYZ(FFF)Lorg/joml/Quaternionf;", "rotationYXZ(FFF)Lorg/joml/Quaternionf;", "rotationZYX(FFF)Lorg/joml/Quaternionf;",
            "rotationAxis(FFFF)Lorg/joml/Quaternionf;", "rotationAxis(FLorg/joml/Vector3fc;)Lorg/joml/Quaternionf;",
            "conjugate()Lorg/joml/Quaternionf;", "conjugate(Lorg/joml/Quaternionf;)Lorg/joml/Quaternionf;",
            "normalize()Lorg/joml/Quaternionf;", "normalize(Lorg/joml/Quaternionf;)Lorg/joml/Quaternionf;",
            "mul(Lorg/joml/Quaternionfc;)Lorg/joml/Quaternionf;", "mul(Lorg/joml/Quaternionfc;Lorg/joml/Quaternionf;)Lorg/joml/Quaternionf;",
            "mul(FFFF)Lorg/joml/Quaternionf;", "mul(FFFFLorg/joml/Quaternionf;)Lorg/joml/Quaternionf;",
            "transform(Lorg/joml/Vector3f;)Lorg/joml/Vector3f;", "transform(Lorg/joml/Vector3fc;Lorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            "transform(FFFLorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            "slerp(Lorg/joml/Quaternionfc;FLorg/joml/Quaternionf;)Lorg/joml/Quaternionf;"
        )
            .extraInputs("slerp(Lorg/joml/Quaternionfc;FLorg/joml/Quaternionf;)Lorg/joml/Quaternionf;", slerpInputs()),
        new InstanceSuite(
            Matrix3f.class,
            "<init>()V", "<init>(FFFFFFFFF)V", "<init>(Lorg/joml/Matrix3fc;)V", "<init>(Lorg/joml/Matrix4fc;)V",
            "set(Lorg/joml/Matrix3fc;)Lorg/joml/Matrix3f;", "set(Lorg/joml/Matrix4fc;)Lorg/joml/Matrix3f;",
            "set(FFFFFFFFF)Lorg/joml/Matrix3f;", "identity()Lorg/joml/Matrix3f;",
            "zero()Lorg/joml/Matrix3f;", "scaling(FFF)Lorg/joml/Matrix3f;", "set(IIF)Lorg/joml/Matrix3f;",
            "mul(Lorg/joml/Matrix3fc;)Lorg/joml/Matrix3f;", "mul(Lorg/joml/Matrix3fc;Lorg/joml/Matrix3f;)Lorg/joml/Matrix3f;",
            "invert()Lorg/joml/Matrix3f;", "invert(Lorg/joml/Matrix3f;)Lorg/joml/Matrix3f;",
            "transpose()Lorg/joml/Matrix3f;", "transpose(Lorg/joml/Matrix3f;)Lorg/joml/Matrix3f;",
            "scale(FFF)Lorg/joml/Matrix3f;", "scale(FFFLorg/joml/Matrix3f;)Lorg/joml/Matrix3f;", "scale(F)Lorg/joml/Matrix3f;",
            "rotateX(F)Lorg/joml/Matrix3f;", "rotateX(FLorg/joml/Matrix3f;)Lorg/joml/Matrix3f;",
            "rotateY(F)Lorg/joml/Matrix3f;", "rotateY(FLorg/joml/Matrix3f;)Lorg/joml/Matrix3f;",
            "rotateZ(F)Lorg/joml/Matrix3f;", "rotateZ(FLorg/joml/Matrix3f;)Lorg/joml/Matrix3f;",
            "rotate(FFFF)Lorg/joml/Matrix3f;", "rotate(FFFFLorg/joml/Matrix3f;)Lorg/joml/Matrix3f;",
            "rotate(FLorg/joml/Vector3fc;)Lorg/joml/Matrix3f;", "rotate(FLorg/joml/Vector3fc;Lorg/joml/Matrix3f;)Lorg/joml/Matrix3f;",
            "rotate(Lorg/joml/Quaternionfc;)Lorg/joml/Matrix3f;", "rotate(Lorg/joml/Quaternionfc;Lorg/joml/Matrix3f;)Lorg/joml/Matrix3f;",
            "transform(Lorg/joml/Vector3f;)Lorg/joml/Vector3f;", "transform(Lorg/joml/Vector3fc;Lorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            "transform(FFFLorg/joml/Vector3f;)Lorg/joml/Vector3f;"
        )
            // JOML writes the element with Unsafe, unchecked, so a column or row past 2 writes past the matrix.
            .where("set(IIF)Lorg/joml/Matrix3f;", args -> elementInMatrix3f((Integer)args[1], (Integer)args[2]))
            .extraInputs("set(IIF)Lorg/joml/Matrix3f;", matrix3fElements()),
        new InstanceSuite(
            Matrix4f.class,
            "<init>()V", "<init>(FFFFFFFFFFFFFFFF)V", "<init>(Lorg/joml/Matrix4fc;)V", "<init>(Lorg/joml/Matrix3fc;)V",
            "properties()I", "determineProperties()Lorg/joml/Matrix4f;", "identity()Lorg/joml/Matrix4f;",
            "set(Lorg/joml/Matrix4fc;)Lorg/joml/Matrix4f;", "set(FFFFFFFFFFFFFFFF)Lorg/joml/Matrix4f;",
            "mul(Lorg/joml/Matrix4fc;)Lorg/joml/Matrix4f;", "mul(Lorg/joml/Matrix4fc;Lorg/joml/Matrix4f;)Lorg/joml/Matrix4f;",
            "translation(FFF)Lorg/joml/Matrix4f;", "setTranslation(FFF)Lorg/joml/Matrix4f;",
            "translate(FFF)Lorg/joml/Matrix4f;", "translate(FFFLorg/joml/Matrix4f;)Lorg/joml/Matrix4f;",
            "scaling(FFF)Lorg/joml/Matrix4f;", "scale(FFF)Lorg/joml/Matrix4f;", "scale(FFFLorg/joml/Matrix4f;)Lorg/joml/Matrix4f;",
            "rotationX(F)Lorg/joml/Matrix4f;", "rotationY(F)Lorg/joml/Matrix4f;", "rotationZ(F)Lorg/joml/Matrix4f;",
            "rotationZYX(FFF)Lorg/joml/Matrix4f;",
            "rotateX(F)Lorg/joml/Matrix4f;", "rotateX(FLorg/joml/Matrix4f;)Lorg/joml/Matrix4f;",
            "rotateY(F)Lorg/joml/Matrix4f;", "rotateY(FLorg/joml/Matrix4f;)Lorg/joml/Matrix4f;",
            "rotateZ(F)Lorg/joml/Matrix4f;", "rotateZ(FLorg/joml/Matrix4f;)Lorg/joml/Matrix4f;",
            "rotationTowardsXY(FF)Lorg/joml/Matrix4f;", "rotateTowardsXY(FFLorg/joml/Matrix4f;)Lorg/joml/Matrix4f;",
            "rotation(FFFF)Lorg/joml/Matrix4f;", "rotation(FLorg/joml/Vector3fc;)Lorg/joml/Matrix4f;",
            "rotate(FFFF)Lorg/joml/Matrix4f;", "rotate(FFFFLorg/joml/Matrix4f;)Lorg/joml/Matrix4f;",
            "rotate(FLorg/joml/Vector3fc;)Lorg/joml/Matrix4f;", "rotate(FLorg/joml/Vector3fc;Lorg/joml/Matrix4f;)Lorg/joml/Matrix4f;",
            "rotation(Lorg/joml/Quaternionfc;)Lorg/joml/Matrix4f;",
            "rotate(Lorg/joml/Quaternionfc;)Lorg/joml/Matrix4f;", "rotate(Lorg/joml/Quaternionfc;Lorg/joml/Matrix4f;)Lorg/joml/Matrix4f;",
            "rotationAround(Lorg/joml/Quaternionfc;FFF)Lorg/joml/Matrix4f;",
            "rotateAround(Lorg/joml/Quaternionfc;FFF)Lorg/joml/Matrix4f;",
            "rotateAround(Lorg/joml/Quaternionfc;FFFLorg/joml/Matrix4f;)Lorg/joml/Matrix4f;",
            "invert()Lorg/joml/Matrix4f;", "invert(Lorg/joml/Matrix4f;)Lorg/joml/Matrix4f;",
            "setPerspective(FFFF)Lorg/joml/Matrix4f;", "setPerspective(FFFFZ)Lorg/joml/Matrix4f;",
            "perspective(FFFF)Lorg/joml/Matrix4f;", "perspective(FFFFZ)Lorg/joml/Matrix4f;",
            "perspective(FFFFLorg/joml/Matrix4f;)Lorg/joml/Matrix4f;", "perspective(FFFFZLorg/joml/Matrix4f;)Lorg/joml/Matrix4f;",
            "setOrtho(FFFFFF)Lorg/joml/Matrix4f;", "setOrtho(FFFFFFZ)Lorg/joml/Matrix4f;",
            "setOrtho2D(FFFF)Lorg/joml/Matrix4f;", "ortho2D(FFFF)Lorg/joml/Matrix4f;", "ortho2D(FFFFLorg/joml/Matrix4f;)Lorg/joml/Matrix4f;",
            "assume(I)Lorg/joml/Matrix4f;",
            "transformProject(Lorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            "transformProject(Lorg/joml/Vector3fc;Lorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            "transformProject(FFFLorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            "transformPosition(Lorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            "transformPosition(Lorg/joml/Vector3fc;Lorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            "transformPosition(FFFLorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            "transformDirection(Lorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            "transformDirection(Lorg/joml/Vector3fc;Lorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            "transformDirection(FFFLorg/joml/Vector3f;)Lorg/joml/Vector3f;",
            "transform(Lorg/joml/Vector4f;)Lorg/joml/Vector4f;", "transform(Lorg/joml/Vector4fc;Lorg/joml/Vector4f;)Lorg/joml/Vector4f;",
            "transformTranspose(Lorg/joml/Vector4f;)Lorg/joml/Vector4f;",
            "transformTranspose(Lorg/joml/Vector4fc;Lorg/joml/Vector4f;)Lorg/joml/Vector4f;",
            "mul(Lorg/joml/Matrix3x2fc;)Lorg/joml/Matrix4f;", "mul(Lorg/joml/Matrix3x2fc;Lorg/joml/Matrix4f;)Lorg/joml/Matrix4f;",
            "invertAffine()Lorg/joml/Matrix4f;", "invertAffine(Lorg/joml/Matrix4f;)Lorg/joml/Matrix4f;", "isFinite()Z",
            "translation(Lorg/joml/Vector3fc;)Lorg/joml/Matrix4f;", "scale(Lorg/joml/Vector3fc;)Lorg/joml/Matrix4f;",
            "getTranslation(Lorg/joml/Vector3f;)Lorg/joml/Vector3f;", "equals(Ljava/lang/Object;)Z", "hashCode()I"
        )
            .extraInputs("equals(Ljava/lang/Object;)Z", matrixEqualsInputs()),
        new InstanceSuite(
            Vector4f.class,
            "<init>()V", "<init>(F)V", "<init>(FFFF)V", "<init>(Lorg/joml/Vector3fc;F)V", "<init>(Lorg/joml/Vector4fc;)V",
            "set(FFFF)Lorg/joml/Vector4f;", "set(Lorg/joml/Vector4fc;)Lorg/joml/Vector4f;", "set(Lorg/joml/Vector3fc;F)Lorg/joml/Vector4f;",
            "mul(F)Lorg/joml/Vector4f;", "mul(FLorg/joml/Vector4f;)Lorg/joml/Vector4f;",
            "mul(Lorg/joml/Vector4fc;)Lorg/joml/Vector4f;", "mul(Lorg/joml/Vector4fc;Lorg/joml/Vector4f;)Lorg/joml/Vector4f;",
            "mul(Lorg/joml/Matrix4fc;)Lorg/joml/Vector4f;", "mul(Lorg/joml/Matrix4fc;Lorg/joml/Vector4f;)Lorg/joml/Vector4f;",
            "mulTranspose(Lorg/joml/Matrix4fc;)Lorg/joml/Vector4f;", "mulTranspose(Lorg/joml/Matrix4fc;Lorg/joml/Vector4f;)Lorg/joml/Vector4f;",
            "div(F)Lorg/joml/Vector4f;", "div(FLorg/joml/Vector4f;)Lorg/joml/Vector4f;",
            "lerp(Lorg/joml/Vector4fc;F)Lorg/joml/Vector4f;", "lerp(Lorg/joml/Vector4fc;FLorg/joml/Vector4f;)Lorg/joml/Vector4f;"
        ),
        new Matrix4fStackSuite(),
        new InstanceSuite(
            Vector2f.class,
            "<init>()V", "<init>(FF)V", "set(FF)Lorg/joml/Vector2f;", "zero()Lorg/joml/Vector2f;",
            "add(FF)Lorg/joml/Vector2f;", "add(FFLorg/joml/Vector2f;)Lorg/joml/Vector2f;",
            "mul(FF)Lorg/joml/Vector2f;", "mul(FFLorg/joml/Vector2f;)Lorg/joml/Vector2f;"
        ),
        new InstanceSuite(
            Matrix3x2f.class,
            "<init>()V", "<init>(Lorg/joml/Matrix3x2fc;)V", "set(Lorg/joml/Matrix3x2fc;)Lorg/joml/Matrix3x2f;",
            "translate(FF)Lorg/joml/Matrix3x2f;", "translate(FFLorg/joml/Matrix3x2f;)Lorg/joml/Matrix3x2f;",
            "scale(FF)Lorg/joml/Matrix3x2f;", "scale(FFLorg/joml/Matrix3x2f;)Lorg/joml/Matrix3x2f;",
            "scale(F)Lorg/joml/Matrix3x2f;", "scale(FLorg/joml/Matrix3x2f;)Lorg/joml/Matrix3x2f;",
            "rotate(F)Lorg/joml/Matrix3x2f;", "rotate(FLorg/joml/Matrix3x2f;)Lorg/joml/Matrix3x2f;",
            "invert(Lorg/joml/Matrix3x2f;)Lorg/joml/Matrix3x2f;",
            "transformPosition(Lorg/joml/Vector2f;)Lorg/joml/Vector2f;",
            "transformPosition(FFLorg/joml/Vector2f;)Lorg/joml/Vector2f;"
        ),
        new Matrix3x2fStackSuite(),
        new InstanceSuite(
            FrustumIntersection.class,
            "<init>()V", "set(Lorg/joml/Matrix4fc;)Lorg/joml/FrustumIntersection;",
            "set(Lorg/joml/Matrix4fc;Z)Lorg/joml/FrustumIntersection;", "intersectAab(FFFFFF)I", "testPoint(FFF)Z"
        )
            .withFields()
            .extraInputs("intersectAab(FFFFFF)I", frustumBoxes())
            .extraInputs("testPoint(FFF)Z", frustumPoints()),
        new InstanceSuite(
            Axis.class,
            "rotation(F)Lorg/joml/Quaternionf;", "rotationDegrees(F)Lorg/joml/Quaternionf;",
            "rotate(Lorg/joml/Matrix3f;F)Lorg/joml/Matrix3f;", "rotateDegrees(Lorg/joml/Matrix3f;F)Lorg/joml/Matrix3f;",
            "rotate(Lorg/joml/Matrix4f;F)Lorg/joml/Matrix4f;", "rotateDegrees(Lorg/joml/Matrix4f;F)Lorg/joml/Matrix4f;"
        ),
        new InstanceSuite(
            GivensParameters.class,
            "<init>(FF)V", "fromUnnormalized(FF)Lcom/mojang/math/GivensParameters;", "fromPositiveAngle(F)Lcom/mojang/math/GivensParameters;",
            "sinHalf()F", "cosHalf()F", "inverse()Lcom/mojang/math/GivensParameters;", "cos()F", "sin()F",
            "aroundX(Lorg/joml/Quaternionf;)Lorg/joml/Quaternionf;", "aroundY(Lorg/joml/Quaternionf;)Lorg/joml/Quaternionf;",
            "aroundZ(Lorg/joml/Quaternionf;)Lorg/joml/Quaternionf;", "aroundX(Lorg/joml/Matrix3f;)Lorg/joml/Matrix3f;",
            "aroundY(Lorg/joml/Matrix3f;)Lorg/joml/Matrix3f;", "aroundZ(Lorg/joml/Matrix3f;)Lorg/joml/Matrix3f;"
        ),
        new StaticSuite(MatrixUtil.class)
            .extraInputs("eigenvalueJacobi(Lorg/joml/Matrix3f;ILorg/joml/Quaternionf;)V", jacobiInputs())
            .extraInputs(
                "svdDecompose(Lorg/joml/Matrix4fc;Lorg/joml/Vector3f;Lorg/joml/Quaternionf;Lorg/joml/Vector3f;Lorg/joml/Quaternionf;)V",
                svdInputs()
            )
            // Each step costs about two matrix products, so billions of them would take minutes. The game takes 5.
            .where("eigenvalueJacobi(Lorg/joml/Matrix3f;ILorg/joml/Quaternionf;)V", args -> (Integer)args[1] <= 1024),
        new InstanceSuite(
            Transformation.class,
            "<init>(Lorg/joml/Matrix4fc;)V", "<init>(Lorg/joml/Vector3fc;Lorg/joml/Quaternionfc;Lorg/joml/Vector3fc;Lorg/joml/Quaternionfc;)V",
            "compose(Lcom/mojang/math/Transformation;)Lcom/mojang/math/Transformation;", "inverse()Lcom/mojang/math/Transformation;",
            "getMatrix()Lorg/joml/Matrix4fc;", "getMatrixCopy()Lorg/joml/Matrix4f;", "translation()Lorg/joml/Vector3fc;",
            "leftRotation()Lorg/joml/Quaternionfc;", "scale()Lorg/joml/Vector3fc;", "rightRotation()Lorg/joml/Quaternionfc;",
            "equals(Ljava/lang/Object;)Z", "hashCode()I", "slerp(Lcom/mojang/math/Transformation;F)Lcom/mojang/math/Transformation;",
            "compose(Lorg/joml/Matrix4fc;Ljava/util/Optional;)Lorg/joml/Matrix4fc;"
        )
            .withFields()
            .extraInputs(
                "<init>(Lorg/joml/Vector3fc;Lorg/joml/Quaternionfc;Lorg/joml/Vector3fc;Lorg/joml/Quaternionfc;)V", transformationPartsInputs()
            )
            .extraInputs("equals(Ljava/lang/Object;)Z", transformationEqualsInputs())
            .extraInputs("slerp(Lcom/mojang/math/Transformation;F)Lcom/mojang/math/Transformation;", transformationSlerpInputs()),
        new PoseStackSuite(),
        new ProjectionSuite(),
        new StaticSuite(Float.class)
            .only("floatToIntBits(F)I", "hashCode(F)I", "toString(F)Ljava/lang/String;")
            .extraInputs("floatToIntBits(F)I", floatNaNs())
            .extraInputs("hashCode(F)I", floatNaNs())
            .extraInputs("toString(F)Ljava/lang/String;", floatToStringInputs()),
        new StaticSuite(Double.class)
            .only("doubleToLongBits(D)J", "hashCode(D)I", "toString(D)Ljava/lang/String;")
            .extraInputs("doubleToLongBits(D)J", doubleNaNs())
            .extraInputs("hashCode(D)I", doubleNaNs())
            .extraInputs("toString(D)Ljava/lang/String;", doubleToStringInputs()),
        // What IntegerProperty.getValue parses a value's name with, which takes any Unicode decimal digit.
        new StaticSuite(Integer.class).only("parseInt(Ljava/lang/String;)I").extraInputs("parseInt(Ljava/lang/String;)I", intStrings()),
        new StaticSuite(Character.class)
            .only("digit(CI)I")
            .where("digit(CI)I", args -> (Integer)args[1] == 10)
            .extraInputs("digit(CI)I", decimalDigits()),
        new InstanceSuite(Vector3i.class, "<init>(III)V", "set(III)Lorg/joml/Vector3i;", "get(I)I", "setComponent(II)Lorg/joml/Vector3i;"),
        InstanceSuite.every(Vec3i.class)
            .extraInputs("closerThan(Lnet/minecraft/core/Vec3i;D)Z", distanceTies(0))
            .extraInputs("closerToCenterThan(Lnet/minecraft/core/Position;D)Z", distanceTies(1)),
        InstanceSuite.every(Direction.class)
            .extraInputs("byName(Ljava/lang/String;)Lnet/minecraft/core/Direction;", names(Direction.values()))
            .extraInputs("valueOf(Ljava/lang/String;)Lnet/minecraft/core/Direction;", names(Direction.values()))
            .extraInputs("getNearest(IIILnet/minecraft/core/Direction;)Lnet/minecraft/core/Direction;", nearestInputs(false))
            .extraInputs("getNearest(Lnet/minecraft/core/Vec3i;Lnet/minecraft/core/Direction;)Lnet/minecraft/core/Direction;", nearestInputs(true))
            .extraInputs("getApproximateNearest(FFF)Lnet/minecraft/core/Direction;", approximateNearestInputs(0))
            .extraInputs("getApproximateNearest(DDD)Lnet/minecraft/core/Direction;", approximateNearestInputs(1))
            .extraInputs("getApproximateNearest(Lnet/minecraft/world/phys/Vec3;)Lnet/minecraft/core/Direction;", approximateNearestInputs(2))
            .extraInputs("fromYRot(D)Lnet/minecraft/core/Direction;", yRotBoundaries())
            .extraInputs("isFacingAngle(F)Z", facingAngles())
            .extraInputs("rotate(Lorg/joml/Matrix4fc;Lnet/minecraft/core/Direction;)Lnet/minecraft/core/Direction;", modelRotations())
            .extraInputs("axisStepOrder(Lnet/minecraft/world/phys/Vec3;)Lcom/google/common/collect/ImmutableList;", axisStepOrderTies()),
        InstanceSuite.every(Direction.Axis.class)
            .extraInputs("byName(Ljava/lang/String;)Lnet/minecraft/core/Direction$Axis;", names(Direction.Axis.values()))
            .extraInputs("valueOf(Ljava/lang/String;)Lnet/minecraft/core/Direction$Axis;", names(Direction.Axis.values()))
            .extraInputs("test(Lnet/minecraft/core/Direction;)Z", nullDirections(Direction.Axis.values())),
        InstanceSuite.every(Direction.AxisDirection.class)
            .extraInputs(
                "valueOf(Ljava/lang/String;)Lnet/minecraft/core/Direction$AxisDirection;", names(Direction.AxisDirection.values())
            ),
        InstanceSuite.every(Direction.Plane.class)
            .extraInputs("valueOf(Ljava/lang/String;)Lnet/minecraft/core/Direction$Plane;", names(Direction.Plane.values()))
            .extraInputs("test(Lnet/minecraft/core/Direction;)Z", nullDirections(Direction.Plane.values())),
        InstanceSuite.every(Vec3.class)
            .extraInputs("closerThan(Lnet/minecraft/core/Position;D)Z", distanceTies(2))
            .extraInputs("closerThan(Lnet/minecraft/world/phys/Vec3;DD)Z", distanceTies(3))
            .extraInputs("normalize()Lnet/minecraft/world/phys/Vec3;", normalizeThresholds())
            .extraInputs("directionFromRotation(FF)Lnet/minecraft/world/phys/Vec3;", lookRotations()),
        InstanceSuite.every(Vec2.class)
            .extraInputs("normalized()Lnet/minecraft/world/phys/Vec2;", normalizedThresholds()),
        InstanceSuite.every(AxisCycle.class)
            .extraInputs("valueOf(Ljava/lang/String;)Lnet/minecraft/core/AxisCycle;", names(AxisCycle.values())),
        InstanceSuite.every(Rotation.class)
            .extraInputs("valueOf(Ljava/lang/String;)Lnet/minecraft/world/level/block/Rotation;", names(Rotation.values()))
            .extraInputs("rotate(II)I", rotationSteps()),
        InstanceSuite.every(BlockPos.class)
            .inherited("toString()Ljava/lang/String;")
            .extraInputs("asLong(III)J", packingLimits())
            .extraInputs("of(J)Lnet/minecraft/core/BlockPos;", packedInputs(0))
            .extraInputs("getX(J)I", packedInputs(0))
            .extraInputs("getY(J)I", packedInputs(0))
            .extraInputs("getZ(J)I", packedInputs(0))
            .extraInputs("getFlatIndex(J)J", packedInputs(0))
            .extraInputs("offset(JLnet/minecraft/core/Direction;)J", packedInputs(1))
            .extraInputs("offset(JIII)J", packedInputs(2))
            .extraInputs("clampLocationWithin(Lnet/minecraft/world/phys/Vec3;)Lnet/minecraft/world/phys/Vec3;", clampBounds())
            .extraInputs("withinManhattan(Lnet/minecraft/core/BlockPos;I)Ljava/lang/Iterable;", manhattanReaches(1))
            .extraInputs("withinClippedManhattan(Lnet/minecraft/core/BlockPos;III)Ljava/lang/Iterable;", manhattanReaches(3))
            .extraInputs("withinBoxByManhattanDistance(Lnet/minecraft/core/BlockPos;III)Ljava/lang/Iterable;", manhattanReaches(3))
            // With a negative reach and a large depth, the search never finds a block, so it runs until the depth wraps.
            .where("withinClippedManhattan(Lnet/minecraft/core/BlockPos;III)Ljava/lang/Iterable;", args -> manhattanEnds(args, Math.max(Math.max((Integer)args[1], (Integer)args[2]), (Integer)args[3])))
            .where("withinBoxByManhattanDistance(Lnet/minecraft/core/BlockPos;III)Ljava/lang/Iterable;", args -> manhattanEnds(args, (Integer)args[1] + (Integer)args[2] + (Integer)args[3]))
            .extraInputs("betweenClosed(IIIIII)Ljava/lang/Iterable;", smallBoxes(false))
            .extraInputs("betweenClosedStream(IIIIII)Ljava/util/stream/Stream;", smallBoxes(false))
            .extraInputs("betweenClosed(Lnet/minecraft/core/BlockPos;Lnet/minecraft/core/BlockPos;)Ljava/lang/Iterable;", smallBoxes(true))
            .extraInputs("betweenClosedStream(Lnet/minecraft/core/BlockPos;Lnet/minecraft/core/BlockPos;)Ljava/util/stream/Stream;", smallBoxes(true))
            .extraInputs("neighborColumn(IIII)Ljava/lang/Iterable;", neighborColumns())
            .extraInputs("spiralAround(Lnet/minecraft/core/BlockPos;ILnet/minecraft/core/Direction;Lnet/minecraft/core/Direction;)Ljava/lang/Iterable;", spirals())
            .extraInputs("betweenCornersInDirection(Lnet/minecraft/core/BlockPos;Lnet/minecraft/core/BlockPos;Lnet/minecraft/world/phys/Vec3;)Ljava/lang/Iterable;", cornerWalks(true))
            .extraInputs("betweenCornersInDirection(IIIIIILnet/minecraft/world/phys/Vec3;)Ljava/lang/Iterable;", cornerWalks(false))
            .extraInputs("randomBetweenClosed(Lnet/minecraft/util/RandomSource;IIIIIII)Ljava/lang/Iterable;", randomBoxes(false))
            .extraInputs("randomInCube(Lnet/minecraft/util/RandomSource;ILnet/minecraft/core/BlockPos;I)Ljava/lang/Iterable;", randomBoxes(true))
            .extraInputs("betweenClosed(Lnet/minecraft/world/phys/AABB;)Ljava/lang/Iterable;", blockAlignedBoxes(0))
            .extraInputs("betweenClosedStream(Lnet/minecraft/world/phys/AABB;)Ljava/util/stream/Stream;", blockAlignedBoxes(0))
            .extraInputs("betweenCornersInDirection(Lnet/minecraft/world/phys/AABB;Lnet/minecraft/world/phys/Vec3;)Ljava/lang/Iterable;", blockAlignedBoxes(1)),
        InstanceSuite.every(BlockPos.MutableBlockPos.class)
            .inherited("toString()Ljava/lang/String;")
            .extraInputs("set(J)Lnet/minecraft/core/BlockPos$MutableBlockPos;", packedInputs(3)),
        InstanceSuite.every(BlockPos.TraversalNodeStatus.class)
            .extraInputs("valueOf(Ljava/lang/String;)Lnet/minecraft/core/BlockPos$TraversalNodeStatus;", names(BlockPos.TraversalNodeStatus.values())),
        InstanceSuite.every(AABB.class)
            .extraInputs("encapsulatingFullBlocks(Lnet/minecraft/core/BlockPos;Lnet/minecraft/core/BlockPos;)Lnet/minecraft/world/phys/AABB;", limitBlockPairs())
            .extraInputs("contract(DDD)Lnet/minecraft/world/phys/AABB;", signedZeroMoves(false))
            .extraInputs("expandTowards(DDD)Lnet/minecraft/world/phys/AABB;", signedZeroMoves(false))
            .extraInputs("expandTowards(Lnet/minecraft/world/phys/Vec3;)Lnet/minecraft/world/phys/AABB;", signedZeroMoves(true))
            .extraInputs("equals(Ljava/lang/Object;)Z", boxEqualsInputs())
            .extraInputs("intersects(Lnet/minecraft/world/phys/AABB;)Z", touchingBoxes(0))
            .extraInputs("intersect(Lnet/minecraft/world/phys/AABB;)Lnet/minecraft/world/phys/AABB;", touchingBoxes(0))
            .extraInputs("minmax(Lnet/minecraft/world/phys/AABB;)Lnet/minecraft/world/phys/AABB;", touchingBoxes(0))
            .extraInputs("distanceToSqr(Lnet/minecraft/world/phys/AABB;)D", touchingBoxes(0))
            .extraInputs("intersects(DDDDDD)Z", touchingBoxes(1))
            .extraInputs("intersects(Lnet/minecraft/world/phys/Vec3;Lnet/minecraft/world/phys/Vec3;)Z", touchingBoxes(2))
            .extraInputs("intersects(Lnet/minecraft/core/BlockPos;)Z", touchingBoxes(3))
            .extraInputs("contains(Lnet/minecraft/world/phys/Vec3;)Z", facePoints(0))
            .extraInputs("contains(DDD)Z", facePoints(1))
            .extraInputs("distanceToSqr(Lnet/minecraft/world/phys/Vec3;)D", facePoints(0))
            .extraInputs("clip(Lnet/minecraft/world/phys/Vec3;Lnet/minecraft/world/phys/Vec3;)Ljava/util/Optional;", clipRays(0))
            .extraInputs("clip(DDDDDDLnet/minecraft/world/phys/Vec3;Lnet/minecraft/world/phys/Vec3;)Ljava/util/Optional;", clipRays(1))
            .extraInputs(
                "clip(Ljava/lang/Iterable;Lnet/minecraft/world/phys/Vec3;Lnet/minecraft/world/phys/Vec3;Lnet/minecraft/core/BlockPos;)Lnet/minecraft/world/phys/BlockHitResult;",
                clipRays(2)
            )
            .extraInputs("collidedAlongVector(Lnet/minecraft/world/phys/Vec3;Ljava/util/List;)Z", collisionPaths()),
        InstanceSuite.every(AABB.Builder.class),
        // A BlockHitResult is the only HitResult so far: an EntityHitResult needs entities.
        InstanceSuite.every(HitResult.class),
        InstanceSuite.every(HitResult.Type.class)
            .extraInputs("valueOf(Ljava/lang/String;)Lnet/minecraft/world/phys/HitResult$Type;", names(HitResult.Type.values())),
        InstanceSuite.every(BlockHitResult.class),
        InstanceSuite.every(Identifier.class)
            .extraInputs("fromNamespaceAndPath(Ljava/lang/String;Ljava/lang/String;)Lnet/minecraft/resources/Identifier;", identifierParts())
            .extraInputs("tryBuild(Ljava/lang/String;Ljava/lang/String;)Lnet/minecraft/resources/Identifier;", identifierParts())
            .extraInputs("parse(Ljava/lang/String;)Lnet/minecraft/resources/Identifier;", identifierStrings())
            .extraInputs("tryParse(Ljava/lang/String;)Lnet/minecraft/resources/Identifier;", identifierStrings())
            .extraInputs("withDefaultNamespace(Ljava/lang/String;)Lnet/minecraft/resources/Identifier;", identifierStrings())
            .extraInputs("isValidPath(Ljava/lang/String;)Z", identifierStrings())
            .extraInputs("isValidNamespace(Ljava/lang/String;)Z", identifierStrings())
            .extraInputs("bySeparator(Ljava/lang/String;C)Lnet/minecraft/resources/Identifier;", separatedStrings())
            .extraInputs("tryBySeparator(Ljava/lang/String;C)Lnet/minecraft/resources/Identifier;", separatedStrings())
            .extraInputs("withPath(Ljava/lang/String;)Lnet/minecraft/resources/Identifier;", identifierAffixes())
            .extraInputs("withPrefix(Ljava/lang/String;)Lnet/minecraft/resources/Identifier;", identifierAffixes())
            .extraInputs("withSuffix(Ljava/lang/String;)Lnet/minecraft/resources/Identifier;", identifierAffixes())
            .extraInputs("equals(Ljava/lang/Object;)Z", identifierPairs())
            .extraInputs("compareTo(Lnet/minecraft/resources/Identifier;)I", identifierPairs())
            .extraInputs("isAllowedInIdentifier(C)Z", everyChar())
            .extraInputs("validPathChar(C)Z", everyChar()),
        new StaticSuite(VarInt.class)
            .extraInputs("getByteSize(I)I", varIntBoundaries(false))
            .extraInputs("write(Lio/netty/buffer/ByteBuf;I)Lio/netty/buffer/ByteBuf;", varIntBoundaries(true))
            .extraInputs("read(Lio/netty/buffer/ByteBuf;)I", varIntEncodings(false)),
        new StaticSuite(VarLong.class)
            .extraInputs("getByteSize(J)I", varLongBoundaries(false))
            .extraInputs("write(Lio/netty/buffer/ByteBuf;J)Lio/netty/buffer/ByteBuf;", varLongBoundaries(true))
            .extraInputs("read(Lio/netty/buffer/ByteBuf;)J", varIntEncodings(true)),
        new StaticSuite(Utf8String.class)
            .extraInputs("read(Lio/netty/buffer/ByteBuf;I)Ljava/lang/String;", utfReadLimits())
            .extraInputs("write(Lio/netty/buffer/ByteBuf;Ljava/lang/CharSequence;I)V", utfWriteLimits()),
        // Netty's buffer, as Unpooled.buffer makes it: the methods the client calls on a ByteBuf.
        new InstanceSuite(
            ByteBuf.class,
            "capacity()I", "maxCapacity()I", "readerIndex()I", "readerIndex(I)Lio/netty/buffer/ByteBuf;", "writerIndex()I",
            "writerIndex(I)Lio/netty/buffer/ByteBuf;", "setIndex(II)Lio/netty/buffer/ByteBuf;", "clear()Lio/netty/buffer/ByteBuf;",
            "isReadable()Z", "isReadable(I)Z", "isWritable()Z", "isWritable(I)Z", "readableBytes()I", "writableBytes()I",
            "maxWritableBytes()I", "markReaderIndex()Lio/netty/buffer/ByteBuf;", "resetReaderIndex()Lio/netty/buffer/ByteBuf;",
            "markWriterIndex()Lio/netty/buffer/ByteBuf;", "resetWriterIndex()Lio/netty/buffer/ByteBuf;",
            "ensureWritable(I)Lio/netty/buffer/ByteBuf;",
            "readBoolean()Z", "readByte()B", "readUnsignedByte()S", "readShort()S", "readUnsignedShort()I", "readInt()I",
            "readUnsignedInt()J", "readLong()J", "readChar()C", "readFloat()F", "readDouble()D",
            "readBytes(I)Lio/netty/buffer/ByteBuf;", "readBytes([B)Lio/netty/buffer/ByteBuf;", "readBytes([BII)Lio/netty/buffer/ByteBuf;",
            "skipBytes(I)Lio/netty/buffer/ByteBuf;",
            "writeBoolean(Z)Lio/netty/buffer/ByteBuf;", "writeByte(I)Lio/netty/buffer/ByteBuf;", "writeShort(I)Lio/netty/buffer/ByteBuf;",
            "writeInt(I)Lio/netty/buffer/ByteBuf;", "writeLong(J)Lio/netty/buffer/ByteBuf;", "writeChar(I)Lio/netty/buffer/ByteBuf;",
            "writeFloat(F)Lio/netty/buffer/ByteBuf;", "writeDouble(D)Lio/netty/buffer/ByteBuf;",
            "writeBytes([B)Lio/netty/buffer/ByteBuf;", "writeBytes([BII)Lio/netty/buffer/ByteBuf;",
            "writeBytes(Lio/netty/buffer/ByteBuf;)Lio/netty/buffer/ByteBuf;", "writeBytes(Lio/netty/buffer/ByteBuf;II)Lio/netty/buffer/ByteBuf;",
            "toString(IILjava/nio/charset/Charset;)Ljava/lang/String;", "slice(II)Lio/netty/buffer/ByteBuf;"
        )
            .extraInputs("readerIndex(I)Lio/netty/buffer/ByteBuf;", bufferIndexes(1))
            .extraInputs("writerIndex(I)Lio/netty/buffer/ByteBuf;", bufferIndexes(1))
            .extraInputs("setIndex(II)Lio/netty/buffer/ByteBuf;", bufferIndexes(2))
            .extraInputs("isReadable(I)Z", bufferIndexes(1))
            .extraInputs("isWritable(I)Z", bufferIndexes(1))
            .extraInputs("ensureWritable(I)Lio/netty/buffer/ByteBuf;", bufferIndexes(1))
            .extraInputs("readBytes(I)Lio/netty/buffer/ByteBuf;", bufferIndexes(1))
            .extraInputs("skipBytes(I)Lio/netty/buffer/ByteBuf;", bufferIndexes(1))
            .extraInputs("toString(IILjava/nio/charset/Charset;)Ljava/lang/String;", bufferRanges())
            .extraInputs("slice(II)Lio/netty/buffer/ByteBuf;", sliceRanges())
            .extraInputs("writeFloat(F)Lio/netty/buffer/ByteBuf;", withBuffers(firsts(floatNaNs()), true))
            .extraInputs("writeDouble(D)Lio/netty/buffer/ByteBuf;", withBuffers(firsts(doubleNaNs()), true))
            .extraInputs("readBytes([BII)Lio/netty/buffer/ByteBuf;", arrayRanges())
            .extraInputs("writeBytes([BII)Lio/netty/buffer/ByteBuf;", arrayRanges())
            .extraInputs("writeBytes(Lio/netty/buffer/ByteBuf;II)Lio/netty/buffer/ByteBuf;", sourceRanges())
            // Vanilla grows the buffer before it finds the source too short, so a length near 2^30 allocates a gigabyte.
            .where("ensureWritable(I)Lio/netty/buffer/ByteBuf;", args -> growsLittle((ByteBuf)args[0], (Integer)args[1]))
            .where("writeBytes([BII)Lio/netty/buffer/ByteBuf;", args -> growsLittle((ByteBuf)args[0], (Integer)args[3]))
            .where("writeBytes(Lio/netty/buffer/ByteBuf;II)Lio/netty/buffer/ByteBuf;", args -> growsLittle((ByteBuf)args[0], (Integer)args[3])),
        // The methods FriendlyByteBuf adds. The ones overriding ByteBuf's forward to the buffer it wraps.
        InstanceSuite.every(FriendlyByteBuf.class)
            .withoutOverridesOf(ByteBuf.class)
            .extraInputs("writeVarInt(I)Lnet/minecraft/network/FriendlyByteBuf;", friendly(varIntBoundaries(true)))
            .extraInputs("writeVarLong(J)Lnet/minecraft/network/FriendlyByteBuf;", friendly(varLongBoundaries(true)))
            .extraInputs("readVarInt()I", friendly(varIntEncodings(false)))
            .extraInputs("readVarLong()J", friendly(varIntEncodings(true)))
            .extraInputs("readUtf()Ljava/lang/String;", friendly(longStrings(false)))
            .extraInputs("writeUtf(Ljava/lang/String;)Lnet/minecraft/network/FriendlyByteBuf;", friendly(longStrings(true)))
            .extraInputs("readUtf(I)Ljava/lang/String;", friendly(utfReadLimits()))
            .extraInputs("writeUtf(Ljava/lang/String;I)Lnet/minecraft/network/FriendlyByteBuf;", friendly(utfWriteLimits()))
            .extraInputs("readIdentifier()Lnet/minecraft/resources/Identifier;", friendly(identifierBuffers()))
            .extraInputs("readByteArray()[B", friendly(byteArrays(false)))
            .extraInputs("readByteArray(I)[B", friendly(byteArrays(true)))
            .extraInputs("readByteArray(Lio/netty/buffer/ByteBuf;)[B", byteArrays(false))
            .extraInputs("readByteArray(Lio/netty/buffer/ByteBuf;I)[B", byteArrays(true))
            .extraInputs("readVarIntArray()[I", friendly(varIntArrays(false)))
            .extraInputs("readVarIntArray(I)[I", friendly(varIntArrays(true)))
            .extraInputs("readLongArray()[J", friendly(longArrays()))
            .extraInputs("readLongArray(Lio/netty/buffer/ByteBuf;)[J", longArrays())
            .extraInputs("readFixedSizeLongArray([J)[J", friendly(fixedLongArrays()))
            .extraInputs("readFixedSizeLongArray(Lio/netty/buffer/ByteBuf;[J)[J", fixedLongArrays())
            // A length prefix in the millions makes vanilla allocate that many bytes before it finds the buffer shorter.
            .where("readByteArray(I)[B", args -> allocatesLittle(args[0], (Integer)args[1]))
            .where("readByteArray(Lio/netty/buffer/ByteBuf;I)[B", args -> allocatesLittle(args[0], (Integer)args[1]))
            .where("readVarIntArray(I)[I", args -> allocatesLittle(args[0], (Integer)args[1]))
            // A fixed-size bit set's reader allocates its size over 8 bytes before it reads, and its writer writes that many.
            .where("readFixedBitSet(I)Ljava/util/BitSet;", args -> (Integer)args[1] <= 1 << 16)
            .where("readFixedBitSet(Lio/netty/buffer/ByteBuf;I)Ljava/util/BitSet;", args -> (Integer)args[1] <= 1 << 16)
            .where("writeFixedBitSet(Ljava/util/BitSet;I)V", args -> (Integer)args[2] <= 1 << 16)
            .where("writeFixedBitSet(Lio/netty/buffer/ByteBuf;Ljava/util/BitSet;I)V", args -> (Integer)args[2] <= 1 << 16)
            // The readers and writers taking a decoder, an encoder or a function get the game's kinds of them, and the
            // readers also read what the writers write.
            .given(READ_OPTIONAL, "VAR_INT", ByteBufCodecs.VAR_INT)
            .extraInputs(
                READ_OPTIONAL + "[VAR_INT]",
                friendly(written(91, Codecs.optionalOf(INTS), (FriendlyByteBuf buffer, Optional<Integer> value) -> buffer.writeOptional(value, ByteBufCodecs.VAR_INT)))
            )
            .given(READ_OPTIONAL, "Vec3's doubles", (StreamDecoder<FriendlyByteBuf, Vec3>)i -> new Vec3(i.readDouble(), i.readDouble(), i.readDouble()))
            .extraInputs(
                READ_OPTIONAL + "[Vec3's doubles]",
                friendly(written(92, Codecs.optionalOf(VEC3S), (FriendlyByteBuf buffer, Optional<Vec3> value) -> buffer.writeOptional(value, Vec3.STREAM_CODEC)))
            )
            .given(WRITE_OPTIONAL, "VAR_INT", Codecs.optionalOf(INTS), ByteBufCodecs.VAR_INT)
            .given(WRITE_OPTIONAL, "Vec3's doubles", Codecs.optionalOf(VEC3S), (StreamEncoder<FriendlyByteBuf, Vec3>)(o, pos) -> {
                o.writeDouble(pos.x());
                o.writeDouble(pos.y());
                o.writeDouble(pos.z());
            })
            .given(READ_NULLABLE, "readIdentifier", (StreamDecoder<FriendlyByteBuf, Identifier>)FriendlyByteBuf::readIdentifier)
            .extraInputs(
                READ_NULLABLE + "[readIdentifier]",
                friendly(written(93, Codecs.nullableOf(IDENTIFIERS), (FriendlyByteBuf buffer, Identifier value) -> buffer.writeNullable(value, FriendlyByteBuf::writeIdentifier)))
            )
            .given(READ_NULLABLE, "readUtf", (StreamDecoder<FriendlyByteBuf, String>)FriendlyByteBuf::readUtf)
            .extraInputs(
                READ_NULLABLE + "[readUtf]",
                friendly(written(94, Codecs.nullableOf(STRINGS), (FriendlyByteBuf buffer, String value) -> buffer.writeNullable(value, FriendlyByteBuf::writeUtf)))
            )
            .given(WRITE_NULLABLE, "writeIdentifier", Codecs.nullableOf(IDENTIFIERS), (StreamEncoder<FriendlyByteBuf, Identifier>)FriendlyByteBuf::writeIdentifier)
            .given(WRITE_NULLABLE, "writeUtf", Codecs.nullableOf(STRINGS), (StreamEncoder<FriendlyByteBuf, String>)FriendlyByteBuf::writeUtf)
            // A property's signature in GAME_PROFILE_PROPERTIES, at most 1024 units long. The reader also reads longer
            // ones, written without the limit.
            .given(READ_NULLABLE_FROM, "Utf8String.read(1024)", NetworkValues.BUFFER, (StreamDecoder<ByteBuf, String>)in -> Utf8String.read(in, 1024))
            .extraInputs(
                READ_NULLABLE_FROM + "[Utf8String.read(1024)]",
                written(95, Codecs.nullableOf(SIGNATURES), (FriendlyByteBuf buffer, String value) -> FriendlyByteBuf.writeNullable(buffer, value, (out, signature) -> Utf8String.write(out, signature, Integer.MAX_VALUE)))
            )
            .given(
                WRITE_NULLABLE_TO, "Utf8String.write(1024)", NetworkValues.BUFFER, Codecs.nullableOf(SIGNATURES),
                (StreamEncoder<ByteBuf, String>)(out, signature) -> Utf8String.write(out, signature, 1024)
            )
            .given(READ_BY_ID, "Direction.BY_ID", Direction.BY_ID)
            .extraInputs(READ_BY_ID + "[Direction.BY_ID]", friendly(written(96, IDS, FriendlyByteBuf::writeVarInt)))
            .given(READ_BY_ID, "Rotation.BY_ID", Rotation.BY_ID)
            .extraInputs(READ_BY_ID + "[Rotation.BY_ID]", friendly(written(97, IDS, FriendlyByteBuf::writeVarInt)))
            .given(WRITE_BY_ID, "Direction::get3DDataValue", (ToIntFunction<Direction>)Direction::get3DDataValue, DIRECTIONS)
            .given(WRITE_BY_ID, "Direction::get2DDataValue", (ToIntFunction<Direction>)Direction::get2DDataValue, DIRECTIONS)
            .given(READ_WITH_COUNT, "readVarInt", (Consumer<FriendlyByteBuf>)FriendlyByteBuf::readVarInt)
            .extraInputs(READ_WITH_COUNT + "[readVarInt]", friendly(written(98, Codecs.listOf(INTS), (FriendlyByteBuf buffer, List<Integer> values) -> {
                buffer.writeVarInt(values.size());
                values.forEach(buffer::writeVarInt);
            })))
            .given(READ_WITH_COUNT, "readUtf", (Consumer<FriendlyByteBuf>)FriendlyByteBuf::readUtf)
            .extraInputs(READ_WITH_COUNT + "[readUtf]", friendly(written(99, Codecs.listOf(STRINGS), (FriendlyByteBuf buffer, List<String> values) -> {
                buffer.writeVarInt(values.size());
                values.forEach(buffer::writeUtf);
            })))
            // A resource pack response's action, by its ordinal.
            .given(READ_ENUM, "ServerboundResourcePackPacket.Action", ServerboundResourcePackPacket.Action.class)
            .extraInputs(READ_ENUM + "[ServerboundResourcePackPacket.Action]", friendly(written(105, IDS, FriendlyByteBuf::writeVarInt)))
            .given(WRITE_ENUM, "ServerboundResourcePackPacket.Action", Values.generator(ServerboundResourcePackPacket.Action.class))
            // What writeNbt writes, which the readers also read.
            .extraInputs("readNbt()Lnet/minecraft/nbt/CompoundTag;", friendly(NbtSuites.nbtBuffers(214)))
            .extraInputs("readNbt(Lio/netty/buffer/ByteBuf;)Lnet/minecraft/nbt/CompoundTag;", NbtSuites.nbtBuffers(215))
            .extraInputs("readNbt(Lnet/minecraft/nbt/NbtAccounter;)Lnet/minecraft/nbt/Tag;", friendly(NbtSuites.accountedNbtBuffers(216)))
            .extraInputs("readNbt(Lio/netty/buffer/ByteBuf;Lnet/minecraft/nbt/NbtAccounter;)Lnet/minecraft/nbt/Tag;", NbtSuites.accountedNbtBuffers(218))
            // An accounter with more than the default quota reading a count in the billions would allocate an array that long.
            .where(
                "readNbt(Lnet/minecraft/nbt/NbtAccounter;)Lnet/minecraft/nbt/Tag;",
                args -> NbtValues.readsLittle(((FriendlyByteBuf)args[0]).unwrap(), (NbtAccounter)args[1], NbtIo::readAnyTag)
            )
            .where(
                "readNbt(Lio/netty/buffer/ByteBuf;Lnet/minecraft/nbt/NbtAccounter;)Lnet/minecraft/nbt/Tag;",
                args -> NbtValues.readsLittle((ByteBuf)args[0], (NbtAccounter)args[1], NbtIo::readAnyTag)
            ),
        // The codecs ByteBufCodecs hands out. The ones made from other codecs or functions get the game's kinds of them.
        new CodecSuite(ByteBufCodecs.class)
            .codec(OPTIONAL, "VAR_INT", leading -> ByteBufCodecs.optional(ByteBufCodecs.VAR_INT), bufferCodec(Codecs.optionalOf(INTS), Optional.class))
            .codec(OPTIONAL, "STRING_UTF8", leading -> ByteBufCodecs.optional(ByteBufCodecs.STRING_UTF8), bufferCodec(Codecs.optionalOf(STRINGS), Optional.class))
            .codec(
                "collection(Ljava/util/function/IntFunction;Lnet/minecraft/network/codec/StreamCodec;)Lnet/minecraft/network/codec/StreamCodec;",
                "ArrayList::new, VAR_INT", leading -> ByteBufCodecs.collection(ArrayList::new, ByteBufCodecs.VAR_INT), bufferCodec(Codecs.listOf(INTS), List.class)
            )
            .codec(
                "collection(Ljava/util/function/IntFunction;Lnet/minecraft/network/codec/StreamCodec;I)Lnet/minecraft/network/codec/StreamCodec;",
                "ArrayList::new, STRING_UTF8", leading -> ByteBufCodecs.collection(ArrayList::new, ByteBufCodecs.STRING_UTF8, (Integer)leading[0]),
                bufferCodec(Codecs.listOf(STRINGS), List.class), INTS
            )
            .codec(
                "collection(Ljava/util/function/IntFunction;)Lnet/minecraft/network/codec/StreamCodec$CodecOperation;", "ArrayList::new, VAR_INT",
                leading -> ByteBufCodecs.VAR_INT.apply(ByteBufCodecs.collection(ArrayList::new)), bufferCodec(Codecs.listOf(INTS), List.class)
            )
            .codec(
                "collection(Ljava/util/function/IntFunction;I)Lnet/minecraft/network/codec/StreamCodec$CodecOperation;", "ArrayList::new, VAR_INT",
                leading -> ByteBufCodecs.VAR_INT.apply(ByteBufCodecs.collection(ArrayList::new, (Integer)leading[0])),
                bufferCodec(Codecs.listOf(INTS), List.class), INTS
            )
            .codec(
                "list()Lnet/minecraft/network/codec/StreamCodec$CodecOperation;", "Direction.STREAM_CODEC",
                leading -> Direction.STREAM_CODEC.apply(ByteBufCodecs.list()), bufferCodec(Codecs.listOf(DIRECTIONS), List.class)
            )
            .codec(
                "list(I)Lnet/minecraft/network/codec/StreamCodec$CodecOperation;", "STRING_UTF8",
                leading -> ByteBufCodecs.STRING_UTF8.apply(ByteBufCodecs.list((Integer)leading[0])), bufferCodec(Codecs.listOf(STRINGS), List.class), INTS
            )
            .codec(
                "fixedSizeCollection(Ljava/util/function/IntFunction;Lnet/minecraft/network/codec/StreamCodec;I)Lnet/minecraft/network/codec/StreamCodec;",
                "ArrayList::new, VAR_INT", leading -> ByteBufCodecs.fixedSizeCollection(ArrayList::new, ByteBufCodecs.VAR_INT, (Integer)leading[0]),
                bufferCodec(Codecs.listOf(INTS), List.class), SIZES
            )
            .codec(
                "fixedSizeCollection(Ljava/util/function/IntFunction;I)Lnet/minecraft/network/codec/StreamCodec$CodecOperation;", "ArrayList::new, FLOAT",
                leading -> ByteBufCodecs.FLOAT.apply(ByteBufCodecs.fixedSizeCollection(ArrayList::new, (Integer)leading[0])),
                bufferCodec(Codecs.listOf(FLOATS), List.class), SIZES
            )
            .codec(
                "fixedSizeList(I)Lnet/minecraft/network/codec/StreamCodec$CodecOperation;", "BOOL",
                leading -> ByteBufCodecs.BOOL.apply(ByteBufCodecs.fixedSizeList((Integer)leading[0])), bufferCodec(Codecs.listOf(BOOLEANS), List.class), SIZES
            )
            // A fixed-size bit set's bytes are its size over 8, rounded up: sizes around the bytes' bounds and the light
            // masks' 26, and none big enough to make vanilla allocate much (a size of 2^31 - 1 is 256 MiB).
            .codec(
                "fixedBitSet(I)Lnet/minecraft/network/codec/StreamCodec;", "sizes up to 2^16",
                leading -> ByteBufCodecs.fixedBitSet((Integer)leading[0]), bufferCodec(Values.BIT_SETS, BitSet.class), BIT_SET_SIZES
            )
            .codec(
                "idMapper(Ljava/util/function/IntFunction;Ljava/util/function/ToIntFunction;)Lnet/minecraft/network/codec/StreamCodec;",
                "Direction::from3DDataValue, Direction::get3DDataValue",
                leading -> ByteBufCodecs.idMapper(Direction::from3DDataValue, Direction::get3DDataValue), bufferCodec(DIRECTIONS, Direction.class)
            )
            // Maps in a HashMap, which writes its entries in its table's order: any, and a server's report details with any
            // maximum.
            .codec(
                "map(Ljava/util/function/IntFunction;Lnet/minecraft/network/codec/StreamCodec;Lnet/minecraft/network/codec/StreamCodec;)Lnet/minecraft/network/codec/StreamCodec;",
                "HashMap::new, STRING_UTF8, VAR_INT",
                leading -> ByteBufCodecs.<ByteBuf, String, Integer, HashMap<String, Integer>>map(HashMap::new, ByteBufCodecs.STRING_UTF8, ByteBufCodecs.VAR_INT),
                bufferCodec(Codecs.hashMapOf(STRINGS, INTS, List.of()), Map.class)
            )
            .codec(
                "map(Ljava/util/function/IntFunction;Lnet/minecraft/network/codec/StreamCodec;Lnet/minecraft/network/codec/StreamCodec;I)Lnet/minecraft/network/codec/StreamCodec;",
                "HashMap::new, stringUtf8(128), stringUtf8(4096)",
                leading -> ByteBufCodecs.<ByteBuf, String, String, HashMap<String, String>>map(
                    HashMap::new, ByteBufCodecs.stringUtf8(128), ByteBufCodecs.stringUtf8(4096), (Integer)leading[0]
                ),
                bufferCodec(PacketValues.REPORT_DETAILS, Map.class), INTS
            )
            // The NBT codecs with the accounters the game gives them: the click action's 32 KiB and 16 deep, and a trusted one.
            // A trusted codec reading a count in the billions from random bytes would allocate an array that long.
            .whereDecode("TRUSTED_TAG.decode", args -> NbtValues.readsLittle((ByteBuf)args[0], null, NbtIo::readAnyTag))
            .whereDecode("TRUSTED_COMPOUND_TAG.decode", args -> NbtValues.readsLittle((ByteBuf)args[0], null, NbtIo::readAnyTag))
            .whereDecode(
                "compoundTagCodec(Ljava/util/function/Supplier;)Lnet/minecraft/network/codec/StreamCodec;",
                args -> NbtValues.readsLittle((ByteBuf)args[0], null, NbtIo::readAnyTag)
            )
            .codec(TAG_CODEC, "new NbtAccounter(32768L, 16)", leading -> ByteBufCodecs.tagCodec(() -> new NbtAccounter(32768L, 16)), bufferCodec(TAGS, Tag.class))
            .codec(
                "optionalTagCodec(Ljava/util/function/Supplier;)Lnet/minecraft/network/codec/StreamCodec;", "new NbtAccounter(32768L, 16)",
                leading -> ByteBufCodecs.optionalTagCodec(() -> new NbtAccounter(32768L, 16)), bufferCodec(Codecs.optionalOf(TAGS), Optional.class)
            )
            .codec(
                "compoundTagCodec(Ljava/util/function/Supplier;)Lnet/minecraft/network/codec/StreamCodec;", "NbtAccounter::unlimitedHeap",
                leading -> ByteBufCodecs.compoundTagCodec(NbtAccounter::unlimitedHeap), bufferCodec(Values.generator(CompoundTag.class), CompoundTag.class)
            )
            // A length prefix in front of the click action's tag, and of a discarded payload, which reads every byte its
            // slice has, with any maximum.
            .codec(
                LENGTH_PREFIXED, CLICK_ACTION_TAGS,
                leading -> ByteBufCodecs.optionalTagCodec(() -> new NbtAccounter(32768L, 16)).apply(ByteBufCodecs.lengthPrefixed((Integer)leading[0])),
                bufferCodec(withUnwritableString(Codecs.optionalOf(TAGS)), Optional.class), PREFIX_SIZES
            )
            .extraInputs(LENGTH_PREFIXED + "[" + CLICK_ACTION_TAGS + "].decode", lengthPrefixes(tagPrefixPayloads(), false))
            .codec(
                LENGTH_PREFIXED_DECORATED, DISCARDED_MARKERS,
                leading -> DiscardedPayload.<FriendlyByteBuf>codec(MARKER, 32767)
                    .apply(ByteBufCodecs.<FriendlyByteBuf, DiscardedPayload>lengthPrefixed((Integer)leading[0], (parent, child) -> new FriendlyByteBuf(child))),
                new Codecs.Codec(NetworkValues.FRIENDLY_BUFFER, Values.generator(DiscardedPayload.class), DiscardedPayload.class), PREFIX_SIZES
            )
            .extraInputs(LENGTH_PREFIXED_DECORATED + "[" + DISCARDED_MARKERS + "].decode", lengthPrefixes(discardedPrefixPayloads(), true))
            // A length prefix in the millions makes vanilla allocate that many bytes before it finds the buffer shorter.
            .whereDecode("byteArray(I)Lnet/minecraft/network/codec/StreamCodec;", args -> allocatesLittle(args[1], (Integer)args[0]))
            // Its writer stops the count at 16 and each string at its limit (64, 32767 and 1024), so the reader also reads
            // what's written without them, and counts it takes or refuses before any property.
            .extraInputs(
                "GAME_PROFILE_PROPERTIES.decode",
                Stream.concat(
                    written(103, ProfileValues.PROPERTY_MAPS, Golden::writePropertiesWithoutLimits).stream(),
                    propertyCounts(new byte[0]).stream()
                ).toList()
            )
            // The same for a profile, whose writer also stops the name at 16 characters.
            .extraInputs(
                "GAME_PROFILE.decode",
                Stream.concat(
                    written(104, ProfileValues.GAME_PROFILES, (FriendlyByteBuf buffer, GameProfile profile) -> {
                        buffer.writeUUID(profile.id());
                        Utf8String.write(buffer, profile.name(), Integer.MAX_VALUE);
                        writePropertiesWithoutLimits(buffer, profile.properties());
                    }).stream(),
                    propertyCounts(NetworkValues.concat(NetworkValues.uuid(NetworkValues.offlinePlayer()), NetworkValues.utf("rustcraft"))).stream()
                ).toList()
            ),
        // StreamCodec's factories and combinators, each given the game's kinds of codecs and functions.
        composites(new CodecSuite(StreamCodec.class))
            .codec(
                "of(Lnet/minecraft/network/codec/StreamEncoder;Lnet/minecraft/network/codec/StreamDecoder;)Lnet/minecraft/network/codec/StreamCodec;",
                "writeUUID, readUUID", leading -> StreamCodec.<ByteBuf, UUID>of(FriendlyByteBuf::writeUUID, FriendlyByteBuf::readUUID),
                bufferCodec(Values.generator(UUID.class), UUID.class)
            )
            .codec(
                "ofMember(Lnet/minecraft/network/codec/StreamMemberEncoder;Lnet/minecraft/network/codec/StreamDecoder;)Lnet/minecraft/network/codec/StreamCodec;",
                "writeIdentifier, readIdentifier",
                leading -> StreamCodec.<FriendlyByteBuf, Identifier>ofMember((id, output) -> output.writeIdentifier(id), FriendlyByteBuf::readIdentifier),
                new Codecs.Codec(NetworkValues.FRIENDLY_BUFFER, Values.generator(Identifier.class), Identifier.class)
            )
            .codec("unit(Ljava/lang/Object;)Lnet/minecraft/network/codec/StreamCodec;", "NORTH", leading -> StreamCodec.unit(Direction.NORTH), bufferCodec(DIRECTIONS, Direction.class))
            .codec(
                "apply(Lnet/minecraft/network/codec/StreamCodec$CodecOperation;)Lnet/minecraft/network/codec/StreamCodec;", "VAR_INT, optional",
                leading -> ByteBufCodecs.VAR_INT.apply(ByteBufCodecs::optional), bufferCodec(Codecs.optionalOf(INTS), Optional.class)
            )
            .codec(
                "map(Ljava/util/function/Function;Ljava/util/function/Function;)Lnet/minecraft/network/codec/StreamCodec;", "BYTE, unpackDegrees, packDegrees",
                leading -> ByteBufCodecs.BYTE.map(Mth::unpackDegrees, Mth::packDegrees), bufferCodec(FLOATS, Float.class)
            )
            .codec(
                "cast()Lnet/minecraft/network/codec/StreamCodec;", "VAR_INT", leading -> ByteBufCodecs.VAR_INT.<FriendlyByteBuf>cast(),
                new Codecs.Codec(NetworkValues.FRIENDLY_BUFFER, INTS, Integer.class)
            )
            // How the protocols put a packet's codec on the ByteBuf a connection reads.
            .codec(
                "mapStream(Ljava/util/function/Function;)Lnet/minecraft/network/codec/StreamCodec;", "ClientIntentionPacket.STREAM_CODEC, FriendlyByteBuf::new",
                leading -> ClientIntentionPacket.STREAM_CODEC.<ByteBuf>mapStream(FriendlyByteBuf::new),
                bufferCodec(Values.generator(ClientIntentionPacket.class), ClientIntentionPacket.class)
            ),
        // ByIdMap's maps by the game's ids, with every strategy for ids out of range, and ones it refuses to make.
        new CodecSuite(ByIdMap.class)
            .function(
                CONTINUOUS, "get3DDataValue, values()",
                leading -> ByIdMap.continuous(Direction::get3DDataValue, Direction.values(), (ByIdMap.OutOfBoundsStrategy)leading[0]), Direction.class,
                Values.generator(ByIdMap.OutOfBoundsStrategy.class)
            )
            .function(
                CONTINUOUS, "get2DDataValue, values()",
                leading -> ByIdMap.continuous(Direction::get2DDataValue, Direction.values(), (ByIdMap.OutOfBoundsStrategy)leading[0]), Direction.class,
                Values.generator(ByIdMap.OutOfBoundsStrategy.class)
            )
            .function(
                CONTINUOUS, "get3DDataValue, []",
                leading -> ByIdMap.continuous(Direction::get3DDataValue, new Direction[0], (ByIdMap.OutOfBoundsStrategy)leading[0]), Direction.class,
                Values.generator(ByIdMap.OutOfBoundsStrategy.class)
            )
            .function(
                SPARSE, "get2DDataValue, [NORTH, EAST, SOUTH, WEST], UP",
                leading -> ByIdMap.sparse(
                    Direction::get2DDataValue, new Direction[]{Direction.NORTH, Direction.EAST, Direction.SOUTH, Direction.WEST}, Direction.UP
                ),
                Direction.class
            )
            .function(
                SPARSE, "get2DDataValue, values(), UP", leading -> ByIdMap.sparse(Direction::get2DDataValue, Direction.values(), Direction.UP), Direction.class
            )
            .function(SPARSE, "get2DDataValue, [], UP", leading -> ByIdMap.sparse(Direction::get2DDataValue, new Direction[0], Direction.UP), Direction.class),
        InstanceSuite.every(ByIdMap.OutOfBoundsStrategy.class)
            .extraInputs("valueOf(Ljava/lang/String;)Lnet/minecraft/util/ByIdMap$OutOfBoundsStrategy;", names(ByIdMap.OutOfBoundsStrategy.values())),
        // The packed section nodes the light engines key their sections by.
        new StaticSuite(SectionPos.class).only(
            "offset(JLnet/minecraft/core/Direction;)J", "offset(JIII)J", "blockToSection(J)J", "getZeroNode(II)J", "getZeroNode(J)J",
            "sectionToChunk(J)J", "x(J)I", "y(J)I", "z(J)I", "asLong(III)J", "blockToSectionCoord(I)I", "sectionRelative(I)I",
            "sectionToBlockCoord(I)I", "sectionToBlockCoord(II)I"
        ),
        new StaticSuite(LpVec3.class)
            .extraInputs("write(Lio/netty/buffer/ByteBuf;Lnet/minecraft/world/phys/Vec3;)V", lpWrites())
            .extraInputs("read(Lio/netty/buffer/ByteBuf;)Lnet/minecraft/world/phys/Vec3;", lpReads())
            .extraInputs("hasContinuationBit(I)Z", everyByte()),
        HandlerSuite.frameDecoder(),
        HandlerSuite.lengthFieldPrepender(),
        // The protocols, and the packets the join phase sends and receives.
        InstanceSuite.every(ConnectionProtocol.class)
            .extraInputs("valueOf(Ljava/lang/String;)Lnet/minecraft/network/ConnectionProtocol;", names(ConnectionProtocol.values())),
        InstanceSuite.every(PacketFlow.class)
            .extraInputs("valueOf(Ljava/lang/String;)Lnet/minecraft/network/protocol/PacketFlow;", names(PacketFlow.values())),
        // Too many types for every pair of edges to be the edge cases, so equals gets them as extra inputs.
        InstanceSuite.every(PacketType.class)
            .exclude("hashCode()I", ENUM_IN_RECORD)
            .extraInputs("equals(Ljava/lang/Object;)Z", edgePairs(PacketValues.PACKET_TYPE)),
        InstanceSuite.every(ClientIntent.class)
            .extraInputs("valueOf(Ljava/lang/String;)Lnet/minecraft/network/protocol/handshake/ClientIntent;", names(ClientIntent.values())),
        InstanceSuite.every(ClientIntentionPacket.class).exclude("hashCode()I", ENUM_IN_RECORD),
        new StaticSuite(HandshakePacketTypes.class),
        new ProtocolSuite(HandshakeProtocols.class),
        // Its writer stops names at 16 characters, so the reader also reads longer ones, written without the limit.
        InstanceSuite.every(ServerboundHelloPacket.class)
            .extraInputs(
                "STREAM_CODEC.decode",
                friendly(written(100, Values.generator(ServerboundHelloPacket.class), (FriendlyByteBuf buffer, ServerboundHelloPacket packet) -> {
                    buffer.writeUtf(packet.name());
                    buffer.writeUUID(packet.profileId());
                }))
            ),
        InstanceSuite.every(ClientboundHelloPacket.class),
        InstanceSuite.every(ServerboundLoginAcknowledgedPacket.class),
        InstanceSuite.every(ClientboundLoginCompressionPacket.class),
        InstanceSuite.every(ServerboundKeyPacket.class),
        // Their readers skip whatever follows the header as the payload, so they also read payloads at the limit and past it.
        InstanceSuite.every(ServerboundCustomQueryAnswerPacket.class).extraInputs("STREAM_CODEC.decode", largePayloads(false)),
        InstanceSuite.every(ClientboundCustomQueryPacket.class).extraInputs("STREAM_CODEC.decode", largePayloads(true)),
        InstanceSuite.every(DiscardedQueryAnswerPayload.class),
        InstanceSuite.every(DiscardedQueryPayload.class),
        InstanceSuite.every(ClientboundLoginFinishedPacket.class),
        new StaticSuite(LoginPacketTypes.class),
        new ProtocolSuite(LoginProtocols.class),
        // The status protocol, the server list's ping, with the ping packets play shares. Its response's status is read with
        // DataFixerUpper's codecs from JSON, which have no generator yet, so its clientbound codec is skipped.
        new StaticSuite(StatusPacketTypes.class),
        new ProtocolSuite(StatusProtocols.class),
        InstanceSuite.every(ServerboundStatusRequestPacket.class),
        InstanceSuite.every(ServerboundPingRequestPacket.class),
        InstanceSuite.every(ClientboundPongResponsePacket.class),
        // The profile the login's last packet carries: authlib's, with its properties in a Guava multimap, which a
        // PropertyMap forwards its Multimap methods to.
        InstanceSuite.every(GameProfile.class),
        InstanceSuite.every(Property.class),
        InstanceSuite.every(PropertyMap.class)
            .inherited(
                "values()Ljava/util/Collection;", "size()I", "isEmpty()Z", "equals(Ljava/lang/Object;)Z", "hashCode()I",
                "toString()Ljava/lang/String;"
            ),
        new StaticSuite(UUIDUtil.class)
            .extraInputs("createOfflinePlayerUUID(Ljava/lang/String;)Ljava/util/UUID;", each(ProfileValues.PLAYER_NAMES.edges()))
            .extraInputs("createOfflineProfile(Ljava/lang/String;)Lcom/mojang/authlib/GameProfile;", each(ProfileValues.PLAYER_NAMES.edges())),
        // The cookie packets, which login, configuration and play share. Their writers stop payloads at 5120 bytes, so the
        // readers also read longer ones, written without the limit.
        InstanceSuite.every(ClientboundCookieRequestPacket.class),
        InstanceSuite.every(ServerboundCookieResponsePacket.class)
            .exclude("hashCode()I", ARRAY_IN_RECORD)
            .exclude("toString()Ljava/lang/String;", ARRAY_IN_RECORD)
            .extraInputs(
                "STREAM_CODEC.decode",
                friendly(written(101, Values.generator(ServerboundCookieResponsePacket.class), (FriendlyByteBuf buffer, ServerboundCookieResponsePacket packet) -> {
                    buffer.writeIdentifier(packet.key());
                    buffer.writeNullable(packet.payload(), (FriendlyByteBuf out, byte[] payload) -> out.writeByteArray(payload));
                }))
            ),
        new StaticSuite(CookiePacketTypes.class),
        InstanceSuite.every(ClientboundStoreCookiePacket.class)
            .exclude("hashCode()I", ARRAY_IN_RECORD)
            .exclude("toString()Ljava/lang/String;", ARRAY_IN_RECORD)
            .extraInputs(
                "STREAM_CODEC.decode",
                friendly(written(102, Values.generator(ClientboundStoreCookiePacket.class), (FriendlyByteBuf buffer, ClientboundStoreCookiePacket packet) -> {
                    buffer.writeIdentifier(packet.key());
                    buffer.writeByteArray(packet.payload());
                }))
            ),
        new StaticSuite(CommonPacketTypes.class),
        // The configuration protocol's serverbound packets, which the client sends after login, and what they hold.
        new StaticSuite(ConfigurationPacketTypes.class),
        InstanceSuite.every(ChatVisiblity.class)
            .extraInputs("valueOf(Ljava/lang/String;)Lnet/minecraft/world/entity/player/ChatVisiblity;", names(ChatVisiblity.values())),
        InstanceSuite.every(HumanoidArm.class)
            .extraInputs("valueOf(Ljava/lang/String;)Lnet/minecraft/world/entity/HumanoidArm;", names(HumanoidArm.values())),
        InstanceSuite.every(ParticleStatus.class)
            .extraInputs("valueOf(Ljava/lang/String;)Lnet/minecraft/server/level/ParticleStatus;", names(ParticleStatus.values())),
        // Its reader's constructor reads what write writes, which doesn't stop the language at 16 characters like it does.
        InstanceSuite.every(ClientInformation.class)
            .exclude("hashCode()I", ENUM_IN_RECORD)
            .extraInputs(
                "<init>(Lnet/minecraft/network/FriendlyByteBuf;)V",
                friendly(written(106, Values.generator(ClientInformation.class), (FriendlyByteBuf buffer, ClientInformation information) -> information.write(buffer)))
            ),
        InstanceSuite.every(ServerboundClientInformationPacket.class).exclude("hashCode()I", ENUM_IN_RECORD),
        // The payloads: what a custom_payload packet's identifier names, and a fallback for any other identifier.
        new CodecSuite(CustomPacketPayload.class)
            .codec(
                "codec(Lnet/minecraft/network/codec/StreamMemberEncoder;Lnet/minecraft/network/codec/StreamDecoder;)Lnet/minecraft/network/codec/StreamCodec;",
                "writeUtf, readUtf",
                leading -> CustomPacketPayload.<FriendlyByteBuf, BrandPayload>codec(
                    (payload, output) -> output.writeUtf(payload.brand()), input -> new BrandPayload(input.readUtf())
                ),
                new Codecs.Codec(NetworkValues.FRIENDLY_BUFFER, Values.generator(BrandPayload.class), BrandPayload.class)
            )
            .codec(
                "codec(Lnet/minecraft/network/protocol/common/custom/CustomPacketPayload$FallbackProvider;Ljava/util/List;)Lnet/minecraft/network/codec/StreamCodec;",
                "DiscardedPayload.codec(id, 32767), [BrandPayload]",
                leading -> CustomPacketPayload.<FriendlyByteBuf>codec(
                    id -> DiscardedPayload.codec(id, 32767), List.of(new CustomPacketPayload.TypeAndCodec<>(BrandPayload.TYPE, BrandPayload.STREAM_CODEC))
                ),
                new Codecs.Codec(NetworkValues.FRIENDLY_BUFFER, Values.generator(CustomPacketPayload.class), CustomPacketPayload.class)
            )
            .codec(
                "codec(Lnet/minecraft/network/protocol/common/custom/CustomPacketPayload$FallbackProvider;Ljava/util/List;)Lnet/minecraft/network/codec/StreamCodec;",
                "DiscardedPayload.codec(id, 0), []", leading -> CustomPacketPayload.<FriendlyByteBuf>codec(id -> DiscardedPayload.codec(id, 0), List.of()),
                new Codecs.Codec(NetworkValues.FRIENDLY_BUFFER, Values.generator(CustomPacketPayload.class), CustomPacketPayload.class)
            ),
        InstanceSuite.every(CustomPacketPayload.Type.class),
        InstanceSuite.every(BrandPayload.class),
        // Its codec skips the rest of the buffer, so it also reads buffers as long as the maximum, and a byte either side.
        InstanceSuite.every(DiscardedPayload.class).extraInputs(DISCARDED_CODEC + ".decode", discardedPayloads()),
        InstanceSuite.every(ServerboundCustomPayloadPacket.class).extraInputs("STREAM_CODEC.decode", customPayloads(32767)),
        InstanceSuite.every(KnownPack.class),
        InstanceSuite.every(ServerboundSelectKnownPacks.class),
        InstanceSuite.every(ServerboundFinishConfigurationPacket.class),
        InstanceSuite.every(ServerboundAcceptCodeOfConductPacket.class),
        InstanceSuite.every(ServerboundKeepAlivePacket.class),
        InstanceSuite.every(ServerboundPongPacket.class),
        InstanceSuite.every(ServerboundResourcePackPacket.class).exclude("hashCode()I", ENUM_IN_RECORD),
        // An EndTag's hashCode is Object's, an identity hash. Its tag is behind a length prefix of at most 65536 bytes,
        // read with 32 KiB and 16 levels to spend, so it also reads prefixes and tags at those limits and past them.
        InstanceSuite.every(ServerboundCustomClickActionPacket.class)
            .where("hashCode()I", args -> !(((ServerboundCustomClickActionPacket)args[0]).payload().orElse(null) instanceof EndTag))
            .extraInputs("STREAM_CODEC.decode", clickActionPayloads()),
        new ProtocolSuite(ConfigurationProtocols.class),
        // The configuration protocol's clientbound packets that need neither text components nor the registries.
        InstanceSuite.every(ClientboundFinishConfigurationPacket.class),
        InstanceSuite.every(ClientboundResetChatPacket.class),
        InstanceSuite.every(ClientboundClearDialogPacket.class),
        InstanceSuite.every(ClientboundKeepAlivePacket.class),
        InstanceSuite.every(ClientboundPingPacket.class),
        InstanceSuite.every(ClientboundSelectKnownPacks.class),
        InstanceSuite.every(ClientboundResourcePackPopPacket.class),
        // Their writers stop strings at 32767 units, so the readers also read longer ones, written without the limit.
        InstanceSuite.every(ClientboundTransferPacket.class)
            .extraInputs(
                "STREAM_CODEC.decode",
                friendly(written(107, Values.generator(ClientboundTransferPacket.class), (FriendlyByteBuf buffer, ClientboundTransferPacket packet) -> {
                    Utf8String.write(buffer, packet.host(), Integer.MAX_VALUE);
                    buffer.writeVarInt(packet.port());
                }))
            ),
        InstanceSuite.every(ClientboundCodeOfConductPacket.class)
            .extraInputs(
                "STREAM_CODEC.decode",
                written(
                    108, Values.generator(ClientboundCodeOfConductPacket.class),
                    (FriendlyByteBuf buffer, ClientboundCodeOfConductPacket packet) -> Utf8String.write(buffer, packet.codeOfConduct(), Integer.MAX_VALUE)
                )
            ),
        InstanceSuite.every(ClientboundPostEffectsPacket.class),
        // Configuration's codec of it skips any payload it doesn't know, up to 1 MiB.
        InstanceSuite.every(ClientboundCustomPayloadPacket.class).extraInputs("CONFIG_STREAM_CODEC.decode", customPayloads(1048576)),
        // Its codec reads the features into a HashSet, which keeps one of each, so it also reads identifiers twice, and counts
        // it refuses to make a set for (negative ones) or reads no identifier after.
        InstanceSuite.every(ClientboundUpdateEnabledFeaturesPacket.class).extraInputs("STREAM_CODEC.decode", featureBuffers()),
        // Its writer stops names at 128 characters, values at 4096 and the count at 32, so the reader also reads what's
        // written without them, a name twice (the HashMap keeps the second value where the first was), and counts with
        // nothing after them.
        InstanceSuite.every(ClientboundCustomReportDetailsPacket.class)
            .extraInputs(
                "STREAM_CODEC.decode",
                Stream.concat(
                    written(109, Values.generator(ClientboundCustomReportDetailsPacket.class), (FriendlyByteBuf buffer, ClientboundCustomReportDetailsPacket packet) -> {
                        buffer.writeVarInt(packet.details().size());
                        packet.details().forEach((key, value) -> {
                            Utf8String.write(buffer, key, Integer.MAX_VALUE);
                            Utf8String.write(buffer, value, Integer.MAX_VALUE);
                        });
                    }).stream(),
                    reportDetailBuffers().stream()
                ).toList()
            ),
        // The game protocol: its packet types, its templates, and the packets the client sends and answers while it joins.
        new StaticSuite(GamePacketTypes.class),
        new StaticSuite(PingPacketTypes.class),
        new ProtocolSuite(GameProtocols.class),
        InstanceSuite.every(ClientboundChunkBatchStartPacket.class),
        InstanceSuite.every(ClientboundChunkBatchFinishedPacket.class),
        InstanceSuite.every(ServerboundChunkBatchReceivedPacket.class),
        InstanceSuite.every(ServerboundClientTickEndPacket.class),
        InstanceSuite.every(ServerboundPlayerLoadedPacket.class),
        InstanceSuite.every(ServerboundConfigurationAcknowledgedPacket.class),
        InstanceSuite.every(ClientboundStartConfigurationPacket.class),
        // Its writer stops commands at 32767 characters, so the reader also reads longer ones, written without the limit.
        InstanceSuite.every(ServerboundChatCommandPacket.class)
            .extraInputs(
                "STREAM_CODEC.decode",
                friendly(written(110, Values.generator(ServerboundChatCommandPacket.class), (FriendlyByteBuf buffer, ServerboundChatCommandPacket packet) ->
                    Utf8String.write(buffer, packet.command(), Integer.MAX_VALUE)
                ))
            ),
        InstanceSuite.every(ServerboundAcceptTeleportationPacket.class),
        InstanceSuite.every(ClientboundGameEventPacket.class),
        // Each kind of move, with the methods of the class they extend.
        InstanceSuite.every(ServerboundMovePlayerPacket.Pos.class).inherited("getX(D)D", "getY(D)D", "getZ(D)D", "getYRot(F)F", "getXRot(F)F", "isOnGround()Z", "horizontalCollision()Z", "hasPosition()Z", "hasRotation()Z", "handle(Lnet/minecraft/network/protocol/game/ServerGamePacketListener;)V"),
        InstanceSuite.every(ServerboundMovePlayerPacket.PosRot.class).inherited("getX(D)D", "getY(D)D", "getZ(D)D", "getYRot(F)F", "getXRot(F)F", "isOnGround()Z", "horizontalCollision()Z", "hasPosition()Z", "hasRotation()Z", "handle(Lnet/minecraft/network/protocol/game/ServerGamePacketListener;)V"),
        InstanceSuite.every(ServerboundMovePlayerPacket.Rot.class).inherited("getX(D)D", "getY(D)D", "getZ(D)D", "getYRot(F)F", "getXRot(F)F", "isOnGround()Z", "horizontalCollision()Z", "hasPosition()Z", "hasRotation()Z", "handle(Lnet/minecraft/network/protocol/game/ServerGamePacketListener;)V"),
        InstanceSuite.every(ServerboundMovePlayerPacket.StatusOnly.class).inherited("getX(D)D", "getY(D)D", "getZ(D)D", "getYRot(F)F", "getXRot(F)F", "isOnGround()Z", "horizontalCollision()Z", "hasPosition()Z", "hasRotation()Z", "handle(Lnet/minecraft/network/protocol/game/ServerGamePacketListener;)V"),
        InstanceSuite.every(PositionMoveRotation.class),
        // What a moving player sends besides its moves: the keys it holds, and its sprint's start and stop.
        InstanceSuite.every(Input.class),
        InstanceSuite.every(ServerboundPlayerInputPacket.class),
        // Its reader takes any action's ordinal, past the last one too, and any data.
        InstanceSuite.every(ServerboundPlayerCommandPacket.class)
            .extraInputs(
                "STREAM_CODEC.decode",
                friendly(written(111, Codecs.tupleOf(IDS, IDS, Values.generator(int.class)), (FriendlyByteBuf buffer, List<Integer> fields) -> {
                    buffer.writeVarInt(fields.get(0));
                    buffer.writeVarInt(fields.get(1));
                    buffer.writeVarInt(fields.get(2));
                }))
            ),
        InstanceSuite.every(ServerboundPlayerCommandPacket.Action.class)
            .extraInputs(
                "valueOf(Ljava/lang/String;)Lnet/minecraft/network/protocol/game/ServerboundPlayerCommandPacket$Action;",
                names(ServerboundPlayerCommandPacket.Action.values())
            ),
        // Its set of relatives is an EnumSet, whose hashCode adds its constants' identity hashes.
        InstanceSuite.every(ClientboundPlayerPositionPacket.class).exclude("hashCode()I", ENUM_IN_RECORD),
        // The game protocol's clientbound packets the server sends while the player joins, and the difficulty.
        InstanceSuite.every(ClientboundSetChunkCacheCenterPacket.class),
        InstanceSuite.every(ClientboundSetChunkCacheRadiusPacket.class),
        InstanceSuite.every(ClientboundSetSimulationDistancePacket.class),
        InstanceSuite.every(ClientboundSetHealthPacket.class),
        InstanceSuite.every(ClientboundSetExperiencePacket.class),
        InstanceSuite.every(ClientboundSetHeldSlotPacket.class),
        InstanceSuite.every(ClientboundTickingStatePacket.class),
        InstanceSuite.every(ClientboundTickingStepPacket.class),
        InstanceSuite.every(ClientboundPlayerAbilitiesPacket.class),
        InstanceSuite.every(ClientboundInitializeBorderPacket.class),
        InstanceSuite.every(Difficulty.class)
            .extraInputs("valueOf(Ljava/lang/String;)Lnet/minecraft/world/Difficulty;", names(Difficulty.values()))
            .extraInputs(
                "byName(Ljava/lang/String;)Lnet/minecraft/world/Difficulty;",
                each(List.of("peaceful", "easy", "normal", "hard", "NORMAL", "Normal", "", "normal ", "custom"))
            ),
        InstanceSuite.every(ClientboundChangeDifficultyPacket.class).exclude("hashCode()I", ENUM_IN_RECORD),
        // The game protocol's packets that read the registries, and what they hold: the registries a RegistryFriendlyByteBuf
        // carries (RegistryValues), the clocks' time, an entity's attributes, the world's spawn and an entity's data.
        new DataResultSuite(),
        new NbtOpsSuite(),
        new RegistryValues.RegistriesSuite(),
        new StaticSuite(Attributes.class),
        InstanceSuite.every(WorldClock.class),
        InstanceSuite.every(ClockNetworkState.class),
        new RegistryValues.ClockManagerSuite(),
        InstanceSuite.every(ClientboundSetTimePacket.class)
            .exclude("hashCode()I", HOLDER_IN_RECORD)
            .extraInputs("STREAM_CODEC.decode", registryBuffers(SET_TIMES)),
        InstanceSuite.every(AttributeModifier.Operation.class),
        InstanceSuite.every(AttributeModifier.class).exclude("hashCode()I", ENUM_IN_RECORD),
        InstanceSuite.every(ClientboundUpdateAttributesPacket.AttributeSnapshot.class).exclude("hashCode()I", HOLDER_IN_RECORD),
        InstanceSuite.every(ClientboundUpdateAttributesPacket.class).extraInputs("STREAM_CODEC.decode", registryBuffers(UPDATE_ATTRIBUTES)),
        InstanceSuite.every(GlobalPos.class).exclude("hashCode()I", KEY_IN_RECORD),
        InstanceSuite.every(LevelData.RespawnData.class).exclude("hashCode()I", KEY_IN_RECORD),
        InstanceSuite.every(ClientboundSetDefaultSpawnPositionPacket.class).exclude("hashCode()I", KEY_IN_RECORD),
        InstanceSuite.every(RecipeBookType.class),
        InstanceSuite.every(RecipeBookSettings.TypeSettings.class),
        InstanceSuite.every(RecipeBookSettings.class),
        // RecipeBookSettings overrides none of equals, hashCode and toString, so the record's compare, hash and print it by
        // identity.
        InstanceSuite.every(ClientboundRecipeBookSettingsPacket.class)
            .exclude("equals(Ljava/lang/Object;)Z", SETTINGS_IN_RECORD)
            .exclude("hashCode()I", SETTINGS_IN_RECORD)
            .exclude("toString()Ljava/lang/String;", SETTINGS_IN_RECORD),
        InstanceSuite.every(Rotations.class),
        InstanceSuite.every(Pose.class),
        InstanceSuite.every(SynchedEntityData.DataValue.class)
            .exclude("hashCode()I", SERIALIZER_IN_RECORD)
            .exclude("toString()Ljava/lang/String;", SERIALIZER_IN_RECORD)
            .where(DATA_VALUE_READ, args -> !RegistryValues.reachesUnported(args[0], false)),
        InstanceSuite.every(ClientboundSetEntityDataPacket.class)
            .exclude("hashCode()I", SERIALIZER_IN_RECORD)
            .exclude("toString()Ljava/lang/String;", SERIALIZER_IN_RECORD)
            .where("STREAM_CODEC.decode", args -> !RegistryValues.reachesUnported(args[0], true))
            .extraInputs("STREAM_CODEC.decode", registryBuffers(RegistryValues.dataValueBuffers())),
        // The registries the server sends, by their keys, and their elements.
        InstanceSuite.every(ResourceKey.class),
        new StaticSuite(Registries.class),
        // An EndTag's hashCode is Object's, an identity hash. Its data is read with the default quota of 2 MiB, so it also
        // reads data at the quota and past it.
        InstanceSuite.every(RegistrySynchronization.PackedRegistryEntry.class)
            .where("hashCode()I", args -> !(((RegistrySynchronization.PackedRegistryEntry)args[0]).data().orElse(null) instanceof EndTag))
            .extraInputs("STREAM_CODEC.decode", registryEntriesAtQuota()),
        // Keys are compared by reference, so equals also compares packets of every pair of the keys' edges, among which
        // are keys with the same identifier in different registries.
        InstanceSuite.every(ClientboundRegistryDataPacket.class)
            .exclude("hashCode()I", KEY_IN_RECORD)
            .extraInputs("equals(Ljava/lang/Object;)Z", registryDataPairs()),
        InstanceSuite.every(ServerboundResourcePackPacket.Action.class)
            .extraInputs(
                "valueOf(Ljava/lang/String;)Lnet/minecraft/network/protocol/common/ServerboundResourcePackPacket$Action;",
                names(ServerboundResourcePackPacket.Action.values())
            ),
        // The light packets' masks: what the codecs and the client's light code call, and what equals and the readers see.
        // set(I) gets indexes up to a few words past the set, since a big one makes it allocate that many.
        new InstanceSuite(
            BitSet.class, "<init>()V", "<init>(I)V", "valueOf([J)Ljava/util/BitSet;", "valueOf([B)Ljava/util/BitSet;", "toByteArray()[B",
            "toLongArray()[J", "get(I)Z", "set(I)V", "clear(II)V", "nextSetBit(I)I", "nextClearBit(I)I", "length()I", "isEmpty()Z",
            "cardinality()I", "equals(Ljava/lang/Object;)Z", "hashCode()I", "toString()Ljava/lang/String;"
        )
            .where("<init>(I)V", args -> (Integer)args[0] <= 1 << 16)
            .where("set(I)V", args -> (Integer)args[1] <= 1 << 10)
            .where("clear(II)V", args -> (Integer)args[2] <= 1 << 16 || (Integer)args[1] > (Integer)args[2]),
        // What a ServerboundHelloPacket's hashCode and toString use, and what makes an offline player's UUID.
        new InstanceSuite(UUID.class, "hashCode()I", "toString()Ljava/lang/String;", "nameUUIDFromBytes([B)Ljava/util/UUID;")
            .extraInputs("nameUUIDFromBytes([B)Ljava/util/UUID;", offlinePlayerNames())
    );

    /** Writes every suite's file to the directory in the first argument, or only the suites the other arguments name. */
    public static void main(final String[] args) throws IOException {
        // Reading NBT wraps an IOException in a crash report, whose system details warn, with a stack trace, about each
        // one they can't get: the game's version, which the generator doesn't set, and the hardware, since
        // tools/golden.py runs it without OSHI. That's four stack traces for each of thousands of reports.
        Configurator.setLevel(SystemReport.class.getName(), Level.ERROR);
        // The game bootstrapped, like its own tests: the configuration protocol's clientbound template holds the dialog
        // packet, whose codec needs the registries. Bootstrapping sends System.out to the log, so the progress goes to the
        // stream from before.
        PrintStream out = System.out;
        SharedConstants.tryDetectVersion();
        Bootstrap.bootStrap();
        Path outputDir = Path.of(args[0]);
        Files.createDirectories(outputDir);
        List<String> only = Arrays.asList(args).subList(1, args.length);
        for (Suite suite : Stream.of(SUITES, NbtSuites.suites(), RenderSuites.suites(), BlockSuites.suites(), ChunkSuites.suites(), ShapeSuites.suites(), LevelChunkSuites.suites(), LightSuites.suites(), BiomeSuites.suites(), PlayerSuites.suites(), ModelSuites.suites()).flatMap(List::stream).toList()) {
            if (!only.isEmpty() && !only.contains(suite.name())) {
                continue;
            }

            Path file = outputDir.resolve(suite.name() + ".jsonl");
            try (GoldenWriter writer = new GoldenWriter(file)) {
                suite.write(writer);
                out.printf("%s: %d lines%n", file.getFileName(), writer.lines());
            }
        }
    }

    // Where Float.toString's digits and format change: every exponent's smallest and largest significand and the one in
    // the middle, powers of ten and their neighbours an ulp either side, the thresholds between the plain and scientific
    // forms (1.0E-3 and 1.0E7), the smallest subnormals (whose decimals come from a tiny significand times 10), whole
    // numbers, and random bit patterns, half of them in the game's range of magnitudes.
    private static List<Object[]> floatToStringInputs() {
        Random random = new Random(37);
        List<Object[]> inputs = new ArrayList<>();
        for (int exponent = 0; exponent < 255; exponent++) {
            for (int significand : new int[]{0, 1, 0x400000, 0x7ffffe, 0x7fffff}) {
                float value = Float.intBitsToFloat(exponent << 23 | significand);
                inputs.add(new Object[]{value});
                inputs.add(new Object[]{-value});
            }
        }
        for (int power = -45; power <= 38; power++) {
            float value = Float.parseFloat("1.0E" + power);
            inputs.add(new Object[]{value});
            inputs.add(new Object[]{Math.nextUp(value)});
            inputs.add(new Object[]{Math.nextDown(value)});
        }
        for (float value : new float[]{1.0E-3F, 9.999999E-4F, 1.0E7F, 9999999.0F, 0.001F, 123456.7F, 0.1F, 0.3F, 2.0E-45F, 3.0E-45F}) {
            inputs.add(new Object[]{value});
            inputs.add(new Object[]{Math.nextUp(value)});
            inputs.add(new Object[]{Math.nextDown(value)});
        }
        for (int i = 0; i < 10; i++) {
            inputs.add(new Object[]{Float.intBitsToFloat(i)});
        }
        for (int i = 0; i < 2000; i++) {
            inputs.add(new Object[]{(float)random.nextInt(2000001) - 1000000});
        }
        for (int i = 0; i < 30000; i++) {
            float value = i % 2 == 0
                ? Float.intBitsToFloat(random.nextInt())
                : (random.nextFloat() * 2.0F - 1.0F) * (float)Math.pow(10.0, random.nextInt(14) - 7);
            inputs.add(new Object[]{value});
        }
        return inputs;
    }

    // The same for Double.toString: its forms change at the same thresholds, and its exponents reach three digits.
    private static List<Object[]> doubleToStringInputs() {
        Random random = new Random(41);
        List<Object[]> inputs = new ArrayList<>();
        for (int exponent = 0; exponent < 2047; exponent++) {
            for (long significand : new long[]{0L, 1L, 1L << 51, (1L << 52) - 2, (1L << 52) - 1}) {
                double value = Double.longBitsToDouble((long)exponent << 52 | significand);
                inputs.add(new Object[]{value});
                inputs.add(new Object[]{-value});
            }
        }
        for (int power = -324; power <= 308; power++) {
            double value = Double.parseDouble("1.0E" + power);
            inputs.add(new Object[]{value});
            inputs.add(new Object[]{Math.nextUp(value)});
            inputs.add(new Object[]{Math.nextDown(value)});
        }
        for (double value : new double[]{1.0E-3, 1.0E7, 0.001, 0.1, 0.2, 0.3, 0.30000001192092896, 2.0E-323, 5.0E-324, 1.0E23, 2.0E23}) {
            inputs.add(new Object[]{value});
            inputs.add(new Object[]{Math.nextUp(value)});
            inputs.add(new Object[]{Math.nextDown(value)});
        }
        for (int i = 0; i < 10; i++) {
            inputs.add(new Object[]{Double.longBitsToDouble(i)});
        }
        for (int i = 0; i < 2000; i++) {
            inputs.add(new Object[]{(double)random.nextInt(2000001) - 1000000});
            inputs.add(new Object[]{(double)random.nextFloat()});
        }
        for (int i = 0; i < 30000; i++) {
            double value = i % 2 == 0
                ? Double.longBitsToDouble(random.nextLong())
                : (random.nextDouble() * 2.0 - 1.0) * Math.pow(10.0, random.nextInt(14) - 7);
            inputs.add(new Object[]{value});
        }
        return inputs;
    }

    // Buffers holding the bytes, each with the registries, for a codec reading a RegistryFriendlyByteBuf to decode.
    private static List<Object[]> registryBuffers(final List<byte[]> payloads) {
        return payloads.stream().map(bytes -> new Object[]{RegistryValues.buffer(bytes)}).toList();
    }

    // One input per entry of Mth's 65536-entry sine table, offset by `shift` entries (cos reads the table a quarter turn ahead).
    private static List<Object[]> sinTableInputs(final int shift) {
        List<Object[]> inputs = new ArrayList<>();
        for (int index = 0; index < 65536; index++) {
            inputs.add(new Object[]{(index + shift + 0.5) / SIN_SCALE});
        }
        return inputs;
    }

    // Angles game code passes, near multiples of pi/64 (where the reduction cancels the most, and
    // tan's poles), every magnitude from 2^low to 2^high, which spans the main path and its
    // short-cuts for small inputs (2^-252 to 90112 for sin and cos, 2^-29 to 12800 for tan), the
    // large-argument path up to 2^1024, tiny and subnormal inputs, and arbitrary bit patterns.
    private static List<Object[]> trigInputs(final long seed, final int low, final int high) {
        Random random = new Random(seed);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < INTRINSIC_INPUTS; i++) {
            double x = switch (i % 6) {
                case 0 -> (random.nextDouble() * 2.0 - 1.0) * 8.0 * Math.PI;
                case 1 -> {
                    double multiple = random.nextInt(1 << 20) * (Math.PI / 64.0);
                    yield multiple + (random.nextInt(9) - 4) * Math.ulp(multiple);
                }
                case 2 -> magnitude(random, low, high);
                case 3 -> magnitude(random, high, 1023);
                case 4 -> magnitude(random, -1074, low);
                default -> randomBits(random);
            };
            inputs.add(new Object[]{random.nextBoolean() ? x : -x});
        }
        return inputs;
    }

    // Around 1, where the result is smallest, (0, 1) like the squared radii nextGaussian takes,
    // every exponent from the subnormals up, and arbitrary bit patterns, negatives included.
    private static List<Object[]> logInputs() {
        Random random = new Random(3);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < INTRINSIC_INPUTS; i++) {
            double x = switch (i % 5) {
                case 0 -> nearOne(random);
                case 1 -> random.nextDouble();
                case 2 -> random.nextDouble() * 1000.0;
                case 3 -> magnitude(random, -1074, 1023);
                default -> randomBits(random);
            };
            inputs.add(new Object[]{x});
        }
        return inputs;
    }

    // Bases and exponents like game code's, small integer exponents of any base, bases near 1
    // (where log2 takes a more accurate path) with large exponents, results across the whole range
    // (subnormal, overflowing and underflowing ones included), negative bases with integer and
    // other exponents, tiny exponents, the exponents with special cases, and arbitrary bit patterns.
    private static List<Object[]> powInputs() {
        Random random = new Random(6);
        double[] specialExponents = {
            0.5, -0.5, 2.0, 1.0, -1.0, 3.0, 0.0, -0.0, Double.POSITIVE_INFINITY, Double.NEGATIVE_INFINITY, Double.NaN
        };
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < INTRINSIC_INPUTS; i++) {
            Object[] input = switch (i % 10) {
                case 0 -> new Object[]{random.nextDouble() * 1000.0, (random.nextDouble() * 2.0 - 1.0) * 8.0};
                case 1 -> new Object[]{sign(random) * magnitude(random, -32, 31), (double)(random.nextInt(41) - 20)};
                case 2 -> new Object[]{sign(random) * nearOne(random), sign(random) * Math.rint(magnitude(random, 0, 63))};
                case 3 -> new Object[]{nearOne(random), sign(random) * magnitude(random, -20, 63)};
                case 4 -> {
                    double x = magnitude(random, -1074, 1023);
                    double log2 = Math.log(x) / Math.log(2.0);
                    yield new Object[]{x, (random.nextDouble() * 2200.0 - 1120.0) / log2};
                }
                case 5 -> new Object[]{-magnitude(random, -1074, 1023), (double)(random.nextInt(41) - 20)};
                case 6 -> {
                    double y = sign(random) * magnitude(random, -4, 59);
                    yield new Object[]{-magnitude(random, -64, 63), random.nextBoolean() ? Math.rint(y) : y};
                }
                case 7 -> new Object[]{magnitude(random, -1074, 1023), sign(random) * magnitude(random, -1074, 0)};
                case 8 -> new Object[]{
                    random.nextBoolean() ? randomBits(random) : sign(random) * nearOne(random),
                    specialExponents[random.nextInt(specialExponents.length)]
                };
                default -> new Object[]{randomBits(random), randomBits(random)};
            };
            inputs.add(input);
        }
        return inputs;
    }

    // Pairs like qrGivensQuat's (floats, often one of them zero), then every path through fdlibm's hypot: exponents
    // about 60 apart (where it returns the sum), both over 2^500 (scaled down first) or under 2^-500 (scaled up),
    // subnormals, the two ways it splits the sum of squares (a - b above b or not, and a = 2b where they meet), the
    // thresholds, and any bits.
    private static List<Object[]> hypotInputs() {
        Random random = new Random(10);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 6000; i++) {
            double[] pair = switch (i % 6) {
                case 0 -> new double[]{JomlValues.component(random), random.nextInt(4) == 0 ? 0.0 : JomlValues.component(random)};
                case 1 -> {
                    double a = magnitude(random, -1000, 1000);
                    yield new double[]{a, Math.scalb(1.0 + random.nextDouble(), Math.getExponent(a) - 56 - random.nextInt(9))};
                }
                case 2 -> new double[]{magnitude(random, 480, 1023), magnitude(random, 400, 1023)};
                case 3 -> new double[]{magnitude(random, -1074, -480), magnitude(random, -1074, -400)};
                case 4 -> {
                    double a = magnitude(random, -60, 60);
                    if (random.nextBoolean()) {
                        double b = a / 2.0;
                        for (int ulps = random.nextInt(5) - 2; ulps != 0; ulps -= Integer.signum(ulps)) {
                            b = ulps > 0 ? Math.nextUp(b) : Math.nextDown(b);
                        }
                        yield new double[]{a, b};
                    }
                    yield new double[]{a, a * (0.25 + random.nextDouble() * 0.75)};
                }
                default -> new double[]{randomBits(random), randomBits(random)};
            };
            int first = random.nextInt(2);
            inputs.add(new Object[]{sign(random) * pair[first], sign(random) * pair[1 - first]});
        }
        double[] thresholds = {
            0x1.00000_ffff_ffffp500, Math.nextUp(0x1.00000_ffff_ffffp500), 0x1.0p-500, Math.nextDown(0x1.0p-500),
            Double.MIN_NORMAL, Math.nextDown(Double.MIN_NORMAL)
        };
        for (double a : thresholds) {
            for (double b : thresholds) {
                inputs.add(new Object[]{a, b});
                inputs.add(new Object[]{a, b * 0x1.0p-30});
            }
        }
        return inputs;
    }

    // Rotations toward ones a little away (where the dot product nears 1, and slerp turns into a lerp at 1 - 1e-6) or
    // anywhere, half of them unit quaternions, a quarter in the other hemisphere (a negative dot product), at Display's
    // progress from 0 to 1 or anything.
    private static List<Object[]> slerpInputs() {
        Random random = new Random(11);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 3000; i++) {
            Quaternionf from = JomlValues.randomQuaternionf(random);
            if (random.nextBoolean()) {
                from.normalize();
            }
            float angle = switch (i % 3) {
                case 0 -> random.nextFloat() * 0.01F;
                case 1 -> Math.scalb(random.nextFloat(), -random.nextInt(24));
                default -> random.nextFloat() * 2.0F * (float)Math.PI;
            };
            Quaternionf to = from.mul(new Quaternionf().rotationAxis(angle, JomlValues.randomVector3f(random).normalize()), new Quaternionf());
            if (random.nextInt(4) == 0) {
                to.set(-to.x, -to.y, -to.z, -to.w);
            }
            float alpha = random.nextInt(8) == 0 ? (Float)Values.FLOATS.random(random) : random.nextFloat();
            inputs.add(new Object[]{from, to, alpha, JomlValues.QUATERNIONF.random(random)});
        }
        return inputs;
    }

    // Matrices next to copies of themselves, equal whatever their properties say, or with one component's zero flipped
    // or moved up an ulp, which makes them differ.
    private static List<Object[]> matrixEqualsInputs() {
        Random random = new Random(12);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 600; i++) {
            Matrix4f matrix = (Matrix4f)JomlValues.MATRIX4F.random(random);
            Matrix4f other = new Matrix4f(matrix);
            switch (i % 3) {
                case 0 -> other.assume(random.nextInt(32));
                case 1 -> {
                    int column = random.nextInt(4);
                    int row = random.nextInt(4);
                    float value = other.get(column, row);
                    other.set(column, row, value == 0.0F ? -value : Math.nextUp(value));
                }
                default -> {
                }
            }
            inputs.add(new Object[]{matrix, other});
        }
        return inputs;
    }

    // What svdDecompose passes: M^T M for the 3x3 part M of a transformation like the game's, for its 5 steps or a few
    // more or fewer. Then symmetric matrices whose first rotation sits where approxGivensQuat switches to a quarter turn,
    // G * a12^2 = (2 * (a11 - a22))^2 give or take a few ulps of a12, which pins down G.
    private static List<Object[]> jacobiInputs() {
        Random random = new Random(13);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 2000; i++) {
            Matrix3f matrix = new Matrix3f(((Transformation)MathValues.TRANSFORMATION.random(random)).getMatrix());
            Matrix3f symmetric = new Matrix3f(matrix).transpose().mul(matrix);
            int steps = i % 4 == 0 ? random.nextInt(11) : 5;
            inputs.add(new Object[]{symmetric, steps, JomlValues.QUATERNIONF.random(random)});
        }
        float g = 3.0F + 2.0F * (float)Math.sqrt(2.0F);
        for (int i = 0; i < 500; i++) {
            float a11 = random.nextFloat() * 8.0F;
            float a22 = random.nextFloat() * 8.0F;
            float ch = 2.0F * (a11 - a22);
            float a12 = (float)(ch / Math.sqrt(g));
            for (int ulps = random.nextInt(7) - 3; ulps != 0; ulps -= Integer.signum(ulps)) {
                a12 = ulps > 0 ? Math.nextUp(a12) : Math.nextDown(a12);
            }
            float a33 = random.nextFloat() * 8.0F;
            inputs.add(new Object[]{new Matrix3f(a11, a12, 0.0F, a12, a22, 0.0F, 0.0F, 0.0F, a33), 1 + random.nextInt(5), new Quaternionf()});
        }
        return inputs;
    }

    // The matrices of transformations like the game's, which Transformation decomposes.
    private static List<Object[]> svdInputs() {
        Random random = new Random(14);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 3000; i++) {
            Transformation transformation = (Transformation)MathValues.TRANSFORMATION.random(random);
            inputs.add(new Object[]{
                new Matrix4f(transformation.getMatrix()), JomlValues.VECTOR3F.random(random), JomlValues.QUATERNIONF.random(random),
                JomlValues.VECTOR3F.random(random), JomlValues.QUATERNIONF.random(random)
            });
        }
        return inputs;
    }

    // Parts like the game's, in every pattern of null (no translation, rotation or scale).
    private static List<Object[]> transformationPartsInputs() {
        Random random = new Random(15);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 1600; i++) {
            int nulls = i % 16;
            inputs.add(new Object[]{
                (nulls & 1) != 0 ? null : MathValues.randomTranslation(random), (nulls & 2) != 0 ? null : MathValues.randomRotation(random),
                (nulls & 4) != 0 ? null : MathValues.randomScale(random), (nulls & 8) != 0 ? null : MathValues.randomRotation(random)
            });
        }
        return inputs;
    }

    // Transformations next to copies of themselves, decomposed or not, or ones built from their matrix with any
    // properties, which are all equal since only the matrix's components count; IDENTITY next to an identity matrix.
    private static List<Object[]> transformationEqualsInputs() {
        Random random = new Random(16);
        List<Object[]> inputs = new ArrayList<>();
        inputs.add(new Object[]{Transformation.IDENTITY, new Transformation(new Matrix4f())});
        for (int i = 0; i < 300; i++) {
            Transformation transformation = (Transformation)MathValues.TRANSFORMATION.random(random);
            Transformation other = switch (i % 3) {
                case 0 -> transformation;
                case 1 -> new Transformation(transformation.getMatrixCopy());
                default -> new Transformation(transformation.getMatrixCopy().assume(random.nextInt(32)));
            };
            inputs.add(new Object[]{transformation, other});
        }
        return inputs;
    }

    // Display entities interpolating from one transformation to the next.
    private static List<Object[]> transformationSlerpInputs() {
        Random random = new Random(17);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 1000; i++) {
            inputs.add(new Object[]{MathValues.TRANSFORMATION.random(random), MathValues.TRANSFORMATION.random(random), random.nextFloat()});
        }
        return inputs;
    }

    // Every angle in every octant often enough to read each entry of Mth.atan2's tables, whose
    // COS_TAB comes from Math.cos, at radii from 2^-20 to 2^20.
    private static List<Object[]> atan2Inputs() {
        Random random = new Random(4);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 4096; i++) {
            double angle = (i + random.nextDouble()) * (2.0 * Math.PI / 4096.0);
            double radius = Math.scalb(1.0 + random.nextDouble(), random.nextInt(41) - 20);
            inputs.add(new Object[]{radius * Math.sin(angle), radius * Math.cos(angle)});
        }
        return inputs;
    }

    // Boxes like the game culls, relative to a camera anywhere in the world: chunk sections around it, entity-sized
    // boxes near it, the cubes offsetToFullyIncludeCameraCube(8) tests (the camera's 8-block cube, stepped back along
    // the view), single points, boxes of any size (inverted ones included), and boxes with a corner on a plane. Then
    // every frustum edge with a box in front of an unrotated camera that has each float edge in each coordinate.
    private static List<Object[]> frustumBoxes() {
        Random random = new Random(8);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 8000; i++) {
            FrustumIntersection frustum = (FrustumIntersection)JomlValues.FRUSTUM_INTERSECTION.random(random);
            if (i % 8 >= 5) {
                inputs.add(boxOnPlane(frustum, random));
                continue;
            }
            double camX = (random.nextDouble() * 2.0 - 1.0) * 1000.0;
            double camY = -64.0 + random.nextDouble() * 384.0;
            double camZ = (random.nextDouble() * 2.0 - 1.0) * 1000.0;
            double[] box = switch (i % 8) {
                case 0 -> {
                    double x = (Math.floor(camX / 16.0) + random.nextInt(25) - 12) * 16.0;
                    double y = (Math.floor(camY / 16.0) + random.nextInt(9) - 4) * 16.0;
                    double z = (Math.floor(camZ / 16.0) + random.nextInt(25) - 12) * 16.0;
                    yield new double[]{x, y, z, x + 16.0, y + 16.0, z + 16.0};
                }
                case 1 -> {
                    double x = camX + (random.nextDouble() * 2.0 - 1.0) * 48.0;
                    double y = camY + (random.nextDouble() * 2.0 - 1.0) * 24.0;
                    double z = camZ + (random.nextDouble() * 2.0 - 1.0) * 48.0;
                    double width = random.nextBoolean() ? 0.6 : random.nextDouble() * 4.0;
                    double height = random.nextBoolean() ? 1.8 : random.nextDouble() * 4.0;
                    yield new double[]{x - width / 2.0, y, z - width / 2.0, x + width / 2.0, y + height, z + width / 2.0};
                }
                case 2 -> {
                    double step = random.nextInt(8) * 4.0;
                    double x = camX - (random.nextDouble() * 2.0 - 1.0) * step;
                    double y = camY - (random.nextDouble() * 2.0 - 1.0) * step;
                    double z = camZ - (random.nextDouble() * 2.0 - 1.0) * step;
                    yield new double[]{
                        Math.floor(x / 8.0) * 8.0, Math.floor(y / 8.0) * 8.0, Math.floor(z / 8.0) * 8.0,
                        Math.ceil(x / 8.0) * 8.0, Math.ceil(y / 8.0) * 8.0, Math.ceil(z / 8.0) * 8.0
                    };
                }
                case 3 -> {
                    // Half of them around the near plane, which is 0.05 away.
                    double range = random.nextBoolean() ? 300.0 : 0.1;
                    double x = camX + (random.nextDouble() * 2.0 - 1.0) * range;
                    double y = camY + (random.nextDouble() * 2.0 - 1.0) * range;
                    double z = camZ + (random.nextDouble() * 2.0 - 1.0) * range;
                    yield new double[]{x, y, z, x, y, z};
                }
                default -> {
                    double[] corners = new double[6];
                    for (int axis = 0; axis < 3; axis++) {
                        double cam = axis == 0 ? camX : axis == 1 ? camY : camZ;
                        double min = cam + (random.nextDouble() * 2.0 - 1.0) * 512.0;
                        corners[axis] = min;
                        corners[axis + 3] = min + (random.nextInt(8) == 0 ? -1.0 : 1.0) * random.nextDouble() * 256.0;
                    }
                    yield corners;
                }
            };
            inputs.add(new Object[]{
                frustum, (float)(box[0] - camX), (float)(box[1] - camY), (float)(box[2] - camZ),
                (float)(box[3] - camX), (float)(box[4] - camY), (float)(box[5] - camZ)
            });
        }

        float[] inFront = {-1.0F, -1.0F, -11.0F, 1.0F, 1.0F, -9.0F};
        for (Object frustum : JomlValues.FRUSTUM_INTERSECTION.edges()) {
            for (int coordinate = 0; coordinate < 6; coordinate++) {
                for (Object edge : Values.FLOATS.edges()) {
                    Object[] input = {frustum, inFront[0], inFront[1], inFront[2], inFront[3], inFront[4], inFront[5]};
                    input[1 + coordinate] = edge;
                    inputs.add(input);
                }
            }
        }
        return inputs;
    }

    // A box of up to 16 blocks a side with its corner furthest along a plane's normal, or nearest, on the plane: which
    // side that corner rounds to depends on the order the plane's terms are added in.
    private static Object[] boxOnPlane(final FrustumIntersection frustum, final Random random) {
        int plane = random.nextInt(6);
        float[] normal = JomlValues.frustumPlane(frustum, plane);
        float[] corner = pointOnPlane(frustum, plane, random, 256.0);
        boolean furthest = random.nextBoolean();
        Object[] input = new Object[7];
        input[0] = frustum;
        for (int axis = 0; axis < 3; axis++) {
            float size = random.nextInt(4) == 0 ? 0.0F : random.nextFloat() * 16.0F;
            // The furthest corner takes the min where the normal is negative, the nearest one the max.
            boolean cornerIsMin = (normal[axis] < 0.0F) == furthest;
            input[1 + axis] = cornerIsMin ? corner[axis] : corner[axis] - size;
            input[4 + axis] = cornerIsMin ? corner[axis] + size : corner[axis];
        }
        return input;
    }

    // Points relative to the camera, near it and far, like Frustum.pointInFrustum's, and twice as many on a plane.
    private static List<Object[]> frustumPoints() {
        Random random = new Random(9);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 6000; i++) {
            FrustumIntersection frustum = (FrustumIntersection)JomlValues.FRUSTUM_INTERSECTION.random(random);
            if (i % 3 != 0) {
                float[] point = pointOnPlane(frustum, random.nextInt(6), random, random.nextBoolean() ? 4.0 : 256.0);
                inputs.add(new Object[]{frustum, point[0], point[1], point[2]});
                continue;
            }
            double range = i % 2 == 0 ? 4.0 : 512.0;
            inputs.add(new Object[]{
                frustum, (float)((random.nextDouble() * 2.0 - 1.0) * range), (float)((random.nextDouble() * 2.0 - 1.0) * range),
                (float)((random.nextDouble() * 2.0 - 1.0) * range)
            });
        }
        return inputs;
    }

    // A point on one of the frustum's planes, as near as floats get, then moved up to two ulps along the axis the plane
    // depends on most, so which side of the plane it's on depends on the order the plane's terms are added in. It's
    // also at a random distance inside one plane of each other pair, so it's often on the frustum's face, where the
    // other planes let it through. Planes that don't pin down a point get the other two coordinates within the range.
    private static float[] pointOnPlane(final FrustumIntersection frustum, final int index, final Random random, final double range) {
        float[] plane = JomlValues.frustumPlane(frustum, index);
        int second = 2 * ((index / 2 + 1) % 3) + random.nextInt(2);
        int third = 2 * ((index / 2 + 2) % 3) + random.nextInt(2);
        double scale = switch (random.nextInt(3)) {
            case 0 -> 0.01;
            case 1 -> 1.0;
            default -> range;
        };
        double[] point = solvePlanes(
            plane, JomlValues.frustumPlane(frustum, second), JomlValues.frustumPlane(frustum, third),
            random.nextDouble() * scale, random.nextDouble() * scale
        );
        int axis = 0;
        for (int i = 1; i < 3; i++) {
            if (Math.abs(plane[i]) > Math.abs(plane[axis])) {
                axis = i;
            }
        }
        if (point == null) {
            point = new double[3];
            double sum = plane[3];
            for (int i = 0; i < 3; i++) {
                if (i != axis) {
                    point[i] = (float)((random.nextDouble() * 2.0 - 1.0) * range);
                    sum += plane[i] * point[i];
                }
            }
            point[axis] = -sum / plane[axis];
        }
        float[] rounded = {(float)point[0], (float)point[1], (float)point[2]};
        for (int ulps = random.nextInt(5) - 2; ulps != 0; ulps -= Integer.signum(ulps)) {
            rounded[axis] = ulps > 0 ? Math.nextUp(rounded[axis]) : Math.nextDown(rounded[axis]);
        }
        return rounded;
    }

    // The point at distance 0 from plane a, b from plane b and c from plane c (planes are [x, y, z, w], and a point's
    // distance is x * px + y * py + z * pz + w), by Cramer's rule, or null when the planes don't meet in one point.
    private static double[] solvePlanes(final float[] a, final float[] b, final float[] c, final double distanceB, final double distanceC) {
        double[][] rows = {{a[0], a[1], a[2]}, {b[0], b[1], b[2]}, {c[0], c[1], c[2]}};
        double[] rhs = {-a[3], distanceB - b[3], distanceC - c[3]};
        double determinant = determinant(rows);
        if (!Double.isFinite(determinant) || Math.abs(determinant) < 1.0E-6) {
            return null;
        }
        double[] point = new double[3];
        for (int axis = 0; axis < 3; axis++) {
            double[][] replaced = {rows[0].clone(), rows[1].clone(), rows[2].clone()};
            for (int row = 0; row < 3; row++) {
                replaced[row][axis] = rhs[row];
            }
            point[axis] = determinant(replaced) / determinant;
        }
        return Double.isFinite(point[0]) && Double.isFinite(point[1]) && Double.isFinite(point[2]) ? point : null;
    }

    private static double determinant(final double[][] m) {
        return m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
            - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
            + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0]);
    }

    // NaNs other than the canonical one, quiet and signalling, of either sign, which floatToIntBits makes canonical.
    private static List<Object[]> floatNaNs() {
        List<Object[]> inputs = new ArrayList<>();
        for (int bits : new int[]{0x7f800001, 0x7fc00001, 0x7fffffff, 0xff800001, 0xffc00000, 0xffffffff}) {
            inputs.add(new Object[]{Float.intBitsToFloat(bits)});
        }
        return inputs;
    }

    private static List<Object[]> doubleNaNs() {
        List<Object[]> inputs = new ArrayList<>();
        for (long bits : new long[]{0x7ff0000000000001L, 0x7ff8000000000001L, 0x7fffffffffffffffL, 0xfff0000000000001L, 0xfff8000000000000L, -1L}) {
            inputs.add(new Object[]{Double.longBitsToDouble(bits)});
        }
        return inputs;
    }

    // An enum's names as byName takes them (serialized, lower case) and as valueOf does (the constant's), in other cases,
    // and ones neither knows.
    private static List<Object[]> names(final Enum<?>[] constants) {
        List<Object[]> inputs = new ArrayList<>();
        for (Enum<?> constant : constants) {
            String name = constant.name();
            String serialized = constant instanceof StringRepresentable representable ? representable.getSerializedName() : name.toLowerCase(Locale.ROOT);
            inputs.add(new Object[]{name});
            inputs.add(new Object[]{serialized});
            inputs.add(new Object[]{name.charAt(0) + name.substring(1).toLowerCase(Locale.ROOT)});
            inputs.add(new Object[]{" " + serialized});
        }
        inputs.add(new Object[]{"minecraft:north"});
        inputs.add(new Object[]{"null"});
        return inputs;
    }

    // Small vectors, so two or three components often tie for the largest, with every fallback, null included.
    private static List<Object[]> nearestInputs(final boolean vec3i) {
        Random random = new Random(18);
        List<Direction> fallbacks = new ArrayList<>(Arrays.asList(Direction.values()));
        fallbacks.add(null);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 700; i++) {
            int x = random.nextInt(7) - 3;
            int y = random.nextInt(7) - 3;
            int z = random.nextInt(7) - 3;
            Direction fallback = fallbacks.get(i % fallbacks.size());
            inputs.add(vec3i ? new Object[]{new Vec3i(x, y, z), fallback} : new Object[]{x, y, z, fallback});
        }
        return inputs;
    }

    // Every vector made of components that tie, are too small for the dot product to pass Float.MIN_VALUE (which is
    // positive, so nothing beats NORTH unless it's above it), or are infinite or NaN, as floats, doubles or a Vec3.
    private static List<Object[]> approximateNearestInputs(final int kind) {
        float[] floats = {0.0F, -0.0F, 1.0F, -1.0F, Float.MIN_VALUE, -Float.MIN_VALUE, 2.0F * Float.MIN_VALUE, Float.POSITIVE_INFINITY, Float.NaN};
        double[] doubles = {0.0, -0.0, 1.0, -1.0, Double.MIN_VALUE, 0x1.0p-149, 0x1.8p-149, 1.0E300, Double.NaN};
        List<Object[]> inputs = new ArrayList<>();
        for (int x = 0; x < 9; x++) {
            for (int y = 0; y < 9; y++) {
                for (int z = 0; z < 9; z++) {
                    inputs.add(switch (kind) {
                        case 0 -> new Object[]{floats[x], floats[y], floats[z]};
                        case 1 -> new Object[]{doubles[x], doubles[y], doubles[z]};
                        default -> new Object[]{new Vec3(doubles[x], doubles[y], doubles[z])};
                    });
                }
            }
        }
        return inputs;
    }

    // Every multiple of 45 degrees from -720 to 720, where fromYRot switches direction, and an ulp either side.
    private static List<Object[]> yRotBoundaries() {
        List<Object[]> inputs = new ArrayList<>();
        for (int k = -16; k <= 16; k++) {
            double angle = k * 45.0;
            inputs.add(new Object[]{Math.nextDown(angle)});
            inputs.add(new Object[]{angle});
            inputs.add(new Object[]{Math.nextUp(angle)});
        }
        return inputs;
    }

    // Each direction facing every degree of a turn and a bit, and the quarter turns where the dot product is about 0.
    private static List<Object[]> facingAngles() {
        List<Object[]> inputs = new ArrayList<>();
        for (Direction direction : Direction.values()) {
            for (int degrees = -450; degrees <= 450; degrees++) {
                inputs.add(new Object[]{direction, (float)degrees});
            }
            for (int k = -4; k <= 4; k++) {
                float angle = k * 90.0F;
                inputs.add(new Object[]{direction, Math.nextDown(angle)});
                inputs.add(new Object[]{direction, Math.nextUp(angle)});
            }
        }
        return inputs;
    }

    // BlockModelRotation's matrices, one per OctahedralGroup, which UnbakedCuboidGeometry turns faces' cull directions by.
    private static boolean elementInMatrix3f(final int column, final int row) {
        return column >= 0 && column <= 2 && row >= 0 && row <= 2;
    }

    // Every element of an identity and of a rotation, set to 1, -0.0 and NaN.
    private static List<Object[]> matrix3fElements() {
        List<Object[]> inputs = new ArrayList<>();
        for (Matrix3f matrix : List.of(new Matrix3f(), new Matrix3f().rotationXYZ(0.5F, 1.0F, 1.5F))) {
            for (int column = 0; column <= 2; column++) {
                for (int row = 0; row <= 2; row++) {
                    for (float value : new float[]{1.0F, -0.0F, Float.NaN}) {
                        inputs.add(new Object[]{new Matrix3f(matrix), column, row, value});
                    }
                }
            }
        }
        return inputs;
    }

    private static List<Object[]> modelRotations() {
        List<Object[]> inputs = new ArrayList<>();
        for (OctahedralGroup group : OctahedralGroup.values()) {
            for (Direction direction : Direction.values()) {
                inputs.add(new Object[]{new Matrix4f(group.transformation()), direction});
            }
        }
        return inputs;
    }

    // Movements whose x and z are as large as each other, of either sign.
    private static List<Object[]> axisStepOrderTies() {
        Random random = new Random(19);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 100; i++) {
            double a = i % 10 == 0 ? 0.0 : (random.nextDouble() * 2.0 - 1.0) * 4.0;
            inputs.add(new Object[]{new Vec3(a, random.nextDouble() - 0.5, random.nextBoolean() ? a : -a)});
        }
        return inputs;
    }

    private static List<Object[]> nullDirections(final Object[] receivers) {
        List<Object[]> inputs = new ArrayList<>();
        for (Object receiver : receivers) {
            inputs.add(new Object[]{receiver, null});
        }
        return inputs;
    }

    // Two points exactly as far apart as the distance they're checked against, or an ulp of the distance either side, which
    // the strict comparisons tell apart: blocks (kind 0), a block and a point that far from its center (1), points (2),
    // and points as far apart horizontally and vertically as the cylinder's radius and half height (3).
    private static List<Object[]> distanceTies(final int kind) {
        Random random = new Random(22 + kind);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 300; i++) {
            Vec3i block = CoreValues.randomVec3i(random);
            block = new Vec3i(block.getX() % 1000000, block.getY() % 1000, block.getZ() % 1000000);
            Vec3 point = new Vec3(block.getX() + random.nextInt(8) / 8.0, block.getY() + random.nextInt(8) / 8.0, block.getZ() + random.nextInt(8) / 8.0);
            int apart = random.nextInt(64) - 32;
            int axis = random.nextInt(3);
            double distance = Math.abs(apart);
            distance = switch (random.nextInt(4)) {
                case 0 -> Math.nextDown(distance);
                case 1 -> Math.nextUp(distance);
                default -> distance;
            };
            inputs.add(switch (kind) {
                case 0 -> new Object[]{block, block.relative(Direction.Axis.VALUES[axis], apart), distance};
                case 1 -> new Object[]{block, Vec3.atCenterOf(block).with(Direction.Axis.VALUES[axis], Vec3.atCenterOf(block).get(Direction.Axis.VALUES[axis]) + apart), distance};
                case 2 -> new Object[]{point, point.with(Direction.Axis.VALUES[axis], point.get(Direction.Axis.VALUES[axis]) + apart), distance};
                default -> {
                    int up = random.nextInt(8) - 4;
                    double height = random.nextBoolean() ? Math.abs(up) : Math.abs(up) + random.nextInt(2) * 0.5;
                    Direction.Axis horizontal = random.nextBoolean() ? Direction.Axis.X : Direction.Axis.Z;
                    Vec3 other = point.with(horizontal, point.get(horizontal) + apart).add(0.0, up, 0.0);
                    yield new Object[]{point, other, distance, height};
                }
            });
        }
        return inputs;
    }

    // Vectors whose length is within a few ulps of normalize's threshold, (double)1.0E-5F, either side of it, first along an
    // axis, where the length comes out exactly (the square root of a rounded square is the number again).
    private static List<Object[]> normalizeThresholds() {
        Random random = new Random(20);
        List<Object[]> inputs = new ArrayList<>();
        for (int ulps = -2; ulps <= 2; ulps++) {
            double length = 1.0E-5F;
            for (int step = ulps; step != 0; step -= Integer.signum(step)) {
                length = step > 0 ? Math.nextUp(length) : Math.nextDown(length);
            }
            for (Direction direction : Direction.values()) {
                inputs.add(new Object[]{direction.getUnitVec3().scale(length)});
            }
        }
        for (int i = 0; i < 500; i++) {
            Vec3 unit = Vec3.directionFromRotation((random.nextFloat() * 2.0F - 1.0F) * 90.0F, (random.nextFloat() * 2.0F - 1.0F) * 180.0F);
            double length = 1.0E-5F;
            for (int ulps = random.nextInt(9) - 4; ulps != 0; ulps -= Integer.signum(ulps)) {
                length = ulps > 0 ? Math.nextUp(length) : Math.nextDown(length);
            }
            inputs.add(new Object[]{new Vec3(unit.x * length, unit.y * length, unit.z * length)});
        }
        return inputs;
    }

    private static List<Object[]> normalizedThresholds() {
        Random random = new Random(21);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 500; i++) {
            float angle = random.nextFloat() * 2.0F * (float)Math.PI;
            float length = 1.0E-4F;
            for (int ulps = random.nextInt(9) - 4; ulps != 0; ulps -= Integer.signum(ulps)) {
                length = ulps > 0 ? Math.nextUp(length) : Math.nextDown(length);
            }
            inputs.add(new Object[]{new Vec2((float)Math.cos(angle) * length, (float)Math.sin(angle) * length)});
        }
        return inputs;
    }

    // The walk scenario's looks, and every whole x rotation (up and down) at y rotations around the circle.
    private static List<Object[]> lookRotations() {
        List<Object[]> inputs = new ArrayList<>();
        inputs.add(new Object[]{0.0F, 0.0F});
        inputs.add(new Object[]{30.0F, 90.0F});
        for (int xRot = -90; xRot <= 90; xRot++) {
            inputs.add(new Object[]{(float)xRot, (xRot + 90) * 2.0F - 180.0F});
        }
        return inputs;
    }

    // Every rotation of every whole step of a turn of 4 (a block's facing) and 16 (a sign's), and of none.
    private static List<Object[]> rotationSteps() {
        List<Object[]> inputs = new ArrayList<>();
        for (Rotation rotation : Rotation.values()) {
            for (int steps : new int[]{4, 16, 0}) {
                for (int index = -1; index <= steps; index++) {
                    inputs.add(new Object[]{rotation, index, steps});
                }
            }
        }
        return inputs;
    }

    // Every block whose coordinates are at BlockPos.asLong's packing limits (26 bits for x and z, 12 for y), or an int
    // past them, which wraps, or near zero.
    private static List<Object[]> packingLimits() {
        int[] horizontal = {-33554433, -33554432, -33554431, -1, 0, 1, 33554431, 33554432};
        int[] vertical = {-2049, -2048, -2047, -64, -1, 0, 1, 319, 2047, 2048};
        List<Object[]> inputs = new ArrayList<>();
        for (int x : horizontal) {
            for (int y : vertical) {
                for (int z : horizontal) {
                    inputs.add(new Object[]{x, y, z});
                }
            }
        }
        return inputs;
    }

    // Blocks packed with BlockPos.asLong: at the packing limits, around the walk scenario's spawn and anywhere a BlockPos
    // packs, and any longs. Kind 0 passes the long alone, 1 with each direction, 2 with steps, and 3 to a
    // MutableBlockPos's set.
    private static List<Object[]> packedInputs(final int kind) {
        Random random = new Random(30 + kind);
        List<Long> packed = new ArrayList<>();
        for (Object[] limits : packingLimits()) {
            packed.add(BlockPos.asLong((Integer)limits[0], (Integer)limits[1], (Integer)limits[2]));
        }
        for (int i = 0; i < 300; i++) {
            packed.add(CoreValues.randomBlockPos(random).asLong());
        }
        for (int i = 0; i < 60; i++) {
            packed.add(random.nextLong());
        }

        List<Object[]> inputs = new ArrayList<>();
        for (long node : packed) {
            switch (kind) {
                case 0 -> inputs.add(new Object[]{node});
                case 1 -> {
                    for (Direction direction : Direction.values()) {
                        inputs.add(new Object[]{node, direction});
                    }
                }
                case 2 -> inputs.add(new Object[]{node, random.nextInt(5) - 2, random.nextInt(5) - 2, random.nextInt(5) - 2});
                default -> inputs.add(new Object[]{CoreValues.randomBlockPos(random).mutable(), node});
            }
        }
        return inputs;
    }

    // Points on the bounds clampLocationWithin works out for a block, an ulp either side, inside and outside, for blocks
    // around the walk scenario's spawn and far out, where the lower bound's float sum rounds (and past 2^24, the block's
    // coordinate too).
    private static List<Object[]> clampBounds() {
        Random random = new Random(33);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 2000; i++) {
            BlockPos block = CoreValues.randomBlockPos(random);
            double[] point = new double[3];
            for (int axis = 0; axis < 3; axis++) {
                int coordinate = block.get(Direction.Axis.VALUES[axis]);
                double min = coordinate + 1.0E-5F;
                double max = coordinate + 1.0 - 1.0E-5F;
                point[axis] = switch (random.nextInt(9)) {
                    case 0 -> min;
                    case 1 -> Math.nextDown(min);
                    case 2 -> Math.nextUp(min);
                    case 3 -> max;
                    case 4 -> Math.nextDown(max);
                    case 5 -> Math.nextUp(max);
                    case 6 -> coordinate + 0.5;
                    case 7 -> coordinate - 3.0;
                    default -> coordinate + 3.0;
                };
            }
            inputs.add(new Object[]{block, new Vec3(point[0], point[1], point[2])});
        }
        return inputs;
    }

    // Reaches like the game's (up to 4 blocks each way, or 8 as one reach), and negative ones, with which the search
    // starts outside the reach or ends at once.
    private static List<Object[]> manhattanReaches(final int reaches) {
        Random random = new Random(34 + reaches);
        List<Object[]> inputs = new ArrayList<>();
        if (reaches == 1) {
            for (int reach = -2; reach <= 8; reach++) {
                for (int i = 0; i < 3; i++) {
                    inputs.add(new Object[]{CoreValues.randomBlockPos(random), reach});
                }
            }
            return inputs;
        }
        for (int x = -2; x <= 4; x++) {
            for (int y = -2; y <= 4; y++) {
                for (int z = -2; z <= 4; z++) {
                    inputs.add(new Object[]{CoreValues.randomBlockPos(random), x, y, z});
                }
            }
        }
        return inputs;
    }

    // The search visits at least one block per depth when every reach is non-negative, so it gets to the 512th block
    // quickly. Otherwise it may find none, and only stops past maxDepth, so that has to be small.
    private static boolean manhattanEnds(final Object[] args, final int maxDepth) {
        return (Integer)args[1] >= 0 && (Integer)args[2] >= 0 && (Integer)args[3] >= 0 || maxDepth <= 64;
    }

    // Boxes up to 9 blocks across around the walk scenario's spawn and anywhere, with their corners either way round (as
    // ints, a min above the max makes the width 0, which leaves nothing to visit, or negative), or as two BlockPos.
    private static List<Object[]> smallBoxes(final boolean blockPositions) {
        Random random = new Random(blockPositions ? 36 : 37);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 300; i++) {
            BlockPos a = CoreValues.randomBlockPos(random);
            BlockPos b = a.offset(random.nextInt(11) - 2, random.nextInt(11) - 2, random.nextInt(11) - 2);
            if (random.nextBoolean()) {
                BlockPos swap = a;
                a = b;
                b = swap;
            }
            inputs.add(blockPositions ? new Object[]{a, b} : new Object[]{a.getX(), a.getY(), a.getZ(), b.getX(), b.getY(), b.getZ()});
        }
        return inputs;
    }

    // Columns from a block up or down to a y up to 16 away, or at it.
    private static List<Object[]> neighborColumns() {
        Random random = new Random(38);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 300; i++) {
            BlockPos start = CoreValues.randomBlockPos(random);
            inputs.add(new Object[]{start.getX(), start.getY(), start.getZ(), start.getY() + random.nextInt(33) - 16});
        }
        return inputs;
    }

    // Every pair of directions (those on one axis throw) with radii up to 3, and none.
    private static List<Object[]> spirals() {
        Random random = new Random(39);
        List<Object[]> inputs = new ArrayList<>();
        for (Direction first : Direction.values()) {
            for (Direction second : Direction.values()) {
                for (int radius = -1; radius <= 3; radius++) {
                    inputs.add(new Object[]{CoreValues.randomBlockPos(random), radius, first, second});
                }
            }
        }
        return inputs;
    }

    // Boxes up to 5 blocks across with their corners either way round, walked along movements like an entity's, ones with
    // zero (of either sign) or NaN components, which pick the order and the start, and ties for axisStepOrder.
    private static List<Object[]> cornerWalks(final boolean blockPositions) {
        Random random = new Random(blockPositions ? 40 : 41);
        double[] components = {0.0, -0.0, 1.0, -1.0, 0.5, -0.5, Double.NaN};
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 500; i++) {
            BlockPos a = CoreValues.randomBlockPos(random);
            BlockPos b = a.offset(random.nextInt(9) - 4, random.nextInt(9) - 4, random.nextInt(9) - 4);
            Vec3 direction = switch (random.nextInt(3)) {
                case 0 -> CoreValues.randomVec3(random);
                case 1 -> new Vec3(components[random.nextInt(7)], components[random.nextInt(7)], components[random.nextInt(7)]);
                default -> {
                    double tie = random.nextDouble() - 0.5;
                    yield new Vec3(tie, random.nextDouble() - 0.5, random.nextBoolean() ? tie : -tie);
                }
            };
            inputs.add(
                blockPositions ? new Object[]{a, b, direction} : new Object[]{a.getX(), a.getY(), a.getZ(), b.getX(), b.getY(), b.getZ(), direction}
            );
        }
        return inputs;
    }

    // Draws up to 20 blocks from boxes up to 5 blocks across, or from a cube up to 4 blocks either way of a center, some
    // empty (a width of 0, which nextInt rejects, but only once a block is drawn).
    private static List<Object[]> randomBoxes(final boolean cube) {
        Random random = new Random(cube ? 42 : 43);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 300; i++) {
            long seed = random.nextLong();
            int limit = random.nextInt(22) - 1;
            BlockPos a = CoreValues.randomBlockPos(random);
            if (cube) {
                inputs.add(new Object[]{seed, limit, a, random.nextInt(6) - 1});
            } else {
                BlockPos b = a.offset(random.nextInt(6) - 1, random.nextInt(6) - 1, random.nextInt(6) - 1);
                inputs.add(new Object[]{seed, limit, a.getX(), a.getY(), a.getZ(), b.getX(), b.getY(), b.getZ()});
            }
        }
        return inputs;
    }

    // Each box against a copy of itself, and against a copy with one side an ulp away, or a zero or NaN of the other sign:
    // equals compares each side with Double.compare, which tells those apart (and any NaN from a number).
    private static List<Object[]> boxEqualsInputs() {
        Random random = new Random(50);
        List<Object> edges = CoreValues.BOX.edges();
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 400; i++) {
            AABB box = i < edges.size() ? (AABB)edges.get(i) : CoreValues.randomBox(random);
            inputs.add(new Object[]{box, box});
            double[] sides = sides(box);
            int side = random.nextInt(6);
            double value = sides[side];
            if (Double.isNaN(value)) {
                sides[side] = Double.doubleToRawLongBits(value) < 0 ? Double.NaN : Double.longBitsToDouble(0xfff8000000000000L);
            } else if (value == 0.0) {
                sides[side] = -value;
            } else {
                sides[side] = random.nextBoolean() ? Math.nextUp(value) : Math.nextDown(value);
            }
            inputs.add(new Object[]{box, new AABB(sides[0], sides[1], sides[2], sides[3], sides[4], sides[5])});
        }
        return inputs;
    }

    // Blocks whose coordinates are at the int limits, where the far side's int sum wraps, or near them or zero.
    private static List<Object[]> limitBlockPairs() {
        Random random = new Random(63);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 300; i++) {
            BlockPos a = limitBlock(random);
            inputs.add(new Object[]{a, random.nextBoolean() ? a : limitBlock(random)});
        }
        return inputs;
    }

    // Moves with zeros of both signs, NaN and small steps, on boxes with -0.0 and 0.0 sides, where whether a zero counts as
    // moving decides the sign of the side it moves.
    private static List<Object[]> signedZeroMoves(final boolean vec3) {
        double[] components = {0.0, -0.0, Double.NaN, 1.0, -1.0, 1.0E-300};
        List<AABB> boxes = List.of(
            new AABB(-0.0, -0.0, -0.0, 0.0, 0.0, 0.0), new AABB(-0.0, -0.0, -0.0, -0.0, -0.0, -0.0), new AABB(0.0, 0.0, 0.0, 0.0, 0.0, 0.0),
            new AABB(-1.0, -0.0, 0.0, -0.0, 0.0, 1.0)
        );
        List<Object[]> inputs = new ArrayList<>();
        for (AABB box : boxes) {
            for (double x : components) {
                for (double y : components) {
                    for (double z : components) {
                        inputs.add(vec3 ? new Object[]{box, new Vec3(x, y, z)} : new Object[]{box, x, y, z});
                    }
                }
            }
        }
        return inputs;
    }

    // Boxes like the game's whose sides meet along an axis, overlap by an ulp, or are an ulp apart, and along another axis
    // too now and then: an entity on a block, against a wall or under a ceiling, and boxes side by side. Kind 0 passes
    // the other box (either one may be the receiver), 1 its sides, 2 two of its corners (either way round along each
    // axis), and 3 the block.
    private static List<Object[]> touchingBoxes(final int kind) {
        Random random = new Random(51 + kind);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 600; i++) {
            BlockPos pos = switch (random.nextInt(4)) {
                case 0 -> limitBlock(random);
                case 1 -> CoreValues.randomBlockPos(random);
                default -> nearSpawn(random);
            };
            AABB other = kind == 3 || random.nextBoolean() ? new AABB(pos) : CoreValues.randomBox(random);
            Vec3 center = other.getCenter();
            Vec3 feet = new Vec3(center.x + spread(random), other.minY + spread(random), center.z + spread(random));
            AABB box = CoreValues.randomEntityBox(random, feet);
            int axis = random.nextInt(3);
            box = meeting(box, other, axis, random);
            if (random.nextInt(3) == 0) {
                box = meeting(box, other, (axis + 1 + random.nextInt(2)) % 3, random);
            }

            double[] sides = sides(other);
            inputs.add(switch (kind) {
                case 0 -> random.nextBoolean() ? new Object[]{box, other} : new Object[]{other, box};
                case 1 -> new Object[]{box, sides[0], sides[1], sides[2], sides[3], sides[4], sides[5]};
                case 2 -> {
                    double[] a = new double[3];
                    double[] b = new double[3];
                    for (int j = 0; j < 3; j++) {
                        boolean swap = random.nextBoolean();
                        a[j] = swap ? sides[j + 3] : sides[j];
                        b[j] = swap ? sides[j] : sides[j + 3];
                    }
                    yield new Object[]{box, new Vec3(a[0], a[1], a[2]), new Vec3(b[0], b[1], b[2])};
                }
                default -> new Object[]{box, pos};
            });
        }
        return inputs;
    }

    // The box moved along the axis so one of its sides is on the other's opposite side, or an ulp either way. Its size
    // along the axis is kept as nearly as doubles allow.
    private static AABB meeting(final AABB box, final AABB other, final int axis, final Random random) {
        double[] sides = sides(box);
        double[] others = sides(other);
        double size = sides[axis + 3] - sides[axis];
        int ulps = random.nextInt(3) - 1;
        if (random.nextBoolean()) {
            sides[axis] = ulps(others[axis + 3], ulps);
            sides[axis + 3] = sides[axis] + size;
        } else {
            sides[axis + 3] = ulps(others[axis], ulps);
            sides[axis] = sides[axis + 3] - size;
        }
        return new AABB(sides[0], sides[1], sides[2], sides[3], sides[4], sides[5]);
    }

    // Points on a box's sides (contains takes its min sides in and its max sides out), an ulp either side, in the middle and
    // outside, for any box. Kind 0 passes a Vec3, 1 the coordinates.
    private static List<Object[]> facePoints(final int kind) {
        Random random = new Random(55 + kind);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 600; i++) {
            AABB box = CoreValues.randomBox(random);
            double[] sides = sides(box);
            double[] point = new double[3];
            for (int axis = 0; axis < 3; axis++) {
                double min = sides[axis];
                double max = sides[axis + 3];
                point[axis] = switch (random.nextInt(8)) {
                    case 0 -> min;
                    case 1 -> ulps(min, random.nextBoolean() ? 1 : -1);
                    case 2 -> max;
                    case 3 -> ulps(max, random.nextBoolean() ? 1 : -1);
                    case 4 -> min - 1.0;
                    case 5 -> max + 1.0;
                    default -> Mth.lerp(random.nextDouble(), min, max);
                };
            }
            inputs.add(kind == 0 ? new Object[]{box, new Vec3(point[0], point[1], point[2])} : new Object[]{box, point[0], point[1], point[2]});
        }
        return inputs;
    }

    // Rays like the game's picks and projectiles: from near a block along a look vector, as far as the block interaction
    // range (4.5) or the entity one (3) or 20 blocks, aimed at a box's center, a point inside it, a corner, an edge or a
    // side, or at a corner along a diagonal, where two sides are hit at the same scale. Also rays along an axis that pass a
    // side at clipPoint's 1.0E-7 margin, or an ulp either side of it, rays that start inside a box or on a side, end on a
    // side or move less than the 1.0E-7 that counts as moving along an axis, and rays of length 0. The boxes are a block's
    // shapes at the block. Kind 0 passes the box, 1 its sides, 2 the block's shapes relative to it and the block, and 3 a
    // ray's start and direction (Mth.rayIntersectsAABB).
    private static List<Object[]> clipRays(final int kind) {
        Random random = new Random(57 + kind);
        List<Object> shapes = CoreValues.BOX_LIST.edges();
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 2000; i++) {
            BlockPos pos = random.nextInt(4) == 0 ? CoreValues.randomBlockPos(random) : nearSpawn(random);
            @SuppressWarnings("unchecked")
            List<AABB> shape = switch (random.nextInt(3)) {
                case 0 -> (List<AABB>)shapes.get(1 + random.nextInt(shapes.size() - 1));
                case 1 -> List.of(CoreValues.blockShape(random));
                default -> List.of(CoreValues.blockShape(random), CoreValues.blockShape(random));
            };
            AABB box = shape.get(random.nextInt(shape.size())).move(pos);
            Vec3[] ray = ray(box, random);
            if (random.nextInt(8) == 0) {
                Object[] threshold = thresholdRay(random);
                pos = (BlockPos)threshold[0];
                shape = List.of(new AABB(0.0, 0.0, 0.0, 1.0, 1.0, 1.0));
                box = new AABB(pos);
                ray = new Vec3[]{(Vec3)threshold[1], (Vec3)threshold[2]};
            }
            double[] sides = sides(box);
            inputs.add(switch (kind) {
                case 0 -> new Object[]{box, ray[0], ray[1]};
                case 1 -> new Object[]{sides[0], sides[1], sides[2], sides[3], sides[4], sides[5], ray[0], ray[1]};
                case 2 -> new Object[]{random.nextInt(50) == 0 ? List.of() : shape, ray[0], ray[1], pos};
                default -> new Object[]{ray[0], ray[1].subtract(ray[0]), box};
            });
        }

        // Rays along an axis that pass one of the other axes' sides at clipPoint's 1.0E-7 margin, give or take two ulps,
        // where the order the margin is added in decides whether they hit.
        Random margins = new Random(70 + kind);
        for (int i = 0; i < 1000; i++) {
            BlockPos pos = margins.nextInt(4) == 0 ? CoreValues.randomBlockPos(margins) : nearSpawn(margins);
            AABB shape = CoreValues.blockShape(margins);
            AABB box = shape.move(pos);
            double[] sides = sides(box);
            int axis = margins.nextInt(3);
            int edge = (axis + 1 + margins.nextInt(2)) % 3;
            double[] point = new double[3];
            for (int other = 0; other < 3; other++) {
                point[other] = Mth.lerp(margins.nextDouble(), sides[other], sides[other + 3]);
            }
            point[edge] = margins.nextBoolean() ? ulps(sides[edge] - 1.0E-7, margins.nextInt(5) - 2) : ulps(sides[edge + 3] + 1.0E-7, margins.nextInt(5) - 2);
            double sign = margins.nextBoolean() ? 1.0 : -1.0;
            point[axis] = sign > 0.0 ? sides[axis] - 0.5 : sides[axis + 3] + 0.5;
            Vec3 from = new Vec3(point[0], point[1], point[2]);
            Vec3 to = from.with(Direction.Axis.VALUES[axis], point[axis] + sign * 2.0);
            inputs.add(switch (kind) {
                case 0 -> new Object[]{box, from, to};
                case 1 -> new Object[]{sides[0], sides[1], sides[2], sides[3], sides[4], sides[5], from, to};
                case 2 -> new Object[]{List.of(shape), from, to, pos};
                default -> new Object[]{from, to.subtract(from), box};
            });
        }
        return inputs;
    }

    // A ray that moves 1.0E-7 along an axis, an ulp more or an ulp less, through a whole block's side at 0, so it counts as
    // moving along the axis or doesn't: it starts and ends half of that either side of the side, which doubles hold exactly
    // near 0. Along the other axes it stays over the side, and now and then starts at -0.0, where the end's sign decides the
    // sign of the hit. Returns the block, the ray's start and its end.
    private static Object[] thresholdRay(final Random random) {
        int axis = random.nextInt(3);
        boolean positive = random.nextBoolean();
        double[] moves = {1.0E-7, Math.nextUp(1.0E-7), Math.nextDown(1.0E-7)};
        double half = moves[random.nextInt(moves.length)] / 2.0;
        BlockPos near = nearSpawn(random);
        int[] block = {near.getX(), near.getY(), near.getZ()};
        double[] from = new double[3];
        double[] to = new double[3];
        for (int other = 0; other < 3; other++) {
            if (other == axis) {
                block[other] = positive ? 0 : -1;
                from[other] = positive ? -half : half;
                to[other] = -from[other];
            } else if (random.nextInt(3) == 0) {
                double[] ends = {-0.0, 0.0, -Double.MIN_VALUE, Double.MIN_VALUE};
                block[other] = random.nextBoolean() ? 0 : -1;
                from[other] = -0.0;
                to[other] = ends[random.nextInt(ends.length)];
            } else {
                from[other] = block[other] + random.nextDouble();
                to[other] = random.nextBoolean() ? from[other] : from[other] + (random.nextDouble() - 0.5) * 0.5;
            }
        }
        return new Object[]{new BlockPos(block[0], block[1], block[2]), new Vec3(from[0], from[1], from[2]), new Vec3(to[0], to[1], to[2])};
    }

    // A ray's start and end around a box.
    private static Vec3[] ray(final AABB box, final Random random) {
        double[] sides = sides(box);
        double[] reaches = {4.5, 3.0, 20.0, 1.0};
        double reach = reaches[random.nextInt(reaches.length)];
        Vec3 look = Vec3.directionFromRotation((random.nextFloat() * 2.0F - 1.0F) * 90.0F, (random.nextFloat() * 2.0F - 1.0F) * 180.0F);
        switch (random.nextInt(8)) {
            case 0, 1 -> {
                Vec3 aim = aim(sides, random);
                Vec3 from = aim.subtract(look.scale(random.nextDouble() * reach));
                return new Vec3[]{from, from.add(look.scale(reach))};
            }
            case 2 -> {
                // At a corner along a diagonal, from a whole number of blocks away, so both sides' scales come out the same.
                double[] corner = new double[3];
                double[] diagonal = new double[3];
                for (int axis = 0; axis < 3; axis++) {
                    boolean max = random.nextBoolean();
                    corner[axis] = max ? sides[axis + 3] : sides[axis];
                    diagonal[axis] = axis == 2 && random.nextBoolean() ? 0.0 : max ? -1.0 : 1.0;
                }
                double distance = 1 + random.nextInt(3);
                Vec3 direction = new Vec3(diagonal[0], diagonal[1], diagonal[2]);
                Vec3 from = new Vec3(corner[0], corner[1], corner[2]).subtract(direction.scale(distance));
                return new Vec3[]{from, from.add(direction.scale(distance * 2.0))};
            }
            case 3 -> {
                // Along an axis, passing the other axes' sides at the margin, or an ulp either side of it.
                int axis = random.nextInt(3);
                double[] point = new double[3];
                for (int other = 0; other < 3; other++) {
                    point[other] = switch (random.nextInt(4)) {
                        case 0 -> ulps(sides[other] - 1.0E-7, random.nextInt(3) - 1);
                        case 1 -> ulps(sides[other + 3] + 1.0E-7, random.nextInt(3) - 1);
                        case 2 -> random.nextBoolean() ? sides[other] : sides[other + 3];
                        default -> Mth.lerp(random.nextDouble(), sides[other], sides[other + 3]);
                    };
                }
                double sign = random.nextBoolean() ? 1.0 : -1.0;
                point[axis] = sign > 0.0 ? sides[axis] - 1.0 - random.nextDouble() : sides[axis + 3] + 1.0 + random.nextDouble();
                Vec3 from = new Vec3(point[0], point[1], point[2]);
                return new Vec3[]{from, from.with(Direction.Axis.VALUES[axis], from.get(Direction.Axis.VALUES[axis]) + sign * reach)};
            }
            case 4 -> {
                // From inside, or from a side, into the box or out of it.
                Vec3 from = random.nextBoolean() ? aim(sides, random) : new Vec3(sides[0], Mth.lerp(0.5, sides[1], sides[4]), Mth.lerp(0.5, sides[2], sides[5]));
                return new Vec3[]{from, from.add(random.nextBoolean() ? look.scale(reach) : new Vec3(1.0, 0.0, 0.0))};
            }
            case 5 -> {
                // Ending on a side.
                Vec3 to = new Vec3(sides[0], Mth.lerp(random.nextDouble(), sides[1], sides[4]), Mth.lerp(random.nextDouble(), sides[2], sides[5]));
                return new Vec3[]{to.subtract(random.nextBoolean() ? new Vec3(1.0, 0.0, 0.0) : look.scale(reach)), to};
            }
            case 6 -> {
                // Moving about 1.0E-7 or less along some axes.
                double[] tiny = {1.0E-7, -1.0E-7, Math.nextUp(1.0E-7), -Math.nextUp(1.0E-7), Math.nextDown(1.0E-7), 0.0, -0.0};
                Vec3 from = aim(sides, random).subtract(look);
                double[] delta = {look.x * 2.0, look.y * 2.0, look.z * 2.0};
                for (int axis = 0; axis < 3; axis++) {
                    if (random.nextBoolean()) {
                        delta[axis] = tiny[random.nextInt(tiny.length)];
                    }
                }
                return new Vec3[]{from, from.add(delta[0], delta[1], delta[2])};
            }
            default -> {
                Vec3 at = random.nextBoolean() ? aim(sides, random) : CoreValues.randomVec3(random);
                return new Vec3[]{at, random.nextBoolean() ? at : CoreValues.randomVec3(random)};
            }
        }
    }

    // The box's center, a point inside it, one of its corners, a point on an edge, or a point on a side.
    private static Vec3 aim(final double[] sides, final Random random) {
        int kind = random.nextInt(5);
        double[] point = new double[3];
        int free = random.nextInt(3);
        for (int axis = 0; axis < 3; axis++) {
            double min = sides[axis];
            double max = sides[axis + 3];
            boolean onSide = kind == 2 || kind == 3 && axis != free || kind == 4 && axis == free;
            point[axis] = kind == 0 ? Mth.lerp(0.5, min, max) : onSide ? random.nextBoolean() ? min : max : Mth.lerp(random.nextDouble(), min, max);
        }
        return new Vec3(point[0], point[1], point[2]);
    }

    // Entities moving like the game's (walking, falling, a projectile's step, toward a box, or about 1.0E-7 along an axis)
    // past a fluid's box (a block with its fluid's height, the only list the game passes) or a block's shapes, starting or
    // ending inside one of them inflated by half the entity, passing through it, or missing it, and entities against one.
    private static List<Object[]> collisionPaths() {
        Random random = new Random(61);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 600; i++) {
            BlockPos pos = random.nextInt(4) == 0 ? CoreValues.randomBlockPos(random) : nearSpawn(random);
            List<AABB> boxes = new ArrayList<>();
            if (random.nextBoolean()) {
                boxes.add(new AABB(pos.getX(), pos.getY(), pos.getZ(), pos.getX() + 1, pos.getY() + (1 + random.nextInt(9)) / 9.0F, pos.getZ() + 1));
            } else {
                for (int count = 1 + random.nextInt(3); count > 0; count--) {
                    boxes.add(CoreValues.blockShape(random).move(pos));
                }
            }
            Vec3 feet = Vec3.atBottomCenterOf(pos).add(spread(random), spread(random), spread(random));
            AABB box = CoreValues.randomEntityBox(random, feet);
            AABB target = boxes.get(random.nextInt(boxes.size()));
            // Against the box, where its center is on the side of the box inflated by half of it, give or take 1.0E-7.
            if (random.nextInt(3) == 0) {
                box = meeting(box, target, random.nextInt(3), random);
            }
            double[] tiny = {1.0E-7, Math.nextDown(1.0E-7), 5.0E-8, 2.0E-7};
            Vec3 vector = switch (random.nextInt(6)) {
                case 0 -> new Vec3((random.nextDouble() * 2.0 - 1.0) * 0.3, random.nextBoolean() ? 0.0 : -0.0784000015258789, (random.nextDouble() * 2.0 - 1.0) * 0.3);
                case 1 -> new Vec3(0.0, -random.nextDouble() * 3.92, 0.0);
                case 2 -> Vec3.directionFromRotation((random.nextFloat() * 2.0F - 1.0F) * 90.0F, (random.nextFloat() * 2.0F - 1.0F) * 180.0F).scale(3.0);
                case 3 -> target.getCenter().subtract(box.getCenter()).scale(random.nextDouble() * 2.0);
                case 4 -> Vec3.ZERO.with(Direction.Axis.VALUES[random.nextInt(3)], tiny[random.nextInt(tiny.length)] * (random.nextBoolean() ? 1.0 : -1.0));
                default -> Vec3.ZERO;
            };
            inputs.add(new Object[]{box, vector, List.copyOf(boxes)});
        }

        // Moves from outside onto the min side of the box inflated by half the entity, which contains takes in and clip
        // doesn't (the move ends there, at a scale of 1). The center and the side are in the same binade, so the move
        // adds back to the side exactly.
        while (inputs.size() < 800) {
            BlockPos pos = nearSpawn(random);
            AABB target = random.nextBoolean() ? new AABB(pos) : CoreValues.blockShape(random).move(pos);
            Direction.Axis axis = Direction.Axis.VALUES[random.nextInt(3)];
            AABB box = CoreValues.randomEntityBox(random, Vec3.atBottomCenterOf(pos).add(spread(random), spread(random), spread(random)));
            AABB inflated = target.inflate(box.getXsize() * 0.5 - 1.0E-7, box.getYsize() * 0.5 - 1.0E-7, box.getZsize() * 0.5 - 1.0E-7);
            Vec3 center = inflated.getCenter();
            box = box.move(center.subtract(box.getCenter())).move(Vec3.ZERO.with(axis, -(inflated.max(axis) - inflated.min(axis)) * 0.5 - random.nextDouble() * 0.5));
            Vec3 from = box.getCenter();
            Vec3 vector = Vec3.ZERO.with(axis, inflated.min(axis) - from.get(axis));
            if (!inflated.contains(from) && from.add(vector).get(axis) == inflated.min(axis) && inflated.contains(from.add(vector))) {
                inputs.add(new Object[]{box, vector, List.of(target)});
            }
        }
        return inputs;
    }

    // Boxes up to 5 blocks across whose sides are on blocks' faces, an ulp or 1.0E-7 inside or outside them, or anywhere in
    // a block, like entities' boxes and the ones nextDeflated and deflate give. Kind 0 passes the box, 1 also a direction
    // to walk it in, like cornerWalks'.
    private static List<Object[]> blockAlignedBoxes(final int kind) {
        Random random = new Random(62 + kind);
        double[] components = {0.0, -0.0, 1.0, -1.0, 0.5, -0.5, Double.NaN};
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 500; i++) {
            BlockPos pos = random.nextBoolean() ? nearSpawn(random) : CoreValues.randomBlockPos(random);
            double[] sides = new double[6];
            for (int side = 0; side < 6; side++) {
                double face = pos.get(Direction.Axis.VALUES[side % 3]) + (side < 3 ? 0.0 : random.nextInt(6));
                sides[side] = switch (random.nextInt(6)) {
                    case 0 -> face + random.nextDouble();
                    case 1 -> face + 1.0E-7;
                    case 2 -> face - 1.0E-7;
                    case 3 -> ulps(face, random.nextBoolean() ? 1 : -1);
                    default -> face;
                };
            }
            AABB box = new AABB(sides[0], sides[1], sides[2], sides[3], sides[4], sides[5]);
            Vec3 direction = random.nextBoolean()
                ? CoreValues.randomVec3(random)
                : new Vec3(components[random.nextInt(7)], components[random.nextInt(7)], components[random.nextInt(7)]);
            inputs.add(kind == 0 ? new Object[]{box} : new Object[]{box, direction});
        }
        return inputs;
    }

    // Namespaces and paths like the game's, and ones that break each rule: empty, "..", upper case, a space, non-ASCII, a
    // slash in a namespace, a second separator in a path.
    // (Methods, since SUITES is built before any static array after it.)
    private static String[] namespaces() {
        return new String[]{"", "minecraft", "realms", "my_mod", "a.b-c0", "..", ".", "Minecraft", "mine craft", "é", "a/b"};
    }

    private static String[] paths() {
        return new String[]{"", "stone", "block/stone", "textures/atlas/blocks.png", "..", "../x", "Stone", "x y", "😀", "-_./09", "a:b"};
    }

    // Every namespace with every path, the way fromNamespaceAndPath and tryBuild take them.
    private static List<Object[]> identifierParts() {
        List<Object[]> inputs = new ArrayList<>();
        for (String namespace : namespaces()) {
            for (String path : paths()) {
                inputs.add(new Object[]{namespace, path});
            }
        }
        return inputs;
    }

    // Every namespace and path joined by the separator, each path alone, and each path after a bare separator.
    private static List<Object[]> identifierStrings() {
        List<Object[]> inputs = new ArrayList<>();
        for (Object[] parts : identifierParts()) {
            inputs.add(new Object[]{parts[0] + ":" + parts[1]});
        }
        for (String path : paths()) {
            inputs.add(new Object[]{path});
            inputs.add(new Object[]{":" + path});
        }
        return inputs;
    }

    // Those strings, and ones joined by other characters, split by the separators the game uses and others: one outside
    // ASCII, and the halves of a supplementary character (indexOf(char) finds a surrogate inside one).
    private static List<Object[]> separatedStrings() {
        char[] separators = {':', '.', '/', '_', 'a', 'é', '\ud83d', '\ude00'};
        List<String> strings = new ArrayList<>();
        for (Object[] input : identifierStrings()) {
            strings.add((String)input[0]);
        }
        strings.addAll(List.of("minecraft.stone", "minecraft/block/stone", "😀:stone", "minecraft:😀", "a😀b", "my_mod.item.x", "minecraftéstone", "éstone", "stoneé"));
        List<Object[]> inputs = new ArrayList<>();
        for (String string : strings) {
            for (char separator : separators) {
                inputs.add(new Object[]{string, separator});
            }
        }
        return inputs;
    }

    // Identifiers with every path piece as a new path, a prefix and a suffix.
    private static List<Object[]> identifierAffixes() {
        Random random = new Random(71);
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 30; i++) {
            Identifier id = i < CoreValues.IDENTIFIER.edges().size() ? (Identifier)CoreValues.IDENTIFIER.edges().get(i) : CoreValues.randomIdentifier(random);
            for (String path : paths()) {
                inputs.add(new Object[]{id, path});
            }
        }
        return inputs;
    }

    // Registry entries whose data is a list of empty compounds, which the default quota of 2 MiB takes (40329 of them,
    // 2097144 bytes accounted: 36 for the list, and 4 and 48 for each compound) or refuses (40330, 2097196 bytes). Each
    // takes a byte to read, so the buffers stay small.
    private static List<Object[]> registryEntriesAtQuota() {
        List<Object[]> inputs = new ArrayList<>();
        for (int count : new int[]{40329, 40330}) {
            FriendlyByteBuf buffer = new FriendlyByteBuf(Unpooled.buffer());
            buffer.writeIdentifier(Identifier.withDefaultNamespace("plains"));
            buffer.writeBoolean(true);
            buffer.writeByte(Tag.TAG_LIST);
            buffer.writeByte(Tag.TAG_COMPOUND);
            buffer.writeInt(count);
            buffer.writeBytes(new byte[count]);
            inputs.add(new Object[]{NetworkValues.buffer(0, 0, NetworkValues.readable(buffer))});
        }
        return inputs;
    }

    // Registry data packets of every pair of the keys' edges, with no entries.
    private static List<Object[]> registryDataPairs() {
        List<Object[]> inputs = new ArrayList<>();
        for (Object a : CoreValues.RESOURCE_KEY.edges()) {
            for (Object b : CoreValues.RESOURCE_KEY.edges()) {
                inputs.add(new Object[]{registryData(a), registryData(b)});
            }
        }
        return inputs;
    }

    @SuppressWarnings("unchecked")
    private static ClientboundRegistryDataPacket registryData(final Object key) {
        return new ClientboundRegistryDataPacket((ResourceKey<? extends Registry<?>>)key, List.of());
    }

    // Every pair of the generator's edges, either way round.
    private static List<Object[]> edgePairs(final Values.Generator values) {
        List<Object[]> inputs = new ArrayList<>();
        for (Object a : values.edges()) {
            for (Object b : values.edges()) {
                inputs.add(new Object[]{a, b});
            }
        }
        return inputs;
    }

    // Identifiers with paths and namespaces in common, or one a prefix of the other, which compareTo and equals tell apart.
    private static List<Object[]> identifierPairs() {
        Random random = new Random(72);
        String[] namespaces = {"minecraft", "realms", "a", "ab", "b", ""};
        String[] paths = {"stone", "stone_bricks", "stonf", "block/stone", "a", ""};
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 400; i++) {
            Identifier a = Identifier.fromNamespaceAndPath(namespaces[random.nextInt(namespaces.length)], paths[random.nextInt(paths.length)]);
            Identifier b = random.nextInt(4) == 0 ? a : Identifier.fromNamespaceAndPath(namespaces[random.nextInt(namespaces.length)], paths[random.nextInt(paths.length)]);
            inputs.add(new Object[]{a, b});
        }
        return inputs;
    }

    // Every ASCII and Latin-1 character and then some, surrogates and the last ones included.
    // Strings around what Integer.parseInt takes: signs, leading zeros, the limits and past them, digits of other scripts
    // (each script's 0 to 9 runs on from its zero), a supplementary digit (two surrogates), and what it refuses.
    private static List<Object[]> intStrings() {
        List<Object[]> inputs = new ArrayList<>();
        for (String s : List.of(
            "", "+", "-", "0", "-0", "+0", "00", "007", "1", "-1", "+1", "9", "10", "99", "2147483647", "2147483648", "-2147483648",
            "-2147483649", "21474836470", "4294967296", "99999999999999999999", " 1", "1 ", "1.0", "0x1", "1e3", "--1", "+-1", "a", "1a",
            "\u0663", "\uff13", "\u0e53", "1\u0660", "\u0967\u0966", "\u0669\u0669", "\ud835\udfce", "\u0039\u0660", "\u2163", "\u00b2"
        )) {
            inputs.add(new Object[]{s});
        }
        return inputs;
    }

    // Every char, in radix 10.
    private static List<Object[]> decimalDigits() {
        List<Object[]> inputs = new ArrayList<>();
        for (int c = 0; c <= 0xffff; c++) {
            inputs.add(new Object[]{(char)c, 10});
        }
        return inputs;
    }

    private static List<Object[]> everyChar() {
        List<Object[]> inputs = new ArrayList<>();
        for (int c = 0; c < 0x180; c++) {
            inputs.add(new Object[]{(char)c});
        }
        for (int c : new int[]{0xd800, 0xdbff, 0xdc00, 0xdfff, 0xfffe, 0xffff}) {
            inputs.add(new Object[]{(char)c});
        }
        return inputs;
    }

    // How far a buffer may grow for a case: a length that overflows the writer index throws before growing anything.
    private static boolean growsLittle(final ByteBuf buffer, final int length) {
        return length <= 1 << 16 || buffer.writerIndex() + length < 0;
    }

    // Whether reading the array at the buffer's reader index allocates little: its length prefix is small, past the
    // maximum (which throws first), or not a whole VarInt.
    private static boolean allocatesLittle(final Object buffer, final int maxSize) {
        Integer size = NetworkValues.leadingVarInt(buffer);
        return size == null || size <= 1 << 16 || size > maxSize;
    }

    // The same inputs with each buffer wrapped in a FriendlyByteBuf, for its instance methods.
    private static List<Object[]> friendly(final List<Object[]> inputs) {
        List<Object[]> wrapped = new ArrayList<>();
        for (Object[] input : inputs) {
            Object[] copy = input.clone();
            copy[0] = new FriendlyByteBuf((ByteBuf)input[0]);
            wrapped.add(copy);
        }
        return wrapped;
    }

    // A profile's properties as GAME_PROFILE_PROPERTIES writes them, without its limits on the count and the strings.
    private static void writePropertiesWithoutLimits(final FriendlyByteBuf buffer, final PropertyMap properties) {
        buffer.writeVarInt(properties.size());
        for (Property property : properties.values()) {
            Utf8String.write(buffer, property.name(), Integer.MAX_VALUE);
            Utf8String.write(buffer, property.value(), Integer.MAX_VALUE);
            FriendlyByteBuf.writeNullable(buffer, property.signature(), (out, signature) -> Utf8String.write(out, signature, Integer.MAX_VALUE));
        }
    }

    // After the prefix, property counts the reader takes without reading a property (negative ones too) or refuses (past
    // 16), with nothing after them.
    private static List<Object[]> propertyCounts(final byte[] prefix) {
        List<Object[]> inputs = new ArrayList<>();
        for (int count : new int[]{0, -1, Integer.MIN_VALUE, 16, 17, Integer.MAX_VALUE}) {
            inputs.add(new Object[]{NetworkValues.buffer(0, 0, NetworkValues.concat(prefix, NetworkValues.varInt(count)))});
        }
        return inputs;
    }

    // Each value as the only argument.
    private static List<Object[]> each(final List<Object> values) {
        return values.stream().map(value -> new Object[]{value}).toList();
    }

    // What createOfflinePlayerUUID hashes for the player names.
    private static List<Object[]> offlinePlayerNames() {
        return ProfileValues.PLAYER_NAMES.edges().stream()
            .map(name -> new Object[]{("OfflinePlayer:" + name).getBytes(StandardCharsets.UTF_8)})
            .toList();
    }

    // Custom query packets whose payload, everything after the transaction id (and the query's identifier), is the most
    // their readers take, and a byte more.
    private static List<Object[]> largePayloads(final boolean identifier) {
        List<Object[]> inputs = new ArrayList<>();
        for (int size : new int[]{1048576, 1048577}) {
            FriendlyByteBuf buffer = new FriendlyByteBuf(Unpooled.buffer());
            buffer.writeVarInt(7);
            if (identifier) {
                buffer.writeIdentifier(Identifier.withDefaultNamespace("query"));
            }
            buffer.writeBytes(new byte[size]);
            inputs.add(new Object[]{new FriendlyByteBuf(NetworkValues.buffer(0, 0, NetworkValues.readable(buffer)))});
        }
        return inputs;
    }

    // A DiscardedPayload's codec, with maximums around 0, a byte, and a custom payload's 32767, reading buffers of the
    // maximum's length and a byte either side.
    private static List<Object[]> discardedPayloads() {
        List<Object[]> inputs = new ArrayList<>();
        for (int max : new int[]{0, 1, 255, 32767}) {
            for (int length = Math.max(0, max - 1); length <= max + 1; length++) {
                Object buffer = new FriendlyByteBuf(NetworkValues.buffer(0, 0, new byte[length]));
                inputs.add(new Object[]{Identifier.fromNamespaceAndPath("rustcraft", "scenario_start"), max, buffer});
            }
        }
        return inputs;
    }

    // Custom payload packets the codec's own encodings never hold: a discarded payload of the most the packet takes
    // (32767 bytes serverbound, 1 MiB clientbound) and a byte more, and brands longer than readUtf's 32767, written without
    // the limit.
    // Counts with nothing after them (negative ones throw making the set), the server's features with one twice, one
    // identifier twice, and a feature that isn't an identifier.
    private static List<Object[]> featureBuffers() {
        List<Object[]> inputs = new ArrayList<>();
        for (int count : new int[]{0, 1, -1, Integer.MIN_VALUE, 65536, 65537, Integer.MAX_VALUE}) {
            inputs.add(new Object[]{NetworkValues.buffer(0, 0, NetworkValues.varInt(count))});
        }
        byte[] vanilla = NetworkValues.utf("minecraft:vanilla");
        byte[] tradeRebalance = NetworkValues.utf("minecraft:trade_rebalance");
        inputs.add(new Object[]{NetworkValues.buffer(0, 0, NetworkValues.concat(NetworkValues.varInt(3), vanilla, tradeRebalance, vanilla))});
        inputs.add(new Object[]{NetworkValues.buffer(0, 0, NetworkValues.concat(NetworkValues.varInt(2), vanilla, vanilla))});
        inputs.add(new Object[]{NetworkValues.buffer(0, 0, NetworkValues.concat(NetworkValues.varInt(1), NetworkValues.utf("Minecraft:Vanilla")))});
        return inputs;
    }

    // Counts with nothing after them, around the limit of 32 and negative (which throws making the map), and a name twice,
    // apart and in a row.
    private static List<Object[]> reportDetailBuffers() {
        List<Object[]> inputs = new ArrayList<>();
        for (int count : new int[]{0, 1, -1, Integer.MIN_VALUE, 32, 33}) {
            inputs.add(new Object[]{NetworkValues.buffer(0, 0, NetworkValues.varInt(count))});
        }
        byte[] server = NetworkValues.utf("Server");
        byte[] mods = NetworkValues.utf("Mod list");
        inputs.add(new Object[]{NetworkValues.buffer(0, 0, NetworkValues.concat(
            NetworkValues.varInt(3), server, NetworkValues.utf("first"), mods, NetworkValues.utf("none"), server, NetworkValues.utf("second")
        ))});
        inputs.add(new Object[]{NetworkValues.buffer(0, 0, NetworkValues.concat(
            NetworkValues.varInt(2), server, NetworkValues.utf("first"), server, NetworkValues.utf("second")
        ))});
        return inputs;
    }

    private static List<Object[]> customPayloads(final int maximum) {
        List<Object[]> inputs = new ArrayList<>();
        for (int size : new int[]{maximum, maximum + 1}) {
            FriendlyByteBuf buffer = new FriendlyByteBuf(Unpooled.buffer());
            buffer.writeIdentifier(Identifier.fromNamespaceAndPath("rustcraft", "scenario_start"));
            buffer.writeBytes(new byte[size]);
            inputs.add(new Object[]{new FriendlyByteBuf(NetworkValues.buffer(0, 0, NetworkValues.readable(buffer)))});
        }
        for (String brand : new String[]{"a".repeat(32768), "😀".repeat(16384), "😀".repeat(16384) + "a"}) {
            FriendlyByteBuf buffer = new FriendlyByteBuf(Unpooled.buffer());
            buffer.writeIdentifier(BrandPayload.TYPE.id());
            Utf8String.write(buffer, brand, Integer.MAX_VALUE);
            inputs.add(new Object[]{new FriendlyByteBuf(NetworkValues.buffer(0, 0, NetworkValues.readable(buffer)))});
        }
        return inputs;
    }

    // Tags or none, and an edge whose writer throws after writing its id: a string tag too long for writeUTF's 65535 bytes.
    // Random draws are the tags' own.
    private static Values.Generator withUnwritableString(final Values.Generator tags) {
        List<Object> edges = new ArrayList<>(tags.edges());
        edges.add(Optional.of(StringTag.valueOf("a".repeat(65536))));
        return new Values.Generator() {
            @Override
            public List<Object> edges() {
                return edges;
            }

            @Override
            public Object random(final Random random) {
                return tags.random(random);
            }

            @Override
            public JsonElement encode(final Object spec) {
                return tags.encode(spec);
            }

            @Override
            public Object instantiate(final Object spec) {
                return tags.instantiate(spec);
            }
        };
    }

    // Tags as the click action's codec writes them: none, an empty compound, a byte, a string and a compound of two.
    private static List<byte[]> tagPrefixPayloads() {
        CompoundTag item = new CompoundTag();
        item.putString("id", "minecraft:stone");
        item.putInt("count", 64);
        return Stream.<Tag>of(EndTag.INSTANCE, new CompoundTag(), ByteTag.valueOf((byte)1), StringTag.valueOf("minecraft:stone"), item)
            .map(NbtValues::anyTagBytes)
            .toList();
    }

    // Payloads a discarded payload's codec skips: none, a byte, and a few.
    private static List<byte[]> discardedPrefixPayloads() {
        return List.of(new byte[0], NetworkValues.bytes(0x2a), NetworkValues.bytes(1, 2, 3, 4, 5));
    }

    // Each payload behind a length prefix that's right, a byte short, a byte long, zero, negative or an int's limit, read
    // with maximums of the payload's length, a byte less and 65536; with no spare capacity or 4 bytes, so a prefix a byte
    // long is past the writer index but inside the array; and with another byte after the payload or none. A byte before
    // the prefix is read already, so a mark the codec set on the buffer would show. Discarded payloads also come at their
    // codec's limit of 32767 bytes and a byte past it.
    private static List<Object[]> lengthPrefixes(final List<byte[]> payloads, final boolean friendly) {
        List<Object[]> inputs = new ArrayList<>();
        for (byte[] payload : payloads) {
            int length = payload.length;
            for (int prefix : new int[]{length, length - 1, length + 1, 0, -1, Integer.MIN_VALUE, Integer.MAX_VALUE}) {
                for (int max : new int[]{length, length - 1, 65536}) {
                    for (int slack : new int[]{0, 4}) {
                        for (byte[] after : new byte[][]{new byte[0], NetworkValues.bytes(0x2a)}) {
                            byte[] bytes = NetworkValues.concat(NetworkValues.bytes(0x7f), NetworkValues.varInt(prefix), payload, after);
                            ByteBuf buffer = NetworkValues.buffer(slack, 1, bytes);
                            inputs.add(new Object[]{max, friendly ? new FriendlyByteBuf(buffer) : buffer});
                        }
                    }
                }
            }
        }
        if (friendly) {
            for (int length : new int[]{32767, 32768}) {
                ByteBuf buffer = NetworkValues.buffer(0, 0, NetworkValues.concat(NetworkValues.varInt(length), new byte[length]));
                inputs.add(new Object[]{65536, new FriendlyByteBuf(buffer)});
            }
        }
        return inputs;
    }

    // Click actions whose tag is behind a length prefix of the codec's limit of 65536 and of a byte past it, with zeros
    // after the tag up to the prefix's length; byte arrays the accounter's 32 KiB just takes (an array accounts 24 bytes
    // and one a byte) and just refuses, and one far over it, which fit the limit; and compounds nested 16 deep, as deep
    // as the accounter reads, and 17.
    private static List<Object[]> clickActionPayloads() {
        byte[] id = NetworkValues.utf("rustcraft:click");
        byte[] small = NbtValues.anyTagBytes(StringTag.valueOf("a"));
        List<Object[]> inputs = new ArrayList<>();
        for (int length : new int[]{65536, 65537}) {
            byte[] padded = Arrays.copyOf(small, length);
            inputs.add(new Object[]{NetworkValues.buffer(0, 0, NetworkValues.concat(id, NetworkValues.varInt(length), padded))});
        }
        List<Tag> tags = new ArrayList<>();
        for (int size : new int[]{32744, 32745, 40000}) {
            tags.add(new ByteArrayTag(new byte[size]));
        }
        for (int depth : new int[]{16, 17}) {
            CompoundTag nested = new CompoundTag();
            for (int i = 1; i < depth; i++) {
                CompoundTag outer = new CompoundTag();
                outer.put("a", nested);
                nested = outer;
            }
            tags.add(nested);
        }
        for (Tag tag : tags) {
            byte[] bytes = NbtValues.anyTagBytes(tag);
            inputs.add(new Object[]{NetworkValues.buffer(0, 0, NetworkValues.concat(id, NetworkValues.varInt(bytes.length), bytes))});
        }
        return inputs;
    }

    // Buffers holding what the writer writes of the values' first edges and random ones, read from the start: whole, with
    // another field after them, cut short by a byte and cut anywhere, with up to 16 bytes of capacity to spare.
    @SuppressWarnings("unchecked")
    private static <T> List<Object[]> written(final long seed, final Values.Generator values, final BiConsumer<FriendlyByteBuf, T> writer) {
        Random random = new Random(seed);
        List<Object> edges = values.edges();
        List<Object[]> inputs = new ArrayList<>();
        for (int i = 0; i < 100; i++) {
            Object value = i < edges.size() ? edges.get(i) : values.random(random);
            FriendlyByteBuf buffer = new FriendlyByteBuf(Unpooled.buffer());
            writer.accept(buffer, (T)values.instantiate(value));
            for (byte[] variant : Codecs.variants(NetworkValues.readable(buffer), random)) {
                inputs.add(new Object[]{NetworkValues.buffer(random.nextInt(17), 0, variant)});
            }
        }
        return inputs;
    }

    // The buffer edges and random buffers, for inputs worked out from each buffer's indexes.
    private static List<ByteBuf> sampleBuffers(final long seed, final int randomBuffers) {
        Random random = new Random(seed);
        List<ByteBuf> buffers = new ArrayList<>();
        NetworkValues.BUFFER.edges().forEach(edge -> buffers.add((ByteBuf)edge));
        for (int i = 0; i < randomBuffers; i++) {
            buffers.add(NetworkValues.randomBuffer(random));
        }
        return buffers;
    }

    // Every value whose VarInt (or VarLong) is one byte longer than the one before's, its neighbours, and its negation.
    private static List<Object[]> varIntBoundaries(final boolean withBuffer) {
        List<Integer> values = new ArrayList<>(List.of(0, -1, Integer.MIN_VALUE, Integer.MAX_VALUE));
        for (int bits = 7; bits < 32; bits += 7) {
            values.addAll(List.of((1 << bits) - 1, 1 << bits, (1 << bits) + 1, -(1 << bits)));
        }
        return withBuffers(values, withBuffer);
    }

    private static List<Object[]> varLongBoundaries(final boolean withBuffer) {
        List<Long> values = new ArrayList<>(List.of(0L, -1L, Long.MIN_VALUE, Long.MAX_VALUE));
        for (int bits = 7; bits < 64; bits += 7) {
            values.addAll(List.of((1L << bits) - 1, 1L << bits, (1L << bits) + 1, -(1L << bits)));
        }
        return withBuffers(values, withBuffer);
    }

    // The first argument of each input, like the NaNs with other payloads, which writeFloat writes as they are.
    private static List<Object> firsts(final List<Object[]> inputs) {
        return inputs.stream().map(input -> input[0]).toList();
    }

    // Each value alone, or written to the game's default buffer and to a full one, which has to grow.
    private static List<Object[]> withBuffers(final List<?> values, final boolean withBuffer) {
        List<Object[]> inputs = new ArrayList<>();
        for (Object value : values) {
            if (withBuffer) {
                inputs.add(new Object[]{Unpooled.buffer(), value});
                inputs.add(new Object[]{NetworkValues.buffer(0, 1, NetworkValues.bytes(1, 2, 3)), value});
            } else {
                inputs.add(new Object[]{value});
            }
        }
        return inputs;
    }

    // VarInts (or VarLongs) of every length up to one past the longest, with any data bits, an overlong zero last byte
    // a quarter of the time, and another field after them half the time.
    private static List<Object[]> varIntEncodings(final boolean varLong) {
        Random random = new Random(varLong ? 82 : 81);
        int longest = varLong ? 10 : 5;
        List<Object[]> inputs = new ArrayList<>();
        for (int length = 1; length <= longest + 1; length++) {
            for (int i = 0; i < 40; i++) {
                byte[] encoding = new byte[length];
                for (int b = 0; b < length - 1; b++) {
                    encoding[b] = (byte)(0x80 | random.nextInt(0x80));
                }
                encoding[length - 1] = (byte)(random.nextInt(4) == 0 ? 0 : random.nextInt(0x80));
                byte[] after = random.nextBoolean() ? NetworkValues.randomField(random) : new byte[0];
                inputs.add(new Object[]{NetworkValues.buffer(random.nextInt(4), 0, NetworkValues.concat(encoding, after))});
            }
        }
        return inputs;
    }

    // Strings whose UTF-16 length and UTF-8 size differ, and malformed UTF-8 whose replacement characters count
    // differently, read with a maximum around each limit: the length, the size over 3 (the most bytes a char takes), and
    // ones whose triple overflows.
    private static List<Object[]> utfReadLimits() {
        List<byte[]> payloads = new ArrayList<>();
        for (String string : utfLimitStrings()) {
            payloads.add(string.getBytes(StandardCharsets.UTF_8));
        }
        payloads.addAll(List.of(NetworkValues.bytes(0xed, 0xa0, 0x80), NetworkValues.bytes(0x80, 0x80, 0x80), NetworkValues.bytes(0xf0, 0x9f, 0x98)));
        List<Object[]> inputs = new ArrayList<>();
        for (byte[] payload : payloads) {
            String decoded = new String(payload, StandardCharsets.UTF_8);
            int length = decoded.length();
            int thirds = (payload.length + 2) / 3;
            for (int max : new int[]{length - 1, length, length + 1, thirds - 1, thirds, 0, -1, 1 << 30, 715827882, 715827883, Integer.MAX_VALUE}) {
                inputs.add(new Object[]{NetworkValues.buffer(0, 0, NetworkValues.lengthPrefixed(payload)), max});
            }
        }
        return inputs;
    }

    private static List<Object[]> utfWriteLimits() {
        List<Object[]> inputs = new ArrayList<>();
        for (String string : utfLimitStrings()) {
            for (int max : new int[]{string.length() - 1, string.length(), string.length() + 1, 0, -1, 1 << 30, 715827882, 715827883, Integer.MAX_VALUE}) {
                inputs.add(new Object[]{NetworkValues.buffer(0, 0, new byte[0]), string, max});
            }
        }
        return inputs;
    }

    // Strings at readUtf's and writeUtf's limit of 32767 units and one past it, in characters of one, two and three
    // bytes (three bytes a unit fill the 98301 bytes the limit allows), and in supplementary characters.
    private static List<Object[]> longStrings(final boolean write) {
        List<Object[]> inputs = new ArrayList<>();
        for (String string : List.of("a".repeat(32767), "a".repeat(32768), "é".repeat(32767), "€".repeat(32767), "€".repeat(32768), "😀".repeat(16384))) {
            inputs.add(write ? new Object[]{Unpooled.buffer(), string} : new Object[]{NetworkValues.buffer(0, 0, NetworkValues.utf(string))});
        }
        return inputs;
    }

    private static List<String> utfLimitStrings() {
        return List.of("", "a", "abc", "é", "ééé", "😀", "😀😀", "a😀", "١٢٣", "\u0000", "minecraft:stone", "x".repeat(200), "é".repeat(100));
    }

    // Identifiers as readIdentifier finds them: valid ones, with and without a namespace, and ones that break a rule.
    private static List<Object[]> identifierBuffers() {
        List<Object[]> inputs = new ArrayList<>();
        for (String id : List.of("minecraft:stone", "stone", "realms:x", ":stone", "minecraft:", "", ":", "Minecraft:stone", "a:b:c", "a/b:c", "😀", "minecraft:block/stone", "..:x")) {
            inputs.add(new Object[]{NetworkValues.buffer(0, 0, NetworkValues.utf(id))});
        }
        return inputs;
    }

    // Byte arrays with their length, one longer than the bytes left, or negative, read with maximums around the length.
    private static List<Object[]> byteArrays(final boolean withMax) {
        Random random = new Random(83);
        List<Object[]> inputs = new ArrayList<>();
        for (int size : new int[]{0, 1, 5, 200}) {
            byte[] payload = new byte[size];
            random.nextBytes(payload);
            for (byte[] bytes : List.of(NetworkValues.lengthPrefixed(payload), NetworkValues.concat(NetworkValues.varInt(size + 1), payload), NetworkValues.concat(NetworkValues.varInt(-1), payload))) {
                if (withMax) {
                    for (int max : new int[]{size - 1, size, size + 1, 0, -1, Integer.MAX_VALUE}) {
                        inputs.add(new Object[]{NetworkValues.buffer(0, 0, bytes), max});
                    }
                } else {
                    inputs.add(new Object[]{NetworkValues.buffer(0, 0, bytes)});
                }
            }
        }
        return inputs;
    }

    // VarInt arrays with their count, one more than there are, or negative, read with maximums around the count.
    private static List<Object[]> varIntArrays(final boolean withMax) {
        Random random = new Random(84);
        List<Object[]> inputs = new ArrayList<>();
        for (int count : new int[]{0, 1, 3, 9}) {
            byte[] values = new byte[0];
            for (int i = 0; i < count; i++) {
                values = NetworkValues.concat(values, NetworkValues.varInt((Integer)Values.INTS.random(random)));
            }
            for (int prefix : new int[]{count, count + 1, -1}) {
                byte[] bytes = NetworkValues.concat(NetworkValues.varInt(prefix), values);
                if (withMax) {
                    for (int max : new int[]{count - 1, count, count + 1, 0, -1, Integer.MAX_VALUE}) {
                        inputs.add(new Object[]{NetworkValues.buffer(0, 0, bytes), max});
                    }
                } else {
                    inputs.add(new Object[]{NetworkValues.buffer(0, 0, bytes)});
                }
            }
        }
        return inputs;
    }

    // Long arrays with their count, one more than the longs left (which readLongArray checks), or negative.
    private static List<Object[]> longArrays() {
        Random random = new Random(85);
        List<Object[]> inputs = new ArrayList<>();
        for (int count : new int[]{0, 1, 2, 5}) {
            byte[] longs = new byte[8 * count];
            random.nextBytes(longs);
            for (int prefix : new int[]{count, count + 1, -1}) {
                for (int extra : new int[]{0, 7}) {
                    inputs.add(new Object[]{NetworkValues.buffer(0, 0, NetworkValues.concat(NetworkValues.varInt(prefix), longs, new byte[extra]))});
                }
            }
        }
        return inputs;
    }

    // Buffers of whole and partial longs, read into arrays of every length up to one past them.
    private static List<Object[]> fixedLongArrays() {
        Random random = new Random(86);
        List<Object[]> inputs = new ArrayList<>();
        for (int bytes : new int[]{0, 8, 12, 24}) {
            byte[] longs = new byte[bytes];
            random.nextBytes(longs);
            for (int length = 0; length <= bytes / 8 + 1; length++) {
                long[] output = new long[length];
                Arrays.fill(output, 0x5555555555555555L);
                inputs.add(new Object[]{NetworkValues.buffer(0, 0, longs), output});
            }
        }
        return inputs;
    }

    // Indexes around a buffer's: negative, zero, its reader and writer indexes, its capacity, one past each, and the
    // most an int holds, one or two at a time.
    private static List<Object[]> bufferIndexes(final int arity) {
        List<Object[]> inputs = new ArrayList<>();
        for (ByteBuf buffer : sampleBuffers(87, 40)) {
            int[] indexes = {
                -1, 0, buffer.readerIndex(), buffer.readerIndex() + 1, buffer.writerIndex(), buffer.writerIndex() + 1,
                buffer.capacity(), buffer.capacity() + 1, buffer.readableBytes(), buffer.readableBytes() + 1, Integer.MAX_VALUE
            };
            for (int first : indexes) {
                if (arity == 1) {
                    inputs.add(new Object[]{buffer, first});
                    continue;
                }
                for (int second : indexes) {
                    inputs.add(new Object[]{buffer, first, second});
                }
            }
        }
        return inputs;
    }

    // Ranges of a buffer's array for toString: its readable bytes, all its capacity, empty at the end, one past the end,
    // and negative starts and lengths, which the decoder checks against the capacity rather than the writer index.
    private static List<Object[]> bufferRanges() {
        List<Object[]> inputs = new ArrayList<>();
        for (ByteBuf buffer : sampleBuffers(88, 40)) {
            int[][] ranges = {
                {buffer.readerIndex(), buffer.readableBytes()}, {0, buffer.capacity()}, {0, buffer.writerIndex()},
                {buffer.capacity(), 0}, {buffer.capacity(), 1}, {0, buffer.capacity() + 1}, {-1, 1}, {1, -1}, {-1, 0}
            };
            for (int[] range : ranges) {
                inputs.add(new Object[]{buffer, range[0], range[1], StandardCharsets.UTF_8});
            }
        }
        return inputs;
    }

    // Ranges of a buffer's array to slice: its readable bytes and one more (past the writer index, inside the array when
    // there's spare capacity), all of it, what's past the writer index, empty at the end, one past the end, negative starts
    // and lengths, and an end past Integer.MAX_VALUE. A slice is checked against the capacity, not the writer index.
    private static List<Object[]> sliceRanges() {
        List<Object[]> inputs = new ArrayList<>();
        for (ByteBuf buffer : sampleBuffers(107, 40)) {
            int[][] ranges = {
                {buffer.readerIndex(), buffer.readableBytes()}, {buffer.readerIndex(), buffer.readableBytes() + 1}, {0, buffer.capacity()},
                {buffer.writerIndex(), buffer.writableBytes()}, {buffer.capacity(), 0}, {buffer.capacity(), 1}, {0, buffer.capacity() + 1},
                {-1, 1}, {1, -1}, {-1, 0}, {Integer.MAX_VALUE, 1}
            };
            for (int[] range : ranges) {
                inputs.add(new Object[]{buffer, range[0], range[1]});
            }
        }
        return inputs;
    }

    // Arrays and ranges of them: all of it, all but the first byte, empty at the end, and ranges past either end.
    private static List<Object[]> arrayRanges() {
        Random random = new Random(89);
        List<Object[]> inputs = new ArrayList<>();
        for (ByteBuf buffer : sampleBuffers(90, 40)) {
            byte[] array = new byte[random.nextInt(12)];
            random.nextBytes(array);
            int[][] ranges = {
                {0, array.length}, {1, array.length - 1}, {array.length, 0}, {array.length, 1}, {0, array.length + 1},
                {-1, 1}, {0, -1}, {0, buffer.readableBytes()}
            };
            for (int[] range : ranges) {
                inputs.add(new Object[]{buffer, array, range[0], range[1]});
            }
        }
        return inputs;
    }

    // Source buffers and ranges of their arrays, which writeBytes checks against the source's capacity.
    private static List<Object[]> sourceRanges() {
        Random random = new Random(91);
        List<ByteBuf> buffers = sampleBuffers(92, 40);
        List<Object[]> inputs = new ArrayList<>();
        for (ByteBuf buffer : buffers) {
            ByteBuf source = buffers.get(random.nextInt(buffers.size()));
            int[][] ranges = {
                {source.readerIndex(), source.readableBytes()}, {0, source.capacity()}, {source.capacity(), 0},
                {source.capacity(), 1}, {-1, 1}, {0, -1}
            };
            for (int[] range : ranges) {
                inputs.add(new Object[]{buffer, source, range[0], range[1]});
            }
        }
        return inputs;
    }

    // A block with each coordinate at an int limit, next to one, or near zero.
    private static BlockPos limitBlock(final Random random) {
        int[] coordinates = {Integer.MAX_VALUE, Integer.MAX_VALUE - 1, Integer.MIN_VALUE, 0, -1};
        return new BlockPos(
            coordinates[random.nextInt(coordinates.length)], coordinates[random.nextInt(coordinates.length)], coordinates[random.nextInt(coordinates.length)]
        );
    }

    // A block within the walk scenario's render distance of its spawn, at a height near the ground.
    private static BlockPos nearSpawn(final Random random) {
        return new BlockPos(random.nextInt(129) - 64, random.nextInt(17) - 68, random.nextInt(129) - 64);
    }

    // Up to a block and a half either way, in sixteenths now and then, so boxes overlap, meet or miss.
    private static double spread(final Random random) {
        return random.nextBoolean() ? (random.nextInt(49) - 24) / 16.0 : (random.nextDouble() * 2.0 - 1.0) * 1.5;
    }

    private static double[] sides(final AABB box) {
        return new double[]{box.minX, box.minY, box.minZ, box.maxX, box.maxY, box.maxZ};
    }

    // The value moved by that many ulps, up or down.
    private static double ulps(final double value, final int ulps) {
        double moved = value;
        for (int step = ulps; step != 0; step -= Integer.signum(step)) {
            moved = step > 0 ? Math.nextUp(moved) : Math.nextDown(moved);
        }
        return moved;
    }

    private static double sign(final Random random) {
        return random.nextBoolean() ? 1.0 : -1.0;
    }

    // 1 to 2 times a power of two from 2^low to 2^high. Below 2^-1022 it rounds to a subnormal.
    private static double magnitude(final Random random, final int low, final int high) {
        return Math.scalb(1.0 + random.nextDouble(), random.nextInt(high - low + 1) + low);
    }

    // 1 plus or minus up to 2^-k, for k from 0 to 52.
    private static double nearOne(final Random random) {
        return 1.0 + (random.nextDouble() * 2.0 - 1.0) * Math.scalb(1.0, -random.nextInt(53));
    }

    // NaN payloads never change game behaviour, so a random NaN becomes the canonical one, like Values does.
    private static double randomBits(final Random random) {
        double value = Double.longBitsToDouble(random.nextLong());
        return Double.isNaN(value) ? Double.NaN : value;
    }

    private static Codecs.Codec bufferCodec(final Values.Generator values, final Class<?> valueType) {
        return new Codecs.Codec(NetworkValues.BUFFER, values, valueType);
    }

    // Every composite, of 1 to 14 fields, from VAR_INT, STRING_UTF8, BOOL and FLOAT in turn. A value is the list of its
    // fields, which the getters read and the constructor (a FunctionN made by Proxy) makes.
    private static CodecSuite composites(final CodecSuite suite) {
        StreamCodec<?, ?>[] codecs = {ByteBufCodecs.VAR_INT, ByteBufCodecs.STRING_UTF8, ByteBufCodecs.BOOL, ByteBufCodecs.FLOAT};
        String[] names = {"VAR_INT", "STRING_UTF8", "BOOL", "FLOAT"};
        Values.Generator[] generators = {INTS, STRINGS, BOOLEANS, FLOATS};
        for (Method method : StreamCodec.class.getMethods()) {
            if (!method.getName().equals("composite")) {
                continue;
            }

            int fields = method.getParameterCount() / 2;
            Object[] args = new Object[method.getParameterCount()];
            Values.Generator[] elements = new Values.Generator[fields];
            List<String> label = new ArrayList<>();
            for (int i = 0; i < fields; i++) {
                int index = i;
                args[2 * i] = codecs[i % codecs.length];
                args[2 * i + 1] = (Function<List<?>, Object>)list -> list.get(index);
                elements[i] = generators[i % generators.length];
                label.add(names[i % names.length]);
            }
            Class<?> constructor = method.getParameterTypes()[2 * fields];
            args[2 * fields] = Proxy.newProxyInstance(Golden.class.getClassLoader(), new Class<?>[]{constructor}, (proxy, invoked, values) -> switch (invoked.getName()) {
                case "apply" -> Arrays.asList(values.clone());
                case "hashCode" -> System.identityHashCode(proxy);
                case "equals" -> proxy == values[0];
                default -> "constructor";
            });
            suite.codec(
                Descriptors.of(method), String.join(", ", label), leading -> method.invoke(null, args),
                bufferCodec(Codecs.tupleOf(elements), List.class)
            );
        }
        return suite;
    }

    // Movements whose largest component is around LpVec3's limits: the smallest it writes and an ulp either side, whole
    // blocks around where the scale needs a VarInt (past 3), the largest it writes and past it, and NaN (written as 0) and
    // infinities. Each with either sign along each axis, the other components zero, smaller or the same.
    private static List<Vec3> lpVectors() {
        double[] magnitudes = {
            0.0, LpVec3.ABS_MIN_VALUE, Math.nextDown(LpVec3.ABS_MIN_VALUE), Math.nextUp(LpVec3.ABS_MIN_VALUE), 1.0E-5, 0.1, 0.5,
            Math.nextDown(1.0), 1.0, Math.nextUp(1.0), 2.0, 2.5, 3.0, Math.nextUp(3.0), 3.5, 4.0, 5.0, 7.0, 8.0, 100.0, 1000.0,
            65535.0, 65536.0, 0x1p31, 0x1p33, Math.nextDown(LpVec3.ABS_MAX_VALUE), LpVec3.ABS_MAX_VALUE, Math.nextUp(LpVec3.ABS_MAX_VALUE),
            1.0E11, Double.MAX_VALUE, Double.POSITIVE_INFINITY, Double.NaN
        };
        Random random = new Random(1013);
        List<Vec3> vectors = new ArrayList<>();
        for (double magnitude : magnitudes) {
            for (double sign : new double[]{1.0, -1.0}) {
                for (int axis = 0; axis < 3; axis++) {
                    for (int others = 0; others < 3; others++) {
                        double[] components = new double[3];
                        for (int i = 0; i < 3; i++) {
                            components[i] = switch (others) {
                                case 0 -> 0.0;
                                case 1 -> (random.nextDouble() * 2.0 - 1.0) * magnitude;
                                default -> magnitude;
                            };
                        }
                        components[axis] = sign * magnitude;
                        vectors.add(new Vec3(components[0], components[1], components[2]));
                    }
                }
            }
        }
        return vectors;
    }

    private static List<Object[]> lpWrites() {
        Random random = new Random(1014);
        List<Object[]> inputs = new ArrayList<>();
        for (Vec3 vector : lpVectors()) {
            inputs.add(new Object[]{NetworkValues.randomBuffer(random), vector});
        }
        return inputs;
    }

    // The vectors' encodings, whole, cut short, and with a field after them.
    private static List<Object[]> lpReads() {
        Random random = new Random(1015);
        List<Object[]> inputs = new ArrayList<>();
        for (Vec3 vector : lpVectors()) {
            ByteBuf buffer = Unpooled.buffer();
            LpVec3.write(buffer, vector);
            byte[] bytes = NetworkValues.readable(buffer);
            inputs.add(new Object[]{NetworkValues.buffer(0, 0, bytes)});
            inputs.add(new Object[]{NetworkValues.buffer(0, 0, Arrays.copyOf(bytes, bytes.length - 1))});
            inputs.add(new Object[]{NetworkValues.buffer(0, 0, NetworkValues.concat(bytes, NetworkValues.randomField(random)))});
        }
        return inputs;
    }

    // Bounds that are signed zeros (0.0 is above -0.0, which throws), NaN, equal, or the wrong way round, around values
    // between, on and past them.
    private static List<Object[]> clampInputs() {
        double[][] bounds = {
            {0.0, -0.0}, {-0.0, 0.0}, {0.0, 0.0}, {-0.0, -0.0}, {1.0, 1.0}, {-1.0, 1.0}, {2.0, 1.0}, {Double.NaN, 1.0}, {1.0, Double.NaN},
            {Double.NEGATIVE_INFINITY, Double.POSITIVE_INFINITY}, {-LpVec3.ABS_MAX_VALUE, LpVec3.ABS_MAX_VALUE}
        };
        double[] values = {-0.0, 0.0, -1.0, 1.0, 0.5, 3.0, Double.NaN, Double.NEGATIVE_INFINITY, Double.POSITIVE_INFINITY};
        List<Object[]> inputs = new ArrayList<>();
        for (double[] bound : bounds) {
            for (double value : values) {
                inputs.add(new Object[]{value, bound[0], bound[1]});
            }
        }
        return inputs;
    }

    private static Object[] asFloats(final Object[] doubles) {
        Object[] floats = new Object[doubles.length];
        for (int i = 0; i < doubles.length; i++) {
            floats[i] = (float)(double)(Double)doubles[i];
        }
        return floats;
    }

    private static List<Object[]> everyByte() {
        List<Object[]> inputs = new ArrayList<>();
        for (int value = 0; value < 256; value++) {
            inputs.add(new Object[]{value});
        }
        return inputs;
    }
}
