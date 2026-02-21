use wgpu::PrimitiveTopology;
use crate::geometry;
use crate::geometry::Primitive;

#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Quad<Vertex: geometry::Vertex>(pub [Vertex; 4]);

#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Tri<Vertex: geometry::Vertex>(pub [Vertex; 3]);

#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Line<Vertex: geometry::Vertex>(pub [Vertex; 2]);

#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct TriStripSegment<Vertex: geometry::Vertex>(pub Vertex);

#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct LineStripSegment<Vertex: geometry::Vertex>(pub Vertex);

#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Point<Vertex: geometry::Vertex>(pub Vertex);

impl<Vertex: geometry::Vertex> Primitive for Quad<Vertex> {
    type Vertex = Vertex;
    const TOPOLOGY: PrimitiveTopology = PrimitiveTopology::TriangleList;
    const VERTICES: u32 = 6;
    const MIN_PRIMITIVES: u32 = 1;
    fn write(&self, buffer: &mut Vec<u8>) { for i in [0, 1, 2, 0, 2, 3] { unsafe { self.0.get_unchecked(i) }.write(buffer) } }
    fn transform(&mut self, transformer: &impl Fn(&mut Self::Vertex)) { for v in &mut self.0 { transformer(v) } }
}

impl<Vertex: geometry::Vertex> Primitive for Tri<Vertex> {
    type Vertex = Vertex;
    const TOPOLOGY: PrimitiveTopology = PrimitiveTopology::TriangleList;
    const VERTICES: u32 = 3;
    const MIN_PRIMITIVES: u32 = 1;
    fn write(&self, buffer: &mut Vec<u8>) { for v in &self.0 { v.write(buffer) } }
    fn transform(&mut self, transformer: &impl Fn(&mut Self::Vertex)) { for v in &mut self.0 { transformer(v) } }
}

impl<Vertex: geometry::Vertex> Primitive for Line<Vertex> {
    type Vertex = Vertex;
    const TOPOLOGY: PrimitiveTopology = PrimitiveTopology::LineList;
    const VERTICES: u32 = 2;
    const MIN_PRIMITIVES: u32 = 1;
    fn write(&self, buffer: &mut Vec<u8>) { for v in &self.0 { v.write(buffer) } }
    fn transform(&mut self, transformer: &impl Fn(&mut Self::Vertex)) { for v in &mut self.0 { transformer(v) } }
}

impl<Vertex: geometry::Vertex> Primitive for TriStripSegment<Vertex> {
    type Vertex = Vertex;
    const TOPOLOGY: PrimitiveTopology = PrimitiveTopology::TriangleStrip;
    const VERTICES: u32 = 1;
    const MIN_PRIMITIVES: u32 = 3;
    fn write(&self, buffer: &mut Vec<u8>) { self.0.write(buffer) }
    fn transform(&mut self, transformer: &impl Fn(&mut Self::Vertex)) { transformer(&mut self.0) }
}

impl<Vertex: geometry::Vertex> Primitive for LineStripSegment<Vertex> {
    type Vertex = Vertex;
    const TOPOLOGY: PrimitiveTopology = PrimitiveTopology::LineStrip;
    const VERTICES: u32 = 1;
    const MIN_PRIMITIVES: u32 = 2;
    fn write(&self, buffer: &mut Vec<u8>) { self.0.write(buffer) }
    fn transform(&mut self, transformer: &impl Fn(&mut Self::Vertex)) { transformer(&mut self.0) }
}

impl<Vertex: geometry::Vertex> Primitive for Point<Vertex> {
    type Vertex = Vertex;
    const TOPOLOGY: PrimitiveTopology = PrimitiveTopology::PointList;
    const VERTICES: u32 = 1;
    const MIN_PRIMITIVES: u32 = 1;
    fn write(&self, buffer: &mut Vec<u8>) { self.0.write(buffer) }
    fn transform(&mut self, transformer: &impl Fn(&mut Self::Vertex)) { transformer(&mut self.0) }
}

#[macro_export]
macro_rules! quad {
    (
        $vert:path:
        $( $a:expr ),* ;
        $( $b:expr ),* ;
        $( $c:expr ),* ;
        $( $d:expr ),* $(;)?
    ) => {
        Quad([
            $vert( $( $a ),* ),
            $vert( $( $b ),* ),
            $vert( $( $c ),* ),
            $vert( $( $d ),* )
        ])
    };
}

#[macro_export]
macro_rules! tri {
    (
        $vert:path:
        $( $a:expr ),* ;
        $( $b:expr ),* ;
        $( $c:expr ),* $(;)?
    ) => {
        Tri([
            $vert( $( $a ),* ),
            $vert( $( $b ),* ),
            $vert( $( $c ),* )
        ])
    };
}

#[macro_export]
macro_rules! line {
    (
        $vert:path:
        $( $a:expr ),* ;
        $( $b:expr ),* $(;)?
    ) => {
        Line([
            $vert( $( $a ),* ),
            $vert( $( $b ),* )
        ])
    };
}

pub use {quad, tri, line};