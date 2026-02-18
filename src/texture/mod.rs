mod atlas;

use glam::Vec4;
use glow::HasContext;

#[derive(Copy, Clone)]
pub struct TextureSettings {
    pub generate_mipmaps: bool,
    pub wrap_s: u32,
    pub wrap_t: u32,
    pub min_filter: u32,
    pub mag_filter: u32,
    pub border_color: Option<Vec4>
}

impl TextureSettings {
    pub fn new(generate_mipmaps: bool, wrap_s: u32, wrap_t: u32, min_filter: u32, mag_filter: u32) -> Self {
        Self {
            generate_mipmaps,
            wrap_s, wrap_t,
            min_filter, mag_filter,
            border_color: None
        }
    }

    pub fn repeat(mut self) -> Self {
        self.border_color = None;
        self.wrap_s = glow::REPEAT;
        self.wrap_t = glow::REPEAT;
        self
    }

    pub fn clamp(mut self) -> Self {
        self.border_color = None;
        self.wrap_s = glow::CLAMP_TO_EDGE;
        self.wrap_t = glow::CLAMP_TO_EDGE;
        self
    }

    pub fn mirror(mut self) -> Self {
        self.border_color = None;
        self.wrap_s = glow::MIRRORED_REPEAT;
        self.wrap_t = glow::MIRRORED_REPEAT;
        self
    }

    pub fn bordered(mut self, color: Vec4) -> Self {
        self.border_color = Some(color);
        self
    }

    pub fn linear(mut self) -> Self {
        self.border_color = None;
        self.min_filter = glow::LINEAR;
        self.mag_filter = glow::LINEAR;
        self
    }

    pub fn nearest(mut self) -> Self {
        self.border_color = None;
        self.min_filter = glow::NEAREST;
        self.mag_filter = glow::NEAREST;
        self
    }

    pub fn mipmap_nearest(mut self) -> Self {
        self.generate_mipmaps = true;
        if matches!(self.min_filter, glow::NEAREST | glow::NEAREST_MIPMAP_NEAREST | glow::NEAREST_MIPMAP_LINEAR) {
            self.min_filter = glow::NEAREST_MIPMAP_NEAREST;
        } else {
            self.min_filter = glow::LINEAR_MIPMAP_NEAREST;
        }
        self
    }

    pub fn mipmap_linear(mut self) -> Self {
        self.generate_mipmaps = true;
        if matches!(self.min_filter, glow::NEAREST | glow::NEAREST_MIPMAP_NEAREST | glow::NEAREST_MIPMAP_LINEAR) {
            self.min_filter = glow::NEAREST_MIPMAP_LINEAR;
        } else {
            self.min_filter = glow::LINEAR_MIPMAP_LINEAR;
        }
        self
    }

}

impl Default for TextureSettings {
    fn default() -> Self {
        Self::new(false, glow::REPEAT, glow::REPEAT, glow::NEAREST, glow::NEAREST)
    }
}

pub struct Texture {
    pub tex: Option<glow::Texture>,
    pub size: (u32, u32)
}

impl Texture {
    pub fn new(gl: &glow::Context, settings: TextureSettings, data: &[u8]) -> Result<Self, String> {
        let img = image::load_from_memory(data)
            .map_err(|e| format!("Failed to load image: {e}"))?
            .to_rgba8();

        let size = img.dimensions();
        
        Self::from_rgba_data(gl, settings, &img, size)
    }
    
    pub fn from_rgba_data(gl: &glow::Context, settings: TextureSettings, data: &[u8], size: (u32, u32)) -> Result<Self, String> {
        unsafe {
            let tex = gl.create_texture()?;
            gl.bind_texture(glow::TEXTURE_2D, Some(tex));

            gl.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                size.0 as i32,
                size.1 as i32,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                glow::PixelUnpackData::Slice(Some(data))
            );

            if settings.generate_mipmaps {
                gl.generate_mipmap(glow::TEXTURE_2D);
            }

            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_S, settings.wrap_s as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_WRAP_T, settings.wrap_t as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MIN_FILTER, settings.min_filter as i32);
            gl.tex_parameter_i32(glow::TEXTURE_2D, glow::TEXTURE_MAG_FILTER, settings.mag_filter as i32);

            if let Some(color) = settings.border_color {
                gl.tex_parameter_f32_slice(glow::TEXTURE_2D, glow::TEXTURE_BORDER_COLOR, color.as_ref());
            }

            gl.bind_texture(glow::TEXTURE_2D, None);

            Ok(Self {
                tex: Some(tex),
                size
            })
        }

    }

    pub fn bind(&self, gl: &glow::Context, unit: u32) {
        unsafe {
            gl.active_texture(glow::TEXTURE0 + unit);
            gl.bind_texture(glow::TEXTURE_2D, self.tex);
        }
    }

    pub fn unbind(&self, gl: &glow::Context) {
        unsafe {
            gl.bind_texture(glow::TEXTURE_2D, None);
        }
    }

    pub fn destroy(&mut self, gl: &glow::Context) {
        unsafe {
            if let Some(tex) = self.tex.take() {
                gl.delete_texture(tex);
            }
        }
    }

}


