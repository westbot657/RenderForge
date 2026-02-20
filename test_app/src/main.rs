// src/main.rs

use bytemuck::{Pod, Zeroable};
use eframe::egui;
use eframe::wgpu;
use eframe::wgpu::util::DeviceExt;
use egui_wgpu::{CallbackResources, CallbackTrait, RenderState, ScreenDescriptor};
use glam::{Mat4, Vec3, Vec4};
use std::sync::{Arc, Mutex};
use wgpu::{CommandBuffer, CommandEncoder, Device, Queue, RenderPass};

// ---- Vertex: pos + color ----
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    pos:   [f32; 3],
    color: [f32; 4],
}

// ---- Instance: one mat4 per cube ----
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Instance {
    model: [[f32; 4]; 4],
}

// ---- Camera uniform ----
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct CameraUniform {
    view: [[f32; 4]; 4],
    proj: [[f32; 4]; 4],
}

// ---- Everything the GPU needs, created once in App::new ----
struct GpuState {
    pipeline:        wgpu::RenderPipeline,
    vertex_buf:      wgpu::Buffer,
    vertex_count:    u32,
    instance_buf:    wgpu::Buffer,
    instance_count:  u32,
    camera_buf:      wgpu::Buffer,
    bind_group:      wgpu::BindGroup,
}

struct App {
    gpu: Arc<Mutex<GpuState>>,
    // camera state
    view: Mat4,
    proj: Mat4,
}

impl App {
    fn new(cc: &eframe::CreationContext) -> Self {
        let wgpu = cc.wgpu_render_state.as_ref().expect("wgpu required");
        let device = &wgpu.device;
        let queue  = &wgpu.queue;

        // ---- Shaders ----
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label:  Some("cube_shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("cube.wgsl").into()),
        });

        // ---- Geometry: CCW cube from 6 quads, each expanded to 2 tris ----
        let verts: &[Vertex] = &[
            // bottom (Y-)
            Vertex { pos: [-1.,-1.,-1.], color: [0.,0.,0.,1.] },
            Vertex { pos: [-1.,-1., 1.], color: [0.,0.,1.,1.] },
            Vertex { pos: [ 1.,-1., 1.], color: [1.,0.,1.,1.] },
            Vertex { pos: [ 1.,-1.,-1.], color: [1.,0.,0.,1.] },
            // top (Y+)
            Vertex { pos: [ 1., 1.,-1.], color: [1.,1.,0.,1.] },
            Vertex { pos: [ 1., 1., 1.], color: [1.,1.,1.,1.] },
            Vertex { pos: [-1., 1., 1.], color: [0.,1.,1.,1.] },
            Vertex { pos: [-1., 1.,-1.], color: [0.,1.,0.,1.] },
            // front (Z+)
            Vertex { pos: [-1.,-1., 1.], color: [0.,0.,1.,1.] },
            Vertex { pos: [ 1.,-1., 1.], color: [1.,0.,1.,1.] },
            Vertex { pos: [ 1., 1., 1.], color: [1.,1.,1.,1.] },
            Vertex { pos: [-1., 1., 1.], color: [0.,1.,1.,1.] },
            // back (Z-)
            Vertex { pos: [ 1.,-1.,-1.], color: [1.,0.,0.,1.] },
            Vertex { pos: [-1.,-1.,-1.], color: [0.,0.,0.,1.] },
            Vertex { pos: [-1., 1.,-1.], color: [0.,1.,0.,1.] },
            Vertex { pos: [ 1., 1.,-1.], color: [1.,1.,0.,1.] },
            // right (X+)
            Vertex { pos: [ 1.,-1., 1.], color: [1.,0.,1.,1.] },
            Vertex { pos: [ 1.,-1.,-1.], color: [1.,0.,0.,1.] },
            Vertex { pos: [ 1., 1.,-1.], color: [1.,1.,0.,1.] },
            Vertex { pos: [ 1., 1., 1.], color: [1.,1.,1.,1.] },
            // left (X-)
            Vertex { pos: [-1.,-1.,-1.], color: [0.,0.,0.,1.] },
            Vertex { pos: [-1.,-1., 1.], color: [0.,0.,1.,1.] },
            Vertex { pos: [-1., 1., 1.], color: [0.,1.,1.,1.] },
            Vertex { pos: [-1., 1.,-1.], color: [0.,1.,0.,1.] },
        ];

