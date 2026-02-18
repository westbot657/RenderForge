use crate::render::*;
use glam::Mat4;
use crate::geometry::{GeoLayout, GeoUnit};

#[derive(Copy, Clone)]
pub struct Data {
    pos: Mat4
}

impl instanced::InstanceData for Data {
    fn write(&self, buffer: &mut Vec<f32>) {
        self.pos.write(buffer)
    }
}

#[derive(Copy, Clone, Default)]
pub struct Layout;

impl instanced::InstanceLayout for Layout {
    type Data = Data;
    fn span(&self) -> usize {
        16
    }
    fn alignments(&self) -> impl Iterator<Item = u32> {
        [4, 4, 4, 4].into_iter()
    }
}

impl Data {
    pub fn new(pos: Mat4) -> Self {
        Self { pos }
    }
}

impl From<Mat4> for Data {
    fn from(value: Mat4) -> Self {
        Self::new(value)
    }
}

impl<Geo, GLayout> instanced::InstancedMesh<Geo, GLayout, Layout>
where
    Geo: GeoUnit<Vert = GLayout::Vert>,
    GLayout: GeoLayout
{
    pub fn add_pos_data(&mut self, pos: Mat4) {
        self.add_data(pos.into())
    }
}
