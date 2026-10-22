package rustcraft.golden;

import java.lang.reflect.Method;
import java.util.Arrays;
import java.util.stream.Collectors;

/** JVM method descriptors, e.g. {@code clamp(III)I}, which keep overloads apart in golden files. */
final class Descriptors {
    private Descriptors() {
    }

    static String of(final Method method) {
        String parameters = Arrays.stream(method.getParameterTypes()).map(Descriptors::of).collect(Collectors.joining());
        return method.getName() + "(" + parameters + ")" + of(method.getReturnType());
    }

    static String of(final Class<?> type) {
        if (type.isArray()) {
            return "[" + of(type.getComponentType());
        }
        if (!type.isPrimitive()) {
            return "L" + type.getName().replace('.', '/') + ";";
        }
        return switch (type.getName()) {
            case "int" -> "I";
            case "long" -> "J";
            case "float" -> "F";
            case "double" -> "D";
            case "boolean" -> "Z";
            case "byte" -> "B";
            case "short" -> "S";
            case "char" -> "C";
            case "void" -> "V";
            default -> throw new IllegalArgumentException(type.getName());
        };
    }
}
