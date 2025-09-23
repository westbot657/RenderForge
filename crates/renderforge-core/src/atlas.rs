use std::collections::HashMap;

use anyhow::Result;
use gl::types::GLuint;
use image::{DynamicImage, RgbaImage};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct AtlasTextureIdentifier(String);

pub struct AtlasRect {
    rect: (u32, u32, u32, u32),
    size: (f32, f32),
}

/// Represents a single texture atlas.
pub struct Atlas {
    tex_id: GLuint,
    position_data: HashMap<AtlasTextureIdentifier, AtlasRect>,
    size: (u32, u32)
}

pub struct AtlasBuilder {
    size: (u32, u32),
    texture: RgbaImage,
    texture_queue: Vec<(AtlasTextureIdentifier, DynamicImage)>,
}


/// Represents a collection of texture atlases
pub struct AtlasSet {
    atlases: Vec<Atlas>
}



impl AtlasRect {
    fn new(size: (u32, u32), rect: (u32, u32, u32, u32)) -> Self {
        Self {
            rect,
            size: (size.0 as f32, size.1 as f32)
        }
    }

    fn coords(&self) -> (u32, u32, u32, u32) {
        self.rect
    }

    fn uvs(&self) -> (f32, f32, f32, f32) {
        (
            self.rect.0 as f32 / self.size.0,
            self.rect.1 as f32 / self.size.1,
            self.rect.2 as f32 / self.size.0,
            self.rect.3 as f32 / self.size.1
        )
    }

}

impl AtlasBuilder {
    pub fn new(size: (u32, u32)) -> Self {
        Self {
            size,
            texture: RgbaImage::new(size.0, size.1),
            texture_queue: Vec::new(),
        }
    }

    /// Turns the AtlasBuilder into an Atlas.
    /// if any images were not able to fit on the atlas, they are returned in the paired Vec.
    pub fn build_overflow(self) -> (Atlas, Vec<AtlasTextureIdentifier, DynamicImage>) {




    }

    /// Turns the AtlasBuilder into an Atlas.
    /// if any images are not able to fit on the atlas, an error is returned and the atlas is not
    /// built or uploaded.
    pub fn build_strict(self) -> Result<Atlas> {

    }

}



