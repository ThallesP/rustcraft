//! org/lwjgl/system/SharedLibraryLoader.java (LWJGL 3.4.3)
//!
//! LWJGL finds a native library on the classpath at the path `Platform.mapLibraryPathBundled`
//! gives it (`macos/arm64/org/lwjgl/shaderc/libshaderc.dylib`), extracts it to a directory of its
//! own and loads it. The launcher puts each module's natives jars for the platform on the classpath
//! (`lwjgl-shaderc-3.4.3-natives-linux.jar`; on a Mac both `-natives-macos.jar`, which holds
//! `macos/x64/`, and `-natives-macos-arm64.jar`), and the class loader finds the path in the first
//! jar that has it. The port does the same with the jars vanilla's launcher would put there, which
//! `vanilla/<version>/classpath.txt` lists, so it runs the very same shaderc, SPIRV-Cross, SDL and
//! stb builds as the game. They're extracted to `target/natives/` and never committed.
//!
//! The jars are zip files whose entries are deflated, read here through their central directory.
//! A library already extracted is kept when its size and CRC-32 match the jar's entry. Each one is
//! written to a temporary file and renamed into place, since test binaries load them in parallel.

use crate::org::lwjgl::system::platform::{Architecture, Platform};
use libloading::Library;
use miniz_oxide::inflate::decompress_to_vec;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};

const MANIFEST_DIR: &str = env!("CARGO_MANIFEST_DIR");
const VERSION: &str = include_str!("../../../../MINECRAFT_VERSION");

/// Every library loaded so far, by file name. They stay loaded for the life of the process, like
/// LWJGL's.
static LOADED: LazyLock<Mutex<BTreeMap<String, &'static Library>>> = LazyLock::new(Mutex::default);

/// Declares a static table of a native library's C functions, looked up the first time one is
/// called: `functions!("shaderc", "shaderc"; shaderc_compiler_initialize: fn() -> Compiler; …)`,
/// the module and the library's name, which the platform maps (`libshaderc.so`). Each binding
/// module wraps them in safe functions, like LWJGL's classes.
macro_rules! functions {
    ($module:literal, $library:literal; $($name:ident: fn($($argument:ty),*) $(-> $result:ty)?;)*) => {
        #[allow(non_snake_case)]
        struct Functions {
            $($name: unsafe extern "C" fn($($argument),*) $(-> $result)?,)*
        }

        static FUNCTIONS: std::sync::LazyLock<Functions> = std::sync::LazyLock::new(|| {
            let library = $crate::org::lwjgl::system::shared_library_loader::load($module, $library)
                .unwrap_or_else(|error| panic!("{error}"));
            // SAFETY: each symbol is declared with its signature from the library's header.
            unsafe {
                Functions {
                    $($name: *library
                        .get(concat!(stringify!($name), "\0").as_bytes())
                        .unwrap_or_else(|error| panic!("{}: {error}", $library)),)*
                }
            }
        });
    };
}
pub(crate) use functions;

/// Loads the library `name` (`shaderc`) of LWJGL's `module` (`shaderc`) from its natives jar,
/// extracting it first.
pub fn load(module: &str, name: &str) -> Result<&'static Library, String> {
    let file_name = Platform::get().map_library_name(name);
    let mut loaded = LOADED
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(library) = loaded.get(&file_name) {
        return Ok(library);
    }

    let path = extract(module, name)?;
    // SAFETY: the library is vanilla's own build of a C library with no initialisers that need
    // anything from the loading thread.
    let opened = unsafe { Library::new(&path) }
        .map_err(|error| format!("can't load {}: {error}", path.display()))?;
    let opened: &'static Library = Box::leak(Box::new(opened));
    loaded.insert(file_name, opened);
    Ok(opened)
}

