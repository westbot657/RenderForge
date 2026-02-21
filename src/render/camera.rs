use bytemuck::Zeroable;
use glam::Mat4;

#[derive(Copy, Clone, bytemuck::Pod, Zeroable)]
#[cfg_attr(feature = "debug", derive(Debug))]
#[repr(C)]
pub struct Camera {
    pub view: Mat4,
    pub proj: Mat4,
}


