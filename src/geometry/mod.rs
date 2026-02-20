use crate::geometry::layout::GeometryLayout;
use crate::geometry::primitive::{Line, LineStripSegment, Point, Primitive, Quad, Tri, TriangleStripSegment};

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
        let size = 4 * self.layout.span() as usize * Prim::VERTEX_COUNT as usize * self.primitives.len();
        buffer.reserve(size);
        for prim in &self.primitives {
            prim.write(buffer)
        }
    }

    pub fn vertex_count(&self) -> u32 {
        self.primitives.len() as u32 * Prim::VERTEX_COUNT as u32
    }

}

impl<Layout> Geometry<Quad<Layout::Vert>, Layout>
where
    Layout: GeometryLayout
{
    pub fn quad(&mut self, a: Layout::Vert, b: Layout::Vert, c: Layout::Vert, d: Layout::Vert) -> &mut Self {
        self.primitives.push(Quad([a, b, c, d]));
        self
    }
    pub fn quads<const N: usize>(&mut self, quads: [Quad<Layout::Vert>; N]) -> &mut Self {
        self.primitives.extend_from_slice(&quads);
        self
    }
}

impl<Layout> Geometry<Tri<Layout::Vert>, Layout>
where
    Layout: GeometryLayout
{
    pub fn tri(&mut self, a: Layout::Vert, b: Layout::Vert, c: Layout::Vert) -> &mut Self {
        self.primitives.push(Tri([a, b, c]));
        self
    }
    pub fn tris<const N: usize>(&mut self, tris: [Tri<Layout::Vert>; N]) -> &mut Self {
        self.primitives.extend_from_slice(&tris);
        self
    }
}

impl<Layout> Geometry<Line<Layout::Vert>, Layout>
where
    Layout: GeometryLayout
{
    pub fn line(&mut self, a: Layout::Vert, b: Layout::Vert) -> &mut Self {
        self.primitives.push(Line([a, b]));
        self
    }
    pub fn lines<const N: usize>(&mut self, lines: [Line<Layout::Vert>; N]) -> &mut Self {
        self.primitives.extend_from_slice(&lines);
        self
    }
}

impl<Layout> Geometry<LineStripSegment<Layout::Vert>, Layout>
where
    Layout: GeometryLayout
{
    pub fn segment(&mut self, segment: Layout::Vert) -> &mut Self {
        self.primitives.push(LineStripSegment(segment));
        self
    }
    pub fn segments(&mut self, segments: impl IntoIterator<Item = Layout::Vert>) -> &mut Self {
        self.primitives.extend(segments.into_iter().map(LineStripSegment));
        self
    }
}

impl<Layout> Geometry<TriangleStripSegment<Layout::Vert>, Layout>
where
    Layout: GeometryLayout
{
    pub fn segment(&mut self, segment: Layout::Vert) -> &mut Self {
        self.primitives.push(TriangleStripSegment(segment));
        self
    }
    pub fn segments(&mut self, segments: impl IntoIterator<Item = Layout::Vert>) -> &mut Self {
        self.primitives.extend(segments.into_iter().map(TriangleStripSegment));
        self
    }
}

impl<Layout> Geometry<Point<Layout::Vert>, Layout>
where
    Layout: GeometryLayout
{
    pub fn point(&mut self, point: Layout::Vert) -> &mut Self {
        self.primitives.push(Point(point));
        self
    }
    pub fn points(&mut self, points: impl IntoIterator<Item = Layout::Vert>) -> &mut Self {
        self.primitives.extend(points.into_iter().map(Point));
        self
    }
}

