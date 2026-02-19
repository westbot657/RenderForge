use crate::geometry::draw;
use crate::geometry::vertex::Vertex;

pub trait GeometryLayout: Sized + Clone + Send + Sync {
    type Vert: Vertex;
    /// Should return the byte alignments of attributes
    fn attributes(&self) -> impl Iterator<Item = (u32, wgpu::VertexFormat)>;

    fn span(&self) -> u64 {
        self.attributes().map(|(_, fmt)| fmt.size()).sum()
    }
}

pub trait InstanceLayout: Sized + Clone + Send + Sync {
    type Data: draw::Data;
    /// Should return the byte alignments of attributes
    fn attributes(&self) -> impl Iterator<Item = (u32, wgpu::VertexFormat)>;

    fn span(&self) -> u64 {
        self.attributes().map(|(_, fmt)| fmt.size()).sum()
    }
}

#[derive(Copy, Clone)]
pub enum UniformKind {
    Buffer,
    StorageBuffer,
    Texture,
    Sampler,
}

#[derive(Clone)]
pub struct UniformEntry {
    pub name: String,
    pub binding: u32,
    pub visibility: wgpu::ShaderStages,
    pub kind: UniformKind
}

pub trait UniformsLayout: Sized + Clone + Sync + Send {
    fn entries(&self) -> impl Iterator<Item = UniformEntry>;
}



