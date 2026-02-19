use wgpu::{CommandBuffer, CommandEncoder, Device, Queue, RenderPass};
use crate::render::camera::Camera;
use crate::render::Renderable;

pub struct Scene<Shared: Sync + Send> {
    components: Vec<Box<dyn Renderable<Shared>>>
}

impl<Shared: Sync + Send> Scene<Shared> {
    pub fn new() -> Self {
        Self::with_components(Vec::new())
    }
    
    pub fn with_components(components: Vec<Box<dyn Renderable<Shared>>>) -> Self {
        Self { components }
    }
}

impl<Shared: Sync + Send> Renderable<Shared> for Scene<Shared> {
    fn prepare(&mut self, device: &Device, queue: &Queue, encoder: &mut CommandEncoder, camera: &Camera, shared: &Shared) -> Vec<CommandBuffer> {
        let mut v = Vec::new();
        for comp in &mut self.components {
            v.append(&mut comp.prepare(device, queue, encoder, camera, shared))
        }
        v
    }

    fn render<'r>(&mut self, pass: &mut RenderPass<'r>, camera: &Camera, shared: &Shared) {
        for comp in &mut self.components {
            comp.render(pass, camera, shared)
        }
    }
}
