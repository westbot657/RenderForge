use std::sync::Arc;
use eframe::{CreationContext, Frame};
use eframe::epaint::mutex::RwLock;
use eframe::epaint::PaintCallbackInfo;
use eframe::wgpu;
use eframe::wgpu::{ColorTargetState, ShaderStages, TextureFormat};
use eframe::wgpu::util::DeviceExt;
use wgpu::{CommandBuffer, CommandEncoder, Device, Queue, RenderPass};
use egui::Context;
use egui_wgpu::{CallbackResources, CallbackTrait, RenderState, ScreenDescriptor};
use egui_wgpu::wgpu::ColorWrites;
use renderforge::{builtin, quad};
use glam::{Mat4, Vec3, Vec4};
use renderforge::builtin::DynamicUniforms;
use renderforge::builtin::geometry::pos_color::Vertex;
use renderforge::builtin::renderer::instanced::{InstancedDrawer, InstancedRenderer};
use renderforge::geometry::layout::UniformKind;
use renderforge::geometry::primitive::Quad;
use renderforge::render::camera::Camera;
use renderforge::render::{shader, DefaultPipelineSelector, Renderable};
use renderforge::render::scene::Scene;
use renderforge::render::shader::{PipelineConfig, Shader};

pub struct SharedState {
    default_camera: Camera,

    pub wgpu: Arc<RwLock<egui_wgpu::Renderer>>
}

impl SharedState {
    pub fn new(camera: Camera, wgpu: &RenderState) -> Self {
        Self {
            default_camera: camera,
            wgpu: Arc::clone(&wgpu.renderer)
        }
    }
}

pub type Shared = Arc<RwLock<SharedState>>;

#[derive(Clone)]
pub struct TestApp {
    shared: Shared,
    scene: Arc<RwLock<Scene<Shared>>>,
}

struct TestDrawer {
    drawer: InstancedDrawer<builtin::instanced::pos::Layout>,
    pos: Mat4,
    uni_buffer: wgpu::Buffer,
}

impl<Shared: Send + Sync> Renderable<Shared> for TestDrawer {
    fn prepare(&mut self, _: &Device, queue: &Queue, _: &mut CommandEncoder, camera: &Camera, _: &Shared) -> Vec<CommandBuffer> {

        queue.write_buffer(&self.uni_buffer, 0, bytemuck::cast_slice(&[*camera]));

        self.drawer.draw(builtin::instanced::pos::Data(self.pos));
        Vec::new()
    }
}

impl TestApp {
    pub fn new(cc: &CreationContext) -> Result<Self, String> {
        let wgpu = cc.wgpu_render_state.clone().unwrap();

        let shared = Arc::new(RwLock::new(
            SharedState::new(Camera::default(), &wgpu)
        ));

        let compiler = shader::create_gl_compiler().map_err(|e| format!("{e}"))?;

        let uniforms = DynamicUniforms::new()
            .with_uniform("Camera", 0, ShaderStages::VERTEX, UniformKind::Buffer);

        let shader = Shader::new_glsl(
            &wgpu.device, &compiler,
            "instanced/pos_col.vsh", "instanced/pos_col.fsh",
            include_str!("../assets/shaders/instanced/pos_col.vsh"),
            include_str!("../assets/shaders/instanced/pos_col.fsh"),
            builtin::geometry::pos_color::Layout,
            builtin::instanced::pos::Layout,
            uniforms,
        ).map_err(|e| format!("{e}"))?;

        let mut geo = shader.create_geometry::<Quad<_>>();

        geo.quads([
            quad![Vertex:
                Vec3::new(-1., -1., -1.), Vec4::new(0., 0., 0., 1.);
                Vec3::new(-1., -1.,  1.), Vec4::new(0., 0., 1., 1.);
                Vec3::new( 1., -1.,  1.), Vec4::new(1., 0., 1., 1.);
                Vec3::new( 1., -1., -1.), Vec4::new(1., 0., 0., 1.);
            ],
            quad![Vertex:
                Vec3::new( 1.,  1., -1.), Vec4::new(1., 1., 0., 1.);
                Vec3::new( 1.,  1.,  1.), Vec4::new(1., 1., 1., 1.);
                Vec3::new(-1.,  1.,  1.), Vec4::new(0., 1., 1., 1.);
                Vec3::new(-1.,  1., -1.), Vec4::new(0., 1., 0., 1.);
            ],
            quad![Vertex:
                Vec3::new(-1., -1.,  1.), Vec4::new(0., 0., 1., 1.);
                Vec3::new( 1., -1.,  1.), Vec4::new(1., 0., 1., 1.);
                Vec3::new( 1.,  1.,  1.), Vec4::new(1., 1., 1., 1.);
                Vec3::new(-1.,  1.,  1.), Vec4::new(0., 1., 1., 1.);
            ],
            quad![Vertex:
                Vec3::new( 1., -1., -1.), Vec4::new(1., 0., 0., 1.);
                Vec3::new(-1., -1., -1.), Vec4::new(0., 0., 0., 1.);
                Vec3::new(-1.,  1., -1.), Vec4::new(0., 1., 0., 1.);
                Vec3::new( 1.,  1., -1.), Vec4::new(1., 1., 0., 1.);
            ],
            quad![Vertex:
                Vec3::new( 1., -1.,  1.), Vec4::new(1., 0., 1., 1.);
                Vec3::new( 1., -1., -1.), Vec4::new(1., 0., 0., 1.);
                Vec3::new( 1.,  1., -1.), Vec4::new(1., 1., 0., 1.);
                Vec3::new( 1.,  1.,  1.), Vec4::new(1., 1., 1., 1.);
            ],
            quad![Vertex:
                Vec3::new(-1., -1., -1.), Vec4::new(0., 0., 0., 1.);
                Vec3::new(-1., -1.,  1.), Vec4::new(0., 0., 1., 1.);
                Vec3::new(-1.,  1.,  1.), Vec4::new(0., 1., 1., 1.);
                Vec3::new(-1.,  1., -1.), Vec4::new(0., 1., 0., 1.);
            ],
        ]);

        let instanced = InstancedRenderer::new(&wgpu.device, geo, builtin::instanced::pos::Layout, 100);

        let instance = instanced.create_drawer();

        let drawer = TestDrawer {
            drawer: instance,
            pos: Mat4::from_translation(Vec3::new(0., 0., 50.)),
            uni_buffer: wgpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: None,
                size: 16 * 2 * 4,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false
            })
        };

        let renderer = shader.create_renderer::<_, DefaultPipelineSelector, Quad<_>, _>(
            &wgpu.device, instanced, &shared, &[
                PipelineConfig {
                    depth: None,
                    cull: None,
                    targets: vec![Some(
                        ColorTargetState {
                            format: TextureFormat::Rgba8Unorm,
                            blend: None,
                            write_mask: ColorWrites::ALL,
                        }
                    )]
                }
            ]
        )?;

        let scene = Scene::with_components(vec![
            Box::new(drawer),
            Box::new(renderer)
        ]);


        Ok(Self {
            shared,
            scene: Arc::new(RwLock::new(scene)),
        })

    }
}

