use std::sync::Arc;
use eframe::{CreationContext, Frame};
use eframe::epaint::mutex::RwLock;
use eframe::epaint::PaintCallbackInfo;
use eframe::wgpu;
use wgpu::{CommandBuffer, CommandEncoder, Device, Queue, RenderPass};
use egui::{Context, Visuals};
use egui_wgpu::{CallbackResources, CallbackTrait, RenderState, ScreenDescriptor};
use renderforge::render::camera::Camera;
use renderforge::render::{Renderable};
use renderforge::render::scene::Scene;

pub struct SharedState {

    pub wgpu: Arc<RwLock<egui_wgpu::Renderer>>
}

impl SharedState {
    pub fn new(wgpu: &RenderState) -> Self {
        Self {
            wgpu: Arc::clone(&wgpu.renderer)
        }
    }
}

pub type Shared = Arc<RwLock<SharedState>>;

#[derive(Clone)]
pub struct TestApp {
    shared: Shared,
    scene: Arc<RwLock<Scene<Shared>>>,
    default_camera: Camera,
}

impl TestApp {
    pub fn new(cc: &CreationContext) -> Result<Self, String> {
        let wgpu = cc.wgpu_render_state.clone().unwrap();

        let shared = Arc::new(RwLock::new(
            SharedState::new(&wgpu)
        ));

        let mut scene = Scene::with_components(vec![

        ]);


        Ok(Self {
            shared,
            scene: Arc::new(RwLock::new(scene)),
            default_camera: Camera::default(),
        })

    }
}

impl CallbackTrait for TestApp {
    fn prepare(&self, device: &Device, queue: &Queue, _screen_descriptor: &ScreenDescriptor, egui_encoder: &mut CommandEncoder, _callback_resources: &mut CallbackResources) -> Vec<CommandBuffer> {
        self.scene.write().prepare(device, queue, egui_encoder, &self.default_camera, &self.shared)
    }

    fn paint(&self, _info: PaintCallbackInfo, render_pass: &mut RenderPass<'static>, _callback_resources: &CallbackResources) {
        self.scene.write().render(render_pass, &self.default_camera, &self.shared);
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

                ui.painter().add(egui_wgpu::Callback::new_paint_callback(
                    rect,
                    self.clone()
                ))

            });

        ctx.request_repaint();
    }

    fn clear_color(&self, _visuals: &Visuals) -> [f32; 4] {
        [0., 0., 0., 1.]
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
                        backends: wgpu::Backends::GL,
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