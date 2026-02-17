use std::borrow::Borrow;
use std::collections::HashMap;
use thiserror::Error;
use crate::geometry::*;
use crate::render::{instanced, GlData};
use crate::render::instanced::InstancedMesh;

#[derive(Clone)]
pub(crate) struct LayoutAttr {
    pub(crate) name: String,
    pub(crate) span: u16,
}

#[derive(Clone)]
pub struct Layout {
    pub(crate) attrs: Vec<LayoutAttr>
}

pub struct RawData<'l> {
    attrs: HashMap<&'l str, &'l dyn GlData>,
    layout: &'l Layout,
}

#[derive(Clone)]
pub struct Data {
    data: Vec<f32>,
}

impl Layout {
    pub fn data(&self) -> RawData<'_> {
        RawData {
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
pub enum DynamicInstanceError {
    #[error("Vertex does not define attribute: '{missing}'")]
    IncompleteVertex { missing: String },
    #[error("Vertex does not have an attribute named '{0}'")]
    InvalidName(String),
    #[error("Expected data to take up {expected} f32s, given data uses {found} f32s")]
    IncompatibleSize { expected: usize, found: usize },
    #[error("Attribute already exists in layout: '{0}'")]
    DuplicateAttribute(String),
}

impl<'l> RawData<'l> {
    
    pub fn set_attr(&mut self, name: impl ToString, value: &'l dyn GlData) -> Result<(), DynamicInstanceError> {
        
        let name = name.to_string();
        
        if !self.layout.contains_name(&name) {
            return Err(DynamicInstanceError::InvalidName(name))
        }
        
        let expected = self.layout.get_expected_span(&name) as usize;
        let found = value.size();
        if expected != found {
            return Err(DynamicInstanceError::IncompatibleSize { expected, found })
        }
        
        self.attrs.insert(
            self.layout.borrow_name(&name),
            value
        );
        
        Ok(())
    }
    
    pub fn with_attr(mut self, name: impl ToString, value: &'l dyn GlData) -> Result<Self, DynamicInstanceError> {
        self.set_attr(name, value)?;
        Ok(self)
    }
    
    pub fn build(self) -> Result<Data, DynamicInstanceError> {
        let mut data = Vec::new();
        
        for LayoutAttr { name, span: _ } in &self.layout.attrs {
            match self.attrs.get(name.as_str()) {
                Some(v) => v.write(&mut data),
                None => return Err(DynamicInstanceError::IncompleteVertex { missing: name.clone() })
            }
        }
        
        Ok(Data { data })
    }
    
}

impl<'l> TryInto<Data> for RawData<'l> {
    type Error = DynamicInstanceError;
    fn try_into(self) -> Result<Data, Self::Error> {
        self.build()
    }
}

impl instanced::InstanceLayout for Layout {
    type Data = Data;
    fn span(&self) -> usize {
        self.attrs.iter().map(|a| a.span as usize).sum()
    }
    fn alignments(&self) -> impl Iterator<Item = u32> {
        self.attrs.iter().map(|a| a.span as u32)
    }
}

impl instanced::InstanceData for Data {
    fn write(&self, buffer: &mut Vec<f32>) {
        buffer.extend_from_slice(self.data.as_slice())
    }
}

impl<Geo, GLayout> InstancedMesh<Geo, GLayout, Layout>
where
    Geo: GeoUnit<Vert = GLayout::Vert>,
    GLayout: GeoLayout
{

}
