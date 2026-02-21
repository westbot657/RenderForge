use std::collections::HashSet;
use std::sync::Arc;
use wgpu::{Device, Queue, RenderPass};
use crate::render::camera::Camera;

pub mod geometry;
pub mod render;
pub mod builtin;

#[cfg(feature = "debug")]
pub trait SizedThreadSafe: Sized + Sync + Send + std::fmt::Debug {}
#[cfg(not(feature = "debug"))]
pub trait SizedThreadSafe: Sized + Sync + Send {}

#[cfg(feature = "debug")]
impl<T> SizedThreadSafe for T where T: Sized + Sync + Send + std::fmt::Debug {}
#[cfg(not(feature = "debug"))]
impl<T> SizedThreadSafe for T where T: Sized + Sync + Send {}


pub trait Renderable<Shared> : Send + Sync {
    fn pre_render(&mut self, device: &Device, queue: &Queue, camera: &Camera, shared: &Shared) {
        let _ = (device, queue, camera, shared);
    }
    fn render(&mut self, device: &Device, pass: &mut RenderPass, camera: &Camera, shared: &Shared) {
        let _ = (device, pass, camera, shared);
    }
}


#[cfg(feature = "egui")]
use egui_winit::winit::{
    application::ApplicationHandler,
    error::EventLoopError,
    event::{DeviceEvent, DeviceId, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{Window, WindowId}
};
use egui_winit::winit::dpi::LogicalSize;
use glam::Mat4;

// core: owns wgpu, winit, egui infrastructure
#[cfg(feature = "egui")]
pub struct CoreApp {
    window:         Arc<Window>,
    surface:        Option<wgpu::Surface<'static>>,
    surface_config: wgpu::SurfaceConfiguration,
    device:         Arc<Device>,
    queue:          Arc<Queue>,
    depth_texture:  wgpu::Texture,
    depth_view:     wgpu::TextureView,
    egui_ctx:       egui::Context,
    egui_winit:     egui_winit::State,
    egui_renderer:  egui_wgpu::Renderer,
    surface_format: wgpu::TextureFormat,
    camera: Camera,
    suppress_keys:  HashSet<KeyCode>,  // keys to not forward to egui
}

#[cfg(feature = "egui")]
impl CoreApp {
    pub fn surface_format(&self) -> wgpu::TextureFormat { self.surface_format }
    pub fn device(&self) -> &Arc<wgpu::Device> { &self.device }
    pub fn queue(&self) -> &Arc<wgpu::Queue> { &self.queue }
    pub fn depth_view(&self) -> &wgpu::TextureView { &self.depth_view }

    pub fn suppress_key(&mut self, key: KeyCode) { self.suppress_keys.insert(key); }
    pub fn unsuppress_key(&mut self, key: KeyCode) { self.suppress_keys.remove(&key); }

    pub fn window(&self) -> &Arc<Window> { &self.window }
    pub fn surface_size(&self) -> (u32, u32) { (self.surface_config.width, self.surface_config.height) }
    pub fn set_camera(&mut self, camera: Camera) { self.camera = camera }
    pub fn camera(&self) -> &Camera { &self.camera }

    // returns consumed flag
    pub fn handle_window_event(&mut self, event: &WindowEvent) -> bool {
        let pass_to_egui = match event {
            WindowEvent::KeyboardInput { event: KeyEvent {
                physical_key: PhysicalKey::Code(code), ..
            }, .. } => !self.suppress_keys.contains(code),
            _ => true,
        };
        if pass_to_egui {
            self.egui_winit.on_window_event(&self.window, event).consumed
        } else {
            false
        }
    }

    pub fn begin_egui(&mut self) -> egui::Context {
        let raw = self.egui_winit.take_egui_input(&self.window);
        self.egui_ctx.begin_pass(raw);
        self.egui_ctx.clone()
    }

    pub fn finish_egui(&mut self, egui_out: egui::FullOutput, view: &wgpu::TextureView) {
        self.egui_winit.handle_platform_output(&self.window, egui_out.platform_output);
        let tris = self.egui_ctx.tessellate(egui_out.shapes, egui_out.pixels_per_point);
        let sd = egui_wgpu::ScreenDescriptor {
            size_in_pixels:   [self.surface_config.width, self.surface_config.height],
            pixels_per_point: egui_out.pixels_per_point,
        };
        for (id, img) in &egui_out.textures_delta.set {
            self.egui_renderer.update_texture(&self.device, &self.queue, *id, img);
        }
        let mut encoder = self.device.create_command_encoder(&Default::default());
        let cmds = self.egui_renderer.update_buffers(&self.device, &self.queue, &mut encoder, &tris, &sd);
        self.queue.submit(std::iter::once(encoder.finish()).chain(cmds));

        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    depth_slice:    None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            }).forget_lifetime();
            self.egui_renderer.render(&mut pass, &tris, &sd);
        }
        self.queue.submit([encoder.finish()]);
        for id in &egui_out.textures_delta.free { self.egui_renderer.free_texture(id); }
    }

    pub fn new(window: Arc<Window>) -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::VULKAN | wgpu::Backends::GL,
            ..Default::default()
        });
        let surface = instance.create_surface(Arc::clone(&window)).unwrap();
        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference:       wgpu::PowerPreference::HighPerformance,
            compatible_surface:     Some(&surface),
            force_fallback_adapter: false,
        })).expect("no adapter");
        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor::default(),
        )).expect("no device");
        let device = Arc::new(device);
        let queue  = Arc::new(queue);
        let size   = window.inner_size();
        let caps   = surface.get_capabilities(&adapter);
        let fmt    = caps.formats.iter().copied().find(|f| f.is_srgb()).unwrap_or(caps.formats[0]);
        let surface_config = wgpu::SurfaceConfiguration {
            usage:                         wgpu::TextureUsages::RENDER_ATTACHMENT,
            format:                        fmt,
            width:                         size.width,
            height:                        size.height,
            present_mode:                  wgpu::PresentMode::Fifo,
            alpha_mode:                    caps.alpha_modes[0],
            view_formats:                  vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surface_config);
        let (depth_texture, depth_view) = create_depth(&device, size.width, size.height);
        let egui_ctx    = egui::Context::default();
        let egui_winit  = egui_winit::State::new(egui_ctx.clone(), egui_ctx.viewport_id(), &window, None, None, None);
        let egui_renderer = egui_wgpu::Renderer::new(&device, fmt, egui_wgpu::RendererOptions {
            msaa_samples: 1, depth_stencil_format: None, dithering: false, predictable_texture_filtering: false,
        });
        Self {
            window, surface: Some(surface), surface_config,
            device, queue, depth_texture, depth_view,
            egui_ctx, egui_winit, egui_renderer,
            surface_format: fmt,
            camera: Camera { view: Mat4::IDENTITY, proj: Mat4::IDENTITY },
            suppress_keys: HashSet::new(),
        }
    }

    pub fn resize(&mut self, w: u32, h: u32) {
        if w == 0 || h == 0 { return; }
        self.surface_config.width  = w;
        self.surface_config.height = h;
        self.surface.as_ref().unwrap().configure(&self.device, &self.surface_config);
        let (dt, dv) = create_depth(&self.device, w, h);
        self.depth_texture = dt;
        self.depth_view    = dv;
    }
}

