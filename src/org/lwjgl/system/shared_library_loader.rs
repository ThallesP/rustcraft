//! org/lwjgl/system/SharedLibraryLoader.java (LWJGL 3.4.3)
//!
//! LWJGL finds a native library in its natives jar on the classpath (`lwjgl-shaderc-3.4.3-natives-
//! linux.jar` holds `linux/x64/org/lwjgl/shaderc/libshaderc.so`), extracts it to a directory of
//! its own and loads it. The port does the same with the jars vanilla's launcher downloaded, which
//! `vanilla/<version>/classpath.txt` lists, so it runs the very same shaderc, SPIRV-Cross and SDL
//! builds as the game. They're extracted to `target/natives/` and never committed.
//!
//! The jars are zip files whose entries are deflated, read here through their central directory.
//! A library already extracted is kept when its size and CRC-32 match the jar's entry. Each one is
//! written to a temporary file and renamed into place, since test binaries load them in parallel.

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
/// called: `functions!("shaderc", "libshaderc.so"; shaderc_compiler_initialize: fn() -> Compiler; …)`.
/// Each binding module wraps them in safe functions, like LWJGL's classes.
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

/// Loads `library` (`libshaderc.so`) from LWJGL's natives jar for `module` (`shaderc`, whose jar
/// is `lwjgl-shaderc-<version>-natives-linux.jar`), extracting it first.
pub fn load(module: &str, library: &str) -> Result<&'static Library, String> {
    let mut loaded = LOADED
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(library) = loaded.get(library) {
        return Ok(library);
    }

    let path = extract(module, library)?;
    // SAFETY: the library is vanilla's own build of a C library with no initialisers that need
    // anything from the loading thread.
    let opened = unsafe { Library::new(&path) }
        .map_err(|error| format!("can't load {}: {error}", path.display()))?;
    let opened: &'static Library = Box::leak(Box::new(opened));
    loaded.insert(library.to_owned(), opened);
    Ok(opened)
}

/// Extracts `library` from `module`'s natives jar into `target/natives/`, unless it's there already.
pub fn extract(module: &str, library: &str) -> Result<PathBuf, String> {
    let jar = natives_jar(module)?;
    let bytes = fs::read(&jar).map_err(|error| format!("can't read {}: {error}", jar.display()))?;
    let entry_name = format!("linux/x64/org/lwjgl/{module}/{library}");
    let entry = find_entry(&bytes, &entry_name)
        .ok_or_else(|| format!("{} has no {entry_name}", jar.display()))?;

    let directory = Path::new(MANIFEST_DIR).join("target").join("natives");
    let path = directory.join(library);
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
    let temporary = directory.join(format!("{library}.{}.tmp", std::process::id()));
    fs::write(&temporary, &contents)
        .and_then(|()| fs::rename(&temporary, &path))
        .map_err(|error| format!("can't write {}: {error}", path.display()))?;
    Ok(path)
}

/// The natives jar of `module` on vanilla's classpath.
fn natives_jar(module: &str) -> Result<PathBuf, String> {
    let classpath = Path::new(MANIFEST_DIR)
        .join("vanilla")
        .join(VERSION.trim())
        .join("classpath.txt");
    let listed = fs::read_to_string(&classpath).map_err(|error| {
        format!(
            "can't read {}: {error}. Run `python3 tools/setup_vanilla.py` first.",
            classpath.display()
        )
    })?;
    let prefix = format!("lwjgl-{module}-");
    listed
        .lines()
        .map(PathBuf::from)
        .find(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| {
                    name.starts_with(&prefix) && name.ends_with("-natives-linux.jar")
                })
        })
        .ok_or_else(|| {
            format!(
                "{} lists no natives jar for lwjgl-{module}",
                classpath.display()
            )
        })
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

    #[test]
    fn extracts_vanillas_natives() {
        for (module, library) in [
            ("shaderc", "libshaderc.so"),
            ("spvc", "libspirv-cross.so"),
            ("sdl", "libSDL3.so"),
        ] {
            let path = extract(module, library).unwrap();
            assert!(path.ends_with(format!("target/natives/{library}")));
        }
    }
}
