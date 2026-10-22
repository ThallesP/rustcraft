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
import java.util.LinkedHashMap;
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
 * object. Static methods get cases like a StaticSuite's, without a receiver, and one making a stream codec from generated
 * arguments gets codec cases like CodecSuite's (DiscardedPayload.codec). A member taking codecs or functions gets cases
 * for the instances listed with given(), like CodecSuite's.
 */
final class InstanceSuite implements Suite {
    private static final int RANDOM_CASES = 500;

    private final Class<?> type;
    private final Set<String> only;
    private final Map<String, List<Object[]>> extraInputs = new HashMap<>();
    private final Map<String, Predicate<Object[]>> domains = new HashMap<>();
    private final Set<String> inherited = new LinkedHashSet<>();
    private final Map<String, List<Given>> given = new LinkedHashMap<>();
    private final Map<String, String> excluded = new HashMap<>();
    private Class<?> wrapped;
    private boolean fields;

    // The instances a member was given, named by the label, and generators for its other arguments.
    private record Given(String label, Object[] arguments) {
    }

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

    /**
     * More cases for a member, each the receiver's spec (none for a constructor or static method) followed by the
     * arguments', or more buffers for a codec constant N to decode, under {@code N.decode}.
     */
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
     * Writes the member as skipped with the reason, for one whose answers change from run to run: a record's hashCode
     * with an enum component hashes the constant's identity hash.
     */
    InstanceSuite exclude(final String member, final String reason) {
        this.excluded.put(member, reason);
        return this;
    }

    /**
     * Also covers public methods the class inherits whose answers depend on the receiver's class, like Vec3i's
     * toString, which prints the class's name, or that it forwards to an object it holds, like PropertyMap's Multimap
     * methods.
     */
    InstanceSuite inherited(final String... members) {
        this.inherited.addAll(Arrays.asList(members));
        return this;
    }

    /**
     * Leaves out the methods that override one of the parent's, for a class that wraps another and forwards its API to
     * it (FriendlyByteBuf to Netty's ByteBuf), whose own suite covers them.
     */
    InstanceSuite withoutOverridesOf(final Class<?> parent) {
        this.wrapped = parent;
        return this;
    }

    /**
     * Cases for a member taking codecs or functions, which have no generator, given the instances the label names:
     * they're named by the member's descriptor and the label, {@code readOptional(…)Ljava/util/Optional;[VAR_INT]}.
     * {@code arguments} has one entry per parameter: the instance itself, which isn't in {@code args}, or a generator
     * for an argument generated like any other (the values the instance writes). The receiver comes first as always,
     * and extra inputs and domains go by the cases' name. A result declared as an Object or an Enum is encoded by its own
     * class.
     */
    InstanceSuite given(final String member, final String label, final Object... arguments) {
        this.given.computeIfAbsent(member, m -> new ArrayList<>()).add(new Given(label, arguments));
        return this;
    }

    @Override
    public String name() {
        return this.type.getName();
    }

