package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonNull;
import com.google.gson.JsonPrimitive;
import java.lang.reflect.Field;
import java.util.ArrayList;
import java.util.List;
import java.util.Random;
import java.util.function.Function;
import org.joml.FrustumIntersection;
import org.joml.Matrix3f;
import org.joml.Matrix3fc;
import org.joml.Matrix3x2f;
import org.joml.Matrix3x2fc;
import org.joml.Matrix4f;
import org.joml.Matrix4fc;
import org.joml.Quaternionf;
import org.joml.Quaternionfc;
import org.joml.Vector2f;
import org.joml.Vector2fc;
import org.joml.Vector3f;
import org.joml.Vector3fc;
import org.joml.Vector4f;
import org.joml.Vector4fc;

/**
 * Arguments and receivers of JOML's types, encoded as their fields in declaration order: a Vector2f is {@code [x, y]},
 * a Vector3f {@code [x, y, z]}, a Vector4f and a Quaternionf {@code [x, y, z, w]}, a Matrix3x2f
 * {@code [m00, m01, m10, m11, m20, m21]}, a Matrix3f {@code [m00, m01, …, m22]}, a Matrix4f
 * {@code [m00, m01, …, m33, properties]}, and a FrustumIntersection its 24 plane components
 * {@code [nxX, nxY, nxZ, nxW, pxX, …, pzW]} followed by its {@code planes} array of six Vector4f. Matrix4f's methods take
 * shortcuts based on its properties, so they're part of its state, and every matrix here gets them from JOML itself,
 * the way the game's do.
 */
final class JomlValues {
    private static final float DEG_TO_RAD = (float)(Math.PI / 180.0);
    private static final List<Field> FRUSTUM_PLANE_FIELDS = frustumFields();
    private static final Field FRUSTUM_PLANES = frustumField("planes");

    static final Values.Generator VECTOR2F = new Mutable<>(
        List.of(
            new Vector2f(), new Vector2f(1.0F, 0.0F), new Vector2f(0.0F, 1.0F), new Vector2f(-1.0F, 0.0F), new Vector2f(-0.0F, -0.0F),
            new Vector2f(1.0F, 2.0F), new Vector2f(0.5F, -60.0F), new Vector2f(427.0F, 240.0F), new Vector2f(1.0E-20F, -1.0E-20F),
            new Vector2f(1.0E20F, -1.0E20F), new Vector2f(Float.MAX_VALUE, 1.0F), new Vector2f(Float.MIN_VALUE, 0.0F),
            new Vector2f(Float.POSITIVE_INFINITY, 1.0F), new Vector2f(Float.NaN, 0.0F)
        ),
        JomlValues::randomVector2f,
        Vector2f::new,
        vector -> floats(vector.x(), vector.y())
    );

    static final Values.Generator VECTOR3F = new Mutable<>(
        List.of(
            new Vector3f(), new Vector3f(1.0F, 0.0F, 0.0F), new Vector3f(0.0F, 1.0F, 0.0F), new Vector3f(0.0F, 0.0F, 1.0F),
            new Vector3f(-1.0F, 0.0F, 0.0F), new Vector3f(0.0F, 0.0F, -1.0F), new Vector3f(-0.0F, -0.0F, -0.0F),
            new Vector3f(1.0F, 2.0F, 3.0F), new Vector3f(0.5F, -60.0F, 0.5F), new Vector3f(0.0F, 0.70710677F, 0.70710677F),
            new Vector3f(1.0E-20F, -1.0E-20F, 1.0E-20F), new Vector3f(1.0E20F, 1.0E20F, -1.0E20F), new Vector3f(Float.MAX_VALUE, 1.0F, 0.0F),
            new Vector3f(Float.MIN_VALUE, 0.0F, 0.0F), new Vector3f(Float.POSITIVE_INFINITY, 1.0F, 0.0F), new Vector3f(Float.NaN, 0.0F, 1.0F)
        ),
        JomlValues::randomVector3f,
        Vector3f::new,
        vector -> floats(vector.x(), vector.y(), vector.z())
    );

