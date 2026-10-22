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
import java.util.ArrayList;
import java.util.Arrays;
import java.util.Comparator;
import java.util.HashSet;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Random;
import java.util.Set;

/**
 * Covers chosen constructors and instance methods of a value class by reflection. Each case calls one method on a
 * fresh receiver, which is {@code args[0]}; {@code after[0]} is the receiver's state after the call, and the other
 * slots hold any argument the method wrote to, unless it's the returned object. Constructors are {@code <init>}
 * cases whose result is the new object.
 */
final class InstanceSuite implements Suite {
    private static final int RANDOM_CASES = 500;

    private final Class<?> type;
    private final Set<String> only;

    /** Big library classes (JOML's Matrix4f has hundreds of methods) list the members the port covers. */
    InstanceSuite(final Class<?> type, final String... members) {
        this.type = type;
        this.only = new LinkedHashSet<>(Arrays.asList(members));
    }

    @Override
    public String name() {
        return this.type.getName();
    }

    @Override
    public void write(final GoldenWriter writer) throws IOException {
        Set<String> written = new HashSet<>();
        for (Executable member : this.members()) {
            String descriptor = descriptor(member);
            if (this.only.contains(descriptor)) {
                written.add(descriptor);
                this.writeCases(writer, member, descriptor);
            }
        }

        Set<String> missing = new HashSet<>(this.only);
        missing.removeAll(written);
        if (!missing.isEmpty()) {
            throw new IllegalStateException(this.type.getName() + " has no public members " + missing);
        }
    }

    private List<Executable> members() {
        List<Executable> members = new ArrayList<>(Arrays.asList(this.type.getConstructors()));
        Arrays.stream(this.type.getDeclaredMethods())
            .filter(method -> Modifier.isPublic(method.getModifiers()) && !Modifier.isStatic(method.getModifiers()))
            .filter(method -> !method.isSynthetic() && !method.isBridge())
            .forEach(members::add);
        members.sort(Comparator.comparing(InstanceSuite::descriptor));
        return members;
    }

    private static String descriptor(final Executable member) {
        return member instanceof Method method ? Descriptors.of(method) : Descriptors.of((Constructor<?>)member);
    }

    private void writeCases(final GoldenWriter writer, final Executable member, final String descriptor) throws IOException {
        boolean constructor = member instanceof Constructor<?>;
        List<Class<?>> types = new ArrayList<>();
        if (!constructor) {
            types.add(this.type);
        }
        types.addAll(Arrays.asList(member.getParameterTypes()));
        Values.Generator[] generators = new Values.Generator[types.size()];
        for (int i = 0; i < generators.length; i++) {
            generators[i] = Values.generator(types.get(i));
            if (generators[i] == null) {
                throw new IllegalStateException(descriptor + ": no generator for " + types.get(i).getName());
            }
        }
        Class<?> returnType = constructor ? this.type : ((Method)member).getReturnType();
        if (returnType != void.class && !Values.canEncode(returnType)) {
            throw new IllegalStateException(descriptor + ": no encoder for " + returnType.getName());
        }

        Random random = new Random(descriptor.hashCode());
        List<Object[]> specs = new ArrayList<>(StaticSuite.edgeTuples(generators, random));
        if (generators.length > 0) {
            for (int i = 0; i < RANDOM_CASES; i++) {
                specs.add(StaticSuite.randomTuple(generators, random));
            }
        }

        for (Object[] spec : specs) {
            JsonArray args = new JsonArray();
            Object[] instances = new Object[spec.length];
            for (int i = 0; i < spec.length; i++) {
                args.add(generators[i].encode(spec[i]));
                instances[i] = generators[i].instantiate(spec[i]);
            }

            Watchdog.begin(this.type.getName() + "." + descriptor + " " + args);
            try {
                Object result = constructor
                    ? ((Constructor<?>)member).newInstance(instances)
                    : ((Method)member).invoke(instances[0], Arrays.copyOfRange(instances, 1, instances.length));
                JsonElement encoded = returnType == void.class ? JsonNull.INSTANCE : Values.encode(result, returnType);
                writer.result(descriptor, args, encoded, this.after(constructor, generators, spec, instances, result));
            } catch (InvocationTargetException e) {
                writer.threw(descriptor, args, e.getCause(), this.after(constructor, generators, spec, instances, null));
            } catch (ReflectiveOperationException e) {
                throw new IllegalStateException(e);
            } finally {
                Watchdog.end();
            }
        }
    }

    // The receiver's state is always recorded, even when it's also the result.
    private JsonArray after(
        final boolean constructor, final Values.Generator[] generators, final Object[] specs, final Object[] instances, final Object result
    ) {
        JsonArray after = StaticSuite.after(generators, specs, instances, result);
        if (constructor) {
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
