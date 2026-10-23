//! net/minecraft/client/resources/model/ResolvableModel.java

use crate::net::minecraft::resources::identifier::Identifier;

/// Something made of models, which it names to the model discovery.
pub trait ResolvableModel {
    fn resolve_dependencies(&self, resolver: &mut dyn Resolver);
}

/// `ResolvableModel.Resolver`.
pub trait Resolver {
    fn mark_dependency(&mut self, id: &Identifier);
}
