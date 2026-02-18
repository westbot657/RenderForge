use glam::Vec4;
use glow::{Context, HasContext};
use crate::render::camera::Camera;
use crate::render::Renderer;
use crate::render::state::GlStateManager;

pub struct BackgroundColor {
    color: Vec4,
    mask: u32,
}
impl BackgroundColor {
    pub fn rgba(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { color: Vec4::new(r, g, b, a), mask: glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT }
    }
    pub fn rgb(r: f32, g: f32, b: f32) -> Self {
        Self::rgba(r, g, b, 1.)
    }
    pub fn splat_rgb(x: f32) -> Self {
        Self::rgb(x, x, x)
    }
    pub fn splat_rgba(x: f32) -> Self {
        Self::rgba(x, x, x, x)
    }
    pub fn persist_depth(mut self) -> Self {
        self.mask &= !glow::DEPTH_BUFFER_BIT;
        self
    }
}

impl<Shared> Renderer<Shared> for BackgroundColor {
    fn render(&mut self, gl: &Context, _state: &mut GlStateManager, _camera: &Camera, _shared_state: &Shared) {
        unsafe {
            gl.clear_color(self.color.x, self.color.y, self.color.z, self.color.w);
            gl.clear(self.mask)
        }
    }
}


pub struct Layer<Shared> {
    components: Vec<Box<dyn Renderer<Shared>>>
}

impl<Shared> Renderer<Shared> for Layer<Shared> {
    fn render(&mut self, gl: &Context, state: &mut GlStateManager, camera: &Camera, shared_state: &Shared) {
        for comp in &mut self.components {
            comp.render(gl, state, camera, shared_state)
        }
    }
}



