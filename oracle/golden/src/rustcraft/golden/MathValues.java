package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonNull;
import com.mojang.math.Axis;
import com.mojang.math.GivensParameters;
import com.mojang.math.OctahedralGroup;
import com.mojang.math.Transformation;
import java.lang.reflect.Field;
import java.util.List;
import java.util.Optional;
import java.util.Random;
import org.joml.Matrix4f;
import org.joml.Quaternionf;
import org.joml.Quaternionfc;
import org.joml.Vector3f;
import org.joml.Vector3fc;

/**
 * Arguments and receivers of com.mojang.math's value classes. A GivensParameters is {@code [sinHalf, cosHalf]}, like any
 * record. A Transformation is its fields in declaration order, {@code [matrix, decomposed, translation, leftRotation,
 * scale, rightRotation]}, the last four null until it's decomposed, followed by whether it's {@code IDENTITY}: the game
 * compares with that constant by reference.
 */
final class MathValues {
    private static final Field DECOMPOSED = transformationField("decomposed");
    private static final Field TRANSLATION = transformationField("translation");
    private static final Field LEFT_ROTATION = transformationField("leftRotation");
    private static final Field SCALE = transformationField("scale");
    private static final Field RIGHT_ROTATION = transformationField("rightRotation");

    // Identities, MatrixUtil's PI_4, half-turns, the ones qrGivensQuat and approxGivensQuat make, and some no rotation is.
    static final Values.Generator GIVENS_PARAMETERS = new Values.Simple(
        List.<Object>of(
            new GivensParameters(0.0F, 1.0F), new GivensParameters(-0.0F, 1.0F), new GivensParameters(1.0F, 0.0F),
            new GivensParameters(0.0F, -1.0F), GivensParameters.fromPositiveAngle((float)(Math.PI / 4)),
            GivensParameters.fromPositiveAngle((float)(Math.PI / 2)), GivensParameters.fromUnnormalized(1.0F, 2.0F),
            new GivensParameters(0.6F, 0.8F), new GivensParameters(0.0F, 0.0F), new GivensParameters(1.0E-20F, 1.0F),
            new GivensParameters(Float.MAX_VALUE, Float.MAX_VALUE), new GivensParameters(Float.NaN, 1.0F),
            new GivensParameters(Float.POSITIVE_INFINITY, 1.0F)
        ),
        random -> switch (random.nextInt(3)) {
            case 0 -> GivensParameters.fromUnnormalized(JomlValues.component(random), JomlValues.component(random));
            case 1 -> GivensParameters.fromPositiveAngle(JomlValues.angle(random));
            default -> new GivensParameters(JomlValues.component(random), JomlValues.component(random));
        },
        value -> Values.encode(value, GivensParameters.class)
    );

    static final Values.Generator TRANSFORMATION = new JomlValues.Mutable<>(
        List.of(
            Transformation.IDENTITY,
            new Transformation(new Matrix4f()),
            new Transformation(null, null, null, null),
            // ChestRenderer, TridentSpecialRenderer, TheEndPortalRenderer, BannerRenderer and BuiltInBlockModels'.
            new Transformation(new Matrix4f().rotationAround(Axis.YP.rotationDegrees(-90.0F), 0.5F, 0.0F, 0.5F)),
            new Transformation(null, null, new Vector3f(1.0F, -1.0F, -1.0F), null),
            new Transformation(new Vector3f(0.0F, 0.375F, 0.0F), null, new Vector3f(1.0F, 0.375F, 1.0F), null),
            banner(-22.5F),
            new Transformation(new Vector3f(0.5F, 0.8125F, 0.5F), Axis.ZP.rotationDegrees(180.0F), null, Axis.XP.rotationDegrees(90.0F)),
            // BlockMath's face transformation for east, and BlockModelRotation's, also moved to the block's corner.
            new Transformation(null, Axis.YP.rotation((float)(Math.PI / 2)), null, null),
            new Transformation(new Matrix4f(OctahedralGroup.ROT_90_Y_NEG.transformation())),
            centerToCorner(new Matrix4f(OctahedralGroup.ROT_120_NPN.transformation())),
            // A banner's matrix, which decomposes into about the same parts, and one decomposed already.
            new Transformation(banner(-22.5F).getMatrixCopy()),
            decomposed(new Transformation(JomlValues.affineMatrix())),
            // Columns of zero, which the decomposition handles apart, and matrices with m33 other than 1.
            new Transformation(new Matrix4f().scaling(0.0F, 1.0F, 1.0F)),
            new Transformation(new Matrix4f().scaling(1.0F, 0.0F, 2.0F)),
            new Transformation(new Matrix4f().zero()),
            new Transformation(new Matrix4f(1.0F, 0.0F, 0.0F, 0.0F, 0.0F, 1.0F, 0.0F, 0.0F, 0.0F, 0.0F, 1.0F, 0.0F, 1.0F, 2.0F, 3.0F, 2.0F)),
            new Transformation(JomlValues.cameraProjection(false)),
            new Transformation(new Matrix4f(Float.NaN, 0.0F, 0.0F, 0.0F, 0.0F, 1.0F, 0.0F, 0.0F, 0.0F, 0.0F, 1.0F, 0.0F, 0.0F, 0.0F, 0.0F, 1.0F)),
            new Transformation(new Matrix4f().translation(Float.POSITIVE_INFINITY, 0.0F, 1.0F))
        ),
        MathValues::randomTransformation,
        MathValues::copyTransformation,
        MathValues::encodeTransformation
    );

