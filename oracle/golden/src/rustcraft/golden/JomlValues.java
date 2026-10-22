package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonPrimitive;
import java.util.List;
import java.util.Random;
import java.util.function.Function;
import org.joml.Matrix3f;
import org.joml.Matrix3fc;
import org.joml.Matrix4f;
import org.joml.Matrix4fc;
import org.joml.Quaternionf;
import org.joml.Quaternionfc;
import org.joml.Vector3f;
import org.joml.Vector3fc;
import org.joml.Vector4f;
import org.joml.Vector4fc;

/**
 * Arguments and receivers of JOML's types, encoded as their fields in declaration order: a Vector3f is
 * {@code [x, y, z]}, a Vector4f and a Quaternionf {@code [x, y, z, w]}, a Matrix3f {@code [m00, m01, …, m22]}, and a Matrix4f
 * {@code [m00, m01, …, m33, properties]}. Matrix4f's methods take shortcuts based on its properties, so they're
 * part of its state, and every matrix here gets them from JOML itself, the way the game's do.
 */
final class JomlValues {
    private static final float DEG_TO_RAD = (float)(Math.PI / 180.0);

    static final Values.Generator VECTOR3F = new Joml<>(
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

    static final Values.Generator VECTOR4F = new Joml<>(
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

    static final Values.Generator QUATERNIONF = new Joml<>(
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

    static final Values.Generator MATRIX3F = new Joml<>(
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

    static final Values.Generator MATRIX4F = new Joml<>(
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

    private JomlValues() {
    }

    static Values.Generator generator(final Class<?> type) {
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
            case Vector3fc vector -> VECTOR3F.encode(vector);
            case Vector4fc vector -> VECTOR4F.encode(vector);
            case Quaternionfc quaternion -> QUATERNIONF.encode(quaternion);
            case Matrix3fc matrix -> MATRIX3F.encode(matrix);
            case Matrix4fc matrix -> MATRIX4F.encode(matrix);
            default -> throw new IllegalArgumentException("no encoder for " + value.getClass().getName());
        };
    }

    /**
     * Specs are JOML objects nobody mutates. Each call gets a copy, and an argument whose state changed afterwards
     * (one the method wrote to) is recorded in {@code after}.
     */
    private record Joml<T>(List<T> edgeList, Function<Random, T> fresh, Function<T, T> copy, Function<T, JsonArray> encoder)
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
            return this.encoder.apply((T)spec);
        }

        @SuppressWarnings("unchecked")
        @Override
        public Object instantiate(final Object spec) {
            return this.copy.apply((T)spec);
        }

        @Override
        public JsonElement after(final Object spec, final Object instance) {
            JsonElement state = this.encode(instance);
            return state.equals(this.encode(spec)) ? null : state;
        }
    }

    private static JsonArray floats(final float... values) {
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
    private static Matrix4f cameraProjection(final boolean zZeroToOne) {
        return new Matrix4f().setPerspective(70.0F * DEG_TO_RAD, 854.0F / 480.0F, 256.0F, 0.05F, zZeroToOne);
    }

    private static Matrix4f affineMatrix() {
        return new Matrix4f().translation(-0.5F, 61.62F, -0.5F).rotateY(0.7F).rotateX(-0.2F).scale(0.5F, 2.0F, -1.0F);
    }

    private static float component(final Random random) {
        return switch (random.nextInt(4)) {
            case 0 -> (Float)Values.FLOATS.random(random);
            case 1 -> random.nextFloat() * 2.0F - 1.0F;
            case 2 -> (random.nextFloat() * 2.0F - 1.0F) * 400.0F;
            default -> random.nextInt(9) - 4;
        };
    }

    private static float angle(final Random random) {
        return random.nextInt(8) == 0 ? (Float)Values.FLOATS.random(random) : (random.nextFloat() * 2.0F - 1.0F) * 4.0F * (float)Math.PI;
    }

    private static Vector3f randomVector3f(final Random random) {
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

    private static Quaternionf randomQuaternionf(final Random random) {
        return switch (random.nextInt(4)) {
            case 0 -> new Quaternionf().rotationYXZ(angle(random), angle(random), angle(random));
            case 1 -> new Quaternionf().rotationAxis(angle(random), randomVector3f(random).normalize());
            case 2 -> new Quaternionf(component(random), component(random), component(random), component(random));
            default -> cameraRotation((random.nextFloat() * 2.0F - 1.0F) * 360.0F, (random.nextFloat() * 2.0F - 1.0F) * 90.0F);
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

    private static Matrix4f randomMatrix4f(final Random random) {
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

    // Transforms like the game's pose stacks build, one to four steps from identity.
    private static Matrix4f composed(final Random random) {
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
