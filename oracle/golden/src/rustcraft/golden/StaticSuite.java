package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import java.io.IOException;
import java.lang.reflect.Field;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Comparator;
import java.util.HashMap;
import java.util.HashSet;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Map;
import java.util.Random;
import java.util.Set;
import java.util.function.Predicate;

/** Covers every public static method and constant of a class by reflection. */
final class StaticSuite implements Suite {
    private static final int RANDOM_CASES = 500;
    static final int MAX_EDGE_PRODUCT = 2048;

    private final Class<?> type;
    private final Map<String, List<Object[]>> extraInputs = new HashMap<>();
    private final Map<String, String> excluded = new HashMap<>();
    private final Map<String, Predicate<Object[]>> domains = new HashMap<>();
    private Set<String> only;

    StaticSuite(final Class<?> type) {
        this.type = type;
    }

    StaticSuite extraInputs(final String method, final List<Object[]> inputs) {
        this.extraInputs.put(method, inputs);
        return this;
    }

    /** Drops inputs outside the domain, for methods that never return on some inputs (vanilla hangs there too). */
    StaticSuite where(final String method, final Predicate<Object[]> domain) {
        this.domains.put(method, domain);
        return this;
    }

    StaticSuite exclude(final String method, final String reason) {
        this.excluded.put(method, reason);
        return this;
    }

    /** Restricts the suite to these methods, for JDK classes where only a few methods matter to the port. */
    StaticSuite only(final String... methods) {
        this.only = new LinkedHashSet<>(Arrays.asList(methods));
        return this;
    }

    @Override
    public String name() {
        return this.type.getName();
    }

    @Override
    public void write(final GoldenWriter writer) throws IOException {
        if (this.only == null) {
            writeFields(this.type, writer);
        }

        Set<String> written = new HashSet<>();
        for (Method method : this.methods()) {
            String descriptor = Descriptors.of(method);
            if (this.only != null && !this.only.contains(descriptor)) {
                continue;
            }

            written.add(descriptor);
            String reason = this.excluded.getOrDefault(descriptor, unsupportedReason(method));
            if (reason == null) {
                this.writeCases(writer, method, descriptor);
            } else {
                writer.skipped(descriptor, reason);
            }
        }

        Set<String> missing = new HashSet<>(this.only == null ? this.extraInputs.keySet() : this.only);
        missing.addAll(this.domains.keySet());
        missing.removeAll(written);
        if (!missing.isEmpty()) {
            throw new IllegalStateException(this.type.getName() + " has no methods " + missing);
        }
    }

    /** Every public constant the class declares itself. */
    static void writeFields(final Class<?> type, final GoldenWriter writer) throws IOException {
        Field[] fields = type.getFields();
        Arrays.sort(fields, Comparator.comparing(Field::getName));
        for (Field field : fields) {
            int modifiers = field.getModifiers();
            if (!Modifier.isStatic(modifiers) || !Modifier.isFinal(modifiers) || field.getDeclaringClass() != type) {
                continue;
            }

            if (Values.canEncode(field.getType())) {
                try {
                    writer.field(field.getName(), Values.encode(field.get(null), field.getType()));
                } catch (IllegalAccessException e) {
                    throw new IllegalStateException(e);
                }
            } else {
                writer.skipped(field.getName(), "no encoder for " + field.getType().getName());
            }
        }
    }

    private List<Method> methods() {
        return Arrays.stream(this.type.getDeclaredMethods())
            .filter(method -> Modifier.isPublic(method.getModifiers()) && Modifier.isStatic(method.getModifiers()))
            .filter(method -> !method.isSynthetic() && !method.isBridge())
            .sorted(Comparator.comparing(Descriptors::of))
            .toList();
    }