    static final Values.Generator VECTOR4F = new Mutable<>(
        List.of(
            new Vector4f(), new Vector4f(0.0F), new Vector4f(1.0F, 1.0F, 1.0F, 1.0F), new Vector4f(0.0F, 0.0F, 1.0F, 0.0F),
            new Vector4f(-1.0F, -1.0F, -1.0F, 1.0F), new Vector4f(1.0F, 1.0F, 1.0F, -1.0F), new Vector4f(0.5F, 0.25F, 1.0F, 0.5F),
            new Vector4f(-0.0F, -0.0F, -0.0F, -0.0F), new Vector4f(-Float.MAX_VALUE, 0.0F, 0.0F, 0.0F), new Vector4f(0.5F, -60.0F, 0.5F, 1.0F),
            new Vector4f(1.0E-20F, -1.0E-20F, 1.0E-20F, 1.0E-20F), new Vector4f(1.0E20F, 1.0E20F, -1.0E20F, 1.0E20F),
            new Vector4f(Float.MIN_VALUE, 0.0F, 0.0F, 1.0F), new Vector4f(Float.POSITIVE_INFINITY, 1.0F, 0.0F, 1.0F),
            new Vector4f(Float.NaN, 0.0F, 1.0F, 1.0F)
        ),
        JomlValues::randomVector4f,
        Vector4f::new,
        vector -> floats(vector.x(), vector.y(), vector.z(), vector.w())
    );

    static final Values.Generator QUATERNIONF = new Mutable<>(
        List.of(
            new Quaternionf(), new Quaternionf(1.0F, 0.0F, 0.0F, 0.0F), new Quaternionf(0.0F, 1.0F, 0.0F, 0.0F),
            new Quaternionf(0.0F, 0.0F, 1.0F, 0.0F), new Quaternionf(0.0F, 0.0F, 0.0F, -1.0F), new Quaternionf(0.0F, 0.0F, 0.0F, 0.0F),
            new Quaternionf(-0.0F, -0.0F, -0.0F, 1.0F), new Quaternionf().rotationX(0.5F), new Quaternionf().rotationY((float)Math.PI),
            new Quaternionf().rotationZ(-2.0F), cameraRotation(0.0F, 0.0F), cameraRotation(90.0F, 10.0F), cameraRotation(-135.5F, -90.0F),
            new Quaternionf(1.0F, 2.0F, 3.0F, 4.0F), new Quaternionf(Float.NaN, 0.0F, 0.0F, 1.0F),
            new Quaternionf(Float.POSITIVE_INFINITY, 0.0F, 0.0F, 1.0F)
        ),
        JomlValues::randomQuaternionf,
        Quaternionf::new,
        quaternion -> floats(quaternion.x(), quaternion.y(), quaternion.z(), quaternion.w())
    );

    // The GUI's poses: identities, pixel translations, the scales and rotations GuiGraphicsExtractor and the HUD apply.
    static final Values.Generator MATRIX3X2F = new Mutable<>(
        List.of(
            new Matrix3x2f(),
            new Matrix3x2f().zero(),
            new Matrix3x2f().translation(0.5F, -60.0F),
            new Matrix3x2f(1.0F, -0.0F, 0.0F, 1.0F, -0.0F, 0.0F),
            new Matrix3x2f().scaling(2.0F),
            new Matrix3x2f().scaling(-1.0F, 1.0F),
            new Matrix3x2f().scaling(0.0F, 1.0F),
            new Matrix3x2f().rotation(0.5F),
            new Matrix3x2f().rotation((float)Math.PI),
            mapDecorationPose(37, -12, 5),
            new Matrix3x2f().translation(213.5F, 120.0F).scale(0.5F),
            new Matrix3x2f(1.0F, 2.0F, 3.0F, 4.0F, 5.0F, 6.0F),
            new Matrix3x2f(Float.NaN, 0.0F, 0.0F, 1.0F, 0.0F, 0.0F),
            new Matrix3x2f().translation(Float.POSITIVE_INFINITY, 1.0F)
        ),
        JomlValues::randomMatrix3x2f,
        matrix -> new Matrix3x2f().set(matrix),
        matrix -> floats(matrix.m00(), matrix.m01(), matrix.m10(), matrix.m11(), matrix.m20(), matrix.m21())
    );

