use std::marker::PhantomData;
use std::sync::{RwLock, Weak};
use crate::render::camera::Camera;
use crate::render::shader::Uniforms;
use crate::render::state::GlStateManager;

pub mod batched;
pub mod buffer;
pub mod instanced;
pub mod shader;
pub mod state;
pub mod camera;
pub mod scene;

pub trait Renderer<SharedState> : Sync + Send {
    fn setup(&mut self, gl: &glow::Context) -> Result<(), String> {
        let _ = gl;
        Ok(())
    }
    fn render(
        &mut self,
        gl: &glow::Context,
        state: &mut GlStateManager,
        camera: &Camera,
        shared_state: &SharedState,
    );
    fn destroy(&mut self, gl: &glow::Context) {
        let _ = gl;
    }
}

pub trait StateController: Sized + Sync + Send {
    type SharedState: Sized + Sync + Send;
    fn set_state(
        &mut self,
        gl: &glow::Context,
        state: &mut GlStateManager,
        uniforms: &Weak<RwLock<Uniforms>>,
        camera: &Camera,
        shared_state: &Self::SharedState,
    ) {
        let _ = gl;
        let _ = state;
        let _ = uniforms;
        let _ = camera;
        let _ = shared_state;
    }
}

pub struct EmptyStateController<Shared>(PhantomData<Shared>);

impl<Shared> StateController for EmptyStateController<Shared>
where
    Shared: Sized + Sync + Send
{
    type SharedState = Shared;
}

pub struct CameraUniformsStateController<Shared> {
    view_uniform: String,
    proj_uniform: String,
    phantom_data: PhantomData<Shared>
}
impl<Shared> CameraUniformsStateController<Shared>
where
    Shared: Sized + Sync + Send
{
    pub fn new(view: impl ToString, proj: impl ToString) -> Self {
        Self {
            view_uniform: view.to_string(),
            proj_uniform: proj.to_string(),
            phantom_data: PhantomData
        }
    }
}

impl<Shared> StateController for CameraUniformsStateController<Shared>
where
    Shared: Sized + Sync + Send
{
    type SharedState = Shared;
    fn set_state(&mut self, gl: &glow::Context, _: &mut GlStateManager, uniforms: &Weak<RwLock<Uniforms>>, camera: &Camera, _: &Self::SharedState) {
        let uniforms = uniforms.upgrade().unwrap();
        let mut uniforms = uniforms.write().unwrap();

        uniforms.set(gl, &self.view_uniform, camera.view());
        uniforms.set(gl, &self.proj_uniform, camera.projection());
    }
}

pub trait GlData: Sync + Send {
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
