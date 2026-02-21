use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex};
use wgpu::{Buffer, BufferUsages, Device, Queue, RenderPass};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use crate::{geometry, Renderable};
use crate::geometry::{Geometry, GeometryLayout};
use crate::render::{Data, InstanceLayout, PipelineSelector, UniformsSetter};
use crate::render::camera::Camera;

pub(crate) struct BaseRenderer
<GLayout, Primitive, Selector, Uniforms, Shared>
where
    GLayout: GeometryLayout,
    Primitive: geometry::Primitive<Vertex=GLayout::Vertex>,
    Selector: PipelineSelector<Uniforms, Shared>,
    Uniforms: UniformsSetter<Shared>,
    Shared: Send + Sync
{
    pub(crate) geometry: Geometry<GLayout, Primitive>,
    pub(crate) selector: Selector,

    pub(crate) vertex_buffer: Buffer,
    pub(crate) vertex_count: u32,

    pub(crate) geometry_dirty: bool,

    pub(crate) _phantom: PhantomData<(Uniforms, Shared)>
}


pub struct InstancedRenderer
<GLayout, ILayout, Primitive, Selector, Uniforms, Shared>
where
    GLayout: GeometryLayout,
    ILayout: InstanceLayout,
    Primitive: geometry::Primitive<Vertex=GLayout::Vertex>,
    Selector: PipelineSelector<Uniforms, Shared>,
    Uniforms: UniformsSetter<Shared>,
    Shared: Send + Sync
{
    pub(crate) base: BaseRenderer<GLayout, Primitive, Selector, Uniforms, Shared>,
    pub(crate) draw_calls: Arc<Mutex<Vec<ILayout::Data>>>,
    pub(crate) instance_buffer: Buffer,
    pub(crate) instance_count: u32,
}

pub struct ImmediateRenderer
<GLayout, Primitive, Selector, Uniforms, Shared>
where
    GLayout: GeometryLayout,
    Primitive: geometry::Primitive<Vertex=GLayout::Vertex>,
    Selector: PipelineSelector<Uniforms, Shared>,
    Uniforms: UniformsSetter<Shared>,
    Shared: Send + Sync
{
    pub(crate) base: BaseRenderer<GLayout, Primitive, Selector, Uniforms, Shared>
}

/// Lets you modify an instanced mesh's geometry.
/// when this is dropped, it will re-upload the geometry
pub struct InstancedGeometryMut
<'geo, GLayout, ILayout, Primitive, Selector, Uniforms, Shared>
where
    GLayout: GeometryLayout,
    ILayout: InstanceLayout,
    Primitive: geometry::Primitive<Vertex=GLayout::Vertex>,
    Selector: PipelineSelector<Uniforms, Shared>,
    Uniforms: UniformsSetter<Shared>,
    Shared: Send + Sync
{
    renderer: &'geo mut InstancedRenderer<GLayout, ILayout, Primitive, Selector, Uniforms, Shared>,
    device: &'geo Device,
    queue: &'geo Queue,
}


/// Lets you modify an immediate mesh's geometry.
/// when this is dropped, it will re-upload the geometry
pub struct ImmediateGeometryMut
<'geo, GLayout, Primitive, Selector, Uniforms, Shared>
where
    GLayout: GeometryLayout,
    Primitive: geometry::Primitive<Vertex=GLayout::Vertex>,
    Selector: PipelineSelector<Uniforms, Shared>,
    Uniforms: UniformsSetter<Shared>,
    Shared: Send + Sync
{
    renderer: &'geo mut ImmediateRenderer<GLayout, Primitive, Selector, Uniforms, Shared>,
    device: &'geo Device,
    queue: &'geo Queue,
}

#[derive(Clone)]
pub struct InstanceDrawer<ILayout>
where
    ILayout: InstanceLayout,
{
    draw_calls: Arc<Mutex<Vec<ILayout::Data>>>
}


impl<GLayout, ILayout, Primitive, Selector, Uniforms, Shared>
InstancedRenderer<GLayout, ILayout, Primitive, Selector, Uniforms, Shared>
where
    GLayout: GeometryLayout,
    ILayout: InstanceLayout,
    Primitive: geometry::Primitive<Vertex=GLayout::Vertex>,
    Selector: PipelineSelector<Uniforms, Shared>,
    Uniforms: UniformsSetter<Shared>,
    Shared: Send + Sync
{
    pub fn geometry_mut<'geo>(
        &'geo mut self, device: &'geo Device, queue: &'geo Queue
    ) -> InstancedGeometryMut<'geo, GLayout, ILayout, Primitive, Selector, Uniforms, Shared> {
        InstancedGeometryMut {
            renderer: self,
            device,
            queue
        }
    }
    pub fn create_drawer(&self) -> InstanceDrawer<ILayout> {
        InstanceDrawer {
            draw_calls: Arc::clone(&self.draw_calls)
        }
    }
}

