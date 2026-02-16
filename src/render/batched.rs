use glam::Mat4;
use crate::geometry::*;

pub struct BatchedMesh<Geo, Layout>
where
    Geo: GeoUnit,
    Geo::Vert: PositionableVertex,
    Layout: GeoLayout
{
    base_geometry: Geometry<Geo, Layout>,
    positions: Vec<Mat4>
}

impl<Geo, Layout> BatchedMesh<Geo, Layout>
where
    Geo: GeoUnit,
    Geo::Vert: PositionableVertex,
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
    Geo: GeoUnit,
    Geo::Vert: PositionableVertex,
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

        for pos in &self.positions {
            let mut mesh = self.base_geometry.clone();
            mesh.transform_vertices(|vert| {
                let p = vert.get_pos_mut();
                let out = pos * glam::Vec3::from_array(*p).extend(1.);
                p[0] = out.x;
                p[1] = out.y;
                p[2] = out.z;
            });
            mesh.write_to_buffer(&mut buffer);
        }

        buffer
    }

}
