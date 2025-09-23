use std::cmp::Ordering;
use std::collections::HashMap;

use anyhow::Result;
use gl::types::GLuint;
use image::{imageops, DynamicImage, GenericImageView, RgbaImage};
use rect_packer::{Config, Packer};

use crate::errors::AtlasError;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct AtlasTextureIdentifier(String);

pub struct AtlasRect {
    rect: (u32, u32, u32, u32),
    size: (f32, f32),
}

/// Represents a single texture atlas. Cannot be modified, use AtlasBuilder to build an Atlas
pub struct Atlas {
    tex_id: GLuint,
    position_data: HashMap<AtlasTextureIdentifier, AtlasRect>,
    size: (u32, u32),
}

pub struct AtlasBuilder {
    size: (u32, u32),
    texture: RgbaImage,
    texture_queue: Vec<(AtlasTextureIdentifier, DynamicImage)>,
    border_padding: u32,
    rectangle_padding: u32,
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
    pub fn new(size: (u32, u32), border_padding: u32, rectangle_padding: u32) -> Self {
        Self {
            size,
            texture: RgbaImage::new(size.0, size.1),
            texture_queue: Vec::new(),
            border_padding,
            rectangle_padding,
        }
    }

    pub fn add(&mut self, id: AtlasTextureIdentifier, img: DynamicImage) {
        self.texture_queue.push((id, img));
    }

    /// Turns the AtlasBuilder into an Atlas.
    /// if any images were not able to fit on the atlas, they are returned in the paired Vec.
    pub fn build_overflow(self) -> (Atlas, Vec<(AtlasTextureIdentifier, DynamicImage)>) {
        self.build(false).unwrap()
    }

    /// Turns the AtlasBuilder into an Atlas.
    /// if any images are not able to fit on the atlas, an error is returned and the atlas is not
    /// built or uploaded.
    pub fn build_strict(self) -> Result<Atlas> {
        let r = self.build(true);
        if let Ok(res) = r {
            Ok(res.0)
        } else {
            Err(r.err().unwrap())
        }
    }

    fn tex_sorter(a: &(AtlasTextureIdentifier, DynamicImage), b: &(AtlasTextureIdentifier, DynamicImage)) -> Ordering {

        let ad = a.1.dimensions();
        let bd = b.1.dimensions();

        (ad.0 * ad.1).cmp(&(bd.0 * bd.1)).reverse()

    }

    fn build(self, error_on_overflow: bool) -> Result<(Atlas, Vec<(AtlasTextureIdentifier, DynamicImage)>)> {
        let mut overflow = Vec::new();

        let mut textures = self.texture_queue.clone();
        textures.sort_by(AtlasBuilder::tex_sorter);

        let config = Config {
            width: self.size.0 as i32,
            height: self.size.1 as i32,
            border_padding: self.border_padding as i32,
            rectangle_padding: self.rectangle_padding as i32,
        };

        let mut packer = Packer::new(config);
        let mut img = RgbaImage::new(self.size.0, self.size.1);
        let mut rectangle_map = HashMap::new();

        for (id, tex) in textures {

            let (w, h) = tex.dimensions();

            if packer.can_pack(w as i32, h as i32, false) {
                let tex = tex.to_rgba8();
                let rect = packer.pack(w as i32, h as i32, false).unwrap();
                rectangle_map.insert(id, (rect.x as u32, rect.y as u32, rect.width as u32, rect.height as u32));

                imageops::overlay(&mut img, &tex, rect.x as i64, rect.y as i64);

            } else {
                if error_on_overflow {
                    return Err(AtlasError::TextureOverflow.into());
                } else {
                    overflow.push((id, tex));
                }
            }


        }



        Err(AtlasError::TextureOverflow.into())

    }

}