    static final Values.Generator MATRIX3F = new Mutable<>(
        List.of(
            new Matrix3f(), new Matrix3f().zero(), new Matrix3f().rotationX(0.5F), new Matrix3f().rotationY(-1.0F),
            new Matrix3f().rotation(cameraRotation(90.0F, 10.0F)), new Matrix3f().scaling(2.0F, 3.0F, 4.0F),
            new Matrix3f().scaling(-1.0F, 1.0F, 1.0F), new Matrix3f().scaling(0.0F, 1.0F, 1.0F),
            new Matrix3f(affineMatrix()).invert().transpose(), new Matrix3f(cameraProjection(false)),
            new Matrix3f(1.0F, 2.0F, 3.0F, 4.0F, 5.0F, 6.0F, 7.0F, 8.0F, 10.0F),
            new Matrix3f(Float.NaN, 0.0F, 0.0F, 0.0F, 1.0F, 0.0F, 0.0F, 0.0F, 1.0F),
            new Matrix3f(Float.POSITIVE_INFINITY, 0.0F, 0.0F, 0.0F, 1.0F, 0.0F, 0.0F, 0.0F, 1.0F)
        ),
        JomlValues::randomMatrix3f,
        matrix -> new Matrix3f().set(matrix),
        matrix -> floats(
            matrix.m00(), matrix.m01(), matrix.m02(), matrix.m10(), matrix.m11(), matrix.m12(), matrix.m20(), matrix.m21(), matrix.m22()
        )
    );

    static final Values.Generator MATRIX4F = new Mutable<>(
        List.of(
            new Matrix4f(),
            new Matrix4f().zero(),
            new Matrix4f().translation(0.5F, -60.0F, 0.5F),
            new Matrix4f().translation(-0.0F, 0.0F, -0.0F),
            // -0.0 compares equal to 0.0, so this one is the identity as far as its properties go.
            new Matrix4f(1.0F, -0.0F, 0.0F, 0.0F, 0.0F, 1.0F, 0.0F, 0.0F, 0.0F, 0.0F, 1.0F, 0.0F, -0.0F, 0.0F, -0.0F, 1.0F),
            new Matrix4f().rotationX(0.5F),
            new Matrix4f().rotationY((float)Math.PI),
            new Matrix4f().rotationZ(-2.0F),
            new Matrix4f().rotation(cameraRotation(90.0F, 10.0F)),
            new Matrix4f().scaling(2.0F, 3.0F, 4.0F),
            new Matrix4f().scaling(-1.0F, 1.0F, 1.0F),
            new Matrix4f().scaling(0.0F, 1.0F, 1.0F),
            affineMatrix(),
            cameraProjection(false),
            cameraProjection(true),
            new Matrix4f().perspective(70.0F * DEG_TO_RAD, 854.0F / 480.0F, 0.05F, Float.POSITIVE_INFINITY),
            // The GUI's projection, also with near and far swapped like Projection.getMatrix does.
            new Matrix4f().setOrtho(0.0F, 427.0F, 240.0F, 0.0F, 21000.0F, 1000.0F, false),
            cameraProjection(false).mul(new Matrix4f().rotation(cameraRotation(90.0F, 10.0F).conjugate())),
            new Matrix4f(1.0F, 2.0F, 3.0F, 4.0F, 5.0F, 6.0F, 7.0F, 8.0F, 9.0F, 10.0F, 11.0F, 12.0F, 13.0F, 14.0F, 15.0F, 17.0F),
            new Matrix4f(Float.NaN, 0.0F, 0.0F, 0.0F, 0.0F, 1.0F, 0.0F, 0.0F, 0.0F, 0.0F, 1.0F, 0.0F, 0.0F, 0.0F, 0.0F, 1.0F),
            new Matrix4f().translation(Float.POSITIVE_INFINITY, 0.0F, 1.0F)
        ),
        JomlValues::randomMatrix4f,
        matrix -> new Matrix4f().set(matrix),
        matrix -> {
            JsonArray array = floats(
                matrix.m00(), matrix.m01(), matrix.m02(), matrix.m03(), matrix.m10(), matrix.m11(), matrix.m12(), matrix.m13(),
                matrix.m20(), matrix.m21(), matrix.m22(), matrix.m23(), matrix.m30(), matrix.m31(), matrix.m32(), matrix.m33()
            );
            array.add(matrix.properties());
            return array;
        }
    );

