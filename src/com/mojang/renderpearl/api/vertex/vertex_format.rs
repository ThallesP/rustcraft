//! com/mojang/renderpearl/api/vertex/VertexFormat.java
//!
//! A builder's errors are panics, where Java throws `IllegalArgumentException` or
//! `IllegalStateException`.

use crate::com::mojang::renderpearl::api::gpu_format::GpuFormat;
use crate::com::mojang::renderpearl::api::vertex::vertex_format_element::VertexFormatElement;
use crate::net::minecraft::util::mth::Mth;
use std::fmt;

/// The attributes of one vertex buffer binding, with their offsets. An attribute with several
/// columns (a matrix) is an element per column under one name.
#[derive(Clone, Debug)]
pub struct VertexFormat {
    vertex_size: i32,
    step_rate: i32,
    element_values: Vec<VertexFormatElement>,
}

impl VertexFormat {
    const VERTEX_ALIGNMENT: i32 = 4;
    pub const MAX_VERTEX_ELEMENTS: usize = 16;

    pub fn builder(step_rate: i32) -> Builder {
        Builder {
            elements: Vec::new(),
            offset: 0,
            step_rate,
        }
    }

    pub fn get_vertex_size(&self) -> i32 {
        self.vertex_size
    }

    pub fn get_step_rate(&self) -> i32 {
        self.step_rate
    }

    pub fn get_elements(&self) -> &[VertexFormatElement] {
        &self.element_values
    }

    /// The first element with the name, as Java's map keeps with `putIfAbsent`.
    pub fn get_element(&self, attribute_name: &str) -> Option<&VertexFormatElement> {
        self.element_values
            .iter()
            .find(|element| element.name == attribute_name)
    }

    pub fn contains(&self, attribute_name: &str) -> bool {
        self.get_element(attribute_name).is_some()
    }
}

/// Java compares the first element of each name and the vertex size.
impl PartialEq for VertexFormat {
    fn eq(&self, other: &VertexFormat) -> bool {
        self.firsts() == other.firsts() && self.vertex_size == other.vertex_size
    }
}

impl VertexFormat {
    /// Java's `elements` map: the first element of each name, by name.
    fn firsts(&self) -> Vec<&VertexFormatElement> {
        let mut firsts: Vec<&VertexFormatElement> = Vec::new();
        for element in &self.element_values {
            if !firsts.iter().any(|first| first.name == element.name) {
                firsts.push(element);
            }
        }
        firsts.sort_by(|a, b| a.name.cmp(&b.name));
        firsts
    }
}

impl fmt::Display for VertexFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names: Vec<&str> = self
            .element_values
            .iter()
            .map(|element| element.name.as_str())
            .collect();
        write!(f, "VertexFormat[{}]", names.join(", "))
    }
}

pub struct Builder {
    elements: Vec<VertexFormatElement>,
    offset: i32,
    step_rate: i32,
}

impl Builder {
    fn create_attribute(&mut self, name: &str, offset: i32, element_format: GpuFormat) {
        assert!(
            self.elements.len() < VertexFormat::MAX_VERTEX_ELEMENTS,
            "Having more than 16 attributes are not supported"
        );
        assert!(
            Mth::is_multiple_of(offset, element_format.byte_alignment()),
            "{name} is not aligned to {} as required by {element_format:?}",
            element_format.byte_alignment()
        );
        self.elements.push(VertexFormatElement {
            name: name.to_owned(),
            offset,
            format: element_format,
        });
    }

    fn validate_unique_name(&self, name: &str) {
        assert!(
            self.elements.iter().all(|element| element.name != name),
            "Another vertex attribute exists with the name {name}"
        );
    }

    pub fn add_attribute(mut self, name: &str, element_format: GpuFormat) -> Builder {
        self.validate_unique_name(name);
        self.create_attribute(name, self.offset, element_format);
        self.offset += element_format.block_size();
        self
    }

    /// `addAttribute(String, int stride, GpuFormat)`.
    pub fn add_attribute_stride(
        mut self,
        name: &str,
        stride: i32,
        element_format: GpuFormat,
    ) -> Builder {
        self.validate_unique_name(name);
        self.create_attribute(name, self.offset, element_format);
        self.offset += stride;
        self
    }

    /// `addAttribute(String, GpuFormat, int columnCount)`.
    pub fn add_attribute_columns(
        mut self,
        name: &str,
        element_format: GpuFormat,
        column_count: i32,
    ) -> Builder {
        self.validate_unique_name(name);
        for _ in 0..column_count {
            self.create_attribute(name, self.offset, element_format);
            self.offset += element_format.block_size();
        }
        self
    }

    /// `addAttribute(String, int offset, int stride, GpuFormat, int columnCount)`.
    pub fn add_attribute_at(
        mut self,
        name: &str,
        offset: i32,
        stride: i32,
        element_format: GpuFormat,
        column_count: i32,
    ) -> Builder {
        self.validate_unique_name(name);
        let mut offset_tracker = offset;
        for _ in 0..column_count {
            self.create_attribute(name, offset_tracker, element_format);
            offset_tracker += stride;
        }
        self.offset = self.offset.max(offset_tracker);
        self
    }

    pub fn build(self) -> VertexFormat {
        let vertex_size = self.offset;
        assert!(
            Mth::is_multiple_of(vertex_size, VertexFormat::VERTEX_ALIGNMENT),
            "Vertex size must be a multiple of 4, was {vertex_size}"
        );
        VertexFormat {
            vertex_size,
            step_rate: self.step_rate,
            element_values: self.elements,
        }
    }
}
