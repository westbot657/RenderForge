use std::collections::HashMap;
use crate::geometry;
use thiserror::Error;

#[derive(Clone)]
struct LayoutAttr {
    name: String,
    span: usize,
}

#[derive(Clone)]
pub struct Layout {
    attrs: Vec<LayoutAttr>,
}

pub struct RawVertex<'l> {
    attrs: HashMap<&'l str, &'l dyn geometry::GlData>,
    layout: &'l Layout,
}

#[derive(Clone)]
pub struct Vertex {
    data: Vec<f32>,
}

impl Layout {

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

    fn get_expected_span(&self, name: &str) -> usize {
        self.attrs.iter().find(|a| a.name == name).unwrap().span
    }

}

#[derive(Error, Debug)]
pub enum DynamicGeometryError {
    #[error("Vertex does not define attributes: {missing}")]
    IncompleteVertex { missing: String },
    #[error("Vertex does not have an attribute named '{0}'")]
    InvalidName(String),
    #[error("Expected data to take up {expected} f32s, given data uses {found}")]
    IncompatibleSize { expected: usize, found: usize }
}

impl<'l> RawVertex<'l> {

    pub fn set_attr(&mut self, name: impl ToString, value: &'l dyn geometry::GlData) -> Result<(), DynamicGeometryError> {

        let name = name.to_string();

        if !self.layout.contains_name(&name) {
            return Err(DynamicGeometryError::InvalidName(name))
        }

        let expected = self.layout.get_expected_span(&name);
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

    pub fn with_attr(mut self, name: impl ToString, value: &'l dyn geometry::GlData) -> Result<Self, DynamicGeometryError> {
        self.set_attr(name, value)?;
        Ok(self)
    }

    pub fn build(self) -> Result<Vertex, DynamicGeometryError> {
        let mut data = Vec::new();

        for LayoutAttr { name, span: _ } in &self.layout.attrs {
            match self.attrs.get(name.as_str()) {
                Some(v) => v.write(&mut data),
                None => return Err(DynamicGeometryError::IncompleteVertex { missing: name.clone() })
            }
        }

        Ok(Vertex { data })
    }
}

impl geometry::GeoLayout for Layout {
    type Vert = Vertex;
    fn span(&self) -> usize {
        self.attrs.iter().map(|a| a.span).sum()
    }
}

impl geometry::Vertex for Vertex {
    fn write(&self, buffer: &mut Vec<f32>) {
        buffer.extend_from_slice(self.data.as_slice())
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


