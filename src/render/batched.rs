use glam::Mat4;
use crate::geometry::*;
use crate::render::buffer::RenderableBuffer;

pub struct BatchedMesh<Geo, Layout>
where
    Geo: GeoUnit<Vert = Layout::Vert>,
    Layout: GeoLayout
{
    base_geometry: Geometry<Geo, Layout>,
    positions: Vec<Mat4>
}

impl<Geo, Layout> BatchedMesh<Geo, Layout>
where
    Geo: GeoUnit<Vert = Layout::Vert>,
    Layout: GeoLayout
{
    pub fn new(base_geometry: Geometry<Geo, Layout>) -> Self {
        Self {
            base_geometry,
            positions: Vec::new(),
        }
    }

    pub fn add_position(&mut self, pos: Mat4) {
        self.positions.push(pos)
    }

    pub fn clear_positions(&mut self) {
        self.positions.clear()
    }
}

impl<Geo, Layout> BufferProvider for BatchedMesh<Geo, Layout>
where
    Geo: GeoUnit<Vert = Layout::Vert>,
    Layout: GeoLayout
{
    fn get_buffer(&self) -> Vec<f32> {
        let size = self.base_geometry.layout.span()
            * Geo::VERTEX_COUNT
            * self.base_geometry.geometry.len()
            * self.positions.len();
        if size == 0 {
            return Vec::new()
        }
        let mut buffer = Vec::with_capacity(size);

        for mat in &self.positions {
            let mut mesh = self.base_geometry.clone();
            let normal_mat = mat.inverse().transpose();
            mesh.transform_vertices(|vert| {
                vert.transform(*mat, normal_mat)
            });
            mesh.write_to_buffer(&mut buffer);
        }

        buffer
    }

}

impl<Geo, Layout> RenderableBuffer for BatchedMesh<Geo, Layout>
where
    Geo: GeoUnit<Vert = Layout::Vert>,
    Layout: GeoLayout
{
    type Geo = Geo;
    type Layout = Layout;
    fn layout(&self) -> &Self::Layout {
        &self.base_geometry.layout
    }
}
