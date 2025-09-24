use gl::types::{GLenum, GLuint};
use image::{DynamicImage, GenericImageView};

use crate::data::Color;

#[derive(Debug, Clone, Copy)]
pub enum MinFilter {
    LinearLinear,
    LinearNearest,
    NearestLinear,
    NearestNearest,
    Nearest,
    Linear,
}

#[derive(Debug, Clone, Copy)]
pub enum MagFilter {
    Nearest,
    Linear,
}

pub enum WrapMode {
    Repeat,
    Mirror,
    ClampToEdge,
    ClampToBorder,
}

pub struct TextureWrap {
    wrap_s: WrapMode,
    wrap_t: WrapMode,
    border_color: [f32; 4],
}


/// 
/// Takes an image::DynamicImage and creates an RGB or RGBA format gl texture and uploads it.
/// if min_filter is one of LinearLinear, LinearNearest, NearestLinear, NearestNearest, then
/// mipmaps are generated as well.
///
/// # Returns
/// (gl id, (texture width, texture height))
pub fn upload_image(img: &DynamicImage, min_filter: MinFilter, mag_filter: MagFilter, texture_wrap: TextureWrap) -> (GLuint, (u32, u32)) {
    unsafe {

        let mut tex_id = 0;

        gl::GenTextures(1, &mut tex_id);
        gl::BindTexture(gl::TEXTURE_2D, tex_id);

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, texture_wrap.wrap_s.to_gl() as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, texture_wrap.wrap_t.to_gl() as i32);

        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, min_filter.to_gl() as i32);
        gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, mag_filter.to_gl() as i32);

        let size = img.dimensions();
        let data;
        let format = if img.has_alpha() {
            let img = img.to_rgba8();
            data = img.into_raw();
            gl::RGBA
        } else {
            let img = img.to_rgb8();
            data = img.into_raw();
            gl::RGB
        };

        gl::TexImage2D(
            gl::TEXTURE_2D,
            0,
            format as i32,
            size.0 as i32,
            size.1 as i32,
            0,
            format,
            gl::UNSIGNED_BYTE,
            data.as_ptr() as *const _,
        );

        if !matches!(min_filter, MinFilter::Nearest | MinFilter::Linear) {
            gl::GenerateMipmap(gl::TEXTURE_2D);
        }

        (tex_id, size)

    }
}


impl MinFilter {
    pub fn to_gl(&self) -> GLenum {
        match self {
            Self::LinearLinear => gl::LINEAR_MIPMAP_LINEAR,
            Self::LinearNearest => gl::LINEAR_MIPMAP_NEAREST,
            Self::NearestLinear => gl::NEAREST_MIPMAP_LINEAR,
            Self::NearestNearest => gl::NEAREST_MIPMAP_NEAREST,
            Self::Linear => gl::LINEAR,
            Self::Nearest => gl::NEAREST
        }
    }
}

impl MagFilter {
    pub fn to_gl(&self) -> GLenum {
        match self {
            Self::Nearest => gl::NEAREST,
            Self::Linear => gl::LINEAR
        }
    }
}

impl WrapMode {
    pub fn to_gl(&self) -> GLenum {
        match self {
            Self::Repeat => gl::REPEAT,
            Self::Mirror => gl::MIRRORED_REPEAT,
            Self::ClampToEdge => gl::CLAMP_TO_EDGE,
            Self::ClampToBorder => gl::CLAMP_TO_BORDER,
        }
    }
}

impl TextureWrap {
    pub fn new(wrap_s: WrapMode, wrap_t: WrapMode) -> Self {
        Self {
            wrap_s,
            wrap_t,
            border_color: [0., 0., 0., 0.],
        }
    }

    pub fn with_border_color(mut self, color: [f32; 4]) -> Self {
        self.border_color = color;
        self
    }

    pub fn set_border_color(&mut self, color: [f32; 4]) {
        self.border_color = color;
    }

}

