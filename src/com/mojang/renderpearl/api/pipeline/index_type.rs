//! com/mojang/renderpearl/api/pipeline/IndexType.java

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum IndexType {
    SHORT,
    INT,
}

impl IndexType {
    pub fn bytes(self) -> i32 {
        match self {
            IndexType::SHORT => 2,
            IndexType::INT => 4,
        }
    }

    /// The smaller type that can index `length` vertices.
    pub fn least(length: i32) -> IndexType {
        if length & -65536 != 0 {
            IndexType::INT
        } else {
            IndexType::SHORT
        }
    }
}
