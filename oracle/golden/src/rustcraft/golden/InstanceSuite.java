package rustcraft.golden;

import com.google.gson.JsonArray;
import com.google.gson.JsonElement;
import com.google.gson.JsonNull;
import java.io.IOException;
import java.lang.reflect.Constructor;
import java.lang.reflect.Executable;
import java.lang.reflect.InvocationTargetException;
import java.lang.reflect.Method;
import java.lang.reflect.Modifier;
import java.lang.reflect.ParameterizedType;
import java.lang.reflect.Type;
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

/**
 * Covers the constructors and methods of a value class by reflection: chosen ones for big library classes, or all its
 * public ones and its constants for the game's own. Each case calls one method on a fresh receiver, which is
 * {@code args[0]}; {@code after[0]} is the receiver's state after the call, and the other slots hold any argument the
 * method wrote to, unless it's the returned object. Constructors are {@code <init>} cases whose result is the new
 * object. Static methods get cases like a StaticSuite's, without a receiver.
 */
final class InstanceSuite implements Suite {
    private static final int RANDOM_CASES = 500;

    private final Class<?> type;
    private final Set<String> only;
    private final Map<String, List<Object[]>> extraInputs = new HashMap<>();
    private final Map<String, Predicate<Object[]>> domains = new HashMap<>();
    private final Set<String> inherited = new LinkedHashSet<>();
    private boolean fields;

    /** Big library classes (JOML's Matrix4f has hundreds of methods) list the members the port covers. */
    InstanceSuite(final Class<?> type, final String... members) {
        this(type, new LinkedHashSet<>(Arrays.asList(members)));
    }

    private InstanceSuite(final Class<?> type, final Set<String> only) {
        this.type = type;
        this.only = only;
    }

    /**
     * Every public constructor and method the class declares, and its constants. A member whose arguments have no
     * generator or whose result has no encoder is written as skipped, which the port's test has to mark pending.
     */
    static InstanceSuite every(final Class<?> type) {
        return new InstanceSuite(type, (Set<String>)null).withFields();
    }

    /** More cases for a member, each the receiver's spec (none for a constructor or static method) followed by the arguments'. */
    InstanceSuite extraInputs(final String member, final List<Object[]> inputs) {
        this.extraInputs.put(member, inputs);
        return this;
    }

    /** Also writes the class's public constants, like a StaticSuite does. */
    InstanceSuite withFields() {
        this.fields = true;
        return this;
    }

    /** Drops inputs outside the domain, like StaticSuite.where, for members whose results never end on some inputs. */
    InstanceSuite where(final String member, final Predicate<Object[]> domain) {
        this.domains.put(member, domain);
        return this;
    }

    /**
     * Also covers public methods the class inherits whose answers depend on the receiver's class, like Vec3i's
     * toString, which prints the class's name.
     */
    InstanceSuite inherited(final String... members) {
        this.inherited.addAll(Arrays.asList(members));
        return this;
    }

    @Override
    public String name() {
        return this.type.getName();
    }

    @Override
    public void write(final GoldenWriter writer) throws IOException {
        if (this.fields) {
            StaticSuite.writeFields(this.type, writer);
        }

        Set<String> written = new HashSet<>();
        for (Executable member : this.members()) {
            String descriptor = descriptor(member);
            if (this.only != null && !this.only.contains(descriptor)) {
                continue;
            }

            written.add(descriptor);
            String reason = this.unsupportedReason(member);
            if (reason == null) {
                this.writeCases(writer, member, descriptor);
            } else if (this.only == null) {
                writer.skipped(descriptor, reason);
            } else {
                throw new IllegalStateException(descriptor + ": " + reason);
            }
        }

        Set<String> missing = new HashSet<>(this.only == null ? Set.of() : this.only);
        missing.addAll(this.extraInputs.keySet());
        missing.addAll(this.domains.keySet());
        missing.addAll(this.inherited);
        missing.removeAll(written);
        if (!missing.isEmpty()) {
            throw new IllegalStateException(this.type.getName() + " has no public members " + missing);
        }
    }

    private List<Executable> members() {
        List<Executable> members = new ArrayList<>(Arrays.asList(this.type.getConstructors()));
        Arrays.stream(this.type.getDeclaredMethods())
            .filter(method -> Modifier.isPublic(method.getModifiers()))
            .filter(method -> !method.isSynthetic() && !method.isBridge())
            .forEach(members::add);
        Arrays.stream(this.type.getMethods())
            .filter(method -> method.getDeclaringClass() != this.type && this.inherited.contains(descriptor(method)))
            .forEach(members::add);
        members.sort(Comparator.comparing(InstanceSuite::descriptor));
        return members;
    }