#[cfg(feature = "egui")]
fn create_depth(device: &Device, w: u32, h: u32) -> (wgpu::Texture, wgpu::TextureView) {
    let tex = device.create_texture(&wgpu::TextureDescriptor {
        label:           Some("depth"),
        size:            wgpu::Extent3d { width: w, height: h, depth_or_array_layers: 1 },
        mip_level_count: 1,
        sample_count:    1,
        dimension:       wgpu::TextureDimension::D2,
        format:          wgpu::TextureFormat::Depth32Float,
        usage:           wgpu::TextureUsages::RENDER_ATTACHMENT,
        view_formats:    &[],
    });
    let view = tex.create_view(&Default::default());
    (tex, view)
}

#[cfg(feature = "egui")]
pub trait GameApp: Sized + 'static {
    fn new(core: &mut CoreApp) -> Self;
    /// Return true to consume event
    fn on_event(&mut self, core: &mut CoreApp, event: &WindowEvent) -> bool;
    fn on_device_event(&mut self, core: &mut CoreApp, event: &DeviceEvent) {}
    fn update(&mut self, core: &mut CoreApp, dt: f32) {}
    fn render(&mut self, core: &mut CoreApp, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView, depth: &wgpu::TextureView);
    fn ui(&mut self, core: &mut CoreApp, ctx: &egui::Context) {}
    fn name() -> &'static str { "App" }
    fn size() -> LogicalSize<u32> { LogicalSize::new(1280, 720) }
}

