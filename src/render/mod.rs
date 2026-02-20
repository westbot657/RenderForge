use std::marker::PhantomData;
use wgpu::{CommandBuffer, CommandEncoder, Device, Queue, RenderPass, RenderPipeline};
use crate::render::camera::Camera;

pub mod camera;
pub mod scene;
pub mod shader;
pub mod state;

pub trait Renderable<Shared: Sync + Send> : Sync + Send {
    fn prepare(&mut self, device: &Device, queue: &Queue, encoder: &mut CommandEncoder, camera: &Camera, shared: &Shared) -> Vec<CommandBuffer> {
        let _ = (device, queue, encoder, camera, shared);
        Vec::new()
    }
    fn render<'r>(&mut self, pass: &mut RenderPass<'r>, camera: &Camera, shared: &Shared) {
        let _ = (pass, camera, shared);
    }
}

pub trait PipelineSelector<Shared: Sync + Send> : Sized + Sync + Send {
    fn create(pipelines: Vec<RenderPipeline>, shared: &Shared) -> Result<Self, String>;
    fn select(&mut self, shared: &Shared) -> &RenderPipeline;
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
    pub fn new(selector: Sel, inner: Render) -> Self {
        Self {
            selector,
            inner,
            phantom: PhantomData,
        }
    }
}

pub struct DefaultPipelineSelector {
    pipeline: RenderPipeline
}

impl<Shared> PipelineSelector<Shared> for DefaultPipelineSelector
where
    Shared: Send + Sync
{
    fn create(pipelines: Vec<RenderPipeline>, shared: &Shared) -> Result<Self, String> {
        let pipeline = pipelines
            .into_iter()
            .next()
            .ok_or_else(|| String::from("No pipeline available"))?;
        Ok(Self { pipeline })
    }
    fn select(&mut self, shared: &Shared) -> &RenderPipeline {
        &self.pipeline
    }
}



