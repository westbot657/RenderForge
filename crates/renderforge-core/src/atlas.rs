use std::cmp::Ordering;
use std::collections::HashMap;
use std::mem;

use anyhow::Result;
use gl::types::GLuint;
use image::{imageops, DynamicImage, GenericImageView, RgbaImage};
use rect_packer::{Config, Packer};

use crate::errors::AtlasError;
use crate::texture::{upload_image, MagFilter, MinFilter, TextureWrap, WrapMode};

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct AtlasTextureIdentifier(String);

#[derive(Debug, Clone, Copy)]
pub struct AtlasRect {
    rect: (u32, u32, u32, u32),
    size: (f32, f32),
}

#[derive(Debug)]
pub struct Atlas {
    tex_id: GLuint,
    position_data: HashMap<AtlasTextureIdentifier, AtlasRect>,
    size: (u32, u32),
}

#[derive(Debug)]
pub struct AtlasBuilder {
    size: (u32, u32),
    texture: RgbaImage,
    texture_queue: Vec<(AtlasTextureIdentifier, DynamicImage)>,
    border_padding: u32,
    rectangle_padding: u32,
    min_filter: MinFilter,
    mag_filter: MagFilter,
}

#[derive(Debug)]
pub struct AtlasSet {
    atlases: Vec<Atlas>
}

#[derive(Debug)]
pub struct AtlasSetBuilder {
    texture_queue: Vec<(AtlasTextureIdentifier, DynamicImage)>,
    size: (u32, u32),
    border_padding: u32,
    rectangle_padding: u32,
    min_filter: MinFilter,
    mag_filter: MagFilter,
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
    /// creates a new AtlasBuilder, used to set up all the data needed to create an Atlas.
    pub fn new(size: (u32, u32), border_padding: u32, rectangle_padding: u32, min_filter: MinFilter, mag_filter: MagFilter) -> Self {
        Self {
            size,
            texture: RgbaImage::new(size.0, size.1),
texture_queue: Vec::new(),
            border_padding,
            rectangle_padding,
            min_filter,
            mag_filter,
        }
    }

    pub fn add(&mut self, id: AtlasTextureIdentifier, img: DynamicImage) -> Result<()> {
        for (id2, _) in &self.texture_queue {
            if id == *id2 {
                return Err(AtlasError::DuplicateId(id.0.to_string()).into());
            }
        }
        self.texture_queue.push((id, img));
        Ok(())
    }

    /// Turns the AtlasBuilder into an Atlas.
    /// if any images were not able to fit on the atlas, they are returned in the paired Vec.
    pub fn build_overflow(self) -> Result<(Atlas, Vec<(AtlasTextureIdentifier, DynamicImage)>)> {
        self.build(false)
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

            if rectangle_map.contains_key(&id) {
                return Err(AtlasError::DuplicateId(id.0.to_string()).into());
            }

            let (w, h) = tex.dimensions();

            if packer.can_pack(w as i32, h as i32, false) {
                let tex = tex.to_rgba8();
                let rect = packer.pack(w as i32, h as i32, false).unwrap();
                rectangle_map.insert(id, AtlasRect::new(self.size, (rect.x as u32, rect.y as u32, rect.width as u32, rect.height as u32)));

                imageops::overlay(&mut img, &tex, rect.x as i64, rect.y as i64);

            } else if error_on_overflow {
                return Err(AtlasError::TextureOverflow.into());
            } else {
                overflow.push((id, tex));
            }


        }


        let d = DynamicImage::ImageRgba8(img);

        let (glid, _) = upload_image(&d, self.min_filter, self.mag_filter, TextureWrap::new(WrapMode::ClampToEdge, WrapMode::ClampToEdge));


        #[cfg(feature = "texture-debug")]
        {
            let name = format!("./texture_debug-atlas-{}.png", glid);
            let res = d.save(name);
            if let Err(e) = res {
                eprintln!("[RenderForge] texture-debug: Failed to save atlas to disk (GL id {})", glid);
            }
        }


        let atlas = Atlas {
            tex_id: glid,
            position_data: rectangle_map,
            size: self.size,
        };

        Ok((atlas, overflow))

    }

}


impl AtlasSetBuilder {
    pub fn new(size: (u32, u32), border_padding: u32, rectangle_padding: u32, min_filter: MinFilter, mag_filter: MagFilter) -> Self {
        Self {
            texture_queue: Vec::new(),
            size,
            border_padding,
            rectangle_padding,
            min_filter,
            mag_filter
        }
    }

    pub fn add(&mut self, id: AtlasTextureIdentifier, texture: DynamicImage) -> Result<()> {
        for (id2, _) in &self.texture_queue {
            if id == *id2 {
                return Err(AtlasError::DuplicateId(id.0.to_string()).into());
            }
        }
        self.texture_queue.push((id, texture));
        Ok(())
    }

    pub fn build(mut self) -> AtlasSet {

        let mut finalized = Vec::new();

        let mut textures = mem::take(&mut self.texture_queue);

        loop {
            let mut builder = AtlasBuilder::new(self.size, self.border_padding, self.rectangle_padding, self.min_filter, self.mag_filter);
            let mut ts = Vec::new();

            mem::swap(&mut ts, &mut textures);
            for tex in ts {
                builder.add(tex.0, tex.1).unwrap();
            }

            let (atlas, textures) = builder.build_overflow().unwrap();

            finalized.push(atlas);

            if textures.is_empty() {
                break AtlasSet {
                    atlases: finalized
                }
            }

        }


    }

}

impl Atlas {
    pub fn has_texture(&self, id: &AtlasTextureIdentifier) -> bool {
        self.position_data.contains_key(id)
    }

    pub fn get_rect(&self, id: &AtlasTextureIdentifier) -> Option<AtlasRect> {
        self.position_data.get(id).copied()
    }

    pub fn get_id(&self) -> GLuint {
        self.tex_id
    }

    pub fn get_size(&self) -> (u32, u32) {
        self.size
    }
}

impl AtlasSet {
    pub fn has_texture(&self, id: &AtlasTextureIdentifier) -> bool {
        for a in &self.atlases {
            if a.has_texture(id) {
                return true
            }
        }
        false
    }

    pub fn get_id_and_rect(&self, id: &AtlasTextureIdentifier) -> Option<(GLuint, AtlasRect)> {

        for a in &self.atlases {
            if a.has_texture(id) {
                return Some((a.get_id(), a.get_rect(id).unwrap()))
            }
        }
        None
    }

}



