pub mod pos;

use wgpu::VertexFormat;
use crate::geometry::{draw, layout};

impl draw::Data for () {
    fn write(&self, _: &mut Vec<u8>) {}
}

impl layout::InstanceLayout for () {
    type Data = ();
    fn attributes(&self) -> impl Iterator<Item=(u32, VertexFormat)> {
        [].into_iter()
    }
}


