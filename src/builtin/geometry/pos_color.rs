use bytemuck::Zeroable;
use glam::{Vec3, Vec4};
use wgpu::VertexFormat;
use crate::geometry::layout::GeometryLayout;
use crate::geometry::vertex;

#[derive(Copy, Clone)]
pub struct Vertex(pub Vec3, pub Vec4);

#[derive(Copy, Clone)]
pub struct Layout;


impl vertex::Vertex for Vertex {
    fn write(&self, buffer: &mut Vec<u8>) {
        let data = [
            self.0.x,
            self.0.y,
            self.0.z,
            
            self.1.x,
            self.1.y,
            self.1.z,
            self.1.w,
        ];
        buffer.extend_from_slice(bytemuck::cast_slice(&data))
    }
}


impl GeometryLayout for Layout {
    type Vert = Vertex;

    fn attributes(&self) -> impl Iterator<Item=(u32, VertexFormat)> {
        [
            (0, VertexFormat::Float32x3),
            (1, VertexFormat::Float32x4),
        ].into_iter()
    }
}