impl<'geo, GLayout, ILayout, Primitive, Selector, Uniforms, Shared>
InstancedGeometryMut<'geo, GLayout, ILayout, Primitive, Selector, Uniforms, Shared>
where
    GLayout: GeometryLayout,
    ILayout: InstanceLayout,
    Primitive: geometry::Primitive<Vertex=GLayout::Vertex>,
    Selector: PipelineSelector<Uniforms, Shared>,
    Uniforms: UniformsSetter<Shared>,
    Shared: Send + Sync
{
    /// Useful if the geometry has interior mutability that might not be caught if the geometry was only borrowed "immutably"
    pub fn mark_dirty(&mut self) {
        self.renderer.base.geometry_dirty = true;
    }
}

impl<'geo, GLayout, ILayout, Primitive, Selector, Uniforms, Shared>
Drop for
InstancedGeometryMut<'geo, GLayout, ILayout, Primitive, Selector, Uniforms, Shared>
where
    GLayout: GeometryLayout,
    ILayout: InstanceLayout,
    Primitive: geometry::Primitive<Vertex=GLayout::Vertex>,
    Selector: PipelineSelector<Uniforms, Shared>,
    Uniforms: UniformsSetter<Shared>,
    Shared: Send + Sync
{
    fn drop(&mut self) {
        self.renderer.base.reupload(self.device, self.queue)
    }
}

impl<'geo, GLayout, ILayout, Primitive, Selector, Uniforms, Shared>
Deref for
InstancedGeometryMut<'geo, GLayout, ILayout, Primitive, Selector, Uniforms, Shared>
where
    GLayout: GeometryLayout,
    ILayout: InstanceLayout,
    Primitive: geometry::Primitive<Vertex=GLayout::Vertex>,
    Selector: PipelineSelector<Uniforms, Shared>,
    Uniforms: UniformsSetter<Shared>,
    Shared: Send + Sync
{
    type Target = Geometry<GLayout, Primitive>;
    fn deref(&self) -> &Self::Target {
        &self.renderer.base.geometry
    }
}

impl<'geo, GLayout, ILayout, Primitive, Selector, Uniforms, Shared>
DerefMut for
InstancedGeometryMut<'geo, GLayout, ILayout, Primitive, Selector, Uniforms, Shared>
where
    GLayout: GeometryLayout,
    ILayout: InstanceLayout,
    Primitive: geometry::Primitive<Vertex=GLayout::Vertex>,
    Selector: PipelineSelector<Uniforms, Shared>,
    Uniforms: UniformsSetter<Shared>,
    Shared: Send + Sync
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.renderer.base.geometry_dirty = true;
        &mut self.renderer.base.geometry
    }
}

impl<ILayout> InstanceDrawer<ILayout>
where
    ILayout: InstanceLayout
{
    pub fn draw(&self, data: ILayout::Data) {
        self.draw_calls.lock().unwrap().push(data)
    }
}




impl<GLayout, Primitive, Selector, Uniforms, Shared>
ImmediateRenderer<GLayout, Primitive, Selector, Uniforms, Shared>
where
    GLayout: GeometryLayout,
    Primitive: geometry::Primitive<Vertex=GLayout::Vertex>,
    Selector: PipelineSelector<Uniforms, Shared>,
    Uniforms: UniformsSetter<Shared>,
    Shared: Send + Sync
{
    pub fn geometry_mut<'geo>(&'geo mut self, device: &'geo Device, queue: &'geo Queue) -> ImmediateGeometryMut<'geo, GLayout, Primitive, Selector, Uniforms, Shared> {
        ImmediateGeometryMut {
            renderer: self,
            device,
            queue
        }
    }
}

impl<'geo, GLayout, Primitive, Selector, Uniforms, Shared>
ImmediateGeometryMut<'geo, GLayout, Primitive, Selector, Uniforms, Shared>
where
    GLayout: GeometryLayout,
    Primitive: geometry::Primitive<Vertex=GLayout::Vertex>,
    Selector: PipelineSelector<Uniforms, Shared>,
    Uniforms: UniformsSetter<Shared>,
    Shared: Send + Sync
{
    /// Useful if the geometry has interior mutability that might not be caught if the geometry was only borrowed "immutably"
    pub fn mark_dirty(&mut self) {
        self.renderer.base.geometry_dirty = true;
    }
}

impl<'geo, GLayout, Primitive, Selector, Uniforms, Shared>
Drop for
ImmediateGeometryMut<'geo, GLayout, Primitive, Selector, Uniforms, Shared>
where
    GLayout: GeometryLayout,
    Primitive: geometry::Primitive<Vertex=GLayout::Vertex>,
    Selector: PipelineSelector<Uniforms, Shared>,
    Uniforms: UniformsSetter<Shared>,
    Shared: Send + Sync
{
    fn drop(&mut self) {
        self.renderer.base.reupload(self.device, self.queue)
    }
}