    // A void method's result is null, and what it wrote to its arguments shows in `after`.
    private static String unsupportedReason(final Method method) {
        if (method.getReturnType() != void.class && !Values.canEncode(method.getReturnType())) {
            return "no encoder for " + method.getReturnType().getName();
        }
        for (Class<?> parameter : method.getParameterTypes()) {
            if (Values.generator(parameter) == null) {
                return "no generator for " + parameter.getName();
            }
        }
        return null;
    }

    private void writeCases(final GoldenWriter writer, final Method method, final String descriptor) throws IOException {
        Values.Generator[] generators = Arrays.stream(method.getParameterTypes()).map(Values::generator).toArray(Values.Generator[]::new);
        Random random = new Random(descriptor.hashCode());

        List<Object[]> specs = new ArrayList<>(edgeTuples(generators, random));
        specs.addAll(this.extraInputs.getOrDefault(descriptor, List.of()));
        if (generators.length > 0) {
            for (int i = 0; i < RANDOM_CASES; i++) {
                specs.add(randomTuple(generators, random));
            }
        }

        Predicate<Object[]> domain = this.domains.getOrDefault(descriptor, spec -> true);
        for (Object[] spec : specs) {
            if (!domain.test(spec)) {
                continue;
            }

            JsonArray args = new JsonArray();
            Object[] instances = new Object[spec.length];
            for (int i = 0; i < spec.length; i++) {
                args.add(generators[i].encode(spec[i]));
                instances[i] = generators[i].instantiate(spec[i]);
            }

            Watchdog.begin(descriptor + " " + args);
            try {
                Object result = method.invoke(null, instances);
                writer.result(descriptor, args, Values.encode(result, method.getReturnType()), after(generators, spec, instances, result));
            } catch (InvocationTargetException e) {
                writer.threw(descriptor, args, e.getCause(), after(generators, spec, instances, null));
            } catch (IllegalAccessException e) {
                throw new IllegalStateException(e);
            } finally {
                Watchdog.end();
            }
        }
    }

    // The full cartesian product of edges when it's small, otherwise each edge once in each position.
    static List<Object[]> edgeTuples(final Values.Generator[] generators, final Random random) {
        long product = edgeProduct(generators);

        List<Object[]> tuples = new ArrayList<>();
        if (product <= MAX_EDGE_PRODUCT) {
            tuples.add(new Object[generators.length]);
            for (int i = 0; i < generators.length; i++) {
                List<Object[]> extended = new ArrayList<>();
                for (Object[] tuple : tuples) {
                    for (Object edge : generators[i].edges()) {
                        Object[] copy = tuple.clone();
                        copy[i] = edge;
                        extended.add(copy);
                    }
                }
                tuples = extended;
            }
            return tuples;
        }

        for (int i = 0; i < generators.length; i++) {
            for (Object edge : generators[i].edges()) {
                Object[] tuple = randomTuple(generators, random);
                tuple[i] = edge;
                tuples.add(tuple);
            }
        }
        return tuples;
    }

    /** How many combinations of edges there are, capped just above the most edgeTuples takes in full. */
    static long edgeProduct(final Values.Generator[] generators) {
        long product = 1;
        for (Values.Generator generator : generators) {
            product = Math.min(product * generator.edges().size(), MAX_EDGE_PRODUCT + 1);
        }
        return product;
    }

    static Object[] randomTuple(final Values.Generator[] generators, final Random random) {
        Object[] tuple = new Object[generators.length];
        for (int i = 0; i < generators.length; i++) {
            tuple[i] = generators[i].random(random);
        }
        return tuple;
    }

    // One slot per argument. The returned object is already the result, so it gets no slot.
    static JsonArray after(final Values.Generator[] generators, final Object[] specs, final Object[] instances, final Object result) {
        JsonArray after = new JsonArray();
        boolean any = false;
        for (int i = 0; i < generators.length; i++) {
            JsonElement state = instances[i] == result ? null : generators[i].after(specs[i], instances[i]);
            any |= state != null;
            after.add(state);
        }
        return any ? after : null;
    }
}
