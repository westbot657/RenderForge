mod shader;

use wgpu::{BindingType, ShaderStages, VertexFormat};
use crate::SizedThreadSafe;

pub trait Data: SizedThreadSafe {
    fn write(&self, buffer: &mut Vec<u8>);
}

pub trait InstanceLayout {
    type Data: Data;
    /// Returns an iterator over attributes' layout location and format
    fn attributes(&self) -> impl Iterator<Item = (u32, VertexFormat)>;
    /// There is almost no reason to override this function
    fn span(&self) -> u64 { self.attributes().map(|(_, format)| format.size()).sum() }
}

pub struct UniformEntry {
    pub name: String,
    pub location: u32,
    pub visibility: ShaderStages,
    pub binding_type: BindingType,
}

pub trait UniformsLayout {
    fn entries(&self) -> impl Iterator<Item = UniformEntry>;
}
