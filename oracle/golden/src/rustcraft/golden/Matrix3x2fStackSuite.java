package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import java.io.IOException;
import java.util.ArrayList;
import java.util.List;
import java.util.Random;
import org.joml.Matrix3x2fStack;

/**
 * Runs random programs against a Matrix3x2fStack, which the GUI's pose is ({@code GuiGraphicsExtractor.pose()}, of
 * size 16). A case is a stack of size {@code args[0]} running the ops in {@code args[1]}; the result has the stack's
 * current matrix after each op. An op that throws ends the program, and a size the constructor rejects makes the
 * whole result one thrown op.
 */
final class Matrix3x2fStackSuite implements Suite {
    private static final int PROGRAMS = 256;
    private static final int OPS_PER_PROGRAM = 48;
    private static final List<Integer> SIZE_EDGES = List.of(16, 1, 2, 3, 0, -1, Integer.MIN_VALUE);

    @Override
    public String name() {
        return Matrix3x2fStack.class.getName();
    }

    @Override
    public void write(final GoldenWriter writer) throws IOException {
        Random random = new Random(0x3f2);
        for (int i = 0; i < PROGRAMS; i++) {
            int size = i < SIZE_EDGES.size() ? SIZE_EDGES.get(i) : 1 + random.nextInt(20);
            List<Op> program = randomProgram(random, size);
            JsonArray args = new JsonArray();
            args.add(size);
            args.add(Op.encode(program));
            writer.result("Matrix3x2fStack", args, run(size, program), null);
        }
    }

    private static JsonArray run(final int size, final List<Op> program) {
        JsonArray steps = new JsonArray();
        Matrix3x2fStack stack;
        try {
            stack = new Matrix3x2fStack(size);
        } catch (RuntimeException e) {
            steps.add(thrown(e));
            return steps;
        }

        for (Op op : program) {
            try {
                apply(stack, op);
            } catch (RuntimeException e) {
                steps.add(thrown(e));
                break;
            }
            steps.add(JomlValues.MATRIX3X2F.encode(stack));
        }
        return steps;
    }

    private static void apply(final Matrix3x2fStack stack, final Op op) {
        Object[] args = op.args();
        switch (op.name()) {
            case "pushMatrix" -> stack.pushMatrix();
            case "popMatrix" -> stack.popMatrix();
            case "translate" -> stack.translate((Float)args[0], (Float)args[1]);
            case "scale" -> stack.scale((Float)args[0], (Float)args[1]);
            case "rotate" -> stack.rotate((Float)args[0]);
            default -> throw new IllegalArgumentException(op.name());
        }
    }

    private static List<Op> randomProgram(final Random random, final int size) {
        List<Op> program = new ArrayList<>();
        int depth = 0;
        while (program.size() < OPS_PER_PROGRAM) {
            Op op = randomOp(random);
            // Popping the bottom matrix or pushing past the top throws and ends the program, so it's rare.
            boolean throwing = op.name().equals("popMatrix") && depth == 0 || op.name().equals("pushMatrix") && depth == size - 1;
            if (throwing && random.nextInt(16) != 0) {
                continue;
            }
            depth = switch (op.name()) {
                case "pushMatrix" -> depth + 1;
                case "popMatrix" -> depth - 1;
                default -> depth;
            };
            program.add(op);
        }
        return program;
    }

    // The GUI pushes, translates to where it draws, sometimes scales or rotates, and pops: those are the only ops the
    // client calls on its stack.
    private static Op randomOp(final Random random) {
        return switch (random.nextInt(16)) {
            case 0, 1, 2, 3 -> new Op("pushMatrix");
            case 4, 5, 6 -> new Op("popMatrix");
            case 7, 8, 9, 10 -> new Op("translate", offset(random), offset(random));
            case 11, 12 -> {
                float scale = scale(random);
                yield new Op("scale", scale, random.nextInt(4) == 0 ? scale(random) : scale);
            }
            case 13 -> new Op("rotate", random.nextInt(8) == 0 ? (Float)Values.FLOATS.random(random) : (random.nextFloat() * 2.0F - 1.0F) * 7.0F);
            default -> new Op("translate", number(random), number(random));
        };
    }

    // Whole and half pixels, like the GUI's offsets.
    private static float offset(final Random random) {
        return (random.nextInt(1025) - 512) * (random.nextBoolean() ? 1.0F : 0.5F);
    }

    private static float scale(final Random random) {
        return switch (random.nextInt(4)) {
            case 0 -> 0.5F;
            case 1 -> 1.0F + random.nextInt(4);
            case 2 -> random.nextFloat() * 4.0F;
            default -> number(random);
        };
    }

    private static float number(final Random random) {
        return random.nextInt(8) == 0 ? (Float)Values.FLOATS.random(random) : (random.nextFloat() * 2.0F - 1.0F) * 16.0F;
    }

    private static JsonObject thrown(final RuntimeException e) {
        JsonObject thrown = new JsonObject();
        thrown.addProperty("throws", e.getClass().getName());
        return thrown;
    }
}