    // Camera's culling frustums (the walk scenario's two looks among them), and frustums of any matrix.
    static final Values.Generator FRUSTUM_INTERSECTION = new Mutable<>(
        List.of(
            new FrustumIntersection(),
            new FrustumIntersection().set(new Matrix4f()),
            cullingFrustum(70.0F, 854.0F / 480.0F, 2048.0F, false, 0.0F, 0.0F),
            cullingFrustum(70.0F, 854.0F / 480.0F, 2048.0F, false, 90.0F, 10.0F),
            cullingFrustum(70.0F, 854.0F / 480.0F, 256.0F, true, -135.5F, -90.0F),
            cullingFrustum(110.0F, 0.5F, 64.0F, false, 180.0F, 90.0F),
            // A perspective that ignores x: every plane's x is zero.
            new FrustumIntersection().set(new Matrix4f().perspective(70.0F * DEG_TO_RAD, 854.0F / 480.0F, 0.05F, 2048.0F).scale(0.0F, 1.0F, 1.0F)),
            new FrustumIntersection().set(cameraProjection(false)),
            new FrustumIntersection().set(new Matrix4f().setOrtho(0.0F, 427.0F, 240.0F, 0.0F, 21000.0F, 1000.0F, false)),
            new FrustumIntersection().set(new Matrix4f().zero()),
            new FrustumIntersection().set(
                new Matrix4f(1.0F, 2.0F, 3.0F, 4.0F, 5.0F, 6.0F, 7.0F, 8.0F, 9.0F, 10.0F, 11.0F, 12.0F, 13.0F, 14.0F, 15.0F, 17.0F)
            ),
            new FrustumIntersection().set(
                new Matrix4f(Float.NaN, 0.0F, 0.0F, 0.0F, 0.0F, 1.0F, 0.0F, 0.0F, 0.0F, 0.0F, 1.0F, 0.0F, 0.0F, 0.0F, 0.0F, 1.0F)
            ),
            new FrustumIntersection().set(new Matrix4f().translation(Float.POSITIVE_INFINITY, 0.0F, 1.0F))
        ),
        JomlValues::randomFrustumIntersection,
        JomlValues::copyFrustum,
        JomlValues::encodeFrustum
    );

    private JomlValues() {
    }

    static Values.Generator generator(final Class<?> type) {
        if (type == Vector2f.class || type == Vector2fc.class) {
            return VECTOR2F;
        }
        if (type == Matrix3x2f.class || type == Matrix3x2fc.class) {
            return MATRIX3X2F;
        }
        if (type == FrustumIntersection.class) {
            return FRUSTUM_INTERSECTION;
        }
        if (type == Vector3f.class || type == Vector3fc.class) {
            return VECTOR3F;
        }
        if (type == Vector4f.class || type == Vector4fc.class) {
            return VECTOR4F;
        }
        if (type == Quaternionf.class || type == Quaternionfc.class) {
            return QUATERNIONF;
        }
        if (type == Matrix3f.class || type == Matrix3fc.class) {
            return MATRIX3F;
        }
        if (type == Matrix4f.class || type == Matrix4fc.class) {
            return MATRIX4F;
        }
        return null;
    }

    static boolean canEncode(final Class<?> type) {
        return generator(type) != null;
    }

    static JsonElement encode(final Object value) {
        return switch (value) {
            case Vector2fc vector -> VECTOR2F.encode(vector);
            case Matrix3x2fc matrix -> MATRIX3X2F.encode(matrix);
            case FrustumIntersection frustum -> FRUSTUM_INTERSECTION.encode(frustum);
            case Vector3fc vector -> VECTOR3F.encode(vector);
            case Vector4fc vector -> VECTOR4F.encode(vector);
            case Quaternionfc quaternion -> QUATERNIONF.encode(quaternion);
            case Matrix3fc matrix -> MATRIX3F.encode(matrix);
            case Matrix4fc matrix -> MATRIX4F.encode(matrix);
            default -> throw new IllegalArgumentException("no encoder for " + value.getClass().getName());
        };
    }

    /**
     * Specs are mutable objects nobody mutates. Each call gets a copy, and an argument whose state changed afterwards
     * (one the method wrote to) is recorded in {@code after}. A null spec (an argument the method lets be null) is
     * passed as null.
     */
    record Mutable<T>(List<T> edgeList, Function<Random, T> fresh, Function<T, T> copy, Function<T, JsonArray> encoder)
        implements Values.Generator {
        @Override
        public List<Object> edges() {
            return List.copyOf(this.edgeList);
        }

        @Override
        public Object random(final Random random) {
            return random.nextInt(8) == 0 ? this.edgeList.get(random.nextInt(this.edgeList.size())) : this.fresh.apply(random);
        }

        @SuppressWarnings("unchecked")
        @Override
        public JsonElement encode(final Object spec) {
            return spec == null ? JsonNull.INSTANCE : this.encoder.apply((T)spec);
        }

        @SuppressWarnings("unchecked")
        @Override
        public Object instantiate(final Object spec) {
            return spec == null ? null : this.copy.apply((T)spec);
        }

        @Override
        public JsonElement after(final Object spec, final Object instance) {
            JsonElement state = this.encode(instance);
            return state.equals(this.encode(spec)) ? null : state;
        }
    }

