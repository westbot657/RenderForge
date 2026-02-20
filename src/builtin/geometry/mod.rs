use wgpu::VertexFormat;
use crate::geometry::{layout, vertex};

pub mod pos;
pub mod pos_color;

impl vertex::Vertex for () {
    fn write(&self, _: &mut Vec<u8>) {}
}

impl layout::GeometryLayout for () {
    type Vert = ();
    fn attributes(&self) -> impl Iterator<Item=(u32, VertexFormat)> {
        [].into_iter()
    }
}
