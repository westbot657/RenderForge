use glam::{IVec2, IVec3, IVec4, Mat3, Mat4, UVec2, UVec3, UVec4, Vec2, Vec3, Vec4};

use crate::types::AssetID;

#[derive(Debug, Clone)]
pub enum UniformValue {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    Bool(bool),
    Vec2(Vec2),
    Vec3(Vec3),
    Vec4(Vec4),
    IVec2(IVec2),
    IVec3(IVec3),
    IVec4(IVec4),
    UVec2(UVec2),
    UVec3(UVec3),
    UVec4(UVec4),
    Mat4(Mat4),
    Mat3(Mat3),
    Sampler2D(glow::Sampler),
}

#[derive(Debug, Clone)]
pub enum AttrDataValue {
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    Bool(bool),
    Vec2(Vec2),
    Vec3(Vec3),
    Vec4(Vec4),
    IVec2(IVec2),
    IVec3(IVec3),
    IVec4(IVec4),
    UVec2(UVec2),
    UVec3(UVec3),
    UVec4(UVec4),
    Mat4(Mat4),
    Mat3(Mat3),
}

#[derive(Debug, Clone)]
pub struct AttrData {
    name: String,
    value: AttrDataValue,
}

#[derive(Debug, Clone)]
pub struct Uniform {
    pub name: String,
    pub value: UniformValue,
}

#[derive(Debug, Clone)]
pub struct Vertex {
    pub attrs: Vec<AttrData>,
}

#[derive(Debug, Clone)]
pub struct Triangle {
    pub attrs: Vec<AttrData>,
    pub vertices: [Vertex; 3],
}

#[derive(Debug, Clone)]
pub struct Mesh {
    pub id: AssetID,
    pub material: AssetID,
    pub uniforms: Vec<Uniform>,
    pub triangles: Vec<Triangle>,
}