    // The only Optional a covered method takes without a generator for its type argument is Transformation.compose's
    // Optional<Transformation>: empty is null.
    static final Values.Generator OPTIONAL_TRANSFORMATION = new Values.Generator() {
        @Override
        public List<Object> edges() {
            return List.of(Optional.empty(), Optional.of(Transformation.IDENTITY), Optional.of(TRANSFORMATION.edges().get(3)));
        }

        @Override
        public Object random(final Random random) {
            return random.nextInt(4) == 0 ? Optional.empty() : Optional.of(TRANSFORMATION.random(random));
        }

        @Override
        public JsonElement encode(final Object spec) {
            return ((Optional<?>)spec).map(TRANSFORMATION::encode).orElse(JsonNull.INSTANCE);
        }

        @Override
        public Object instantiate(final Object spec) {
            return ((Optional<?>)spec).map(TRANSFORMATION::instantiate);
        }

        @Override
        public JsonElement after(final Object spec, final Object instance) {
            JsonElement state = this.encode(instance);
            return state.equals(this.encode(spec)) ? null : state;
        }
    };

    private MathValues() {
    }

    static Values.Generator generator(final Class<?> type) {
        if (type == GivensParameters.class) {
            return GIVENS_PARAMETERS;
        }
        if (type == Transformation.class) {
            return TRANSFORMATION;
        }
        if (type == Optional.class) {
            return OPTIONAL_TRANSFORMATION;
        }
        return null;
    }

    static boolean canEncode(final Class<?> type) {
        return type == Transformation.class;
    }

    static JsonElement encode(final Object value) {
        return TRANSFORMATION.encode(value);
    }

    // BannerRenderer's transformation for a banner turned by angle degrees.
    private static Transformation banner(final float angle) {
        return new Transformation(
            new Vector3f(0.5F, 0.0F, 0.5F), Axis.YP.rotationDegrees(angle), new Vector3f(0.6666667F, -0.6666667F, -0.6666667F), null
        );
    }

    // BlockMath.blockCenterToCorner's matrix.
    private static Transformation centerToCorner(final Matrix4f matrix) {
        return new Transformation(new Matrix4f().translation(0.5F, 0.5F, 0.5F).mul(matrix).translate(-0.5F, -0.5F, -0.5F));
    }

    private static Transformation decomposed(final Transformation transformation) {
        transformation.translation();
        return transformation;
    }

    // Any JOML matrix, the parts entities and block entities build from (any of them null), the matrices of those parts
    // (which have to be decomposed back), and blocks' rotations. One in four is decomposed already.
    private static Transformation randomTransformation(final Random random) {
        Transformation transformation = switch (random.nextInt(5)) {
            case 0 -> new Transformation(new Matrix4f((Matrix4f)JomlValues.MATRIX4F.random(random)));
            case 1 -> randomParts(random);
            case 2, 3 -> new Transformation(randomParts(random).getMatrixCopy());
            default -> {
                Matrix4f matrix = switch (random.nextInt(3)) {
                    case 0 -> new Matrix4f(OctahedralGroup.values()[random.nextInt(OctahedralGroup.values().length)].transformation());
                    case 1 -> new Matrix4f().rotationAround(Axis.YP.rotationDegrees(random.nextInt(16) * 22.5F), 0.5F, random.nextInt(2) * 0.5F, 0.5F);
                    default -> new Matrix4f().rotationAround(JomlValues.randomQuaternionf(random), 0.5F, 0.5F, 0.5F);
                };
                yield random.nextBoolean() ? centerToCorner(matrix) : new Transformation(matrix);
            }
        };
        return random.nextInt(4) == 0 ? decomposed(transformation) : transformation;
    }

