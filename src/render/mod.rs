use std::cell::RefCell;
use std::rc;
use crate::render::shader::Uniforms;
use crate::render::state::GlStateManager;

pub mod batched;
pub mod buffer;
pub mod instanced;
pub mod shader;
mod state;

pub trait Renderer: Sized {
    fn setup(&mut self, gl: &glow::Context) -> Result<(), String> {
        let _ = gl;
        Ok(())
    }
    fn render(&mut self, gl: &glow::Context, state: &mut GlStateManager);
    fn destroy(self, gl: &glow::Context) {
        let _ = gl;
    }
}

pub trait StateController: Sized {
    fn set_state(&mut self, state: &mut GlStateManager, uniforms: &rc::Weak<RefCell<Uniforms>>);
}

pub trait GlData {
    fn size(&self) -> usize;
    fn write(&self, buffer: &mut Vec<f32>);
}

impl GlData for f32 {
    fn size(&self) -> usize { 1 }
    fn write(&self, buffer: &mut Vec<f32>){
        buffer.push(*self)
    }
}
impl GlData for [f32; 2] {
    fn size(&self) -> usize { 2 }
    fn write(&self, buffer: &mut Vec<f32>) {
        buffer.extend_from_slice(self)
    }
}
impl GlData for [f32; 3] {
    fn size(&self) -> usize { 3 }
    fn write(&self, buffer: &mut Vec<f32>) {
        buffer.extend_from_slice(self)
    }
}
impl GlData for [f32; 4] {
    fn size(&self) -> usize { 4 }
    fn write(&self, buffer: &mut Vec<f32>) {
        buffer.extend_from_slice(self)
    }
}
impl GlData for [f32; 16] {
    fn size(&self) -> usize { 16 }
    fn write(&self, buffer: &mut Vec<f32>) {
        buffer.extend_from_slice(self)
    }
}
impl GlData for glam::Vec2 {
    fn size(&self) -> usize { 2 }
    fn write(&self, buffer: &mut Vec<f32>) {
        buffer.extend_from_slice(self.as_ref())
    }
}
impl GlData for glam::Vec3 {
    fn size(&self) -> usize { 3 }
    fn write(&self, buffer: &mut Vec<f32>) {
        buffer.extend_from_slice(self.as_ref())
    }
}
impl GlData for glam::Vec4 {
    fn size(&self) -> usize { 4 }
    fn write(&self, buffer: &mut Vec<f32>) {
        buffer.extend_from_slice(self.as_ref())
    }
}
impl GlData for glam::Quat {
    fn size(&self) -> usize { 4 }
    fn write(&self, buffer: &mut Vec<f32>) {
        buffer.extend_from_slice(self.as_ref())
    }
}
impl GlData for glam::Mat4 {
    fn size(&self) -> usize { 16 }
    fn write(&self, buffer: &mut Vec<f32>) {
        buffer.extend_from_slice(self.as_ref())
    }
}

impl GlData for u32 {
    fn size(&self) -> usize { 1 }
    fn write(&self, buffer: &mut Vec<f32>) {
        buffer.push(f32::from_bits(*self))
    }
}
impl GlData for i32 {
    fn size(&self) -> usize { 1 }
    fn write(&self, buffer: &mut Vec<f32>) {
        buffer.push(f32::from_bits(*self as u32))
    }
}
impl GlData for bool {
    fn size(&self) -> usize { 1 }
    fn write(&self, buffer: &mut Vec<f32>) {
        buffer.push(f32::from_bits(*self as u32))
    }
}
