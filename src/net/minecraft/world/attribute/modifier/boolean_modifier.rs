//! net/minecraft/world/attribute/modifier/BooleanModifier.java

use crate::net::minecraft::world::attribute::modifier::attribute_modifier::OperationId;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BooleanModifier {
    AND,
    NAND,
    OR,
    NOR,
    XOR,
    XNOR,
}

impl BooleanModifier {
    pub fn apply(&self, subject: bool, argument: bool) -> bool {
        match self {
            BooleanModifier::AND => argument && subject,
            BooleanModifier::NAND => !argument || !subject,
            BooleanModifier::OR => argument || subject,
            BooleanModifier::NOR => !argument && !subject,
            BooleanModifier::XOR => argument ^ subject,
            BooleanModifier::XNOR => argument == subject,
        }
    }

    /// `AttributeModifier.BOOLEAN_LIBRARY`.
    pub fn by_operation(operation: OperationId) -> Option<BooleanModifier> {
        match operation {
            OperationId::AND => Some(BooleanModifier::AND),
            OperationId::NAND => Some(BooleanModifier::NAND),
            OperationId::OR => Some(BooleanModifier::OR),
            OperationId::NOR => Some(BooleanModifier::NOR),
            OperationId::XOR => Some(BooleanModifier::XOR),
            OperationId::XNOR => Some(BooleanModifier::XNOR),
            _ => None,
        }
    }
}
