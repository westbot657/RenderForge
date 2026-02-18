
pub trait GeoLayout: Sized + Clone + Sync + Send {
    type Vert: Vertex;
    fn span(&self) -> usize;
    fn alignments(&self) -> impl Iterator<Item = u32>;
}

pub trait Vertex: Sized + Clone + Sync + Send {
    fn write(&self, buffer: &mut Vec<f32>);
    fn transform(&mut self, transform: glam::Mat4, normal_transform: glam::Mat4) {
        let _ = transform;
        let _ = normal_transform;
    }
}


pub trait GeoUnit: Sized + Clone + Sync + Send {
    const VERTEX_COUNT: usize;
    const MODE: u32;
    const MIN_PRIMITIVE_COUNT: usize;
    type Vert: Vertex;
    fn write(&self, buffer: &mut Vec<f32>);
    fn transform_vertices(&mut self, transformer: &impl Fn(&mut Self::Vert));
}

pub trait BufferProvider {
    fn get_buffer(&self) -> Vec<f32>;
}

#[derive(Clone)]
pub struct Tri<Vert: Vertex> {
    pub(crate) vertices: [Vert; 3],
}

#[derive(Clone)]
pub struct Quad<Vert: Vertex> {
    pub(crate) vertices: [Vert; 4],
}

#[derive(Clone)]
pub struct Lines<Vert: Vertex> {
    pub(crate) vertices: [Vert; 2],
}

#[derive(Clone)]
pub struct LineStripSegment<Vert: Vertex> {
    pub(crate) vertex: Vert
}

#[derive(Clone)]
pub struct LineLoopSegment<Vert: Vertex> {
    pub(crate) vertex: Vert
}

#[derive(Clone)]
pub struct TriangleStripSegment<Vert: Vertex> {
    pub(crate) vertex: Vert
}

#[derive(Clone)]
pub struct TriangleFanSegment<Vert: Vertex> {
    pub(crate) vertex: Vert
}

#[derive(Clone)]
pub struct Point<Vert: Vertex> {
    pub(crate) vertex: Vert
}

impl<Vert: Vertex> Tri<Vert> {
    pub fn new(a: Vert, b: Vert, c: Vert) -> Self {
        Self { vertices: [a, b, c] }
    }
}

impl<Vert: Vertex> Quad<Vert> {
    pub fn new(a: Vert, b: Vert, c: Vert, d: Vert) -> Self {
        Self { vertices: [a, b, c, d] }
    }

    pub fn to_triangles(self) -> [Tri<Vert>; 2] {
        let [a, b, c, d] = self.vertices;
        [
            Tri { vertices: [
                a.clone(),
                b,
                c.clone(),
            ] },
            Tri { vertices: [
                a,
                c,
                d,
            ] }
        ]
    }
}

impl<Vert: Vertex> Lines<Vert> {
    pub fn new(a: Vert, b: Vert) -> Self {
        Self { vertices: [a, b] }
    }
}

impl<Vert: Vertex> LineStripSegment<Vert> {
    pub fn new(vertex: Vert) -> Self {
        Self { vertex }
    }
}

impl<Vert: Vertex> LineLoopSegment<Vert> {
    pub fn new(vertex: Vert) -> Self {
        Self { vertex }
    }
}

impl<Vert: Vertex> TriangleStripSegment<Vert> {
    pub fn new(vertex: Vert) -> Self {
        Self { vertex }
    }
}

impl<Vert: Vertex> TriangleFanSegment<Vert> {
    pub fn new(vertex: Vert) -> Self {
        Self { vertex }
    }
}

impl<Vert: Vertex> Point<Vert> {
    pub fn new(vertex: Vert) -> Self {
        Self { vertex }
    }
}


impl<Vert: Vertex> GeoUnit for Tri<Vert> {
    const VERTEX_COUNT: usize = 3;
    const MODE: u32 = glow::TRIANGLES;
    const MIN_PRIMITIVE_COUNT: usize = 1;
    type Vert = Vert;

    fn write(&self, buffer: &mut Vec<f32>) {
        for v in &self.vertices {
            v.write(buffer)
        }
    }
    fn transform_vertices(&mut self, transformer: &impl Fn(&mut Self::Vert)) {
        for v in &mut self.vertices {
            transformer(v)
        }
    }
}

impl<Vert: Vertex> GeoUnit for Quad<Vert> {
    const VERTEX_COUNT: usize = 6;
    const MODE: u32 = glow::TRIANGLES;
    const MIN_PRIMITIVE_COUNT: usize = 1;
    type Vert = Vert;

    fn write(&self, buffer: &mut Vec<f32>) {
        for i in [0, 1, 2, 0, 2, 3] {
            self.vertices[i].write(buffer)
        }
    }
    fn transform_vertices(&mut self, transformer: &impl Fn(&mut Self::Vert)) {
        for v in &mut self.vertices {
            transformer(v)
        }
    }
}

impl<Vert: Vertex> GeoUnit for Lines<Vert> {
    const VERTEX_COUNT: usize = 2;
    const MODE: u32 = glow::LINES;
    const MIN_PRIMITIVE_COUNT: usize = 1;
    type Vert = Vert;

    fn write(&self, buffer: &mut Vec<f32>) {
        for v in &self.vertices {
            v.write(buffer)
        }
    }
    fn transform_vertices(&mut self, transformer: &impl Fn(&mut Self::Vert)) {
        for v in &mut self.vertices {
            transformer(v)
        }
    }
}

