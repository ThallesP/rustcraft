//! net/minecraft/client/resources/model/ModelDebugName.java

/// What a model is called in the bake's warnings.
pub trait ModelDebugName {
    fn debug_name(&self) -> String;
}
