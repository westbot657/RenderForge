use crate::types::AssetID;

#[derive(Debug, Copy, Clone)]
pub enum UniformType {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Bool,
    Vec2,
    Vec3,
    Vec4,
    IVec2,
    IVec3,
    IVec4,
    UVec2,
    UVec3,
    UVec4,
    Mat4,
    Mat3,
    Sampler2D,
}

#[derive(Debug, Copy, Clone)]
pub enum AttrDataType {
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Bool,
    Vec2,
    Vec3,
    Vec4,
    IVec2,
    IVec3,
    IVec4,
    UVec2,
    UVec3,
    UVec4,
    Mat4,
    Mat3,
}

#[derive(Debug, Clone)]
pub struct UniformDef {
    name: String,
    data_type: UniformType,
}

#[derive(Debug, Clone)]
pub struct AttrDef {
    name: String,
    data_type: AttrDataType,
}

#[derive(Debug, Clone)]
pub struct MaterialDef {
    pub id: AssetID,
    pub uniforms: Vec<UniformDef>,
    pub vertex_attrs: Vec<AttrDef>,
    pub triangle_attrs: Vec<AttrDef>,
}







