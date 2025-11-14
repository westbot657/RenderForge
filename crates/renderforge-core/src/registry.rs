use std::collections::HashMap;
use std::hash::Hash;

use anyhow::Result;

use crate::mesh::{InstancedMesh, InstancedMeshData, InstancedMeshTrait, MeshController};
use crate::window::Window;


#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum ResourceIdentifier {
    Texture(String),
    Atlas(String),
    InstancedMesh(String),
    Window(String),
    VertexBuffer(String),
}

#[derive(Debug)]
pub enum Resource {
    InstancedMesh(Box<dyn InstancedMeshTrait>),
    Window(Window)
}

#[derive(Debug)]
pub struct Registry {
   resources: HashMap<ResourceIdentifier, Resource> 
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

impl Registry {
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    pub fn get(&self, id: &ResourceIdentifier) -> Option<&Resource> {
        self.resources.get(id)
    }

    pub fn get_mut(&mut self, id: &ResourceIdentifier) -> Option<&mut Resource> {
        self.resources.get_mut(id)
    }

    pub fn add(&mut self, id: impl ToString, resource: Resource) {
        let id = id.to_string();
        let loc = match &resource {
            Resource::Window(..) => ResourceIdentifier::Window(id),
            Resource::InstancedMesh(..) => ResourceIdentifier::InstancedMesh(id),
        };

        self.resources.insert(loc, resource);

    }

}


