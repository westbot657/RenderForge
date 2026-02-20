use wgpu::VertexFormat;
use crate::SizedThreadSafe;

pub mod primitive;

pub trait Vertex: SizedThreadSafe {
    fn write(&self, buffer: &mut Vec<u8>);
}

pub trait Primitive: SizedThreadSafe {
    type Vertex: Vertex;
    const TOPOLOGY: wgpu::PrimitiveTopology;
    const VERTICES: u32;
    const MIN_PRIMITIVES: u32;
    fn write(&self, buffer: &mut Vec<u8>);
    fn transform(&mut self, transformer: &impl Fn(&mut Self::Vertex));
}

pub trait GeometryLayout: SizedThreadSafe + Clone {
    type Vertex: Vertex;
    /// Returns an iterator over attributes' layout location and format
    fn attributes(&self) -> impl Iterator<Item = (u32, VertexFormat)>;
    /// There is almost no reason to override this function
    fn span(&self) -> u64 { self.attributes().map(|(_, format)| format.size()).sum() }
}

impl Vertex for () {
    fn write(&self, _: &mut Vec<u8>) {}
}

impl GeometryLayout for () {
    type Vertex = ();
    fn attributes(&self) -> impl Iterator<Item=(u32, VertexFormat)> { [].into_iter() }
}

#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Geometry<Layout, Primitive>
where
    Layout: GeometryLayout,
    Primitive: crate::geometry::Primitive<Vertex = Layout::Vertex>,
{
    pub primitives: Vec<Primitive>,
    pub layout: Layout,
}

impl<Layout, Primitive> Geometry<Layout, Primitive>
where
    Layout: GeometryLayout,
    Primitive: crate::geometry::Primitive<Vertex = Layout::Vertex>,
{
    pub fn new(layout: Layout) -> Self {
        Self {
            primitives: Vec::new(),
            layout,
        }
    }
}
