

use std::collections::HashMap;
use glow::HasContext;
use rect_packer::Packer;

pub struct RawAtlas {
    textures: Vec<RawTexture>,
    next_id: u32,
    max_size: u32,
}

struct RawTexture {
    id: u32,
    data: Vec<u8>,
    width: u32,
    height: u32,
}

#[derive(Clone, Copy)]
pub struct AtlasRegion {
    pub min_u: f32,
    pub min_v: f32,
    pub max_u: f32,
    pub max_v: f32,
}

pub struct TextureAtlas {
    pub texture: crate::texture::Texture,
    pub regions: HashMap<u32, AtlasRegion>,
    pub atlas_width: u32,
    pub atlas_height: u32,
}

impl RawAtlas {
    pub fn new(max_size: u32) -> Self {
        Self {
            textures: Vec::new(),
            next_id: 0,
            max_size,
        }
    }

    pub fn submit_rgba(&mut self, data: Vec<u8>, width: u32, height: u32) -> u32 {
        let id = self.next_id;
        self.next_id += 1;

        self.textures.push(RawTexture {
            id,
            data,
            width,
            height,
        });

        id
    }

    pub fn submit_image(&mut self, bytes: &[u8]) -> Result<u32, String> {
        let img = image::load_from_memory(bytes)
            .map_err(|e| format!("Failed to load image: {}", e))?
            .to_rgba8();

        let (width, height) = img.dimensions();
        Ok(self.submit_rgba(img.into_raw(), width, height))
    }

    pub fn build(
        self,
        gl: &glow::Context,
        settings: crate::texture::TextureSettings,
    ) -> Result<TextureAtlas, String> {
        if self.textures.is_empty() {
            return Err("Cannot build empty atlas".to_string());
        }

        let config = rect_packer::Config {
            width: self.max_size as i32,
            height: self.max_size as i32,
            border_padding: 0,
            rectangle_padding: 0,
        };

        let mut packer = Packer::new(config);

        let mut packed_rects = Vec::new();
        for texture in &self.textures {
            if let Some(rect) = packer.pack(texture.width as i32, texture.height as i32, false) {
                packed_rects.push((texture, rect));
            } else {
                return Err(format!(
                    "Failed to pack texture {}x{} into atlas (max size: {})",
                    texture.width, texture.height, self.max_size
                ));
            }
        }

        let atlas_width = self.max_size;
        let atlas_height = self.max_size;

        let mut atlas_data = vec![0u8; (atlas_width * atlas_height * 4) as usize];
        let mut regions = HashMap::new();

        for (texture, rect) in packed_rects {
            for y in 0..texture.height {
                for x in 0..texture.width {
                    let src_idx = ((y * texture.width + x) * 4) as usize;
                    let dst_x = rect.x as u32 + x;
                    let dst_y = rect.y as u32 + y;
                    let dst_idx = ((dst_y * atlas_width + dst_x) * 4) as usize;

                    atlas_data[dst_idx..dst_idx + 4]
                        .copy_from_slice(&texture.data[src_idx..src_idx + 4]);
                }
            }

            regions.insert(texture.id, AtlasRegion {
                min_u: rect.x as f32 / atlas_width as f32,
                min_v: rect.y as f32 / atlas_height as f32,
                max_u: (rect.x + rect.width) as f32 / atlas_width as f32,
                max_v: (rect.y + rect.height) as f32 / atlas_height as f32,
            });
        }

        let texture = crate::texture::Texture::from_rgba_data(
            gl,
            settings,
            &atlas_data,
            (atlas_width, atlas_height),
        )?;

        Ok(TextureAtlas {
            texture,
            regions,
            atlas_width,
            atlas_height,
        })
    }
}

impl TextureAtlas {
    pub fn get_region(&self, id: u32) -> Option<&AtlasRegion> {
        self.regions.get(&id)
    }

    pub fn bind(&self, gl: &glow::Context, unit: u32) {
        self.texture.bind(gl, unit);
    }

    pub fn unbind(&self, gl: &glow::Context) {
        self.texture.unbind(gl);
    }

    pub fn destroy(&mut self, gl: &glow::Context) {
        self.texture.destroy(gl);
    }
}