package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonNull;
import com.google.gson.JsonObject;
import com.google.gson.JsonPrimitive;
import com.mojang.blaze3d.vertex.PoseStack;
import com.mojang.math.Axis;
import com.mojang.math.Transformation;
import java.io.IOException;
import java.lang.reflect.Field;
import java.util.ArrayList;
import java.util.List;
import java.util.Random;
import org.joml.Matrix4f;
import org.joml.Quaternionf;
import org.joml.Vector3f;

/**
 * Runs random programs against a PoseStack. A case is a fresh stack running the ops in {@code args[0]}; the result
 * has one step per op, {@code [output, pose, normal, trustedNormals]}: what the op returned, then the top pose's
 * state afterwards. An op that throws ends the program. mulPose's output is its matrix argument afterwards, since
 * MatrixUtil.checkProperty adds to the properties of the matrix it's given; with a Transformation, it's the
 * transformation afterwards, whose own matrix that changes.
 */
final class PoseStackSuite implements Suite {
    private static final int PROGRAMS = 512;
    // Programs that also multiply by transformations, which come after the others so those stay as they were.
    private static final int TRANSFORMATION_PROGRAMS = 256;
    private static final int OPS_PER_PROGRAM = 48;
    private static final Field TRUSTED_NORMALS;

    static {
        try {
            TRUSTED_NORMALS = PoseStack.Pose.class.getDeclaredField("trustedNormals");
            TRUSTED_NORMALS.setAccessible(true);
        } catch (NoSuchFieldException e) {
            throw new IllegalStateException(e);
        }
    }

    @Override
    public String name() {
        return PoseStack.class.getName();
    }

    @Override
    public void write(final GoldenWriter writer) throws IOException {
        Random random = new Random(0x9053);
        for (int i = 0; i < PROGRAMS + TRANSFORMATION_PROGRAMS; i++) {
            // One program in four takes any float, NaN and infinities included, which spoil the rest of it.
            List<Op> program = randomProgram(random, i % 4 == 3, i >= PROGRAMS);
            JsonArray args = new JsonArray();
            args.add(Op.encode(program));
            writer.result("PoseStack", args, run(program), null);
        }
    }

    private static JsonArray run(final List<Op> program) {
        PoseStack stack = new PoseStack();
        JsonArray steps = new JsonArray();
        for (Op op : program) {
            JsonElement output;
            try {
                output = apply(stack, op);
            } catch (RuntimeException e) {
                JsonObject thrown = new JsonObject();
                thrown.addProperty("throws", e.getClass().getName());
                steps.add(thrown);
                break;
            }

            PoseStack.Pose last = stack.last();
            JsonArray step = new JsonArray();
            step.add(output);
            step.add(JomlValues.MATRIX4F.encode(last.pose()));
            step.add(JomlValues.MATRIX3F.encode(last.normal()));
            try {
                step.add(TRUSTED_NORMALS.getBoolean(last));
            } catch (IllegalAccessException e) {
                throw new IllegalStateException(e);
            }
            steps.add(step);
        }
        return steps;
    }

    private static JsonElement apply(final PoseStack stack, final Op op) {
        Object[] args = op.args();
        switch (op.name()) {
            case "pushPose" -> stack.pushPose();
            case "popPose" -> stack.popPose();
            case "translate" -> stack.translate((Float)args[0], (Float)args[1], (Float)args[2]);
            case "translateDouble" -> stack.translate((Double)args[0], (Double)args[1], (Double)args[2]);
            case "scale" -> stack.scale((Float)args[0], (Float)args[1], (Float)args[2]);
            case "rotate" -> stack.rotate((Quaternionf)args[0]);
            case "rotateAxis" -> stack.rotate((Axis)args[0], (Float)args[1]);
            case "rotateDegrees" -> stack.rotateDegrees((Axis)args[0], (Float)args[1]);
            case "rotateAround" -> stack.rotateAround((Quaternionf)args[0], (Float)args[1], (Float)args[2], (Float)args[3]);
            case "setIdentity" -> stack.setIdentity();
            case "mulPose" -> {
                Matrix4f matrix = (Matrix4f)JomlValues.MATRIX4F.instantiate(args[0]);
                stack.mulPose(matrix);
                return JomlValues.MATRIX4F.encode(matrix);
            }
            case "mulPoseTransformation" -> {
                Transformation transformation = (Transformation)MathValues.TRANSFORMATION.instantiate(args[0]);
                stack.mulPose(transformation);
                return MathValues.TRANSFORMATION.encode(transformation);
            }
            case "transformNormal" -> {
                return JomlValues.VECTOR3F.encode(stack.last().transformNormal((Float)args[0], (Float)args[1], (Float)args[2], new Vector3f()));
            }
            case "transformNormalVector" -> {
                return JomlValues.VECTOR3F.encode(stack.last().transformNormal((Vector3f)args[0], new Vector3f()));
            }
            case "isEmpty" -> {
                return new JsonPrimitive(stack.isEmpty());
            }
            default -> throw new IllegalArgumentException(op.name());
        }
        return JsonNull.INSTANCE;
    }

