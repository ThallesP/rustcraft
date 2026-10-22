package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonObject;
import java.io.IOException;
import java.util.ArrayList;
import java.util.List;
import java.util.Random;
import org.joml.Matrix4f;
import org.joml.Matrix4fStack;
import org.joml.Quaternionf;

/**
 * Runs random programs against a Matrix4fStack, which the game's model-view stack is. A case is a stack of size
 * {@code args[0]} running the ops in {@code args[1]}; the result has the stack's current matrix after each op. An op
 * that throws ends the program, and a size the constructor rejects makes the whole result one thrown op.
 */
final class Matrix4fStackSuite implements Suite {
    private static final int PROGRAMS = 256;
    private static final int OPS_PER_PROGRAM = 48;
    private static final List<Integer> SIZE_EDGES = List.of(16, 1, 2, 3, 0, -1, Integer.MIN_VALUE);

    @Override
    public String name() {
        return Matrix4fStack.class.getName();
    }

    @Override
    public void write(final GoldenWriter writer) throws IOException {
        Random random = new Random(0x57ac);
        for (int i = 0; i < PROGRAMS; i++) {
            int size = i < SIZE_EDGES.size() ? SIZE_EDGES.get(i) : 1 + random.nextInt(20);
            List<Op> program = randomProgram(random, size);
            JsonArray args = new JsonArray();
            args.add(size);
            args.add(Op.encode(program));
            writer.result("Matrix4fStack", args, run(size, program), null);
        }
    }

    private static JsonArray run(final int size, final List<Op> program) {
        JsonArray steps = new JsonArray();
        Matrix4fStack stack;
        try {
            stack = new Matrix4fStack(size);
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
            steps.add(JomlValues.MATRIX4F.encode(stack));
        }
        return steps;
    }

    private static void apply(final Matrix4fStack stack, final Op op) {
        Object[] args = op.args();
        switch (op.name()) {
            case "pushMatrix" -> stack.pushMatrix();
            case "popMatrix" -> stack.popMatrix();
            case "clear" -> stack.clear();
            case "identity" -> stack.identity();
            case "translate" -> stack.translate((Float)args[0], (Float)args[1], (Float)args[2]);
            case "scale" -> stack.scale((Float)args[0], (Float)args[1], (Float)args[2]);
            case "rotateY" -> stack.rotateY((Float)args[0]);
            case "rotate" -> stack.rotate((Quaternionf)args[0]);
            case "mul" -> stack.mul((Matrix4f)args[0]);
            case "set" -> stack.set((Matrix4f)args[0]);
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
                case "clear" -> 0;
                default -> depth;
            };
            program.add(op);
        }
        return program;
    }

    // The game pushes, transforms and pops; set and identity overwrite the current matrix, and clear the whole stack.
    private static Op randomOp(final Random random) {
        return switch (random.nextInt(16)) {
            case 0, 1, 2, 3 -> new Op("pushMatrix");
            case 4, 5, 6 -> new Op("popMatrix");
            case 7 -> new Op("clear");
            case 8 -> new Op("identity");
            case 9, 10 -> new Op("translate", number(random), number(random), number(random));
            case 11 -> new Op("scale", number(random), number(random), number(random));
            case 12 -> new Op("rotateY", number(random));
            case 13 -> new Op("rotate", JomlValues.QUATERNIONF.random(random));
            case 14 -> new Op("mul", JomlValues.MATRIX4F.random(random));
            default -> new Op("set", JomlValues.MATRIX4F.random(random));
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