/// Extracts the library `name` of `module` into `target/natives/`, unless it's there already.
pub fn extract(module: &str, name: &str) -> Result<PathBuf, String> {
    let classpath = classpath_file("classpath.txt");
    let bundled = find_bundled(
        &read_classpath(&classpath)?,
        Platform::get(),
        Architecture::current(),
        module,
        name,
    )
    .map_err(|error| format!("{} {error}", classpath.display()))?;
    let Bundled {
        jar,
        bytes,
        entry,
        path: entry_name,
        file_name,
    } = bundled;

    let directory = Path::new(MANIFEST_DIR).join("target").join("natives");
    let path = directory.join(&file_name);
    if let Ok(existing) = fs::read(&path)
        && existing.len() as u64 == entry.size
        && crc32(&existing) == entry.crc32
    {
        return Ok(path);
    }

    let contents = entry.read(&bytes)?;
    if contents.len() as u64 != entry.size || crc32(&contents) != entry.crc32 {
        return Err(format!("{entry_name} in {} is corrupt", jar.display()));
    }
    fs::create_dir_all(&directory)
        .map_err(|error| format!("can't create {}: {error}", directory.display()))?;
    let temporary = directory.join(format!("{file_name}.{}.tmp", std::process::id()));
    fs::write(&temporary, &contents)
        .and_then(|()| fs::rename(&temporary, &path))
        .map_err(|error| format!("can't write {}: {error}", path.display()))?;
    Ok(path)
}

/// A file of vanilla's version directory listing a classpath: `classpath.txt`, this platform's, or
/// `classpath-macos-arm64.txt`, another's (`setup_vanilla.py --natives macos-arm64`).
fn classpath_file(name: &str) -> PathBuf {
    Path::new(MANIFEST_DIR)
        .join("vanilla")
        .join(VERSION.trim())
        .join(name)
}

fn read_classpath(file: &Path) -> Result<Vec<PathBuf>, String> {
    let listed = fs::read_to_string(file).map_err(|error| {
        format!(
            "can't read {}: {error}. Run `python3 tools/setup_vanilla.py` first.",
            file.display()
        )
    })?;
    Ok(listed.lines().map(PathBuf::from).collect())
}

/// A library in a natives jar, read into memory.
struct Bundled {
    jar: PathBuf,
    bytes: Vec<u8>,
    entry: Entry,
    /// The entry's name, `mapLibraryPathBundled`'s path.
    path: String,
    /// `mapLibraryName`'s: `libshaderc.dylib`.
    file_name: String,
}

/// Finds the library `name` of `module` the way the class loader finds LWJGL's resource for
/// `platform` and `architecture`: in the first of the module's natives jars on the classpath
/// (`lwjgl-<module>-<version>-natives-…`) that has the bundled path.
fn find_bundled(
    classpath: &[PathBuf],
    platform: Platform,
    architecture: Architecture,
    module: &str,
    name: &str,
) -> Result<Bundled, String> {
    let file_name = platform.map_library_name(name);
    let path =
        platform.map_library_path_bundled(architecture, &format!("org/lwjgl/{module}/{file_name}"));
    let prefix = format!("lwjgl-{module}-");
    for jar in classpath {
        let is_natives_jar = jar
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with(&prefix) && name.contains("-natives-"));
        if !is_natives_jar {
            continue;
        }
        let bytes =
            fs::read(jar).map_err(|error| format!("can't read {}: {error}", jar.display()))?;
        if let Some(entry) = find_entry(&bytes, &path) {
            return Ok(Bundled {
                jar: jar.clone(),
                bytes,
                entry,
                path,
                file_name,
            });
        }
    }
    Err(format!(
        "lists no natives jar of lwjgl-{module} with {path}"
    ))
}

struct Entry {
    method: u16,
    crc32: u32,
    compressed_size: u64,
    size: u64,
    local_header: u64,
}