        // expand quads to tris (6 faces * 2 tris * 3 verts = 36)
        let mut final_verts: Vec<Vertex> = Vec::with_capacity(36);
        for face in 0..6usize {
            let b = face * 4;
            for i in [0,1,2, 0,2,3] {
                final_verts.push(verts[b + i]);
            }
        }

        let vertex_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label:    Some("vertex_buf"),
            contents: bytemuck::cast_slice(&final_verts),
            usage:    wgpu::BufferUsages::VERTEX,
        });

        // ---- 10 instance positions ----
        let positions = [
            Vec3::new(  0.,  0., -10.),
            Vec3::new(  3.,  0., -10.),
            Vec3::new( -3.,  0., -10.),
            Vec3::new(  6.,  0., -15.),
            Vec3::new( -6.,  0., -15.),
            Vec3::new(  0.,  3., -12.),
            Vec3::new(  0., -3., -12.),
            Vec3::new(  4.,  4., -18.),
            Vec3::new( -4., -4., -18.),
            Vec3::new(  0.,  0., -20.),
        ];

        let instances: Vec<Instance> = positions.iter().map(|&p| Instance {
            model: Mat4::from_translation(p).to_cols_array_2d(),
        }).collect();

        let instance_buf = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label:    Some("instance_buf"),
            contents: bytemuck::cast_slice(&instances),
            usage:    wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        });

        // ---- Camera uniform buffer ----
        let camera_buf = device.create_buffer(&wgpu::BufferDescriptor {
            label:              Some("camera_buf"),
            size:               std::mem::size_of::<CameraUniform>() as u64,
            usage:              wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        // ---- Bind group layout + bind group ----
        let bgl = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label:   Some("bgl"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding:    0,
                visibility: wgpu::ShaderStages::VERTEX,
                ty: wgpu::BindingType::Buffer {
                    ty:                 wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size:   None,
                },
                count: None,
            }],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label:   Some("bind_group"),
            layout:  &bgl,
            entries: &[wgpu::BindGroupEntry {
                binding:  0,
                resource: camera_buf.as_entire_binding(),
            }],
        });

        // ---- Pipeline layout ----
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label:                Some("pipeline_layout"),
            bind_group_layouts:   &[&bgl],
            push_constant_ranges: &[],
        });

        // ---- Vertex buffer layouts ----
        // geo: location 0 = pos (Float32x3), location 1 = color (Float32x4)
        let geo_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as u64,
            step_mode:    wgpu::VertexStepMode::Vertex,
            attributes:   &[
                wgpu::VertexAttribute { shader_location: 0, offset:  0, format: wgpu::VertexFormat::Float32x3 },
                wgpu::VertexAttribute { shader_location: 1, offset: 12, format: wgpu::VertexFormat::Float32x4 },
            ],
        };

        // instance: locations 2-5 = mat4 (4 x Float32x4)
        let inst_layout = wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Instance>() as u64,
            step_mode:    wgpu::VertexStepMode::Instance,
            attributes:   &[
                wgpu::VertexAttribute { shader_location: 2, offset:  0, format: wgpu::VertexFormat::Float32x4 },
                wgpu::VertexAttribute { shader_location: 3, offset: 16, format: wgpu::VertexFormat::Float32x4 },
                wgpu::VertexAttribute { shader_location: 4, offset: 32, format: wgpu::VertexFormat::Float32x4 },
                wgpu::VertexAttribute { shader_location: 5, offset: 48, format: wgpu::VertexFormat::Float32x4 },
            ],
        };

        // ---- Pipeline ----
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label:  Some("pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module:              &shader,
                entry_point:         Some("vs_main"),
                buffers:             &[geo_layout, inst_layout],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module:              &shader,
                entry_point:         Some("fs_main"),
                targets:             &[Some(wgpu::ColorTargetState {
                    format:     wgpu.target_format,
                    blend:      Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology:           wgpu::PrimitiveTopology::TriangleList,
                cull_mode:          Some(wgpu::Face::Back),
                front_face:         wgpu::FrontFace::Ccw,
                polygon_mode:       wgpu::PolygonMode::Fill,
                strip_index_format: None,
                unclipped_depth:    false,
                conservative:       false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count:                     4,
                mask:                      !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
            cache:     None,
        });

        let gpu = Arc::new(Mutex::new(GpuState {
            pipeline,
            vertex_buf,
            vertex_count:   36,
            instance_buf,
            instance_count: instances.len() as u32,
            camera_buf,
            bind_group,
        }));

        // register the gpu state so the paint callback can access it
        wgpu.renderer.write().callback_resources.insert(gpu.clone());

        let view = Mat4::look_at_rh(Vec3::new(10., 20., 20.), Vec3::ZERO, Vec3::Y);
        let proj = Mat4::perspective_rh(60f32.to_radians(), 1.0, 0.1, 1000.0);

        Self { gpu, view, proj }
    }
}

