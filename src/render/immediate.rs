use crate::geometry::*;

pub struct ImmediateBuffer<Geo, Layout>
where
    Geo: GeoUnit,
    Layout: GeoLayout
{
    inner: Geometry<Geo, Layout>
}

impl<Geo, Layout> ImmediateBuffer<Geo, Layout>
where
    Geo: GeoUnit,
    Layout: GeoLayout + Default
{
    pub fn new() -> Self {
        Self::new_with_layout(Layout::default())
    }
}

impl<Geo, Layout> ImmediateBuffer<Geo, Layout>
where
    Geo: GeoUnit,
    Layout: GeoLayout
{
    pub fn new_with_layout(layout: Layout) -> Self {
        Self {
            inner: Geometry::new_with_layout(layout)
        }
    }

    pub fn clear(&mut self) {
        self.inner.geometry.clear();
    }

}

impl<Geo, Layout> BufferProvider for ImmediateBuffer<Geo, Layout>
where
    Geo: GeoUnit,
    Layout: GeoLayout
{
    fn get_buffer(&self) -> Vec<f32> {
        self.inner.get_buffer()
    }
}


impl<Layout, Vert> ImmediateBuffer<Quad<Vert>, Layout>
where
    Layout: GeoLayout,
    Vert: Vertex
{
    pub fn add_quad(&mut self, quad: Quad<Vert>) {
        self.inner.add_quad(quad)
    }
}

impl<Layout, Vert> ImmediateBuffer<Tri<Vert>, Layout>
where
    Layout: GeoLayout,
    Vert: Vertex
{
    pub fn add_tri(&mut self, tri: Tri<Vert>) {
        self.inner.add_tri(tri)
    }
}