    private static String descriptor(final Executable member) {
        return member instanceof Method method ? Descriptors.of(method) : Descriptors.of((Constructor<?>)member);
    }

    // The receiver's type first, unless it's a constructor or a static method. equals(Object) compares with another
    // object of the same class. A parameterized type stays one, so its elements can pick the generator (List<AABB>).
    private List<Type> argumentTypes(final Executable member) {
        List<Type> types = new ArrayList<>();
        if (!(member instanceof Constructor<?>) && !Modifier.isStatic(member.getModifiers())) {
            types.add(this.type);
        }
        Class<?>[] raw = member.getParameterTypes();
        Type[] generic = member.getGenericParameterTypes();
        for (int i = 0; i < raw.length; i++) {
            if (raw[i] == Object.class) {
                types.add(this.type);
            } else {
                types.add(generic.length == raw.length && generic[i] instanceof ParameterizedType ? generic[i] : raw[i]);
            }
        }
        return types;
    }

    private Class<?> returnType(final Executable member) {
        return member instanceof Method method ? method.getReturnType() : this.type;
    }

    private String unsupportedReason(final Executable member) {
        Class<?> returnType = this.returnType(member);
        if (returnType != void.class && !Values.canEncode(returnType)) {
            return "no encoder for " + returnType.getName();
        }
        for (Type type : this.argumentTypes(member)) {
            if (Values.generator(type) == null) {
                Class<?> raw = type instanceof ParameterizedType parameterized ? (Class<?>)parameterized.getRawType() : (Class<?>)type;
                return "no generator for " + raw.getName();
            }
        }
        return null;
    }

    private void writeCases(final GoldenWriter writer, final Executable member, final String descriptor) throws IOException {
        boolean constructor = member instanceof Constructor<?>;
        boolean receiver = !constructor && !Modifier.isStatic(member.getModifiers());
        Values.Generator[] generators = this.argumentTypes(member).stream().map(Values::generator).toArray(Values.Generator[]::new);
        Class<?> returnType = this.returnType(member);

        Random random = new Random(descriptor.hashCode());
        List<Object[]> specs = new ArrayList<>(StaticSuite.edgeTuples(generators, random));
        specs.addAll(this.extraInputs.getOrDefault(descriptor, List.of()));
        // When every argument is an enum, the edges were every combination already.
        boolean exhaustive = Arrays.stream(generators).allMatch(Values.Generator::exhaustive)
            && StaticSuite.edgeProduct(generators) <= StaticSuite.MAX_EDGE_PRODUCT;
        if (generators.length > 0 && !exhaustive) {
            for (int i = 0; i < RANDOM_CASES; i++) {
                specs.add(StaticSuite.randomTuple(generators, random));
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

            Watchdog.begin(this.type.getName() + "." + descriptor + " " + args);
            try {
                Object result;
                if (constructor) {
                    result = ((Constructor<?>)member).newInstance(instances);
                } else if (receiver) {
                    result = ((Method)member).invoke(instances[0], Arrays.copyOfRange(instances, 1, instances.length));
                } else {
                    result = ((Method)member).invoke(null, instances);
                }
                JsonElement encoded = returnType == void.class ? JsonNull.INSTANCE : Values.encode(result, returnType);
                writer.result(descriptor, args, encoded, this.after(receiver, generators, spec, instances, result));
            } catch (InvocationTargetException e) {
                writer.threw(descriptor, args, e.getCause(), this.after(receiver, generators, spec, instances, null));
            } catch (Values.ElementThrew e) {
                writer.threw(descriptor, args, e.getCause(), this.after(receiver, generators, spec, instances, null));
            } catch (ReflectiveOperationException e) {
                throw new IllegalStateException(e);
            } finally {
                Watchdog.end();
            }
        }
    }

    // The receiver's state is always recorded, even when it's also the result.
    private JsonArray after(
        final boolean receiver, final Values.Generator[] generators, final Object[] specs, final Object[] instances, final Object result
    ) {
        JsonArray after = StaticSuite.after(generators, specs, instances, result);
        if (!receiver) {
            return after;
        }
        if (after == null) {
            after = new JsonArray();
            for (int i = 0; i < generators.length; i++) {
                after.add(JsonNull.INSTANCE);
            }
        }
        after.set(0, generators[0].encode(instances[0]));
        return after;
    }
}
