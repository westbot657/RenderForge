use bytemuck::Zeroable;
use glam::Vec3;
use wgpu::VertexFormat;
use crate::geometry::layout::GeometryLayout;
use crate::geometry::vertex;

#[derive(Copy, Clone, bytemuck::Pod, Zeroable)]
#[repr(transparent)]
pub struct Vertex(pub Vec3);

#[derive(Copy, Clone)]
pub struct Layout;


impl vertex::Vertex for Vertex {
    fn write(&self, buffer: &mut Vec<u8>) {
        buffer.extend_from_slice(bytemuck::cast_slice(&[*self]))
    }
}


impl GeometryLayout for Layout {
    type Vert = Vertex;

    fn attributes(&self) -> impl Iterator<Item=(u32, VertexFormat)> {
        [(0, VertexFormat::Float32x3)].into_iter()
    }
}

