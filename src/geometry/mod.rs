use crate::geometry::layout::GeometryLayout;
use crate::geometry::primitive::Primitive;

pub mod primitive;
pub mod vertex;
pub mod layout;
pub mod draw;

pub struct Geometry<Prim, Layout>
where
    Prim: Primitive<Vert = Layout::Vert>,
    Layout: GeometryLayout,
{
    pub primitives: Vec<Prim>,
    pub layout: Layout
}

impl<Prim, Layout> Geometry<Prim, Layout>
where
    Prim: Primitive<Vert = Layout::Vert>,
    Layout: GeometryLayout
{
    pub fn new_with_layout(layout: Layout) -> Self {
        Self {
            primitives: Vec::new(),
            layout,
        }
    }

    pub fn mutate_vertices(&mut self, modifier: impl Fn(&mut Prim::Vert)) {
        for prim in &mut self.primitives { prim.mutate_vertices(&modifier) }
    }

    pub fn write(&self, buffer: &mut Vec<u8>) {
        let size = self.layout.span() * Prim::VERTEX_COUNT * self.primitives.len() as u64;
        buffer.reserve(size as usize);
        for prim in &self.primitives {
            prim.write(buffer)
        }
    }

}