    static JsonArray floats(final float... values) {
        JsonArray array = new JsonArray();
        for (float value : values) {
            array.add(new JsonPrimitive(Values.floatBits(value)));
        }
        return array;
    }

    // Camera.setRotation's quaternion for a look direction in degrees.
    private static Quaternionf cameraRotation(final float yRot, final float xRot) {
        return new Quaternionf().rotationYXZ((float)Math.PI - yRot * DEG_TO_RAD, -xRot * DEG_TO_RAD, 0.0F);
    }

    // Projection.getMatrix's perspective for the walk scenario, which swaps near and far (reversed depth).
    static Matrix4f cameraProjection(final boolean zZeroToOne) {
        return new Matrix4f().setPerspective(70.0F * DEG_TO_RAD, 854.0F / 480.0F, 256.0F, 0.05F, zZeroToOne);
    }

    // Camera.createProjectionMatrixForCulling's perspective times the view rotation, which is the frustum the game culls with.
    static FrustumIntersection cullingFrustum(
        final float fov, final float aspect, final float depthFar, final boolean zZeroToOne, final float yRot, final float xRot
    ) {
        Matrix4f projection = new Matrix4f().perspective(fov * DEG_TO_RAD, aspect, 0.05F, depthFar, zZeroToOne);
        Matrix4f view = new Matrix4f().rotation(cameraRotation(yRot, xRot).conjugate(new Quaternionf()));
        return new FrustumIntersection().set(projection.mul(view, new Matrix4f()));
    }

    // GuiGraphicsExtractor's pose for a map decoration at (x, y) with rotation rot (in sixteenths of a turn).
    private static Matrix3x2f mapDecorationPose(final int x, final int y, final int rot) {
        return new Matrix3x2f()
            .translate(x / 2.0F + 64.0F, y / 2.0F + 64.0F)
            .rotate((float)(Math.PI / 180.0) * rot * 360.0F / 16.0F)
            .scale(4.0F, 4.0F)
            .translate(-0.125F, 0.125F);
    }

    static Matrix4f affineMatrix() {
        return new Matrix4f().translation(-0.5F, 61.62F, -0.5F).rotateY(0.7F).rotateX(-0.2F).scale(0.5F, 2.0F, -1.0F);
    }

    static float component(final Random random) {
        return switch (random.nextInt(4)) {
            case 0 -> (Float)Values.FLOATS.random(random);
            case 1 -> random.nextFloat() * 2.0F - 1.0F;
            case 2 -> (random.nextFloat() * 2.0F - 1.0F) * 400.0F;
            default -> random.nextInt(9) - 4;
        };
    }

    static float angle(final Random random) {
        return random.nextInt(8) == 0 ? (Float)Values.FLOATS.random(random) : (random.nextFloat() * 2.0F - 1.0F) * 4.0F * (float)Math.PI;
    }

    // Mouse and screen positions, whole and in between pixels, and anything else.
    private static Vector2f randomVector2f(final Random random) {
        return switch (random.nextInt(4)) {
            case 0 -> new Vector2f(component(random), component(random));
            case 1 -> new Vector2f(random.nextInt(855), random.nextInt(481));
            case 2 -> new Vector2f(random.nextFloat() * 854.0F, random.nextFloat() * 480.0F);
            default -> new Vector2f((Float)Values.FLOATS.random(random), (Float)Values.FLOATS.random(random));
        };
    }

    static Vector3f randomVector3f(final Random random) {
        return switch (random.nextInt(3)) {
            case 0 -> new Vector3f(component(random), component(random), component(random));
            case 1 -> new Vector3f(random.nextFloat() * 2.0F - 1.0F, random.nextFloat() * 2.0F - 1.0F, random.nextFloat() * 2.0F - 1.0F).normalize();
            default -> new Vector3f((Float)Values.FLOATS.random(random), (Float)Values.FLOATS.random(random), (Float)Values.FLOATS.random(random));
        };
    }

