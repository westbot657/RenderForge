use crate::geometry::*;
use crate::render::shader::{NullInstanceLayout, Shader};

pub struct ImmediateBuffer<Geo, Layout>
where
    Geo: GeoUnit<Vert = Layout::Vert>,
    Layout: GeoLayout
{
    inner: Geometry<Geo, Layout>
}

impl<Geo, Layout> ImmediateBuffer<Geo, Layout>
where
    Geo: GeoUnit<Vert = Layout::Vert>,
    Layout: GeoLayout + Default
{
    pub fn new() -> Self {
        Self::new_with_layout(Layout::default())
    }
}

impl<Geo, Layout> ImmediateBuffer<Geo, Layout>
where
    Geo: GeoUnit<Vert = Layout::Vert>,
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
    Geo: GeoUnit<Vert = Layout::Vert>,
    Layout: GeoLayout
{
    fn get_buffer(&self) -> Vec<f32> {
        self.inner.get_buffer()
    }
}


impl<Layout> ImmediateBuffer<Quad<Layout::Vert>, Layout>
where
    Layout: GeoLayout
{
    pub fn add_quad(&mut self, quad: Quad<Layout::Vert>) {
        self.inner.add_quad(quad)
    }
}

impl<Layout> ImmediateBuffer<Tri<Layout::Vert>, Layout>
where
    Layout: GeoLayout
{
    pub fn add_tri(&mut self, tri: Tri<Layout::Vert>) {
        self.inner.add_tri(tri)
    }
}

impl<Layout> Shader<Layout, NullInstanceLayout>
where
    Layout: GeoLayout
{
    pub fn get_immediate_renderer<Geo>(&self) -> ImmediateBuffer<Geo, Layout>
    where
        Geo: GeoUnit<Vert = Layout::Vert>
    {
        ImmediateBuffer::new_with_layout(self.layout.clone())
    }
}

