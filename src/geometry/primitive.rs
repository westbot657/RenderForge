use wgpu::PrimitiveTopology;
use crate::geometry;
use crate::geometry::vertex::Vertex;

pub trait Primitive: Sized + Clone + Send + Sync {
    type Vert: Vertex;
    const TOPOLOGY: PrimitiveTopology;
    const VERTEX_COUNT: u32;
    const MIN_PRIMITIVE_COUNT: usize;

    fn mutate_vertices(&mut self, modifier: &impl Fn(&mut Self::Vert));
    fn write(&self, buffer: &mut Vec<u8>);
}


#[derive(Clone)]
pub struct Quad<Vert: Vertex>(pub [Vert; 4]);

#[derive(Clone)]
pub struct Tri<Vert: Vertex>(pub [Vert; 3]);

#[derive(Clone)]
pub struct Line<Vert: Vertex>(pub [Vert; 2]);

#[derive(Clone)]
pub struct LineStripSegment<Vert: Vertex>(pub Vert);

#[derive(Clone)]
pub struct TriangleStripSegment<Vert: Vertex>(pub Vert);

#[derive(Clone)]
pub struct Point<Vert: Vertex>(pub Vert);


impl<Vert: Vertex> Primitive for Quad<Vert> {
    type Vert = Vert;
    const TOPOLOGY: PrimitiveTopology = PrimitiveTopology::TriangleList;
    const VERTEX_COUNT: u32 = 6;
    const MIN_PRIMITIVE_COUNT: usize = 1;

    fn mutate_vertices(&mut self, modifier: &impl Fn(&mut Self::Vert)) {
        for v in &mut self.0 { modifier(v) }
    }
    fn write(&self, buffer: &mut Vec<u8>) {
        for i in [0, 1, 2, 0, 2, 3] { self.0[i].write(buffer) }
    }
}

impl<Vert: Vertex> Primitive for Tri<Vert> {
    type Vert = Vert;
    const TOPOLOGY: PrimitiveTopology = PrimitiveTopology::TriangleList;
    const VERTEX_COUNT: u32 = 3;
    const MIN_PRIMITIVE_COUNT: usize = 1;

    fn mutate_vertices(&mut self, modifier: &impl Fn(&mut Self::Vert)) {
        for v in &mut self.0 { modifier(v) }
    }
    fn write(&self, buffer: &mut Vec<u8>) {
        for v in &self.0 { v.write(buffer) }
    }
}

impl<Vert: Vertex> Primitive for Line<Vert> {
    type Vert = Vert;
    const TOPOLOGY: PrimitiveTopology = PrimitiveTopology::LineList;
    const VERTEX_COUNT: u32 = 2;
    const MIN_PRIMITIVE_COUNT: usize = 1;

    fn mutate_vertices(&mut self, modifier: &impl Fn(&mut Self::Vert)) {
        for v in &mut self.0 { modifier(v) }
    }
    fn write(&self, buffer: &mut Vec<u8>) {
        for v in &self.0 { v.write(buffer) }
    }
}

impl<Vert: Vertex> Primitive for LineStripSegment<Vert> {
    type Vert = Vert;
    const TOPOLOGY: PrimitiveTopology = PrimitiveTopology::LineStrip;
    const VERTEX_COUNT: u32 = 1;
    const MIN_PRIMITIVE_COUNT: usize = 2;

    fn mutate_vertices(&mut self, modifier: &impl Fn(&mut Self::Vert)) {
        modifier(&mut self.0)
    }
    fn write(&self, buffer: &mut Vec<u8>) {
        self.0.write(buffer)
    }
}

impl<Vert: Vertex> Primitive for TriangleStripSegment<Vert> {
    type Vert = Vert;
    const TOPOLOGY: PrimitiveTopology = PrimitiveTopology::TriangleStrip;
    const VERTEX_COUNT: u32 = 1;
    const MIN_PRIMITIVE_COUNT: usize = 3;

    fn mutate_vertices(&mut self, modifier: &impl Fn(&mut Self::Vert)) {
        modifier(&mut self.0)
    }
    fn write(&self, buffer: &mut Vec<u8>) {
        self.0.write(buffer)
    }
}

impl<Vert: Vertex> Primitive for Point<Vert> {
    type Vert = Vert;
    const TOPOLOGY: PrimitiveTopology = PrimitiveTopology::PointList;
    const VERTEX_COUNT: u32 = 1;
    const MIN_PRIMITIVE_COUNT: usize = 1;

    fn mutate_vertices(&mut self, modifier: &impl Fn(&mut Self::Vert)) {
        modifier(&mut self.0)
    }
    fn write(&self, buffer: &mut Vec<u8>) {
        self.0.write(buffer)
    }
}

#[macro_export]
macro_rules! quad {
    (
        $vert:path:
        $( $a:expr ),*;
        $( $b:expr ),*;
        $( $c:expr ),*;
        $( $d:expr ),* $(;)?
    ) => {
        Quad([
            $vert( $( $a ),* ),
            $vert( $( $b ),* ),
            $vert( $( $c ),* ),
            $vert( $( $d ),* ),
        ])
    };
}

#[macro_export]
macro_rules! tri {
    (
        $vert:path:
        $( $a:expr ),*;
        $( $b:expr ),*;
        $( $c:expr ),*$(;)?
    ) => {
        Tri([
            $vert( $( $a ),* ),
            $vert( $( $b ),* ),
            $vert( $( $c ),* ),
        ])
    };
}
#[macro_export]
macro_rules! line {
    (
        $vert:path:
        $( $a:expr ),*;
        $( $b:expr ),*$(;)?
    ) => {
        Line([
            $vert( $( $a ),* ),
            $vert( $( $b ),* ),
        ])
    };
}


pub use {quad, tri, line};

