use glam::{Vec2, Vec3, Vec4};
use crate::*;

#[derive(Copy, Clone)]
pub struct Vertex {
    pub pos: Vec3,
    pub uv: Vec2,
    pub color: Vec4,
}

#[derive(Copy, Clone, Default)]
pub struct Layout;

impl geometry::GeoLayout for Layout {
    type Vert = Vertex;
    fn span(&self) -> usize {
        9
    }
}

impl geometry::Vertex for Vertex {
    fn write(&self, buffer: &mut Vec<f32>) {
        buffer.extend_from_slice(&[
            self.pos.x,
            self.pos.y,
            self.pos.z,
            
            self.uv.x,
            self.uv.y,
            
            self.color.x,
            self.color.y,
            self.color.z,
            self.color.w,
        ])
    }
}

impl Vertex {
    pub fn new(pos: Vec3, uv: Vec2, color: Vec4) -> Self {
        Self {
            pos,
            uv,
            color,
        }
    }
}

impl<Geo> geometry::Geometry<Geo, Layout>
where
    Geo: geometry::GeoUnit<Vert = Vertex>
{
    pub fn new() -> Self {
        Self::new_with_layout(Layout)
    }
}