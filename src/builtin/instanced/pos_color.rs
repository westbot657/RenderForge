use std::borrow::Borrow;
use glam::{Mat4, Vec4};
use crate::geometry::{GeoLayout, GeoUnit};
use crate::render::*;

#[derive(Copy, Clone)]
pub struct Data {
    pos: Mat4,
    color: Vec4,
}

#[derive(Copy, Clone, Default)]
pub struct Layout;

impl instanced::InstanceData for Data {
    fn write(&self, buffer: &mut Vec<f32>) {
        self.pos.write(buffer);
        self.color.write(buffer);
    }
}

impl instanced::InstanceLayout for Layout {
    type Data = Data;
    fn span(&self) -> usize {
        16 + 4
    }
    fn alignments(&self) -> impl Iterator<Item = u32> {
        [16, 4].into_iter()
    }
}

impl Data {
    pub fn new(pos: Mat4, color: Vec4) -> Self {
        Self { pos, color }
    }
}

impl<Geo, GLayout> instanced::InstancedMesh<Geo, GLayout, Layout>
where
    Geo: GeoUnit<Vert = GLayout::Vert>,
    GLayout: GeoLayout
{
    pub fn add_pos_col_data(&mut self, pos: Mat4, col: Vec4) {
        self.data.push(Data::new(pos, col))
    }
}

