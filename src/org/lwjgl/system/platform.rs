//! org/lwjgl/system/Platform.java (LWJGL 3.4.3)
//!
//! The platform and architecture LWJGL picks its natives by: a library's name (`mapLibraryName`:
//! `libSDL3.so`, `libSDL3.dylib`, `SDL3.dll`) and its path in a natives jar
//! (`mapLibraryPathBundled`: `macos/arm64/org/lwjgl/sdl/libSDL3.dylib`). Java reads `os.name` and
//! `os.arch`; the port is built for one platform, so they're the target's. `mapLibraryName` keeps a
//! name that already looks mapped (its regex), which no caller passes, so only the mapping is
//! ported. The bundled name and path mappers are LWJGL's defaults, which the game doesn't change.

#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Platform {
    FREEBSD,
    LINUX,
    MACOSX,
    WINDOWS,
}

/// `Platform.Architecture`.
#[allow(clippy::upper_case_acronyms)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Architecture {
    X64,
    X86,
    ARM64,
    ARM32,
    PPC64LE,
    RISCV64,
}

impl Platform {
    /// The platform the port runs on, `Platform.get()`.
    pub const fn get() -> Platform {
        if cfg!(target_os = "macos") {
            Platform::MACOSX
        } else if cfg!(target_os = "windows") {
            Platform::WINDOWS
        } else if cfg!(target_os = "freebsd") {
            Platform::FREEBSD
        } else if cfg!(target_os = "linux") {
            Platform::LINUX
        } else {
            panic!("Unknown platform")
        }
    }

    /// `getArchitecture()`.
    pub const fn get_architecture() -> Architecture {
        Architecture::current()
    }

    /// The directory a natives jar keeps this platform's libraries under: `macos`.
    pub fn native_path(self) -> &'static str {
        match self {
            Platform::FREEBSD => "freebsd",
            Platform::LINUX => "linux",
            Platform::MACOSX => "macos",
            Platform::WINDOWS => "windows",
        }
    }

    /// `mapLibraryName(name)`, which is `System.mapLibraryName` for a name that isn't mapped
    /// already: `lib<name>.so`, `lib<name>.dylib` or `<name>.dll`.
    pub fn map_library_name(self, name: &str) -> String {
        match self {
            Platform::FREEBSD | Platform::LINUX => format!("lib{name}.so"),
            Platform::MACOSX => format!("lib{name}.dylib"),
            Platform::WINDOWS => format!("{name}.dll"),
        }
    }

    /// `mapLibraryPathBundled(path)` on this platform and `architecture`: the platform's native
    /// path, the architecture's name in lower case, then the path.
    pub fn map_library_path_bundled(self, architecture: Architecture, path: &str) -> String {
        format!(
            "{}/{}/{path}",
            self.native_path(),
            architecture.lower_case_name()
        )
    }
}

impl Architecture {
    /// The architecture the port runs on, `Architecture.current`.
    pub const fn current() -> Architecture {
        if cfg!(target_arch = "x86_64") {
            Architecture::X64
        } else if cfg!(target_arch = "x86") {
            Architecture::X86
        } else if cfg!(target_arch = "aarch64") {
            Architecture::ARM64
        } else if cfg!(target_arch = "arm") {
            Architecture::ARM32
        } else if cfg!(all(target_arch = "powerpc64", target_endian = "little")) {
            Architecture::PPC64LE
        } else if cfg!(target_arch = "riscv64") {
            Architecture::RISCV64
        } else {
            panic!("Unknown architecture")
        }
    }

    /// `name().toLowerCase()`, as the bundled paths spell it: `arm64`.
    pub fn lower_case_name(self) -> &'static str {
        match self {
            Architecture::X64 => "x64",
            Architecture::X86 => "x86",
            Architecture::ARM64 => "arm64",
            Architecture::ARM32 => "arm32",
            Architecture::PPC64LE => "ppc64le",
            Architecture::RISCV64 => "riscv64",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_names_and_paths_like_lwjgl() {
        assert_eq!(Platform::LINUX.map_library_name("SDL3"), "libSDL3.so");
        assert_eq!(Platform::MACOSX.map_library_name("SDL3"), "libSDL3.dylib");
        assert_eq!(Platform::WINDOWS.map_library_name("SDL3"), "SDL3.dll");
        assert_eq!(
            Platform::MACOSX
                .map_library_path_bundled(Architecture::ARM64, "org/lwjgl/stb/liblwjgl_stb.dylib"),
            "macos/arm64/org/lwjgl/stb/liblwjgl_stb.dylib"
        );
        assert_eq!(
            Platform::LINUX
                .map_library_path_bundled(Architecture::X64, "org/lwjgl/shaderc/libshaderc.so"),
            "linux/x64/org/lwjgl/shaderc/libshaderc.so"
        );
    }
}
