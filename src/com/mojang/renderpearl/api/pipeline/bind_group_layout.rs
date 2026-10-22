//! com/mojang/renderpearl/api/pipeline/BindGroupLayout.java
//!
//! A builder's errors are panics, where Java throws `IllegalArgumentException`.

use crate::com::mojang::renderpearl::api::gpu_format::GpuFormat;
use crate::com::mojang::renderpearl::api::pipeline::uniform_type::UniformType;
use std::collections::BTreeSet;

/// The uniforms a pipeline binds together: uniform buffers, samplers and texel buffers by name.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct BindGroupLayout {
    pub uniforms: Vec<UniformDescription>,
}

impl BindGroupLayout {
    pub fn new(uniforms: Vec<UniformDescription>) -> BindGroupLayout {
        BindGroupLayout { uniforms }
    }

    pub fn builder() -> Builder {
        Builder {
            uniforms: Vec::new(),
        }
    }

    pub fn flatten_uniforms(bind_group_layouts: &[BindGroupLayout]) -> Vec<UniformDescription> {
        bind_group_layouts
            .iter()
            .flat_map(|layout| layout.uniforms.iter().cloned())
            .collect()
    }

    pub fn ensure_compatible(bind_group_layouts: &[BindGroupLayout]) {
        let mut names = BTreeSet::new();
        for (layout_index, layout) in bind_group_layouts.iter().enumerate() {
            for uniform in &layout.uniforms {
                if !names.insert(uniform.name.as_str()) {
                    panic!(
                        "Duplicate bind name '{}' in bind group layout {layout_index}",
                        uniform.name
                    );
                }
            }
        }
    }
}

pub struct Builder {
    uniforms: Vec<UniformDescription>,
}

impl Builder {
    pub fn with_uniform(mut self, name: &str, uniform_type: UniformType) -> Builder {
        assert!(
            uniform_type != UniformType::TEXEL_BUFFER,
            "Cannot use texel buffer without specifying texture format"
        );
        self.uniforms
            .push(UniformDescription::new(name, uniform_type));
        self
    }

    /// `withUniform(String, UniformType, GpuFormat)`.
    pub fn with_uniform_format(
        mut self,
        name: &str,
        uniform_type: UniformType,
        format: GpuFormat,
    ) -> Builder {
        assert!(
            uniform_type == UniformType::TEXEL_BUFFER,
            "Only texel buffer can specify texture format"
        );
        self.uniforms
            .push(UniformDescription::texel_buffer(name, format));
        self
    }

    pub fn build(self) -> BindGroupLayout {
        BindGroupLayout::new(self.uniforms)
    }
}

/// `BindGroupLayout.UniformDescription`.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct UniformDescription {
    pub name: String,
    pub uniform_type: UniformType,
    pub gpu_format: Option<GpuFormat>,
}

impl UniformDescription {
    pub fn new(name: &str, uniform_type: UniformType) -> UniformDescription {
        assert!(
            uniform_type != UniformType::TEXEL_BUFFER,
            "Texel buffer needs a texture format"
        );
        UniformDescription {
            name: name.to_owned(),
            uniform_type,
            gpu_format: None,
        }
    }

    /// `UniformDescription(String, GpuFormat)`.
    pub fn texel_buffer(name: &str, gpu_format: GpuFormat) -> UniformDescription {
        UniformDescription {
            name: name.to_owned(),
            uniform_type: UniformType::TEXEL_BUFFER,
            gpu_format: Some(gpu_format),
        }
    }
}
