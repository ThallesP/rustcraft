//! com/mojang/renderpearl/api/GpuFormat.java

/// A texel or vertex attribute format: its component type and how many components it has.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum GpuFormat {
    R8_UNORM,
    R8_SNORM,
    RG8_UNORM,
    RG8_SNORM,
    RGB8_UNORM,
    RGB8_SNORM,
    RGBA8_UNORM,
    RGBA8_SNORM,
    R16_UNORM,
    R16_SNORM,
    RG16_UNORM,
    RG16_SNORM,
    RGB16_UNORM,
    RGB16_SNORM,
    RGBA16_UNORM,
    RGBA16_SNORM,
    R8_UINT,
    R8_SINT,
    RG8_UINT,
    RG8_SINT,
    RGB8_UINT,
    RGB8_SINT,
    RGBA8_UINT,
    RGBA8_SINT,
    R16_UINT,
    R16_SINT,
    RG16_UINT,
    RG16_SINT,
    RGB16_UINT,
    RGB16_SINT,
    RGBA16_UINT,
    RGBA16_SINT,
    R32_UINT,
    R32_SINT,
    RG32_UINT,
    RG32_SINT,
    RGB32_UINT,
    RGB32_SINT,
    RGBA32_UINT,
    RGBA32_SINT,
    R16_FLOAT,
    RG16_FLOAT,
    RGB16_FLOAT,
    RGBA16_FLOAT,
    R32_FLOAT,
    RG32_FLOAT,
    RGB32_FLOAT,
    RGBA32_FLOAT,
    RGB10A2_UNORM,
    RGB10A2_UINT,
    RG11B10_FLOAT,
    D32_FLOAT,
    D32_FLOAT_S8_UINT,
    D24_UNORM_S8_UINT,
    D16_UNORM,
    S8_UINT,
}

use ComponentType::*;
use GpuFormat::*;

// Each constant's constructor arguments, in declaration order.
const DATA: [(ComponentType, i32); 56] = [
    (UNORM_8, 1),
    (SNORM_8, 1),
    (UNORM_8, 2),
    (SNORM_8, 2),
    (UNORM_8, 3),
    (SNORM_8, 3),
    (UNORM_8, 4),
    (SNORM_8, 4),
    (UNORM_16, 1),
    (SNORM_16, 1),
    (UNORM_16, 2),
    (SNORM_16, 2),
    (UNORM_16, 3),
    (SNORM_16, 3),
    (UNORM_16, 4),
    (SNORM_16, 4),
    (UINT_8, 1),
    (SINT_8, 1),
    (UINT_8, 2),
    (SINT_8, 2),
    (UINT_8, 3),
    (SINT_8, 3),
    (UINT_8, 4),
    (SINT_8, 4),
    (UINT_16, 1),
    (SINT_16, 1),
    (UINT_16, 2),
    (SINT_16, 2),
    (UINT_16, 3),
    (SINT_16, 3),
    (UINT_16, 4),
    (SINT_16, 4),
    (UINT_32, 1),
    (SINT_32, 1),
    (UINT_32, 2),
    (SINT_32, 2),
    (UINT_32, 3),
    (SINT_32, 3),
    (UINT_32, 4),
    (SINT_32, 4),
    (FLOAT_16, 1),
    (FLOAT_16, 2),
    (FLOAT_16, 3),
    (FLOAT_16, 4),
    (FLOAT_32, 1),
    (FLOAT_32, 2),
    (FLOAT_32, 3),
    (FLOAT_32, 4),
    (OPAQUE_32, 1),
    (OPAQUE_32, 1),
    (OPAQUE_32, 1),
    (OPAQUE_32, 1),
    (OPAQUE_64, 1),
    (OPAQUE_32, 1),
    (OPAQUE_16, 1),
    (OPAQUE_8, 1),
];

impl GpuFormat {
    pub fn block_size(self) -> i32 {
        self.component_type().byte_size() * self.component_count()
    }

    pub fn byte_alignment(self) -> i32 {
        self.component_type().byte_size()
    }

    pub fn component_type(self) -> ComponentType {
        DATA[self as usize].0
    }

    pub fn component_count(self) -> i32 {
        DATA[self as usize].1
    }

    pub fn has_color_aspect(self) -> bool {
        !self.has_depth_aspect() && !self.has_stencil_aspect()
    }

    pub fn has_depth_aspect(self) -> bool {
        matches!(
            self,
            D32_FLOAT | D32_FLOAT_S8_UINT | D24_UNORM_S8_UINT | D16_UNORM
        )
    }

    pub fn has_stencil_aspect(self) -> bool {
        matches!(self, S8_UINT | D32_FLOAT_S8_UINT | D24_UNORM_S8_UINT)
    }

    /// Java's `name()`.
    pub fn name(self) -> String {
        format!("{self:?}")
    }
}

/// `GpuFormat.ComponentType`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum ComponentType {
    UNORM_8,
    SNORM_8,
    UINT_8,
    SINT_8,
    UNORM_16,
    SNORM_16,
    UINT_16,
    SINT_16,
    FLOAT_16,
    UINT_32,
    SINT_32,
    FLOAT_32,
    OPAQUE_8,
    OPAQUE_16,
    OPAQUE_32,
    OPAQUE_64,
}

impl ComponentType {
    pub fn byte_size(self) -> i32 {
        match self {
            UNORM_8 | SNORM_8 | UINT_8 | SINT_8 | OPAQUE_8 => 1,
            UNORM_16 | SNORM_16 | UINT_16 | SINT_16 | FLOAT_16 | OPAQUE_16 => 2,
            UINT_32 | SINT_32 | FLOAT_32 | OPAQUE_32 => 4,
            OPAQUE_64 => 8,
        }
    }
}
