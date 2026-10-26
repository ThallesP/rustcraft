//! com/mojang/renderpearl/api/pipeline/PrimitiveTopology.java

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PrimitiveTopology {
    LINES,
    DEBUG_LINES,
    DEBUG_LINE_STRIP,
    POINTS,
    TRIANGLES,
    TRIANGLE_STRIP,
    TRIANGLE_FAN,
    QUADS,
}

impl PrimitiveTopology {
    /// `values()`.
    pub const VALUES: [PrimitiveTopology; 8] = [
        PrimitiveTopology::LINES,
        PrimitiveTopology::DEBUG_LINES,
        PrimitiveTopology::DEBUG_LINE_STRIP,
        PrimitiveTopology::POINTS,
        PrimitiveTopology::TRIANGLES,
        PrimitiveTopology::TRIANGLE_STRIP,
        PrimitiveTopology::TRIANGLE_FAN,
        PrimitiveTopology::QUADS,
    ];
}

use PrimitiveTopology::*;

// Each constant's `primitiveLength`, `primitiveStride` and `connectedPrimitives`.
const DATA: [(i32, i32, bool); 8] = [
    (2, 2, false),
    (2, 2, false),
    (2, 1, true),
    (1, 1, false),
    (3, 3, false),
    (3, 1, true),
    (3, 1, true),
    (4, 4, false),
];

impl PrimitiveTopology {
    pub fn primitive_length(self) -> i32 {
        DATA[self as usize].0
    }

    pub fn primitive_stride(self) -> i32 {
        DATA[self as usize].1
    }

    pub fn connected_primitives(self) -> bool {
        DATA[self as usize].2
    }

    pub fn index_count(self, vertex_count: i32) -> i32 {
        match self {
            LINES | QUADS => vertex_count / 4 * 6,
            DEBUG_LINES | DEBUG_LINE_STRIP | POINTS | TRIANGLES | TRIANGLE_STRIP | TRIANGLE_FAN => {
                vertex_count
            }
        }
    }
}
