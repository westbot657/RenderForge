use std::collections::HashMap;
use wgpu::{Buffer, Device, Queue, ShaderStages};
use crate::render::{UniformEntry, UniformHandle, UniformType, UniformsLayout, UniformsSetter};
use crate::render::camera::Camera;

#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct CameraUniformLayout {
    pub name: String,
    pub location: u32,
}

impl UniformsLayout for CameraUniformLayout {
    fn entries(&self) -> impl Iterator<Item=UniformEntry> {
        [
            UniformEntry {
                name: self.name.clone(),
                location: self.location,
                visibility: ShaderStages::VERTEX,
                uniform_type: UniformType::Buffer {
                    has_dynamic_offset: false,
                    min_binding_size: None,
                    size: size_of::<Camera>() as u64,
                },
            }
        ].into_iter()
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct CameraUniformSetter {
    layout: CameraUniformLayout,
    handle: Option<Buffer>
}

impl Clone for CameraUniformSetter {
    fn clone(&self) -> Self {
        Self {
            layout: self.layout.clone(),
            handle: None,
        }
    }
}

impl<Shared: Send + Sync> UniformsSetter<Shared> for CameraUniformSetter {
    fn bind(&mut self, _: &Device, _: &Queue, mut uniforms: HashMap<String, UniformHandle>) -> Result<(), String> {
        let handle = uniforms.remove(&self.layout.name)
            .ok_or_else(|| String::from("Camera uniform not found"))?;

        let buffer = match handle {
            UniformHandle::Buffer(b) => b,
            _ => return Err(String::from("Invalid uniform type"))
        };

        self.handle = Some(buffer);

        Ok(())
    }
    fn set(&self, _: &Device, queue: &Queue, camera: &Camera, _: &Shared) {
        if let Some(buffer) = &self.handle {
            queue.write_buffer(buffer, 0, bytemuck::bytes_of(camera))
        }
    }
}
