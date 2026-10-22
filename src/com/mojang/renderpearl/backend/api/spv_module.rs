//! com/mojang/renderpearl/backend/api/SpvModule.java
//!
//! Java's reflection hands out objects that read and write the module's SPIR-V words in place
//! (`binding(int)` remaps a descriptor's binding). Here the reflection is data with the words'
//! offsets, and the module whose words they are is a parameter: `descriptor.set_binding(module,
//! binding)`. The rest of what Java asks SPIRV-Cross on demand (`decoration(int)`, the types'
//! getters) is asked once, when the module is reflected; only the decorations the game reads
//! are kept (flat and component).

use crate::com::mojang::renderpearl::api::pipeline::shader_type::ShaderType;
use crate::com::mojang::renderpearl::util::shader_compile_exception::ShaderCompileException;

/// A shader compiled to SPIR-V.
pub trait SpvModule: Send + Sync {
    /// The SPIR-V, in the machine's byte order like LWJGL's buffers.
    fn spv(&self) -> &[u8];

    fn shader_type(&self) -> ShaderType;

    fn reflect(&mut self) -> Result<&Reflection, ShaderCompileException>;

    fn get_reflection_info_if_available(&self) -> Option<&Reflection>;

    /// The SPIR-V's word at `offset`, which the reflection's setters write.
    fn word(&self, offset: usize) -> u32;

    fn set_word(&mut self, offset: usize, value: u32);
}

/// `SpvModule.Reflection`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reflection {
    pub inputs: Vec<InterfaceVariable>,
    pub outputs: Vec<InterfaceVariable>,
    /// By resource type, in `SpvUtil.DESCRIPTOR_TYPES`' order.
    pub descriptors_by_type: Vec<(i32, Vec<Descriptor>)>,
    pub push_constants: Vec<PushConstant>,
}

impl Reflection {
    pub fn descriptors_of_type(&self, descriptor_type: i32) -> &[Descriptor] {
        self.descriptors_by_type
            .iter()
            .find(|(resource_type, _)| *resource_type == descriptor_type)
            .map_or(&[], |(_, descriptors)| descriptors)
    }

    /// Every descriptor, type after type.
    pub fn descriptors(&self) -> Vec<Descriptor> {
        self.descriptors_by_type
            .iter()
            .flat_map(|(_, descriptors)| descriptors.iter().cloned())
            .collect()
    }
}

/// `SpvModule.Reflection.InterfaceVariable`: a stage input or output.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct InterfaceVariable {
    pub name: String,
    pub variable_type: Type,
    /// The word holding its `Location` decoration's operand.
    pub location_offset: usize,
    /// `decoration(SpvDecorationFlat)`.
    pub flat: u32,
    /// `decoration(SpvDecorationComponent)`.
    pub component: u32,
}

impl InterfaceVariable {
    pub fn location(&self, module: &dyn SpvModule) -> i32 {
        module.word(self.location_offset) as i32
    }

    pub fn set_location(&self, module: &mut dyn SpvModule, location: i32) {
        module.set_word(self.location_offset, location as u32);
    }

    pub fn has_flat_decoration(&self) -> bool {
        self.flat != 0
    }
}

/// `SpvModule.Reflection.Descriptor`: a uniform buffer, image or sampler.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Descriptor {
    pub name: String,
    pub descriptor_type: Type,
    pub resource_type: i32,
    pub set_offset: usize,
    pub binding_offset: usize,
}

impl Descriptor {
    pub fn descriptor_set_index(&self, module: &dyn SpvModule) -> i32 {
        module.word(self.set_offset) as i32
    }

    pub fn set_descriptor_set_index(&self, module: &mut dyn SpvModule, index: i32) {
        module.set_word(self.set_offset, index as u32);
    }

    pub fn binding(&self, module: &dyn SpvModule) -> i32 {
        module.word(self.binding_offset) as i32
    }

    pub fn set_binding(&self, module: &mut dyn SpvModule, binding: i32) {
        module.set_word(self.binding_offset, binding as u32);
    }
}

/// `SpvModule.Reflection.PushConstant`: a push constant block's declared size.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PushConstant {
    pub size: i32,
}

/// `SpvModule.Reflection.Type`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Type {
    pub base_type: i32,
    /// The image's `SpvDim`, or `Integer.MAX_VALUE` for a type that isn't an image.
    pub dimensions: i32,
    pub vector_size: i32,
    pub array_lengths: Vec<i32>,
}

impl Type {
    pub fn array_dimensions(&self) -> i32 {
        self.array_lengths.len() as i32
    }

    pub fn array_length(&self, dimension_index: usize) -> i32 {
        self.array_lengths[dimension_index]
    }
}