impl Entry {
    fn read(&self, zip: &[u8]) -> Result<Vec<u8>, String> {
        let header = self.local_header as usize;
        if u32_at(zip, header) != Some(0x0403_4b50) {
            return Err("bad local file header".to_owned());
        }
        let name_length = u16_at(zip, header + 26).ok_or("truncated zip")? as usize;
        let extra_length = u16_at(zip, header + 28).ok_or("truncated zip")? as usize;
        let start = header + 30 + name_length + extra_length;
        let data = zip
            .get(start..start + self.compressed_size as usize)
            .ok_or("truncated zip")?;
        match self.method {
            0 => Ok(data.to_vec()),
            8 => decompress_to_vec(data).map_err(|error| format!("can't inflate: {error:?}")),
            method => Err(format!("unsupported compression method {method}")),
        }
    }
}

/// Finds `name` in the zip's central directory, which the end of central directory record at the
/// end of the file points to.
fn find_entry(zip: &[u8], name: &str) -> Option<Entry> {
    let end = (0..=zip.len().checked_sub(22)?)
        .rev()
        .find(|&offset| u32_at(zip, offset) == Some(0x0605_4b50))?;
    let count = u16_at(zip, end + 10)? as usize;
    let mut offset = u32_at(zip, end + 16)? as usize;
    for _ in 0..count {
        if u32_at(zip, offset)? != 0x0201_4b50 {
            return None;
        }
        let name_length = u16_at(zip, offset + 28)? as usize;
        let extra_length = u16_at(zip, offset + 30)? as usize;
        let comment_length = u16_at(zip, offset + 32)? as usize;
        if zip.get(offset + 46..offset + 46 + name_length)? == name.as_bytes() {
            return Some(Entry {
                method: u16_at(zip, offset + 10)?,
                crc32: u32_at(zip, offset + 16)?,
                compressed_size: u32_at(zip, offset + 20)? as u64,
                size: u32_at(zip, offset + 24)? as u64,
                local_header: u32_at(zip, offset + 42)? as u64,
            });
        }
        offset += 46 + name_length + extra_length + comment_length;
    }
    None
}

fn u16_at(bytes: &[u8], offset: usize) -> Option<u16> {
    Some(u16::from_le_bytes(
        bytes.get(offset..offset + 2)?.try_into().ok()?,
    ))
}

fn u32_at(bytes: &[u8], offset: usize) -> Option<u32> {
    Some(u32::from_le_bytes(
        bytes.get(offset..offset + 4)?.try_into().ok()?,
    ))
}

