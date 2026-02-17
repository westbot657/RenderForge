use std::collections::HashMap;
use glam::{Mat4, Vec3};
use crate::{geometry, render};
use thiserror::Error;

#[derive(Clone)]
pub(crate) struct LayoutAttr {
    pub(crate) name: String,
    pub(crate) span: u16,
}

#[derive(Clone)]
pub struct Layout {
    pub(crate) attrs: Vec<LayoutAttr>,
    pub(crate) position_marker: Option<u16>,
    pub(crate) normal_marker: Option<u16>
}

pub struct RawLayout {
    attrs: Vec<LayoutAttr>
}

pub struct RawVertex<'l> {
    attrs: HashMap<&'l str, &'l dyn render::GlData>,
    layout: &'l Layout,
}


#[derive(Clone)]
pub struct Vertex {
    data: Vec<f32>,
    position_marker: Option<u16>,
    normal_marker: Option<u16>,
}

impl RawLayout {

    fn get_idx(&self, target: Option<&str>) -> Result<Option<u16>, DynamicGeometryError> {
        Ok(match target {
            Some(s) => {
                let s = s.to_string();
                let Some(idx) = self.attrs
                    .iter()
                    .enumerate()
                    .find(|(i, a)| a.name == s)
                    .map(|(i, _)| i) else {
                    return Err(DynamicGeometryError::InvalidMetaMarker(s))
                };
                Some(idx as u16)
            }
            None => None
        })
    }

    pub fn add_attribute(&mut self, name: impl ToString, span: u16) -> Result<(), DynamicGeometryError> {
        let name = name.to_string();
        if self.attrs.iter().any(|a| a.name == name) {
            return Err(DynamicGeometryError::DuplicateAttribute(name))
        }
        self.attrs.push(LayoutAttr { name, span });
        Ok(())
    }

    pub fn build(self, position_marker: Option<&str>, normal_marker: Option<&str>) -> Result<Layout, DynamicGeometryError> {
        let position_marker = self.get_idx(position_marker)?;
        let normal_marker = self.get_idx(normal_marker)?;

        Ok(Layout {
            attrs: self.attrs,
            position_marker,
            normal_marker
        })
    }
}

impl Layout {

    pub fn new() -> RawLayout {
        RawLayout {
            attrs: Vec::new(),
        }
    }

    pub fn vertex(&self) -> RawVertex<'_> {
        RawVertex {
            attrs: HashMap::new(),
            layout: self
        }
    }

    fn contains_name(&self, name: &str) -> bool {
        self.attrs.iter().any(|attr| attr.name == name)
    }

    fn borrow_name(&self, name: &str) -> &str {
        &self.attrs.iter().find(|a| a.name == name).unwrap().name
    }

    fn get_expected_span(&self, name: &str) -> u16 {
        self.attrs.iter().find(|a| a.name == name).unwrap().span
    }

}

#[derive(Error, Debug)]
pub enum DynamicGeometryError {
    #[error("Vertex does not define attribute: '{missing}'")]
    IncompleteVertex { missing: String },
    #[error("Vertex does not have an attribute named '{0}'")]
    InvalidName(String),
    #[error("Expected data to take up {expected} f32s, given data uses {found} f32s")]
    IncompatibleSize { expected: usize, found: usize },
    #[error("Name '{0}' not found in layout")]
    InvalidMetaMarker(String),
    #[error("Attribute already exists in layout: '{0}'")]
    DuplicateAttribute(String),
}

impl<'l> RawVertex<'l> {

    pub fn set_attr(&mut self, name: impl ToString, value: &'l dyn render::GlData) -> Result<(), DynamicGeometryError> {

        let name = name.to_string();

        if !self.layout.contains_name(&name) {
            return Err(DynamicGeometryError::InvalidName(name))
        }

        let expected = self.layout.get_expected_span(&name) as usize;
        let found = value.size();
        if expected != found {
            return Err(DynamicGeometryError::IncompatibleSize { expected, found })
        }

        self.attrs.insert(
            self.layout.borrow_name(&name),
            value
        );

        Ok(())
    }

    pub fn with_attr(mut self, name: impl ToString, value: &'l dyn render::GlData) -> Result<Self, DynamicGeometryError> {
        self.set_attr(name, value)?;
        Ok(self)
    }

    pub fn build(self) -> Result<Vertex, DynamicGeometryError> {
        let mut data = Vec::new();
        let mut position_marker = None;
        let mut normal_marker = None;

        let mut total_offset = 0;

        for (i, LayoutAttr { name, span }) in self.layout.attrs.iter().enumerate() {
            match self.attrs.get(name.as_str()) {
                Some(v) => v.write(&mut data),
                None => return Err(DynamicGeometryError::IncompleteVertex { missing: name.clone() })
            }
            if let Some(p) = self.layout.position_marker && p as usize == i {
                position_marker = Some(total_offset);
            }
            if let Some(n) = self.layout.normal_marker && n as usize == i {
                normal_marker = Some(total_offset);
            }
            total_offset += span;
        }

        Ok(Vertex { data, position_marker, normal_marker })
    }
}

impl<'l> TryInto<Vertex> for RawVertex<'l> {
    type Error = DynamicGeometryError;
    fn try_into(self) -> Result<Vertex, Self::Error> {
        self.build()
    }
}

impl geometry::GeoLayout for Layout {
    type Vert = Vertex;
    fn span(&self) -> usize {
        self.attrs.iter().map(|a| a.span as usize).sum()
    }
    fn alignments(&self) -> impl Iterator<Item = u32> {
        self.attrs.iter().map(|a| a.span as u32)
    }
}

impl geometry::Vertex for Vertex {
    fn write(&self, buffer: &mut Vec<f32>) {
        buffer.extend_from_slice(self.data.as_slice())
    }
    fn transform(&mut self, transform: Mat4, normal_transform: Mat4) {
        if let Some(idx) = self.position_marker {
            let idx = idx as usize;
            let pos = Vec3::from_slice(&self.data[idx..idx+3]);
            let transformed = transform.transform_point3(pos);
            self.data[idx..idx+3].copy_from_slice(&transformed.to_array());
        }
        if let Some(idx) = self.normal_marker {
            let idx = idx as usize;
            let normal = Vec3::from_slice(&self.data[idx..idx+3]);
            let transformed = normal_transform.transform_vector3(normal).normalize();
            self.data[idx..idx+3].copy_from_slice(&transformed.to_array());
        }
    }
}

impl<Geo> geometry::Geometry<Geo, Layout>
where
    Geo: geometry::GeoUnit<Vert = Vertex>
{
    pub fn new(layout: Layout) -> Self {
        Self::new_with_layout(layout)
    }

    pub fn layout(&self) -> &Layout {
        &self.layout
    }
}

pub trait TryIntoVertex: TryInto<Vertex, Error = DynamicGeometryError> {}

impl geometry::Tri<Vertex> {
    pub fn from_vertices(
        a: impl TryIntoVertex,
        b: impl TryIntoVertex,
        c: impl TryIntoVertex,
    ) -> Result<Self, DynamicGeometryError> {
        Ok(Self { vertices: [
            a.try_into()?,
            b.try_into()?,
            c.try_into()?,
        ] })
    }
}

impl geometry::Quad<Vertex> {
    pub fn from_vertices(
        a: impl TryIntoVertex,
        b: impl TryIntoVertex,
        c: impl TryIntoVertex,
        d: impl TryIntoVertex,
    ) -> Result<Self, DynamicGeometryError> {
        Ok(Self { vertices: [
            a.try_into()?,
            b.try_into()?,
            c.try_into()?,
            d.try_into()?,
        ] })
    }
}