    // Colours, clip-space points like the frustum's corners, and anything else.
    private static Vector4f randomVector4f(final Random random) {
        return switch (random.nextInt(3)) {
            case 0 -> new Vector4f(random.nextFloat(), random.nextFloat(), random.nextFloat(), random.nextFloat());
            case 1 -> new Vector4f(component(random), component(random), component(random), random.nextBoolean() ? 1.0F : component(random));
            default -> new Vector4f(
                (Float)Values.FLOATS.random(random), (Float)Values.FLOATS.random(random), (Float)Values.FLOATS.random(random),
                (Float)Values.FLOATS.random(random)
            );
        };
    }

    static Quaternionf randomQuaternionf(final Random random) {
        return switch (random.nextInt(4)) {
            case 0 -> new Quaternionf().rotationYXZ(angle(random), angle(random), angle(random));
            case 1 -> new Quaternionf().rotationAxis(angle(random), randomVector3f(random).normalize());
            case 2 -> new Quaternionf(component(random), component(random), component(random), component(random));
            default -> cameraRotation((random.nextFloat() * 2.0F - 1.0F) * 360.0F, (random.nextFloat() * 2.0F - 1.0F) * 90.0F);
        };
    }

    // Poses like the GUI's, one to four steps from identity, or any components.
    private static Matrix3x2f randomMatrix3x2f(final Random random) {
        if (random.nextInt(4) == 0) {
            return new Matrix3x2f(component(random), component(random), component(random), component(random), component(random), component(random));
        }
        Matrix3x2f matrix = new Matrix3x2f();
        int steps = 1 + random.nextInt(4);
        for (int i = 0; i < steps; i++) {
            switch (random.nextInt(5)) {
                case 0 -> matrix.translate(random.nextInt(855) - 427 + (random.nextBoolean() ? 0.5F : 0.0F), random.nextInt(481) - 240);
                case 1 -> matrix.translate(component(random), component(random));
                case 2 -> matrix.scale(guiScale(random));
                case 3 -> matrix.scale(component(random), component(random));
                default -> matrix.rotate(angle(random));
            }
        }
        return matrix;
    }

    private static float guiScale(final Random random) {
        return switch (random.nextInt(4)) {
            case 0 -> 0.5F;
            case 1 -> 1.0F + random.nextInt(4);
            case 2 -> 1.0F / (1.0F + random.nextInt(4));
            default -> random.nextFloat() * 4.0F;
        };
    }

    private static Matrix3f randomMatrix3f(final Random random) {
        return switch (random.nextInt(4)) {
            case 0 -> new Matrix3f(randomMatrix4f(random));
            case 1 -> new Matrix3f().rotation(randomQuaternionf(random));
            case 2 -> new Matrix3f(composed(random)).invert().transpose();
            default -> new Matrix3f(
                component(random), component(random), component(random), component(random), component(random),
                component(random), component(random), component(random), component(random)
            );
        };
    }

    static Matrix4f randomMatrix4f(final Random random) {
        return switch (random.nextInt(5)) {
            case 0, 1 -> composed(random);
            case 2 -> {
                float fov = (random.nextFloat() * 100.0F + 20.0F) * DEG_TO_RAD;
                float aspect = random.nextFloat() * 3.0F + 0.25F;
                Matrix4f projection = random.nextBoolean()
                    ? new Matrix4f().setPerspective(fov, aspect, random.nextFloat() * 1000.0F, random.nextFloat() * 0.1F, random.nextBoolean())
                    : new Matrix4f().setOrtho(0.0F, random.nextFloat() * 1000.0F, random.nextFloat() * 1000.0F, 0.0F, 21000.0F, 1000.0F, random.nextBoolean());
                yield random.nextBoolean() ? projection : projection.mul(composed(random));
            }
            case 3 -> new Matrix4f(
                component(random), component(random), component(random), component(random),
                component(random), component(random), component(random), component(random),
                component(random), component(random), component(random), component(random),
                component(random), component(random), component(random), component(random)
            );
            // Affine, but built from components, so its rotation part isn't one.
            default -> new Matrix4f(
                component(random), component(random), component(random), 0.0F,
                component(random), component(random), component(random), 0.0F,
                component(random), component(random), component(random), 0.0F,
                component(random), component(random), component(random), 1.0F
            );
        };
    }

