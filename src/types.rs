use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use glow::Texture;
use uuid::Uuid;

use crate::material::MaterialDef;
use crate::mesh::Mesh;

#[derive(PartialEq, Eq, Hash, Debug, Clone)]
pub enum AssetID {
    Name(String),
    Uuid(Uuid),
}

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum AssetHandle {
    MaterialDef(AssetID),
    Texture(AssetID),
    Mesh(AssetID),
}

pub enum Asset {
    MaterialDef(MaterialDef),
    Texture(Texture),
    Mesh(Mesh),
}

pub struct AssetStore {
    assets: HashMap<AssetID, Asset>,
}


pub struct AssetLibrary {
    store: Rc<RefCell<AssetStore>>,
}

impl AssetStore {
    fn new() -> Self {
        Self {
            assets: HashMap::new(),
        }
    }
}

impl AssetLibrary {

    pub fn new() -> Self {
        Self {
            store: Rc::new(RefCell::new(AssetStore::new()))
        }
    }

    pub fn get_asset(&self, handle: AssetHandle) -> Option<&Asset> {
        let a = self.store.borrow();
    }

}

