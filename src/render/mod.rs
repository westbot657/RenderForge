use std::marker::PhantomData;
use std::ops::Add;
use std::sync::{RwLock, Weak};
use glow::Context;
use crate::render::camera::Camera;
use crate::render::shader::{UniformUploader, Uniforms};
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

pub trait StateController<Shared> : Sync + Send
where
    Shared: Sized + Sync + Send
{
    // type SharedState: Sized + Sync + Send;
    fn set_state(
        &mut self,
        gl: &glow::Context,
        state: &mut GlStateManager,
        uniforms: &Weak<RwLock<Uniforms>>,
        camera: &Camera,
        shared_state: &Shared,
    ) {
        let _ = gl;
        let _ = state;
        let _ = uniforms;
        let _ = camera;
        let _ = shared_state;
    }
}

pub struct EmptyStateController<Shared>(PhantomData<Shared>);

impl<Shared> EmptyStateController<Shared>
where
    Shared: Sized + Sync + Send
{
    pub fn new() -> Self {
        Self(PhantomData)
    }
}

impl<Shared> StateController<Shared> for EmptyStateController<Shared>
where
    Shared: Sized + Sync + Send
{
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

impl<Shared> StateController<Shared> for CameraUniformsStateController<Shared>
where
    Shared: Sized + Sync + Send
{
    // type SharedState = Shared;
    fn set_state(&mut self, gl: &glow::Context, _: &mut GlStateManager, uniforms: &Weak<RwLock<Uniforms>>, camera: &Camera, _: &Shared) {
        let uniforms = uniforms.upgrade().unwrap();
        let mut uniforms = uniforms.write().unwrap();

        uniforms.set(gl, &self.view_uniform, camera.view());
        uniforms.set(gl, &self.proj_uniform, camera.projection());
    }
}

pub struct MultiStateController<Shared> {
    controllers: Vec<Box<dyn StateController<Shared>>>
}

impl<Shared> MultiStateController<Shared>
where
    Shared: Sized + Sync + Send
{
    pub fn new() -> Self {
        Self { controllers: Vec::new() }
    }
}

impl<Shared> StateController<Shared> for MultiStateController<Shared>
where
    Shared: Sized + Sync + Send
{
    fn set_state(&mut self, gl: &Context, state: &mut GlStateManager, uniforms: &Weak<RwLock<Uniforms>>, camera: &Camera, shared_state: &Shared) {
        for controller in &mut self.controllers {
            controller.set_state(gl, state, uniforms, camera, shared_state)
        }
    }
}

pub trait MergeableStateController<Shared: Sync + Send>
where
    Self: StateController<Shared> + Sized + 'static,
    Shared: Sync + Send
{
    fn flatten(self) -> Vec<Box<dyn StateController<Shared>>> {
        vec![Box::new(self)]
    }
}

impl<T, Shared> MergeableStateController<Shared> for T
where
    T: StateController<Shared> + 'static,
    Shared: Sync + Send
{

}

impl<B, Shared> Add<B> for MultiStateController<Shared>
where
    B: MergeableStateController<Shared>,
    Shared: Sync + Send,
{
    type Output = MultiStateController<Shared>;
    fn add(self, rhs: B) -> Self::Output {
        let mut v = self.controllers;
        v.append(&mut rhs.flatten());
        MultiStateController { controllers: v }
    }
}


pub trait GlData : Sync + Send {
    type DataType: UniformUploader;
    fn size(&self) -> usize;
    fn write(&self, buffer: &mut Vec<Self::DataType>);
}

impl GlData for f32 {
    type DataType = f32;
    fn size(&self) -> usize { 1 }
    fn write(&self, buffer: &mut Vec<Self::DataType>){
        buffer.push(*self)
    }
}
impl GlData for [f32; 2] {
    type DataType = f32;
    fn size(&self) -> usize { 2 }
    fn write(&self, buffer: &mut Vec<Self::DataType>) {
        buffer.extend_from_slice(self)
    }
}
impl GlData for [f32; 3] {
    type DataType = f32;
    fn size(&self) -> usize { 3 }
    fn write(&self, buffer: &mut Vec<Self::DataType>) {
        buffer.extend_from_slice(self)
    }
}
impl GlData for [f32; 4] {
    type DataType = f32;
    fn size(&self) -> usize { 4 }
    fn write(&self, buffer: &mut Vec<Self::DataType>) {
        buffer.extend_from_slice(self)
    }
}
impl GlData for [f32; 16] {
    type DataType = f32;
    fn size(&self) -> usize { 16 }
    fn write(&self, buffer: &mut Vec<Self::DataType>) {
        buffer.extend_from_slice(self)
    }
}
impl GlData for glam::Vec2 {
    type DataType = f32;
    fn size(&self) -> usize { 2 }
    fn write(&self, buffer: &mut Vec<Self::DataType>) {
        buffer.extend_from_slice(self.as_ref())
    }
}
impl GlData for glam::Vec3 {
    type DataType = f32;
    fn size(&self) -> usize { 3 }
    fn write(&self, buffer: &mut Vec<Self::DataType>) {
        buffer.extend_from_slice(self.as_ref())
    }
}
impl GlData for glam::Vec4 {
    type DataType = f32;
    fn size(&self) -> usize { 4 }
    fn write(&self, buffer: &mut Vec<Self::DataType>) {
        buffer.extend_from_slice(self.as_ref())
    }
}
impl GlData for glam::Quat {
    type DataType = f32;
    fn size(&self) -> usize { 4 }
    fn write(&self, buffer: &mut Vec<Self::DataType>) {
        buffer.extend_from_slice(self.as_ref())
    }
}
impl GlData for glam::Mat4 {
    type DataType = f32;
    fn size(&self) -> usize { 16 }
    fn write(&self, buffer: &mut Vec<Self::DataType>) {
        buffer.extend_from_slice(self.as_ref())
    }
}

impl GlData for u32 {
    type DataType = u32;
    fn size(&self) -> usize { 1 }
    fn write(&self, buffer: &mut Vec<Self::DataType>) {
        buffer.push(*self)
    }
}
impl GlData for i32 {
    type DataType = i32;
    fn size(&self) -> usize { 1 }
    fn write(&self, buffer: &mut Vec<Self::DataType>) {
        buffer.push(*self)
    }
}
impl GlData for bool {
    type DataType = u32;
    fn size(&self) -> usize { 1 }
    fn write(&self, buffer: &mut Vec<Self::DataType>) {
        buffer.push(if *self { 1 } else { 0 })
    }
}
