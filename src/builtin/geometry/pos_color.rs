use bytemuck::Zeroable;
use glam::{Vec3, Vec4};
use wgpu::VertexFormat;
use crate::geometry;

#[derive(Copy, Clone, bytemuck::Pod, Zeroable)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[repr(C)]
pub struct Vertex {
    pub pos: [f32; 3],
    pub color: [f32; 4],
}

impl Vertex {
    pub fn new(pos: Vec3, color: Vec4) -> Self {
        Self {
            pos: pos.to_array(),
            color: color.to_array(),
        }
    }
}

impl geometry::Vertex for Vertex {
    fn write(&self, buffer: &mut Vec<u8>) {
        buffer.extend_from_slice(bytemuck::cast_slice(&[*self]))
    }
}

#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Layout(pub u32);

impl geometry::GeometryLayout for Layout {
    type Vertex = Vertex;
    fn attributes(&self) -> impl Iterator<Item=(u32, VertexFormat)> {
        [
            (self.0, VertexFormat::Float32x3),
            (self.0+1, VertexFormat::Float32x4),
        ].into_iter()
    }
}
