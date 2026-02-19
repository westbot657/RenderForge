use std::marker::PhantomData;
use wgpu::{CommandBuffer, CommandEncoder, Device, Queue, RenderPass};
use crate::render::camera::Camera;

pub mod camera;
pub mod scene;
pub mod shader;
pub mod state;

pub trait Renderable<Shared: Sync + Send> : Sync + Send {
    fn prepare(&mut self, device: &Device, queue: &Queue, encoder: &mut CommandEncoder, camera: &Camera, shared: &Shared) -> Vec<CommandBuffer>;
    fn render<'r>(&mut self, pass: &mut RenderPass<'r>, camera: &Camera, shared: &Shared);
}

pub trait PipelineSelector<Shared: Sync + Send> : Sized + Sync + Send {
    fn create(pipelines: Vec<wgpu::RenderPipeline>, shared: &Shared) -> Self;
    fn select<'r>(&mut self, shared: &Shared) -> &'r wgpu::RenderPipeline;
}

pub struct Renderer<Selector, Render, Shared>
where
    Selector: PipelineSelector<Shared>,
    Render: Renderable<Shared>,
    Shared: Sync + Send
{
    selector: Selector,
    inner: Render,
    phantom: PhantomData<Shared>
}

impl<Sel, Render, Shared> Renderable<Shared> for Renderer<Sel, Render, Shared>
where
    Sel: PipelineSelector<Shared>,
    Render: Renderable<Shared>,
    Shared: Sync + Send
{
    fn prepare(&mut self, device: &Device, queue: &Queue, encoder: &mut CommandEncoder, camera: &Camera, shared: &Shared) -> Vec<CommandBuffer> {
        self.inner.prepare(device, queue, encoder, camera, shared)
    }
    fn render<'r>(&mut self, pass: &mut RenderPass<'r>, camera: &Camera, shared: &Shared) {
        let pipeline = self.selector.select(shared);
        pass.set_pipeline(pipeline);
        self.inner.render(pass, camera, shared)
    }
}

impl<Sel, Render, Shared> Renderer<Sel, Render, Shared>
where
    Sel: PipelineSelector<Shared>,
    Render: Renderable<Shared>,
    Shared: Sync + Send
{

}

