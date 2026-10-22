//! com/mojang/renderpearl/api/vertex/VertexFormatElement.java

use crate::com::mojang::renderpearl::api::gpu_format::GpuFormat;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct VertexFormatElement {
    pub name: String,
    pub offset: i32,
    pub format: GpuFormat,
}

impl fmt::Display for VertexFormatElement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {:?} offset:{}", self.name, self.format, self.offset)
    }
}