impl<Vert: Vertex> GeoUnit for LineStripSegment<Vert> {
    const VERTEX_COUNT: usize = 1;
    const MODE: u32 = glow::LINE_STRIP;
    const MIN_PRIMITIVE_COUNT: usize = 2;
    type Vert = Vert;

    fn write(&self, buffer: &mut Vec<f32>) {
        self.vertex.write(buffer)
    }
    fn transform_vertices(&mut self, transformer: &impl Fn(&mut Self::Vert)) {
        transformer(&mut self.vertex)
    }
}

impl<Vert: Vertex> GeoUnit for LineLoopSegment<Vert> {
    const VERTEX_COUNT: usize = 1;
    const MODE: u32 = glow::LINE_LOOP;
    const MIN_PRIMITIVE_COUNT: usize = 2;
    type Vert = Vert;

    fn write(&self, buffer: &mut Vec<f32>) {
        self.vertex.write(buffer)
    }
    fn transform_vertices(&mut self, transformer: &impl Fn(&mut Self::Vert)) {
        transformer(&mut self.vertex)
    }
}

impl<Vert: Vertex> GeoUnit for TriangleStripSegment<Vert> {
    const VERTEX_COUNT: usize = 1;
    const MODE: u32 = glow::TRIANGLE_STRIP;
    const MIN_PRIMITIVE_COUNT: usize = 3;
    type Vert = Vert;

    fn write(&self, buffer: &mut Vec<f32>) {
        self.vertex.write(buffer)
    }
    fn transform_vertices(&mut self, transformer: &impl Fn(&mut Self::Vert)) {
        transformer(&mut self.vertex)
    }
}

impl<Vert: Vertex> GeoUnit for TriangleFanSegment<Vert> {
    const VERTEX_COUNT: usize = 1;
    const MODE: u32 = glow::TRIANGLE_FAN;
    const MIN_PRIMITIVE_COUNT: usize = 3;
    type Vert = Vert;

    fn write(&self, buffer: &mut Vec<f32>) {
        self.vertex.write(buffer)
    }
    fn transform_vertices(&mut self, transformer: &impl Fn(&mut Self::Vert)) {
        transformer(&mut self.vertex)
    }
}

impl<Vert: Vertex> GeoUnit for Point<Vert> {
    const VERTEX_COUNT: usize = 1;
    const MODE: u32 = glow::POINTS;
    const MIN_PRIMITIVE_COUNT: usize = 1;
    type Vert = Vert;

    fn write(&self, buffer: &mut Vec<f32>) {
        self.vertex.write(buffer)
    }
    fn transform_vertices(&mut self, transformer: &impl Fn(&mut Self::Vert)) {
        transformer(&mut self.vertex)
    }
}


#[derive(Clone)]
pub struct Geometry<Geo, Layout>
where
    Geo: GeoUnit<Vert = Layout::Vert>,
    Layout: GeoLayout
{
    pub(crate) geometry: Vec<Geo>,
    pub(crate) layout: Layout,
}

impl<Geo, Layout> Default for Geometry<Geo, Layout>
where
    Geo: GeoUnit<Vert = Layout::Vert>,
    Layout: GeoLayout + Default
{
    fn default() -> Self {
        Self::new_with_layout(Layout::default())
    }
}

impl<Geo, Layout> BufferProvider for Geometry<Geo, Layout>
where
    Geo: GeoUnit<Vert = Layout::Vert>,
    Layout: GeoLayout
{
    fn get_buffer(&self) -> Vec<f32> {
        let size = self.layout.span() * Geo::VERTEX_COUNT * self.geometry.len();
        let mut buf = Vec::with_capacity(size);
        self.write_to_buffer(&mut buf);
        buf
    }
}

impl<Geo, Layout> Geometry<Geo, Layout>
where
    Geo: GeoUnit<Vert = Layout::Vert>,
    Layout: GeoLayout
{

    pub(crate) fn new_with_layout(layout: Layout) -> Self {
        Self {
            geometry: Vec::new(),
            layout,
        }
    }

    pub fn transform_vertices(&mut self, transformer: impl Fn(&mut Geo::Vert)) {
        for geo in &mut self.geometry {
            geo.transform_vertices(&transformer)
        }
    }

    pub fn get_raw_geo(&self) -> &[Geo] {
        self.geometry.as_slice()
    }

    pub fn get_raw_geo_mut(&mut self) -> &mut Vec<Geo> {
        &mut self.geometry
    }

    pub fn write_to_buffer(&self, buffer: &mut Vec<f32>) {
        let size = self.layout.span() * Geo::VERTEX_COUNT * self.geometry.len();
        buffer.reserve(size);
        for unit in &self.geometry {
            unit.write(buffer);
        }
    }

}

impl<Layout> Geometry<Tri<Layout::Vert>, Layout>
where
    Layout: GeoLayout,
{
    pub fn add_tri(&mut self, tri: Tri<Layout::Vert>) {
        self.geometry.push(tri)
    }
    pub fn add_tris(&mut self, tris: &[Tri<Layout::Vert>]) {
        self.geometry.extend_from_slice(tris)
    }
}

impl<Layout> Geometry<Quad<Layout::Vert>, Layout>
where
    Layout: GeoLayout,
{
    pub fn add_quad(&mut self, quad: Quad<Layout::Vert>) {
        self.geometry.push(quad)
    }
    pub fn add_quads(&mut self, quads: &[Quad<Layout::Vert>]) {
        self.geometry.extend_from_slice(quads)
    }
}