    private static List<Op> randomProgram(final Random random, final boolean wild, final boolean transformations) {
        List<Op> program = new ArrayList<>();
        int depth = 0;
        while (program.size() < OPS_PER_PROGRAM) {
            Op op = randomOp(random, wild, transformations);
            // Popping the last pose throws and ends the program, so it's rare.
            if (op.name().equals("popPose") && depth == 0 && random.nextInt(16) != 0) {
                continue;
            }
            depth += op.name().equals("pushPose") ? 1 : op.name().equals("popPose") ? -1 : 0;
            program.add(op);
        }
        return program;
    }

    private static Op randomOp(final Random random, final boolean wild, final boolean transformations) {
        return switch (random.nextInt(transformations ? 23 : 20)) {
            case 0, 1, 2 -> new Op("pushPose");
            case 3, 4 -> new Op("popPose");
            case 5, 6 -> new Op("translate", number(random, wild), number(random, wild), number(random, wild));
            case 7 -> new Op("translateDouble", doubleNumber(random, wild), doubleNumber(random, wild), doubleNumber(random, wild));
            case 8, 9 -> {
                float scale = number(random, wild);
                yield switch (random.nextInt(3)) {
                    // Uniform scales keep the normal matrix trusted, flipping it when one is negative.
                    case 0 -> new Op("scale", scale, scale, scale);
                    case 1 -> new Op("scale", sign(random) * scale, sign(random) * scale, sign(random) * scale);
                    default -> new Op("scale", scale, number(random, wild), number(random, wild));
                };
            }
            case 10 -> new Op("rotate", quaternion(random, wild));
            case 11 -> new Op("rotateAxis", Values.AXES.random(random), angle(random, wild));
            case 12 -> new Op("rotateDegrees", Values.AXES.random(random), number(random, wild) * 45.0F);
            case 13 -> new Op("rotateAround", quaternion(random, wild), number(random, wild), number(random, wild), number(random, wild));
            case 14 -> new Op("setIdentity");
            case 15, 16 -> new Op("mulPose", JomlValues.MATRIX4F.random(random));
            case 17 -> new Op("transformNormal", number(random, wild), number(random, wild), number(random, wild));
            case 18 -> new Op("transformNormalVector", JomlValues.VECTOR3F.random(random));
            case 19 -> new Op("isEmpty");
            default -> new Op("mulPoseTransformation", MathValues.TRANSFORMATION.random(random));
        };
    }

    // Whole blocks, sixteenths of one like model coordinates, and anything in between.
    private static float number(final Random random, final boolean wild) {
        if (wild && random.nextInt(4) == 0) {
            return (Float)Values.FLOATS.random(random);
        }
        return switch (random.nextInt(4)) {
            case 0 -> random.nextInt(33) - 16;
            case 1 -> (random.nextInt(33) - 16) / 16.0F;
            default -> (random.nextFloat() * 2.0F - 1.0F) * 4.0F;
        };
    }

    // Camera-relative positions, which lose bits when translate(DDD) narrows them to floats.
    private static double doubleNumber(final Random random, final boolean wild) {
        if (wild && random.nextInt(4) == 0) {
            return (Double)Values.generator(double.class).random(random);
        }
        return (random.nextDouble() * 2.0 - 1.0) * 64.0;
    }

    private static float angle(final Random random, final boolean wild) {
        return wild && random.nextInt(4) == 0 ? (Float)Values.FLOATS.random(random) : (random.nextFloat() * 2.0F - 1.0F) * 2.0F * (float)Math.PI;
    }

    private static Quaternionf quaternion(final Random random, final boolean wild) {
        return wild ? (Quaternionf)JomlValues.QUATERNIONF.random(random) : ((Axis)Values.AXES.random(random)).rotation(angle(random, false));
    }

    private static float sign(final Random random) {
        return random.nextBoolean() ? 1.0F : -1.0F;
    }
}
