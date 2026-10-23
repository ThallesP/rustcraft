package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonNull;
import com.google.gson.JsonObject;
import com.google.gson.JsonPrimitive;
import java.io.IOException;
import java.util.List;
import java.util.Map;
import java.util.Random;
import net.minecraft.core.Holder;
import net.minecraft.resources.Identifier;
import net.minecraft.world.entity.ai.attributes.Attribute;
import net.minecraft.world.entity.ai.attributes.AttributeInstance;
import net.minecraft.world.entity.ai.attributes.AttributeModifier;
import net.minecraft.world.entity.ai.attributes.Attributes;

/**
 * The player's movement's own suites (TESTING.md). An attribute is its registered name, a modifier operation its
 * constant's name.
 */
final class PlayerSuites {
    // The attributes the player's movement reads, by registered name.
    private static final Map<String, Holder<Attribute>> ATTRIBUTES = Map.ofEntries(
        Map.entry("movement_speed", Attributes.MOVEMENT_SPEED),
        Map.entry("sneaking_speed", Attributes.SNEAKING_SPEED),
        Map.entry("jump_strength", Attributes.JUMP_STRENGTH),
        Map.entry("gravity", Attributes.GRAVITY),
        Map.entry("step_height", Attributes.STEP_HEIGHT),
        Map.entry("scale", Attributes.SCALE),
        Map.entry("friction_modifier", Attributes.FRICTION_MODIFIER),
        Map.entry("air_drag_modifier", Attributes.AIR_DRAG_MODIFIER),
        Map.entry("bounciness", Attributes.BOUNCINESS),
        Map.entry("movement_efficiency", Attributes.MOVEMENT_EFFICIENCY),
        Map.entry("water_movement_efficiency", Attributes.WATER_MOVEMENT_EFFICIENCY),
        Map.entry("max_health", Attributes.MAX_HEALTH)
    );
    private static final List<String> NAMES = ATTRIBUTES.keySet().stream().sorted().toList();
    // Each operation's modifiers come from one id, so an instance never holds two of one operation: their order is an
    // Object2ObjectOpenHashMap's, which the port doesn't follow yet.
    private static final List<String> IDS = List.of("minecraft:added", "minecraft:base_multiple", "minecraft:sprinting");
    private static final double[] AMOUNTS = {
        0.3F, 0.2, 0.4, -0.15, 1.0, -1.0, 0.0, -0.0, 0.5, 2.0, 1.0E-7, 100.0, -100.0, 3000.0, Double.NaN, 0.1F, -0.3F, 0.42F, 0.08
    };
    private static final double[] BASES = {
        0.1F, 0.10000000149011612, 0.7, 0.0, -0.0, 1.0, 20.0, 0.42F, 0.08, 0.6, 0.3, 1024.0, 1025.0, -1.0, 2048.0, 0.0625, 16.0,
        Double.NaN, Double.POSITIVE_INFINITY, Double.NEGATIVE_INFINITY, 1.0E-300
    };

    private PlayerSuites() {
    }

    static List<Suite> suites() {
        return List.of(new AttributeInstanceSuite());
    }

    /**
     * Programs on an AttributeInstance of one of the attributes, `[attribute, ops]`, each step its output and the value
     * afterwards: `["base", value]` (setBaseValue), `["add", id, amount, operation]` (addTransientModifier, which throws
     * for an id that's on already), `["remove", id]` (removeModifier, whether there was one), `["removeAll"]` and
     * `["value"]`.
     */
    static final class AttributeInstanceSuite implements Suite {
        @Override
        public String name() {
            return AttributeInstance.class.getName();
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            Random random = new Random(0x41545452L);
            for (int program = 0; program < 1500; program++) {
                String name = NAMES.get(program < NAMES.size() * 4 ? program % NAMES.size() : random.nextInt(NAMES.size()));
                AttributeInstance instance = new AttributeInstance(ATTRIBUTES.get(name), changed -> {
                });
                JsonArray ops = new JsonArray();
                JsonArray steps = new JsonArray();
                for (int count = 1 + random.nextInt(12); count > 0; count--) {
                    JsonArray op = new JsonArray();
                    Runnable run;
                    JsonElement[] output = {JsonNull.INSTANCE};
                    switch (random.nextInt(5)) {
                        case 0 -> {
                            double base = random.nextInt(3) == 0 ? random.nextDouble() * 4.0 - 2.0 : BASES[random.nextInt(BASES.length)];
                            op.add("base");
                            op.add(ShapeSuites.d(base));
                            run = () -> instance.setBaseValue(base);
                        }
                        case 1, 2 -> {
                            int which = random.nextInt(3);
                            double amount = random.nextInt(3) == 0 ? random.nextDouble() * 2.0 - 1.0 : AMOUNTS[random.nextInt(AMOUNTS.length)];
                            AttributeModifier.Operation operation = AttributeModifier.Operation.values()[which];
                            op.add("add");
                            op.add(IDS.get(which));
                            op.add(ShapeSuites.d(amount));
                            op.add(operation.name());
                            run = () -> instance.addTransientModifier(new AttributeModifier(Identifier.parse(IDS.get(which)), amount, operation));
                        }
                        case 3 -> {
                            if (random.nextInt(4) == 0) {
                                op.add("removeAll");
                                run = instance::removeModifiers;
                            } else {
                                String id = IDS.get(random.nextInt(IDS.size()));
                                op.add("remove");
                                op.add(id);
                                run = () -> output[0] = new JsonPrimitive(instance.removeModifier(Identifier.parse(id)));
                            }
                        }
                        default -> {
                            op.add("value");
                            run = () -> output[0] = ShapeSuites.d(instance.getValue());
                        }
                    }
                    ops.add(op);
                    try {
                        run.run();
                    } catch (RuntimeException e) {
                        JsonObject thrown = new JsonObject();
                        thrown.addProperty("throws", e.getClass().getName());
                        steps.add(thrown);
                        break;
                    }
                    steps.add(BlockSuites.array(output[0], ShapeSuites.d(instance.getValue())));
                }
                writer.result("programs", BlockSuites.array(new JsonPrimitive(name), ops), steps, null);
            }
        }
    }
}