impl<'geo, GLayout, Primitive, Selector, Uniforms, Shared>
Deref for
ImmediateGeometryMut<'geo, GLayout, Primitive, Selector, Uniforms, Shared>
where
    GLayout: GeometryLayout,
    Primitive: geometry::Primitive<Vertex=GLayout::Vertex>,
    Selector: PipelineSelector<Uniforms, Shared>,
    Uniforms: UniformsSetter<Shared>,
    Shared: Send + Sync
{
    type Target = Geometry<GLayout, Primitive>;
    fn deref(&self) -> &Self::Target {
        &self.renderer.base.geometry
    }
}

impl<'geo, GLayout, Primitive, Selector, Uniforms, Shared>
DerefMut for
ImmediateGeometryMut<'geo, GLayout, Primitive, Selector, Uniforms, Shared>
where
    GLayout: GeometryLayout,
    Primitive: geometry::Primitive<Vertex=GLayout::Vertex>,
    Selector: PipelineSelector<Uniforms, Shared>,
    Uniforms: UniformsSetter<Shared>,
    Shared: Send + Sync
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.renderer.base.geometry_dirty = true;
        &mut self.renderer.base.geometry
    }
}


impl<GLayout, Primitive, Selector, Uniforms, Shared>
BaseRenderer<GLayout, Primitive, Selector, Uniforms, Shared>
where
    GLayout: GeometryLayout,
    Primitive: geometry::Primitive<Vertex=GLayout::Vertex>,
    Selector: PipelineSelector<Uniforms, Shared>,
    Uniforms: UniformsSetter<Shared>,
    Shared: Send + Sync
{
    fn setup(&self, device: &Device, queue: &Queue, camera: &Camera, shared: &Shared) {
        self.selector.select(shared).setup(device, queue, camera, shared)
    }
    fn render(&mut self, _: &Device, pass: &mut RenderPass, _: &Camera, shared: &Shared) {
        self.selector.select(shared).render(pass);
        pass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
    }

    fn reupload(&mut self, device: &Device, queue: &Queue) {
        if !self.geometry_dirty { return; }
        self.geometry_dirty = false;

        let mut data = Vec::new();
        self.geometry.write(&mut data);

        self.vertex_count = self.geometry.primitives.len() as u32 * Primitive::VERTICES;

        if data.len() > self.vertex_buffer.size() as usize {
            self.vertex_buffer.destroy();
            self.vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("vertex buffer"),
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                contents: data.as_slice()
            })
        } else {
            queue.write_buffer(&self.vertex_buffer, 0, data.as_slice())
        }

    }

}

impl<GLayout, ILayout, Primitive, Selector, Uniforms, Shared>
Renderable<Shared> for
InstancedRenderer<GLayout, ILayout, Primitive, Selector, Uniforms, Shared>
where
    GLayout: GeometryLayout,
    ILayout: InstanceLayout,
    Primitive: geometry::Primitive<Vertex=GLayout::Vertex>,
    Selector: PipelineSelector<Uniforms, Shared>,
    Uniforms: UniformsSetter<Shared>,
    Shared: Send + Sync
{
    fn pre_render(&mut self, device: &Device, queue: &Queue, camera: &Camera, shared: &Shared) {
        self.base.setup(device, queue, camera, shared);

        let data = {
            let mut draws = self.draw_calls.lock().unwrap();
            let mut data = Vec::new();
            for draw in &*draws {
                draw.write(&mut data);
            }
            self.instance_count = draws.len() as u32;
            draws.clear();
            data
        };

        if data.len() > self.instance_buffer.size() as usize {
            self.instance_buffer = device.create_buffer_init(&BufferInitDescriptor {
                label: Some("instance buffer"),
                usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
                contents: data.as_slice(),
            });
        } else {
            queue.write_buffer(&self.instance_buffer, 0, data.as_slice());
        }

    }

    fn render(&mut self, device: &Device, pass: &mut RenderPass, camera: &Camera, shared: &Shared) {
        self.base.render(device, pass, camera, shared);
        pass.set_vertex_buffer(1, self.instance_buffer.slice(..));
        pass.draw(0..self.base.vertex_count, 0..self.instance_count);
    }
}

impl<GLayout, Primitive, Selector, Uniforms, Shared>
Renderable<Shared> for
ImmediateRenderer<GLayout, Primitive, Selector, Uniforms, Shared>
where
    GLayout: GeometryLayout,
    Primitive: geometry::Primitive<Vertex=GLayout::Vertex>,
    Selector: PipelineSelector<Uniforms, Shared>,
    Uniforms: UniformsSetter<Shared>,
    Shared: Send + Sync
{
    fn pre_render(&mut self, device: &Device, queue: &Queue, camera: &Camera, shared: &Shared) {
        self.base.setup(device, queue, camera, shared);
    }

    fn render(&mut self, device: &Device, pass: &mut RenderPass, camera: &Camera, shared: &Shared) {
        self.base.render(device, pass, camera, shared);
        pass.draw(0..self.base.vertex_count, 0..1);
    }
}