impl CallbackTrait for TestApp {
    fn prepare(&self, device: &Device, queue: &Queue, _screen_descriptor: &ScreenDescriptor, egui_encoder: &mut CommandEncoder, _callback_resources: &mut CallbackResources) -> Vec<CommandBuffer> {
        let camera = { self.shared.read().default_camera };
        self.scene.write().prepare(device, queue, egui_encoder, &camera, &self.shared)
    }

    fn paint(&self, _info: PaintCallbackInfo, render_pass: &mut RenderPass<'static>, _callback_resources: &CallbackResources) {
        let camera = { self.shared.read().default_camera };
        self.scene.write().render(render_pass, &camera, &self.shared);
    }
}

impl eframe::App for TestApp {
    fn update(&mut self, ctx: &Context, frame: &mut Frame) {

        egui::TopBottomPanel::top("Top")
            .show(ctx, |ui| {
                ui.heading("Test App - Spinning Cube");
                let (_, _) = ui.allocate_exact_size(
                    ui.available_size(),
                    egui::Sense::empty()
                );
            });

        egui::TopBottomPanel::bottom("Bottom")
            .show(ctx, |ui| {
                let (_, _) = ui.allocate_exact_size(
                    ui.available_size(),
                    egui::Sense::empty()
                );
            });

        egui::SidePanel::left("Left")
            .default_width(200.)
            .resizable(true)
            .show(ctx, |ui| {
                let (_, _) = ui.allocate_exact_size(
                    ui.available_size(),
                    egui::Sense::empty()
                );
            });

        egui::SidePanel::right("Right")
            .default_width(200.)
            .resizable(true)
            .show(ctx, |ui| {
                let (_, _) = ui.allocate_exact_size(
                    ui.available_size(),
                    egui::Sense::empty()
                );
            });

        egui::CentralPanel::default()
            .show(ctx, |ui| {
                let (rect, _) = ui.allocate_exact_size(
                    ui.available_size(),
                    egui::Sense::empty(),
                );

                self.shared.write().default_camera.proj = Mat4::perspective_lh(90f32.to_radians(), rect.aspect_ratio(), 0.1, 4000.);

                ui.painter().add(egui_wgpu::Callback::new_paint_callback(
                    rect,
                    self.clone()
                ))

            });

        ctx.request_repaint();
    }
}


pub fn main() -> eframe::Result<()> {

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800., 600.]),
        multisampling: 4,
        renderer: eframe::Renderer::Wgpu,
        wgpu_options: egui_wgpu::WgpuConfiguration {
            wgpu_setup: egui_wgpu::WgpuSetup::CreateNew(
                egui_wgpu::WgpuSetupCreateNew {
                    instance_descriptor: wgpu::InstanceDescriptor {
                        backends: wgpu::Backends::VULKAN | wgpu::Backends::GL,
                        ..Default::default()
                    },
                    ..Default::default()
                },
            ),
            ..Default::default()
        },
        ..Default::default()
    };

    eframe::run_native(
        "Test App",
        options,
        Box::new(|cc| Ok(Box::new(TestApp::new(cc)?)))
    )

}