    // Mostly cameras looking anywhere, with any FOV the options and effects give, near and far render distances, and
    // either depth range; otherwise the frustum of any matrix.
    private static FrustumIntersection randomFrustumIntersection(final Random random) {
        if (random.nextInt(4) == 0) {
            return new FrustumIntersection().set(randomMatrix4f(random));
        }
        float fov = random.nextBoolean() ? 70.0F : 30.0F + random.nextFloat() * 80.0F;
        float aspect = random.nextBoolean() ? 854.0F / 480.0F : random.nextFloat() * 3.0F + 0.25F;
        float depthFar = (2 + random.nextInt(31)) * 64.0F;
        float yRot = (random.nextFloat() * 2.0F - 1.0F) * 360.0F;
        float xRot = random.nextInt(8) == 0 ? (random.nextBoolean() ? 90.0F : -90.0F) : (random.nextFloat() * 2.0F - 1.0F) * 90.0F;
        return cullingFrustum(fov, aspect, depthFar, random.nextBoolean(), yRot, xRot);
    }

    private static List<Field> frustumFields() {
        List<Field> fields = new ArrayList<>();
        for (String plane : List.of("nx", "px", "ny", "py", "nz", "pz")) {
            for (String component : List.of("X", "Y", "Z", "W")) {
                fields.add(frustumField(plane + component));
            }
        }
        return fields;
    }

    private static Field frustumField(final String name) {
        try {
            Field field = FrustumIntersection.class.getDeclaredField(name);
            field.setAccessible(true);
            return field;
        } catch (NoSuchFieldException e) {
            throw new IllegalStateException(e);
        }
    }

    /** Plane {@code index} of a frustum (PLANE_NX, …) as {@code [x, y, z, w]}. */
    static float[] frustumPlane(final FrustumIntersection frustum, final int index) {
        float[] plane = new float[4];
        try {
            for (int i = 0; i < 4; i++) {
                plane[i] = FRUSTUM_PLANE_FIELDS.get(index * 4 + i).getFloat(frustum);
            }
        } catch (IllegalAccessException e) {
            throw new IllegalStateException(e);
        }
        return plane;
    }

    private static FrustumIntersection copyFrustum(final FrustumIntersection frustum) {
        FrustumIntersection copy = new FrustumIntersection();
        try {
            for (Field field : FRUSTUM_PLANE_FIELDS) {
                field.setFloat(copy, field.getFloat(frustum));
            }
            Vector4f[] planes = (Vector4f[])FRUSTUM_PLANES.get(frustum);
            Vector4f[] copyPlanes = (Vector4f[])FRUSTUM_PLANES.get(copy);
            for (int i = 0; i < planes.length; i++) {
                copyPlanes[i].set(planes[i]);
            }
        } catch (IllegalAccessException e) {
            throw new IllegalStateException(e);
        }
        return copy;
    }

    private static JsonArray encodeFrustum(final FrustumIntersection frustum) {
        JsonArray array = new JsonArray();
        try {
            for (Field field : FRUSTUM_PLANE_FIELDS) {
                array.add(new JsonPrimitive(Values.floatBits(field.getFloat(frustum))));
            }
            JsonArray planes = new JsonArray();
            for (Vector4f plane : (Vector4f[])FRUSTUM_PLANES.get(frustum)) {
                planes.add(VECTOR4F.encode(plane));
            }
            array.add(planes);
        } catch (IllegalAccessException e) {
            throw new IllegalStateException(e);
        }
        return array;
    }

    // Transforms like the game's pose stacks build, one to four steps from identity.
    static Matrix4f composed(final Random random) {
        Matrix4f matrix = new Matrix4f();
        int steps = 1 + random.nextInt(4);
        for (int i = 0; i < steps; i++) {
            switch (random.nextInt(6)) {
                case 0 -> matrix.translate(component(random), component(random), component(random));
                case 1 -> matrix.rotate(randomQuaternionf(random));
                case 2 -> {
                    switch (random.nextInt(3)) {
                        case 0 -> matrix.rotateX(angle(random));
                        case 1 -> matrix.rotateY(angle(random));
                        default -> matrix.rotateZ(angle(random));
                    }
                }
                case 3 -> {
                    float scale = component(random);
                    if (random.nextBoolean()) {
                        matrix.scale(scale);
                    } else {
                        matrix.scale(scale, component(random), component(random));
                    }
                }
                case 4 -> matrix.rotate(angle(random), randomVector3f(random).normalize());
                default -> matrix.mul(new Matrix4f().rotationYXZ(angle(random), angle(random), angle(random)));
            }
        }
        return matrix;
    }
}
