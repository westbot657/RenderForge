use bytemuck::Zeroable;
use glam::Mat4;
use wgpu::VertexFormat;
use crate::geometry::{draw, layout};

#[derive(Copy, Clone, bytemuck::Pod, Zeroable)]
#[repr(transparent)]
pub struct Data(pub Mat4);

#[derive(Copy, Clone)]
pub struct Layout;

impl draw::Data for Data {
    fn write(&self, buffer: &mut Vec<u8>) {
        buffer.extend_from_slice(bytemuck::cast_slice(&[*self]))
    }
}


impl layout::InstanceLayout for Layout {
    type Data = Data;
    fn attributes(&self) -> impl Iterator<Item=(u32, VertexFormat)> {
        [
            (0,  VertexFormat::Float32x4),
            (1,  VertexFormat::Float32x4),
            (2,  VertexFormat::Float32x4),
            (3, VertexFormat::Float32x4),
        ].into_iter()
    }
}