/// The zip format's CRC-32 (reflected, polynomial 0xEDB88320).
fn crc32(bytes: &[u8]) -> u32 {
    static TABLE: LazyLock<[u32; 256]> = LazyLock::new(|| {
        let mut table = [0; 256];
        for (index, slot) in table.iter_mut().enumerate() {
            let mut crc = index as u32;
            for _ in 0..8 {
                crc = if crc & 1 != 0 {
                    0xEDB8_8320 ^ (crc >> 1)
                } else {
                    crc >> 1
                };
            }
            *slot = crc;
        }
        table
    });
    !bytes.iter().fold(!0u32, |crc, &byte| {
        TABLE[((crc ^ byte as u32) & 0xFF) as usize] ^ (crc >> 8)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc32_matches_the_zip_format() {
        assert_eq!(crc32(b""), 0);
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
    }

    /// The libraries the port loads, by module and name, as their bindings name them.
    const LIBRARIES: [(&str, &str); 5] = [
        ("shaderc", "shaderc"),
        ("spvc", "spirv-cross"),
        ("sdl", "SDL3"),
        ("stb", "lwjgl_stb"),
        ("vma", "lwjgl_vma"),
    ];

    #[test]
    fn extracts_vanillas_natives() {
        for (module, name) in LIBRARIES {
            let path = extract(module, name).unwrap();
            let file_name = Platform::get().map_library_name(name);
            assert!(path.ends_with(format!("target/natives/{file_name}")));
        }
    }

    /// What a Mac's loader finds, in the jars vanilla's launcher puts on a Mac's classpath:
    /// `setup_vanilla.py --natives macos-arm64 --natives macos` fetches them and writes their
    /// classpaths, which CI does. The entries are the jars' (`unzip -l`), and each is a 64-bit
    /// Mach-O dylib for its architecture, the Apple Silicon ones signed, since a Mac on ARM only
    /// runs signed code. Nothing is loaded. With them, the Vulkan loader LWJGL loads on a Mac:
    /// MoltenVK, from `lwjgl-vulkan`'s natives.
    #[test]
    fn finds_each_library_in_the_natives_jars_of_macos() {
        const CPU_TYPE_X86_64: u32 = 0x0100_0007;
        const CPU_TYPE_ARM64: u32 = 0x0100_000C;
        const MH_DYLIB: u32 = 6;
        const LC_CODE_SIGNATURE: u32 = 0x1D;
        for (classifier, architecture, cpu_type) in [
            ("macos-arm64", Architecture::ARM64, CPU_TYPE_ARM64),
            ("macos", Architecture::X64, CPU_TYPE_X86_64),
        ] {
            let file = classpath_file(&format!("classpath-{classifier}.txt"));
            if !file.exists() {
                assert!(
                    std::env::var_os("CI").is_none(),
                    "CI fetches {classifier}'s natives, but {} is missing",
                    file.display()
                );
                eprintln!(
                    "skipped: no {}; `python3 tools/setup_vanilla.py --natives {classifier}` writes it",
                    file.display()
                );
                continue;
            }
            let classpath = read_classpath(&file).unwrap();
            let arch = architecture.lower_case_name();
            let libraries = LIBRARIES.into_iter().chain([("vulkan", "MoltenVK")]);
            for ((module, name), entry_name) in libraries.zip([
                format!("macos/{arch}/org/lwjgl/shaderc/libshaderc.dylib"),
                format!("macos/{arch}/org/lwjgl/spvc/libspirv-cross.dylib"),
                format!("macos/{arch}/org/lwjgl/sdl/libSDL3.dylib"),
                format!("macos/{arch}/org/lwjgl/stb/liblwjgl_stb.dylib"),
                format!("macos/{arch}/org/lwjgl/vma/liblwjgl_vma.dylib"),
                format!("macos/{arch}/org/lwjgl/vulkan/libMoltenVK.dylib"),
            ]) {
                let bundled =
                    find_bundled(&classpath, Platform::MACOSX, architecture, module, name).unwrap();
                let jar_name = bundled.jar.file_name().unwrap().to_str().unwrap();
                assert!(
                    jar_name.starts_with(&format!("lwjgl-{module}-"))
                        && jar_name.ends_with(&format!("-natives-{classifier}.jar")),
                    "{jar_name}"
                );
                assert_eq!(bundled.path, entry_name);
                assert_eq!(bundled.file_name, entry_name.rsplit('/').next().unwrap());

                let contents = bundled.entry.read(&bundled.bytes).unwrap();
                assert_eq!(contents.len() as u64, bundled.entry.size);
                assert_eq!(crc32(&contents), bundled.entry.crc32);
                assert_eq!(u32_at(&contents, 0), Some(0xFEED_FACF), "{entry_name}");
                assert_eq!(u32_at(&contents, 4), Some(cpu_type), "{entry_name}");
                assert_eq!(u32_at(&contents, 12), Some(MH_DYLIB), "{entry_name}");
                let command_count = u32_at(&contents, 16).unwrap();
                let mut offset = 32;
                let mut signed = false;
                for _ in 0..command_count {
                    signed |= u32_at(&contents, offset) == Some(LC_CODE_SIGNATURE);
                    offset += u32_at(&contents, offset + 4).unwrap() as usize;
                }
                assert_eq!(signed, architecture == Architecture::ARM64, "{entry_name}");
            }
        }
    }
}
