use wgpu::{Device, Queue, RenderPass};
use crate::render::camera::Camera;

pub mod geometry;
pub mod render;
pub mod builtin;

#[cfg(feature = "debug")]
pub trait SizedThreadSafe: Sized + Sync + Send + std::fmt::Debug {}
#[cfg(not(feature = "debug"))]
pub trait SizedThreadSafe: Sized + Sync + Send {}

#[cfg(feature = "debug")]
impl<T> SizedThreadSafe for T where T: Sized + Sync + Send + std::fmt::Debug {}
#[cfg(not(feature = "debug"))]
impl<T> SizedThreadSafe for T where T: Sized + Sync + Send {}


pub trait Renderable<Shared> : Send + Sync {
    fn pre_render(&mut self, device: &Device, queue: &Queue, camera: &Camera, shared: &Shared) {
        let _ = (device, queue, camera, shared);
    }
    fn render(&mut self, device: &Device, pass: &mut RenderPass, camera: &Camera, shared: &Shared) {
        let _ = (device, pass, camera, shared);
    }
}


struct BaseApp {

}




