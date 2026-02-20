use std::marker::PhantomData;
use std::sync::{Arc, RwLock};
use wgpu::util::DeviceExt;
use wgpu::{CommandBuffer, CommandEncoder, Device, Queue, RenderPass};
use crate::geometry::draw::Data;
use crate::geometry::Geometry;
use crate::geometry::layout::{GeometryLayout, InstanceLayout};
use crate::geometry::primitive::Primitive;
use crate::render::camera::Camera;
use crate::render::Renderable;

pub struct InstancedRenderer<GLayout, ILayout, Prim>
where
    GLayout: GeometryLayout,
    ILayout: InstanceLayout,
    Prim: Primitive<Vert = GLayout::Vert>,
{
    pub layout: ILayout,
    draws: Arc<RwLock<Vec<ILayout::Data>>>,
    geometry: Geometry<Prim, GLayout>,
    
    vertex_buf: wgpu::Buffer,
    vertex_count: u32,
    instance_buf: wgpu::Buffer,
    max_instances: u32,

    _phantom: PhantomData<(GLayout, Prim)>,
}

pub struct InstancedDrawer<ILayout>
where
    ILayout: InstanceLayout,
{
    draws: Arc<RwLock<Vec<ILayout::Data>>>,
}

impl<ILayout: InstanceLayout> InstancedDrawer<ILayout> {
    pub fn draw(&self, data: ILayout::Data) {
        self.draws.write().unwrap().push(data);
    }

    pub fn draw_many(&self, items: impl IntoIterator<Item = ILayout::Data>) {
        self.draws.write().unwrap().extend(items);
    }
}

impl<GLayout, ILayout, Prim> InstancedRenderer<GLayout, ILayout, Prim>
where
    GLayout: GeometryLayout,
    ILayout: InstanceLayout,
    Prim: Primitive<Vert = GLayout::Vert>,
{
    pub fn new(
        device: &Device,
        geometry: Geometry<Prim, GLayout>,
        layout: ILayout,
        max_instances: u32,
    ) -> Self {
        
        let mut vertex_bytes = Vec::new();
        geometry.write(&mut vertex_bytes);
        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("instanced_vertex_buf"),
            contents: &vertex_bytes,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        let instance_stride = layout.span() as u64;
        let instance_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("instanced_instance_buf"),
            size: instance_stride * max_instances as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            layout,
            draws: Arc::new(RwLock::new(Vec::new())),
            vertex_buf,
            vertex_count: geometry.vertex_count(),
            geometry,
            instance_buf,
            max_instances,
            _phantom: PhantomData
        }
    }

    pub fn create_drawer(&self) -> InstancedDrawer<ILayout> {
        InstancedDrawer { draws: Arc::clone(&self.draws) }
    }
}

impl<GLayout, ILayout, Prim, Shared> Renderable<Shared> for InstancedRenderer<GLayout, ILayout, Prim>
where
    GLayout: GeometryLayout,
    ILayout: InstanceLayout,
    Prim: Primitive<Vert = GLayout::Vert>,
    Shared: Send + Sync,
{
    fn prepare(
        &mut self,
        _device: &Device,
        queue:   &Queue,
        _encoder: &mut CommandEncoder,
        _camera: &Camera,
        _shared: &Shared,
    ) -> Vec<CommandBuffer> {
        let mut draws = self.draws.write().unwrap();

        if !draws.is_empty() {
            let count = draws.len().min(self.max_instances as usize);
            
            // TODO: re-allocate buffer if there's too many instances to fit currently?
            
            let mut draw_data = Vec::new();
            for draw in draws.iter() {
                draw.write(&mut draw_data);
            }
            
            queue.write_buffer(
                &self.instance_buf,
                0,
                bytemuck::cast_slice(draw_data.as_slice()),
            );
        }

        draws.clear();

        Vec::new()
    }

    fn render<'r>(&mut self, pass: &mut RenderPass<'r>, _camera: &Camera, _shared: &Shared) {
        let instance_count = {
            self.draws.read().unwrap().len() as u32
        };

        if instance_count == 0 { return; }

        pass.set_vertex_buffer(0, self.vertex_buf.slice(..));
        pass.set_vertex_buffer(1, self.instance_buf.slice(..));
        pass.draw(0..self.vertex_count, 0..instance_count.min(self.max_instances));
    }
}