    @Override
    public void write(final GoldenWriter writer) throws IOException {
        Set<String> written = new HashSet<>();
        if (this.fields) {
            written.addAll(StaticSuite.writeFields(this.type, writer, this.extraInputs));
        }

        for (Executable member : this.members()) {
            String descriptor = descriptor(member);
            if (this.only != null && !this.only.contains(descriptor)) {
                continue;
            }

            written.add(descriptor);
            List<Given> listed = this.given.get(descriptor);
            if (listed != null) {
                for (Given instance : listed) {
                    String name = descriptor + "[" + instance.label() + "]";
                    written.add(name);
                    this.writeCases(writer, member, name, this.withReceiver(member, instance.arguments()));
                }
                continue;
            }

            List<Type> argumentTypes = this.argumentTypes(member);
            Codecs.Codec codec = member instanceof Method method && Modifier.isStatic(method.getModifiers())
                ? Codecs.codecOf(method.getGenericReturnType())
                : null;
            if (codec != null && argumentTypes.stream().allMatch(type -> Values.generator(type) != null)) {
                Values.Generator[] leading = argumentTypes.stream().map(Values::generator).toArray(Values.Generator[]::new);
                written.add(descriptor + ".decode");
                Codecs.writeCodec(
                    writer, descriptor, leading, args -> ((Method)member).invoke(null, args), codec, spec -> true,
                    this.extraInputs.getOrDefault(descriptor + ".decode", List.of())
                );
                continue;
            }

            String reason = this.excluded.containsKey(descriptor) ? this.excluded.get(descriptor) : this.unsupportedReason(member);
            if (this.excluded.containsKey(descriptor)) {
                writer.skipped(descriptor, reason);
            } else if (reason == null) {
                Object[] generators = this.argumentTypes(member).stream().map(Values::generator).toArray();
                this.writeCases(writer, member, descriptor, generators);
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
        missing.addAll(this.given.keySet());
        missing.addAll(this.excluded.keySet());
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
            .filter(method -> this.wrapped == null || Modifier.isStatic(method.getModifiers()) || !overrides(this.wrapped, method))
            .forEach(members::add);
        Arrays.stream(this.type.getMethods())
            .filter(method -> method.getDeclaringClass() != this.type && this.inherited.contains(descriptor(method)))
            .forEach(members::add);
        members.sort(Comparator.comparing(InstanceSuite::descriptor));
        return members;
    }

    private static boolean overrides(final Class<?> parent, final Method method) {
        try {
            parent.getMethod(method.getName(), method.getParameterTypes());
            return true;
        } catch (NoSuchMethodException e) {
            return false;
        }
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

    // A listed instance's arguments, after the receiver's generator unless the member is a constructor or static.
    private Object[] withReceiver(final Executable member, final Object[] arguments) {
        if (arguments.length != member.getParameterCount()) {
            throw new IllegalStateException(descriptor(member) + " takes " + member.getParameterCount() + " arguments, not " + arguments.length);
        }
        if (member instanceof Constructor<?> || Modifier.isStatic(member.getModifiers())) {
            return arguments;
        }
        Object[] withReceiver = new Object[arguments.length + 1];
        withReceiver[0] = Values.generator(this.type);
        System.arraycopy(arguments, 0, withReceiver, 1, arguments.length);
        return withReceiver;
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

    // Each argument, receiver first, is a generator, or an instance the member was given, which isn't in the cases.
    private void writeCases(final GoldenWriter writer, final Executable member, final String name, final Object[] arguments) throws IOException {
        boolean constructor = member instanceof Constructor<?>;
        boolean receiver = !constructor && !Modifier.isStatic(member.getModifiers());
        Values.Generator[] generators = Arrays.stream(arguments)
            .filter(argument -> argument instanceof Values.Generator)
            .toArray(Values.Generator[]::new);
        Class<?> returnType = this.returnType(member);

        Random random = new Random(name.hashCode());
        List<Object[]> specs = new ArrayList<>(StaticSuite.edgeTuples(generators, random));
        specs.addAll(this.extraInputs.getOrDefault(name, List.of()));
        // When every argument is an enum, the edges were every combination already.
        boolean exhaustive = Arrays.stream(generators).allMatch(Values.Generator::exhaustive)
            && StaticSuite.edgeProduct(generators) <= StaticSuite.MAX_EDGE_PRODUCT;
        if (generators.length > 0 && !exhaustive) {
            for (int i = 0; i < RANDOM_CASES; i++) {
                specs.add(StaticSuite.randomTuple(generators, random));
            }
        }

        Predicate<Object[]> domain = this.domains.getOrDefault(name, spec -> true);
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
            Object[] all = arguments.clone();
            for (int i = 0, generated = 0; i < all.length; i++) {
                if (all[i] instanceof Values.Generator) {
                    all[i] = instances[generated++];
                }
            }

            Watchdog.begin(this.type.getName() + "." + name + " " + args);
            try {
                Object result;
                if (constructor) {
                    result = ((Constructor<?>)member).newInstance(all);
                } else if (receiver) {
                    result = ((Method)member).invoke(all[0], Arrays.copyOfRange(all, 1, all.length));
                } else {
                    result = ((Method)member).invoke(null, all);
                }
                JsonElement encoded = returnType == void.class ? JsonNull.INSTANCE : Values.encode(result, encodedAs(result, returnType));
                writer.result(name, args, encoded, this.after(receiver, generators, spec, instances, result));
            } catch (InvocationTargetException e) {
                writer.threw(name, args, e.getCause(), this.after(receiver, generators, spec, instances, null));
            } catch (Values.ElementThrew e) {
                writer.threw(name, args, e.getCause(), this.after(receiver, generators, spec, instances, null));
            } catch (ReflectiveOperationException e) {
                throw new IllegalStateException(e);
            } finally {
                Watchdog.end();
            }
        }
    }

    // A generic result (readNullable's, readEnum's) by its own class, an enum constant by its enum's.
    private static Class<?> encodedAs(final Object result, final Class<?> returnType) {
        if ((returnType != Object.class && returnType != Enum.class) || result == null) {
            return returnType;
        }
        return result instanceof Enum<?> constant ? constant.getDeclaringClass() : result.getClass();
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
