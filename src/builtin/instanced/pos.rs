use bytemuck::Zeroable;
use glam::Mat4;
use wgpu::VertexFormat;
use crate::render;

#[derive(Copy, Clone, bytemuck::Pod, Zeroable)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[repr(C)]
pub struct Data(pub Mat4);

impl render::Data for Data {
    fn write(&self, buffer: &mut Vec<u8>) {
        buffer.extend_from_slice(bytemuck::cast_slice(&[*self]))
    }
}


#[derive(Copy, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Layout(pub u32);

impl render::InstanceLayout for Layout {
    type Data = Data;
    fn attributes(&self) -> impl Iterator<Item=(u32, VertexFormat)> {
        [
            (self.0, VertexFormat::Float32x4),
            (self.0+1, VertexFormat::Float32x4),
            (self.0+2, VertexFormat::Float32x4),
            (self.0+3, VertexFormat::Float32x4),
        ].into_iter()
    }
}