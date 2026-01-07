use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::rc::{Rc, Weak};
use glam::{
    Quat,
    Mat2, Mat3, Mat4,
    Vec2, Vec3, Vec4,
    IVec2, IVec3, IVec4,
    UVec2, UVec3, UVec4,
};
use thiserror::Error;
use anyhow::{anyhow, Result};
use glow::UniformLocation;
use indexmap::IndexMap;
use indexmap::map::Iter;

#[derive(Debug, Clone)]
pub struct Shared<T>(Rc<RefCell<T>>);

#[derive(Debug, Clone)]
pub struct WeakShared<T>(Weak<RefCell<T>>);

impl<T> Shared<T> {
    pub fn new(t: T) -> Shared<T> {
        Shared(Rc::new(RefCell::new(t)))
    }
}

impl<T> Deref for Shared<T> {
    type Target = Rc<RefCell<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for Shared<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<Rc<RefCell<T>>> for Shared<T> {
    fn from(value: Rc<RefCell<T>>) -> Self {
        Self(value)
    }
}

impl<T> From<T> for Shared<T> {
    fn from(value: T) -> Self {
        Self(Rc::new(RefCell::new(value)))
    }
}

impl<T> Deref for WeakShared<T> {
    type Target = Weak<RefCell<T>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T> DerefMut for WeakShared<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<T> From<Weak<RefCell<T>>> for WeakShared<T> {
    fn from(value: Weak<RefCell<T>>) -> Self {
        Self(value)
    }
}

impl<T> From<&Rc<RefCell<T>>> for WeakShared<T> {
    fn from(value: &Rc<RefCell<T>>) -> Self {
        Self(Rc::downgrade(value))
    }
}

#[derive(Debug, Clone)]
pub enum MaybeDefaultOption<T> {
    NoneDefault(Option<T>),
    SomeDefault(Option<T>, T)
}

impl<T: Clone> MaybeDefaultOption<T> {
    pub fn get(&self) -> Result<T> {
        Ok(self.get_ref()?.clone())
    }
}

impl<T> MaybeDefaultOption<T> {
    pub fn get_ref(&self) -> Result<&T> {
        match self {
            MaybeDefaultOption::NoneDefault(Some(v)) => Ok(v),
            MaybeDefaultOption::SomeDefault(Some(v), _) => Ok(v),
            MaybeDefaultOption::SomeDefault(None, v) => Ok(v),
            _ => Err(anyhow!("No value associated with option"))
        }
    }
}

#[derive(Error, Debug)]
pub enum DataTypeError {
    #[error("Mismatched data types: '{0}' and '{1}'")]
    MismatchedTypes(MaterialDataType, MaterialDataType)
}

macro_rules! typed_data {
    ( $( $variant:tt ( $tp:ty ): $x:tt => $conv:expr );+ $(;)? ) => {
        #[derive(PartialEq, Eq, Debug, Clone, Copy)]
        pub enum MaterialDataType {
            $( $variant ),+,
        }

        #[derive(Debug, Clone)]
        pub enum MaterialData {
            $( $variant($tp) ),+,
        }

        #[derive(Debug, Clone)]
        pub enum TypedMaterialData {
            $( $variant(MaybeDefaultOption<$tp>) ),+,
        }

        impl std::fmt::Display for MaterialDataType {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
                match self {
                    $( MaterialDataType::$variant => write!(f, stringify!($tp)) ),+,
                }
            }
        }

        impl MaterialDataType {
            pub fn as_typed_material(&self) -> Result<TypedMaterialData> {
                self.as_typed_material_with_default(None)
            }

            pub fn as_typed_material_with_default(&self, default: Option<MaterialData>) -> Result<TypedMaterialData> {
                if let Some(opt) = default {
                    let ty = opt.get_type();
                    if *self != ty {
                        return Err(DataTypeError::MismatchedTypes(self.clone(), ty.clone()).into())
                    }
                    Ok(match self {
                        $( MaterialDataType::$variant => TypedMaterialData::$variant(MaybeDefaultOption::SomeDefault(None, opt.into_raw::<$tp>()?)) ),+,
                    })
                } else {
                    Ok(match self {
                        $( MaterialDataType::$variant => TypedMaterialData::$variant(MaybeDefaultOption::NoneDefault(None)) ),+,
                    })
                }
            }
        }

        impl MaterialData {
            pub fn get_type(&self) -> MaterialDataType {
                match self {
                    $( MaterialData::$variant(_) => MaterialDataType::$variant ),+,
                }
            }

            pub fn into_raw<T: FromMaterialData>(self) -> Result<T> {
                T::from_data(self)
            }

            pub fn upload_to_vec(&self, data: &mut Vec<f32>, position: u32, size: u32) {
                match self {
                    $( MaterialData::$variant($x) => data[position as usize..(position+size) as usize].copy_from_slice($conv) ),+,
                }
            }

        }

        impl TypedMaterialData {
            pub fn get_type(&self) -> MaterialDataType {
                match self {
                    $( TypedMaterialData::$variant(_) => MaterialDataType::$variant ),+,
                }
            }

            pub fn as_data(&self) -> Result<MaterialData> {
                Ok(match self {
                    $(
                    TypedMaterialData::$variant(x) => MaterialData::$variant(x.get()?)
                    ),+,
                })
            }

        }

        pub trait FromMaterialData
        where
            Self: Sized
        {
            fn from_data(data: MaterialData) -> Result<Self>;
        }
        $(
            impl FromMaterialData for $tp {
                fn from_data(data: MaterialData) -> Result<Self> {
                    match data {
                        MaterialData::$variant(x) => Ok(x),
                        _ => Err(anyhow::anyhow!("Invalid data type"))
                    }
                }
            }
        )+

    };
}

typed_data! {
    Mat2(Mat2)   : x => &x.to_cols_array();
    Mat3(Mat3)   : x => &x.to_cols_array();
    Mat4(Mat4)   : x => &x.to_cols_array();
    Quat(Quat)   : x => &x.to_array();
    F32(f32)     : x => &[*x];
    I32(i32)     : x => &[f32::from_bits(*x as u32)];
    U32(u32)     : x => &[f32::from_bits(*x)];
    Bool(bool)   : x => &[f32::from_bits(if *x { 1 } else { 0 })];
    Vec2(Vec2)   : x => &x.to_array();
    Vec3(Vec3)   : x => &x.to_array();
    Vec4(Vec4)   : x => &x.to_array();
    IVec2(IVec2) : x => &unsafe { std::mem::transmute::<_, [_; 2]>(x.to_array()) };
    IVec3(IVec3) : x => &unsafe { std::mem::transmute::<_, [_; 3]>(x.to_array()) };
    IVec4(IVec4) : x => &unsafe { std::mem::transmute::<_, [_; 4]>(x.to_array()) };
    UVec2(UVec2) : x => &unsafe { std::mem::transmute::<_, [_; 2]>(x.to_array()) };
    UVec3(UVec3) : x => &unsafe { std::mem::transmute::<_, [_; 3]>(x.to_array()) };
    UVec4(UVec4) : x => &unsafe { std::mem::transmute::<_, [_; 4]>(x.to_array()) };
}


#[derive(Debug)]
struct MaterialDef {
    name: String,
    program: glow::Program,
    instance_types: IndexMap<String, MaterialDataType>,
    global_types: IndexMap<String, MaterialDataType>,
    triangle_types: IndexMap<String, MaterialDataType>,
    vertex_types: IndexMap<String, MaterialDataType>,
    uniforms: IndexMap<String, (UniformLocation, MaterialDataType)>,
    layout: GeometryLayout,
}

#[derive(Debug)]
pub struct MaterialBase {
    base: Shared<MaterialDef>,
}

#[derive(Debug)]
pub struct Material {
    base: Shared<MaterialDef>,

}


#[derive(Debug)]
pub struct Vertex {
    attributes: IndexMap<String, Shared<TypedMaterialData>>
}

#[derive(Debug)]
pub struct Triangle {
    vertices: [Vertex; 3],
    attributes: IndexMap<String, Shared<TypedMaterialData>>,
}

#[derive(Debug)]
pub struct Geometry {
    triangles: Vec<Triangle>,
    material: Material,
    attributes: IndexMap<String, Shared<TypedMaterialData>>,
    uniforms: IndexMap<String, Shared<TypedMaterialData>>
}


#[derive(Debug, Clone)]
pub struct AttributeLocation {
    pub offset: u32,
    pub size: u32,
}

#[derive(Debug, Clone)]
pub struct AttributeLayout {
    pub attributes: Vec<AttributeLocation>,
}

impl AttributeLayout {
    pub fn size(&self) -> u32 {
        let mut max = 0;
        for attr in &self.attributes {
            max = max.max(attr.offset + attr.size)
        }
        max
    }
}

#[derive(Debug, Clone)]
pub struct GeometryLayout {
    pub static_layout: Vec<AttributeLayout>,
    pub dynamic_layout: Vec<AttributeLayout>,
}

pub trait Renderable {
    fn get_layout(&self) -> GeometryLayout;
    fn get_static_data(&self) -> Result<Vec<Vec<f32>>>;
    fn get_dynamic_data(&self) -> Result<Vec<Vec<f32>>>;
    fn get_uniforms(&self) -> Result<Vec<(UniformLocation, MaterialData)>>;
}

fn inlay_data(l: &AttributeLayout, data: &mut Vec<f32>, base: usize, attrs: Iter<String, Shared<TypedMaterialData>>) -> Result<()> {
    for (a, (_, v)) in l.attributes.iter().zip(attrs) {
        let val = v.borrow().as_data()?;
        val.upload_to_vec(data, base as u32 + a.offset, a.size)
    }
    Ok(())
}

impl Geometry {
    fn get_data(&self, data: &Vec<AttributeLayout>) -> Result<Vec<Vec<f32>>> {
        let mut data_buffers = Vec::new();
        for l in data {
            let span = l.size() as usize;
            let sz = span * 3 * self.triangles.len();
            let mut data = Vec::with_capacity(sz);

            for tri in &self.triangles {
                for vert in &tri.vertices {
                    let base = data.len();
                    data.resize(base + span, 0f32);
                    inlay_data(l, &mut data, base, self.attributes.iter())?;
                    inlay_data(l, &mut data, base, tri.attributes.iter())?;
                    inlay_data(l, &mut data, base, vert.attributes.iter())?;
                }
            }
            data_buffers.push(data);
        }
        Ok(data_buffers)
    }
}

impl Renderable for Geometry {
    fn get_layout(&self) -> GeometryLayout {
        self.material.base.borrow().layout.clone()
    }

    fn get_static_data(&self) -> Result<Vec<Vec<f32>>> {
        let mat = self.material.base.borrow();
        let layout = mat.layout.clone();
        self.get_data(&layout.static_layout)
    }

    fn get_dynamic_data(&self) -> Result<Vec<Vec<f32>>> {
        let mat = self.material.base.borrow();
        let layout = mat.layout.clone();
        self.get_data(&layout.dynamic_layout)
    }

    fn get_uniforms(&self) -> Result<Vec<(UniformLocation, MaterialData)>> {
        let mut v = Vec::with_capacity(self.uniforms.len());
        let mat = self.material.base.borrow();
        for (k, u) in &self.uniforms {
            let loc = mat.uniforms.get(k).ok_or_else(|| anyhow!("Uniform is not defined by the material: '{}'", k))?.0;
            v.push((loc, u.borrow().as_data()?))
        }
        Ok(v)
    }
}





