// ---- Paint callback ----

struct CubeCallback {
    camera: CameraUniform,
}

impl CallbackTrait for CubeCallback {
    fn prepare(
        &self,
        _device:      &Device,
        queue:        &Queue,
        _screen:      &ScreenDescriptor,
        _encoder:     &mut CommandEncoder,
        resources:    &mut CallbackResources,
    ) -> Vec<CommandBuffer> {
        let gpu = resources.get::<Arc<Mutex<GpuState>>>().unwrap();
        let gpu = gpu.lock().unwrap();
        queue.write_buffer(&gpu.camera_buf, 0, bytemuck::bytes_of(&self.camera));
        vec![]
    }

    fn paint<'a>(
        &'a self,
        _info:      egui::PaintCallbackInfo,
        pass:       &mut RenderPass<'static>,
        resources:  &'a CallbackResources,
    ) {
        let gpu = resources.get::<Arc<Mutex<GpuState>>>().unwrap();
        let gpu = gpu.lock().unwrap();

        pass.set_pipeline(&gpu.pipeline);
        pass.set_bind_group(0, &gpu.bind_group, &[]);
        pass.set_vertex_buffer(0, gpu.vertex_buf.slice(..));
        pass.set_vertex_buffer(1, gpu.instance_buf.slice(..));
        pass.draw(0..gpu.vertex_count, 0..gpu.instance_count);
    }
}

// ---- eframe App ----

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let (rect, _) = ui.allocate_exact_size(ui.available_size(), egui::Sense::empty());

            // update proj for current aspect ratio
            self.proj = Mat4::perspective_rh(
                60f32.to_radians(),
                rect.width() / rect.height().max(1.0),
                0.1,
                1000.0,
            );


            let camera = CameraUniform {
                view: self.view.to_cols_array_2d(),
                proj: self.proj.to_cols_array_2d(),
            };

            ui.painter().add(egui_wgpu::Callback::new_paint_callback(
                rect,
                CubeCallback { camera },
            ));
        });

        ctx.request_repaint();
    }
}

// ---- main ----

fn main() -> eframe::Result<()> {
    // env_logger::init();

    eframe::run_native(
        "Cube Instances",
        eframe::NativeOptions {
            multisampling: 4,
            renderer: eframe::Renderer::Wgpu,
            wgpu_options: egui_wgpu::WgpuConfiguration {
                wgpu_setup: egui_wgpu::WgpuSetup::CreateNew(egui_wgpu::WgpuSetupCreateNew {
                    instance_descriptor: wgpu::InstanceDescriptor {
                        backends: wgpu::Backends::VULKAN | wgpu::Backends::GL,
                        ..Default::default()
                    },
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        },
        Box::new(|cc| Ok(Box::new(App::new(cc)))),
    )
}