    private static Transformation randomParts(final Random random) {
        return new Transformation(
            random.nextInt(4) == 0 ? null : randomTranslation(random), random.nextInt(4) == 0 ? null : randomRotation(random),
            random.nextInt(4) == 0 ? null : randomScale(random), random.nextInt(4) == 0 ? null : randomRotation(random)
        );
    }

    // Model coordinates in sixteenths of a block, or anything.
    static Vector3f randomTranslation(final Random random) {
        return random.nextBoolean()
            ? new Vector3f(random.nextInt(33) - 16, random.nextInt(33) - 16, random.nextInt(33) - 16).div(16.0F)
            : JomlValues.randomVector3f(random);
    }

    // Turns by multiples of 22.5 degrees around an axis like signs and banners, or any rotation.
    static Quaternionf randomRotation(final Random random) {
        return random.nextBoolean()
            ? ((Axis)Values.AXES.random(random)).rotationDegrees(random.nextInt(16) * 22.5F)
            : JomlValues.randomQuaternionf(random);
    }

    // Uniform scales, mirrors, flattened ones, and anything.
    static Vector3f randomScale(final Random random) {
        float scale = random.nextBoolean() ? 0.6666667F : random.nextFloat() * 4.0F;
        return switch (random.nextInt(4)) {
            case 0 -> new Vector3f(scale);
            case 1 -> new Vector3f(scale, -scale, -scale);
            case 2 -> new Vector3f(1.0F, random.nextInt(3) == 0 ? 0.0F : scale, 1.0F);
            default -> JomlValues.randomVector3f(random);
        };
    }

    private static Transformation copyTransformation(final Transformation transformation) {
        if (transformation == Transformation.IDENTITY) {
            return transformation;
        }
        Transformation copy = new Transformation(new Matrix4f(transformation.getMatrix()));
        try {
            DECOMPOSED.setBoolean(copy, DECOMPOSED.getBoolean(transformation));
            TRANSLATION.set(copy, JomlValues.VECTOR3F.instantiate(TRANSLATION.get(transformation)));
            LEFT_ROTATION.set(copy, JomlValues.QUATERNIONF.instantiate(LEFT_ROTATION.get(transformation)));
            SCALE.set(copy, JomlValues.VECTOR3F.instantiate(SCALE.get(transformation)));
            RIGHT_ROTATION.set(copy, JomlValues.QUATERNIONF.instantiate(RIGHT_ROTATION.get(transformation)));
        } catch (IllegalAccessException e) {
            throw new IllegalStateException(e);
        }
        return copy;
    }

    private static JsonArray encodeTransformation(final Transformation transformation) {
        JsonArray array = new JsonArray();
        try {
            array.add(JomlValues.MATRIX4F.encode(transformation.getMatrix()));
            array.add(DECOMPOSED.getBoolean(transformation));
            array.add(JomlValues.VECTOR3F.encode((Vector3fc)TRANSLATION.get(transformation)));
            array.add(JomlValues.QUATERNIONF.encode((Quaternionfc)LEFT_ROTATION.get(transformation)));
            array.add(JomlValues.VECTOR3F.encode((Vector3fc)SCALE.get(transformation)));
            array.add(JomlValues.QUATERNIONF.encode((Quaternionfc)RIGHT_ROTATION.get(transformation)));
        } catch (IllegalAccessException e) {
            throw new IllegalStateException(e);
        }
        array.add(transformation == Transformation.IDENTITY);
        return array;
    }

    private static Field transformationField(final String name) {
        try {
            Field field = Transformation.class.getDeclaredField(name);
            field.setAccessible(true);
            return field;
        } catch (NoSuchFieldException e) {
            throw new IllegalStateException(e);
        }
    }
}
