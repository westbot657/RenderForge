
pub trait GeoLayout: Sized + Clone {
    type Vert: Vertex;
    fn span(&self) -> usize;
}

pub trait Vertex: Sized + Clone {
    fn write(&self, buffer: &mut Vec<f32>);
    fn transform(&mut self, transform: glam::Mat4, normal_transform: glam::Mat4) {
        let _ = transform;
        let _ = normal_transform;
    }
}

pub trait GlData {
    fn size(&self) -> usize;
    fn write(&self, buffer: &mut Vec<f32>);
}

pub trait GeoUnit: Sized + Clone {
    const VERTEX_COUNT: usize;
    type Vert: Vertex;
    fn write(&self, buffer: &mut Vec<f32>);
    fn transform_vertices(&mut self, transformer: &impl Fn(&mut Self::Vert));
}

pub trait BufferProvider {
    fn get_buffer(&self) -> Vec<f32>;
}

#[derive(Clone)]
pub struct Tri<Vert>
where
    Vert: Vertex
{
    pub(crate) vertices: [Vert; 3],
}

#[derive(Clone)]
pub struct Quad<Vert>
where
    Vert: Vertex
{
    pub(crate) vertices: [Vert; 4],
}

impl<Vert> Tri<Vert>
where
    Vert: Vertex
{

}

impl<Vert> Quad<Vert>
where
    Vert: Vertex
{
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

impl<Vert> GeoUnit for Tri<Vert>
where
    Vert: Vertex
{
    const VERTEX_COUNT: usize = 3;

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

impl<Vert> GeoUnit for Quad<Vert>
where
    Vert: Vertex
{
    const VERTEX_COUNT: usize = 4;

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


#[derive(Clone)]
pub struct Geometry<Geo, Layout>
where
    Geo: GeoUnit,
    Layout: GeoLayout
{
    pub(crate) geometry: Vec<Geo>,
    pub(crate) layout: Layout,
}

impl<Geo, Layout> Default for Geometry<Geo, Layout>
where
    Geo: GeoUnit,
    Layout: GeoLayout + Default
{
    fn default() -> Self {
        Self::new_with_layout(Layout::default())
    }
}

impl<Geo, Layout> BufferProvider for Geometry<Geo, Layout>
where
    Geo: GeoUnit,
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
    Geo: GeoUnit,
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

impl<Layout, Vert> Geometry<Tri<Vert>, Layout>
where
    Vert: Vertex,
    Layout: GeoLayout,
{
    pub fn add_tri(&mut self, tri: Tri<Vert>) {
        self.geometry.push(tri)
    }
    pub fn add_tris(&mut self, tris: &[Tri<Vert>]) {
        self.geometry.extend_from_slice(tris)
    }
}

impl<Layout, Vert> Geometry<Quad<Vert>, Layout>
where
    Vert: Vertex,
    Layout: GeoLayout,
{
    pub fn add_quad(&mut self, quad: Quad<Vert>) {
        self.geometry.push(quad)
    }
    pub fn add_quads(&mut self, quads: &[Quad<Vert>]) {
        self.geometry.extend_from_slice(quads)
    }
}




impl GlData for f32 {
    fn size(&self) -> usize { 1 }
    fn write(&self, buffer: &mut Vec<f32>){
        buffer.push(*self)
    }
}
impl GlData for [f32; 2] {
    fn size(&self) -> usize { 2 }
    fn write(&self, buffer: &mut Vec<f32>) {
        buffer.extend_from_slice(self)
    }
}
impl GlData for [f32; 3] {
    fn size(&self) -> usize { 3 }
    fn write(&self, buffer: &mut Vec<f32>) {
        buffer.extend_from_slice(self)
    }
}
impl GlData for [f32; 4] {
    fn size(&self) -> usize { 4 }
    fn write(&self, buffer: &mut Vec<f32>) {
        buffer.extend_from_slice(self)
    }
}
impl GlData for [f32; 16] {
    fn size(&self) -> usize { 16 }
    fn write(&self, buffer: &mut Vec<f32>) {
        buffer.extend_from_slice(self)
    }
}
impl GlData for glam::Vec2 {
    fn size(&self) -> usize { 2 }
    fn write(&self, buffer: &mut Vec<f32>) {
        buffer.extend_from_slice(self.as_ref())
    }
}
impl GlData for glam::Vec3 {
    fn size(&self) -> usize { 3 }
    fn write(&self, buffer: &mut Vec<f32>) {
        buffer.extend_from_slice(self.as_ref())
    }
}
impl GlData for glam::Vec4 {
    fn size(&self) -> usize { 4 }
    fn write(&self, buffer: &mut Vec<f32>) {
        buffer.extend_from_slice(self.as_ref())
    }
}
impl GlData for glam::Quat {
    fn size(&self) -> usize { 4 }
    fn write(&self, buffer: &mut Vec<f32>) {
        buffer.extend_from_slice(self.as_ref())
    }
}
impl GlData for glam::Mat4 {
    fn size(&self) -> usize { 16 }
    fn write(&self, buffer: &mut Vec<f32>) {
        buffer.extend_from_slice(self.as_ref())
    }
}

impl GlData for u32 {
    fn size(&self) -> usize { 1 }
    fn write(&self, buffer: &mut Vec<f32>) {
        buffer.push(f32::from_bits(*self))
    }
}
impl GlData for i32 {
    fn size(&self) -> usize { 1 }
    fn write(&self, buffer: &mut Vec<f32>) {
        buffer.push(f32::from_bits(*self as u32))
    }
}
impl GlData for bool {
    fn size(&self) -> usize { 1 }
    fn write(&self, buffer: &mut Vec<f32>) {
        buffer.push(f32::from_bits(*self as u32))
    }
}

