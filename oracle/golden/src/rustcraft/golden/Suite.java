package rustcraft.golden;

import java.io.IOException;

interface Suite {
    /** Binary name of the class the suite covers, which is also the golden file's name. */
    String name();

    void write(GoldenWriter writer) throws IOException;
}
