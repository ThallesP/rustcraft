package rustcraft.golden;

/** Kills the run when a single call hangs, naming it, since some vanilla methods loop forever on extreme inputs. */
final class Watchdog {
    private static final long LIMIT_MILLIS = 5000;

    private static volatile String current;
    private static volatile long startedAt;

    static {
        Thread thread = new Thread(Watchdog::watch, "golden-watchdog");
        thread.setDaemon(true);
        thread.start();
    }

    private Watchdog() {
    }

    static void begin(final String call) {
        startedAt = System.currentTimeMillis();
        current = call;
    }

    static void end() {
        current = null;
    }

    private static void watch() {
        while (true) {
            String call = current;
            if (call != null && System.currentTimeMillis() - startedAt > LIMIT_MILLIS) {
                System.err.printf("%s ran for over %d ms. Restrict its inputs with StaticSuite.where().%n", call, LIMIT_MILLIS);
                Runtime.getRuntime().halt(1);
            }

            try {
                Thread.sleep(250);
            } catch (InterruptedException e) {
                return;
            }
        }
    }
}