#[cfg(feature = "egui")]
struct AppRunner<G: GameApp> {
    core:  Option<CoreApp>,
    game:  Option<G>,
    last:  std::time::Instant,
}

#[cfg(feature = "egui")]
impl<G: GameApp> ApplicationHandler for AppRunner<G> {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop.create_window(
                egui_winit::winit::window::WindowAttributes::default()
                    .with_title(G::name())
                    .with_inner_size(G::size())
            ).unwrap()
        );
        let mut core = CoreApp::new(window);
        let game = G::new(&mut core);
        self.core = Some(core);
        self.game = Some(game);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _: WindowId, event: WindowEvent) {
        let (core, game) = match (self.core.as_mut(), self.game.as_mut()) {
            (Some(c), Some(g)) => (c, g),
            _ => return,
        };

        // game gets first chance to consume
        let consumed = game.on_event(core, &event);
        if consumed { return; }

        // then core handles it (egui + resize etc)
        let consumed = core.handle_window_event(&event);

        match &event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::Resized(size) => core.resize(size.width, size.height),
            WindowEvent::RedrawRequested => {
                let now = std::time::Instant::now();
                let dt  = (now - self.last).as_secs_f32().min(0.1);
                self.last = now;

                let (core, game) = (self.core.as_mut().unwrap(), self.game.as_mut().unwrap());

                game.update(core, dt);

                // get surface texture
                let output = match core.surface.as_ref().unwrap().get_current_texture() {
                    Ok(o)  => o,
                    Err(_) => { core.window.request_redraw(); return; }
                };
                let view = output.texture.create_view(&Default::default());

                // scene — game can borrow core freely here
                let mut encoder = core.device.create_command_encoder(&Default::default());
                game.render(core, &mut encoder, &view, &core.depth_view.clone()); // clone view ref to avoid borrow issue
                core.queue.submit([encoder.finish()]);

                // egui — game can borrow core freely here too
                let raw = core.egui_winit.take_egui_input(&core.window);
                let egui_ctx = core.egui_ctx.clone(); // egui::Context is Arc internally, cheap clone
                let egui_out = egui_ctx.run(raw, |ctx| game.ui(core, ctx));

                core.finish_egui(egui_out, &view);

                output.present();
                core.window.request_redraw();
            }
            _ => {}
        }
    }

    fn device_event(&mut self, _: &ActiveEventLoop, _: DeviceId, event: DeviceEvent) {
        if let (Some(c), Some(g)) = (self.core.as_mut(), self.game.as_mut()) {
            g.on_device_event(c, &event);
        }
    }

    fn exiting(&mut self, _: &ActiveEventLoop) {
        self.game = None;
        self.core = None;
    }
}

// entry point helper
#[cfg(feature = "egui")]
pub fn run<G: GameApp>() -> Result<(), EventLoopError> {
    let event_loop = EventLoop::new()?;
    let mut runner = AppRunner::<G> {
        core: None, game: None, last: std::time::Instant::now()
    };
    event_loop.run_app(&mut runner)
}



