<img src="assets/icon.svg" width="64" height="64" alt="">

# rustcraft

Created in [T3 Code](https://t3.codes).

## The prompt

This project started from this prompt:

> I want to re-create the Minecraft Client in Rust, wondering if we can setup the environment first.
>
> can we get the decompiled version of Minecraft Java (the latest version), I was thinking of doing a Java to Rust translation, but keeping the Java mannerism (like folder structures and etc).
>
> But i'm also scared of you getting some parts wrong, and even when I tell you to optimize properly, so I'm wondering what we can do here to fix it, maybe screenshots comparison (in a pixel perfect way?), comparing network packets to the original client and maybe even tests from the minecraft server client in C++
>
> what do you think here?

## Vanilla reference

The port is translated from the vanilla Java client. Fetch and decompile it with:

```sh
python3 tools/setup_vanilla.py          # latest release
python3 tools/setup_vanilla.py 26.3     # a specific version
```

This needs Python 3.12+ and about 2 GB of disk. It downloads its own JDK and [Vineflower](https://github.com/Vineflower/vineflower), and everything lands in `vanilla/` (gitignored, because Mojang's code and assets can't be redistributed):

| Path | Contents |
| --- | --- |
| `vanilla/<version>/src/` | Decompiled client (it includes the integrated server) |
| `vanilla/<version>/resources/` | Assets and data bundled in the client jar |
| `vanilla/<version>/generated/` | Data generator output: `reports/` has block states, registries, packet ids and commands |
| `vanilla/<version>/client.jar`, `server.jar` | The original jars, for running the real game as a test oracle |
| `vanilla/libraries/` | The client's libraries (LWJGL, Netty, fastutil, DFU…) |
| `vanilla/toolchain/` | The JDK the version requires, and Vineflower |
