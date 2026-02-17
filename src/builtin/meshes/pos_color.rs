use glam::{Mat4, Vec3, Vec4};
use crate::*;

#[derive(Copy, Clone)]
pub struct Vertex {
    pub pos: Vec3,
    pub color: Vec4,
}

#[derive(Copy, Clone, Default)]
pub struct Layout;

impl geometry::GeoLayout for Layout {
    type Vert = Vertex;
    fn span(&self) -> usize {
        3 + 4
    }
    fn alignments(&self) -> impl Iterator<Item=u32> {
        [3, 4].into_iter()
    }
}

impl geometry::Vertex for Vertex {
    fn write(&self, buffer: &mut Vec<f32>) {
        buffer.extend_from_slice(&[
            self.pos.x,
            self.pos.y,
            self.pos.z,

            self.color.x,
            self.color.y,
            self.color.z,
            self.color.w,
        ])
    }
    fn transform(&mut self, transform: Mat4, _: Mat4) {
        self.pos = transform.transform_point3(self.pos);
    }
}

impl Vertex {
    pub fn new(pos: Vec3, color: Vec4) -> Self {
        Self {
            pos,
            color
        }
    }
}

impl<Geo> geometry::Geometry<Geo, Layout>
where
    Geo: geometry::GeoUnit<Vert = Vertex>,
{
    pub fn new() -> Self {
        Self::new_with_layout(Layout)
    }
}