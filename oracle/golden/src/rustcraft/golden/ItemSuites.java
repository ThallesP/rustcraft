package rustcraft.golden;

import com.google.gson.JsonArray;
import java.io.IOException;
import java.util.List;
import net.minecraft.core.component.DataComponentType;
import net.minecraft.core.registries.BuiltInRegistries;
import net.minecraft.world.item.Item;

/** The items' and the data component types' registries, which the port has as tables of names. */
final class ItemSuites {
    private ItemSuites() {
    }

    static List<Suite> suites() {
        return List.of(new ItemsSuite(), new DataComponentsSuite());
    }

    /** {@code BuiltInRegistries.ITEM}'s keys by id, as {@code NAMES}. */
    private static final class ItemsSuite implements Suite {
        @Override
        public String name() {
            return "net.minecraft.world.item.Items";
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            JsonArray names = new JsonArray();
            for (Item item : BuiltInRegistries.ITEM) {
                names.add(BuiltInRegistries.ITEM.getKey(item).toString());
            }
            writer.field("NAMES", names);
        }
    }

    /** {@code BuiltInRegistries.DATA_COMPONENT_TYPE}'s keys by id, each with whether it's transient, as {@code TYPES}. */
    private static final class DataComponentsSuite implements Suite {
        @Override
        public String name() {
            return "net.minecraft.core.component.DataComponents";
        }

        @Override
        public void write(final GoldenWriter writer) throws IOException {
            JsonArray types = new JsonArray();
            for (DataComponentType<?> type : BuiltInRegistries.DATA_COMPONENT_TYPE) {
                types.add(RenderSuites.array(
                    new com.google.gson.JsonPrimitive(BuiltInRegistries.DATA_COMPONENT_TYPE.getKey(type).toString()),
                    new com.google.gson.JsonPrimitive(type.isTransient())
                ));
            }
            writer.field("TYPES", types);
        }
    }
}
