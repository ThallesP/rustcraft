//! com/mojang/renderpearl/api/pipeline/ShaderSource.java
//!
//! `CachedIncludeSource` holds the `shaderc_include_result` shaderc reads an include from, with
//! the bytes it points to: the include's id (`minecraft:fog.glsl`) as its source name and its text,
//! both UTF-8 without a terminator (`memUTF8(…, false)`). Java frees them in `close()`; here they're
//! dropped with it.

use crate::com::mojang::renderpearl::api::pipeline::shader_type::ShaderType;
use crate::net::minecraft::resources::identifier::Identifier;
use crate::org::lwjgl::util::shaderc::shaderc::ShadercIncludeResult;

/// Where a pipeline's shaders and their includes come from.
pub trait ShaderSource: Sync {
    fn get_shader(&self, id: &Identifier, shader_type: ShaderType) -> Option<&str>;

    fn get_include(&self, id: &Identifier) -> Option<&CachedIncludeSource>;
}

/// `ShaderSource.CachedIncludeSource`.
pub struct CachedIncludeSource {
    result: Box<ShadercIncludeResult>,
    _source_name: Box<[u8]>,
    _content: Box<[u8]>,
}

// SAFETY: the result only points into the boxes it's kept with, which nothing changes after
// `new`; shaderc reads it from whichever thread compiles.
unsafe impl Send for CachedIncludeSource {}
unsafe impl Sync for CachedIncludeSource {}

impl CachedIncludeSource {
    fn new(source_name: &str, content: &str) -> CachedIncludeSource {
        let source_name: Box<[u8]> = source_name.as_bytes().into();
        let content: Box<[u8]> = content.as_bytes().into();
        let result = Box::new(ShadercIncludeResult {
            source_name: source_name.as_ptr().cast(),
            source_name_length: source_name.len(),
            content: content.as_ptr().cast(),
            content_length: content.len(),
            user_data: std::ptr::null_mut(),
        });
        CachedIncludeSource {
            result,
            _source_name: source_name,
            _content: content,
        }
    }

    pub fn create(id: &Identifier, source: &str) -> CachedIncludeSource {
        CachedIncludeSource::new(&id.to_string(), source)
    }

    /// An error for shaderc: an empty source name, with the message as the content.
    pub fn create_error(message: &str) -> CachedIncludeSource {
        CachedIncludeSource::new("", message)
    }

    pub fn include_result_ptr(&self) -> *mut ShadercIncludeResult {
        (&*self.result as *const ShadercIncludeResult).cast_mut()
    }
}
