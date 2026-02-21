// src/main.rs

use bytemuck::{Pod, Zeroable};
use egui_wgpu::{RendererOptions, ScreenDescriptor};
use glam::{Mat4, Vec3, Vec4};
use std::sync::Arc;
use wgpu::{ColorTargetState, CompareFunction, DepthStencilState, Device, ExperimentalFeatures, Face, FrontFace, Queue, RenderPass, TextureFormat, Trace};
use wgpu::util::DeviceExt;
use winit::{
    application::ApplicationHandler,
    event::{DeviceEvent, DeviceId, ElementState, KeyEvent, WindowEvent},
    event_loop::{ActiveEventLoop, EventLoop},
    keyboard::{KeyCode, PhysicalKey},
    window::{CursorGrabMode, Window, WindowId},
};
use renderforge::builtin::{geometry, instanced, uniforms};
use renderforge::builtin::uniforms::{CameraUniformLayout, CameraUniformSetter};
use renderforge::geometry::primitive::Quad;
use renderforge::{quad, Renderable};
use renderforge::render::camera::Camera;
use renderforge::render::renderer::InstanceDrawer;
use renderforge::render::shader::{PipelineConfig, Shader, ShaderLayout, Source};
use renderforge::render::{Scene, SinglePipelineSelector};
// ---- GPU types ----

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Vertex {
    pos:   [f32; 3],
    color: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Instance {
    model: [[f32; 4]; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct CameraUniform {
    view: [[f32; 4]; 4],
    proj: [[f32; 4]; 4],
}

// ---- Input state ----

#[derive(Default)]
struct Keys {
    w:     bool,
    a:     bool,
    s:     bool,
    d:     bool,
    shift: bool,
    space: bool,
}

// ---- App ----

struct State {
    window:         Arc<Window>,
    surface:        wgpu::Surface<'static>,
    surface_config: wgpu::SurfaceConfiguration,
    device:         Arc<wgpu::Device>,
    queue:          Arc<wgpu::Queue>,

    // scene
    scene: Scene<()>,

    depth_texture:    wgpu::Texture,
    depth_view:       wgpu::TextureView,

    // egui
    egui_ctx:      egui::Context,
    egui_winit:    egui_winit::State,
    egui_renderer: egui_wgpu::Renderer,

    // camera
    pos:   Vec3,
    yaw:   f32,
    pitch: f32,

    // input
    keys:           Keys,
    mouse_captured: bool,
    last_time:      std::time::Instant,
}

struct CubeDrawer {
    drawer: InstanceDrawer<instanced::pos::Layout>,
    pos: Mat4
}

impl<Shared: Sync + Send> Renderable<Shared> for CubeDrawer {
    fn pre_render(&mut self, _: &Device, _: &Queue, _: &Camera, _: &Shared) {
        self.drawer.draw(instanced::pos::Data(self.pos))
    }
}


impl State {
    fn new(window: Arc<Window>) -> Self {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::GL | wgpu::Backends::VULKAN,
            ..Default::default()
        });

        let surface = instance.create_surface(Arc::clone(&window)).unwrap();

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference:       wgpu::PowerPreference::HighPerformance,
            compatible_surface:     Some(&surface),
            force_fallback_adapter: false,
        })).expect("no adapter");

        let (device, queue) = pollster::block_on(adapter.request_device(
            &wgpu::DeviceDescriptor {
                label:             None,
                required_features: wgpu::Features::empty(),
                required_limits:   wgpu::Limits::downlevel_defaults(),
                memory_hints:      Default::default(),
                experimental_features: ExperimentalFeatures::disabled(),
                trace: Trace::Off,
            },
        )).expect("no device");

        let device = Arc::new(device);
        let queue  = Arc::new(queue);

        let size = window.inner_size();
        let caps = surface.get_capabilities(&adapter);
        let fmt  = caps.formats.iter().copied()
            .find(|f| f.is_srgb())
            .unwrap_or(caps.formats[0]);

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

        // ---- Depth texture ----
        let (depth_texture, depth_view) = create_depth(&device, size.width, size.height);

        let uniforms_layout = CameraUniformLayout {
            name: String::from("Camera"),
            location: 0,
        };

        let camera_setter = uniforms_layout.create_setter();

        let cube_shader = Shader::new_wgsl(
            device.as_ref(),
            Source {
                name: Some("cube.wgsl"),
                source: include_str!("cube.wgsl")
            },
            "vs_main", "fs_main",
            ShaderLayout {
                geometry_layout: geometry::pos_color::Layout(0),
                instance_layout: instanced::pos::Layout(2),
                uniforms_layout
            }
        ).unwrap();

        let mut cube = cube_shader.create_geometry::<Quad<_>>();
        cube.primitives.extend_from_slice(&[
            quad![geometry::pos_color::Vertex::new:
                Vec3::new(-1., -1., -1.), Vec4::new(0., 0., 0., 1.);
                Vec3::new(-1., -1.,  1.), Vec4::new(0., 0., 1., 1.);
                Vec3::new( 1., -1.,  1.), Vec4::new(1., 0., 1., 1.);
                Vec3::new( 1., -1., -1.), Vec4::new(1., 0., 0., 1.);
            ],
            quad![geometry::pos_color::Vertex::new:
                Vec3::new( 1., 1.,-1.), Vec4::new(1.,1.,0.,1.);
                Vec3::new( 1., 1., 1.), Vec4::new(1.,1.,1.,1.);
                Vec3::new(-1., 1., 1.), Vec4::new(0.,1.,1.,1.);
                Vec3::new(-1., 1.,-1.), Vec4::new(0.,1.,0.,1.);
            ],
            quad![geometry::pos_color::Vertex::new:
                Vec3::new(-1., 1., 1.), Vec4::new(0.,1.,1.,1.);
                Vec3::new( 1., 1., 1.), Vec4::new(1.,1.,1.,1.);
                Vec3::new( 1.,-1., 1.), Vec4::new(1.,0.,1.,1.);
                Vec3::new(-1.,-1., 1.), Vec4::new(0.,0.,1.,1.);
            ],
            quad![geometry::pos_color::Vertex::new:
                Vec3::new( 1., 1.,-1.), Vec4::new(1.,1.,0.,1.);
                Vec3::new(-1., 1.,-1.), Vec4::new(0.,1.,0.,1.);
                Vec3::new(-1.,-1.,-1.), Vec4::new(0.,0.,0.,1.);
                Vec3::new( 1.,-1.,-1.), Vec4::new(1.,0.,0.,1.);
            ],
            quad![geometry::pos_color::Vertex::new:
                Vec3::new( 1., 1., 1.), Vec4::new(1.,1.,1.,1.);
                Vec3::new( 1., 1.,-1.), Vec4::new(1.,1.,0.,1.);
                Vec3::new( 1.,-1.,-1.), Vec4::new(1.,0.,0.,1.);
                Vec3::new( 1.,-1., 1.), Vec4::new(1.,0.,1.,1.);
            ],
            quad![geometry::pos_color::Vertex::new:
                Vec3::new(-1., 1.,-1.), Vec4::new(0.,1.,0.,1.);
                Vec3::new(-1., 1., 1.), Vec4::new(0.,1.,1.,1.);
                Vec3::new(-1.,-1., 1.), Vec4::new(0.,0.,1.,1.);
                Vec3::new(-1.,-1.,-1.), Vec4::new(0.,0.,0.,1.);
            ]
        ]);

        let instanced = cube_shader.create_instanced_renderer::<SinglePipelineSelector<_, ()>, _, _, _>(
            &device, &queue, &[
                PipelineConfig {
                    cull_mode: Some(Face::Back),
                    front_face: FrontFace::Ccw,
                    targets: vec![Some(ColorTargetState {
                        format: fmt,
                        blend: None,
                        write_mask: Default::default(),
                    })],
                    depth_stencil: Some(DepthStencilState {
                        format: TextureFormat::Depth32Float,
                        depth_write_enabled: true,
                        depth_compare: CompareFunction::Less,
                        stencil: Default::default(),
                        bias: Default::default(),
                    })
                }
            ], cube, camera_setter, 20
        ).unwrap();

        let drawer = instanced.create_drawer();

        let c1 = CubeDrawer {
            drawer: drawer.clone(),
            pos: Mat4::IDENTITY,
        };

        let c2 = CubeDrawer {
            drawer,
            pos: Mat4::from_translation(Vec3::new(3., 0., 0.)) * Mat4::from_scale(Vec3::splat(0.5))
        };

        let scene = Scene::with_components(vec![
            Box::new(c1),
            Box::new(c2),

            Box::new(instanced),
        ]);


        // ---- egui ----
        let egui_ctx = egui::Context::default();
        let egui_winit = egui_winit::State::new(
            egui_ctx.clone(),
            egui_ctx.viewport_id(),
            &window,
            None,
            None,
            None,
        );
        let egui_renderer = egui_wgpu::Renderer::new(&device, fmt, RendererOptions {
            msaa_samples: 1,
            depth_stencil_format: None,
            dithering: false,
            predictable_texture_filtering: false,
        });

        Self {
            window,
            surface,
            surface_config,
            device,
            queue,
            scene,
            depth_texture,
            depth_view,
            egui_ctx,
            egui_winit,
            egui_renderer,
            pos:            Vec3::new(0., 0., -5.),
            yaw:            0.,
            pitch:          0.,
            keys:           Keys::default(),
            mouse_captured: false,
            last_time:      std::time::Instant::now(),
        }
    }

    fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 { return; }
        self.surface_config.width  = width;
        self.surface_config.height = height;
        self.surface.configure(&self.device, &self.surface_config);
        let (dt, dv) = create_depth(&self.device, width, height);
        self.depth_texture = dt;
        self.depth_view    = dv;
    }

    fn set_mouse_captured(&mut self, captured: bool) {
        self.mouse_captured = captured;
        if captured {
            let _ = self.window.set_cursor_grab(CursorGrabMode::Locked)
                .or_else(|_| self.window.set_cursor_grab(CursorGrabMode::Confined));
            self.window.set_cursor_visible(false);
        } else {
            let _ = self.window.set_cursor_grab(CursorGrabMode::None);
            self.window.set_cursor_visible(true);
        }
    }

    fn view_matrix(&self) -> Mat4 {
        // LH: positive Z forward
        let dir = Vec3::new(
            self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.cos() * self.pitch.cos(),
        ).normalize();
        Mat4::look_at_lh(self.pos, self.pos + dir, Vec3::Y)
    }

    fn render(&mut self) {
        let now = std::time::Instant::now();
        let dt  = (now - self.last_time).as_secs_f32().min(0.1);
        self.last_time = now;

        // ---- movement ----
        let speed = 8.0 * dt;
        let forward = Vec3::new( self.yaw.sin(), 0.,  self.yaw.cos());
        let right   = Vec3::new( self.yaw.cos(), 0., -self.yaw.sin());

        if self.keys.w     { self.pos += forward * speed; }
        if self.keys.s     { self.pos -= forward * speed; }
        if self.keys.a     { self.pos -= right   * speed; }
        if self.keys.d     { self.pos += right   * speed; }
        if self.keys.shift { self.pos.y -= speed; }
        if self.keys.space { self.pos.y += speed; }

        // ---- camera uniform ----
        let w = self.surface_config.width  as f32;
        let h = self.surface_config.height as f32;
        let camera = Camera {
            view: self.view_matrix(),
            proj: Mat4::perspective_lh(60f32.to_radians(), w / h.max(1.0), 0.1, 1000.0),
        };

        self.scene.pre_render(&self.device, &self.queue, &camera, &());

        // ---- egui ----
        let raw_input = self.egui_winit.take_egui_input(&self.window);
        let egui_output = self.egui_ctx.run(raw_input, |ctx| {
            egui::Window::new("Controls").show(ctx, |ui| {
                ui.label(if self.mouse_captured {
                    "ESC — release mouse"
                } else {
                    "Click to capture mouse"
                });
                ui.label("WASD — move  |  Shift/Space — down/up");
            });
        });

        // ---- surface ----
        let output = match self.surface.get_current_texture() {
            Ok(o)  => o,
            Err(_) => { self.window.request_redraw(); return; }
        };
        let view = output.texture.create_view(&Default::default());


        // ---- scene pass — submit immediately ----
        {
            let mut encoder = self.device.create_command_encoder(&Default::default());
            {
                let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("scene_pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view:           &view,
                        resolve_target: None,
                        depth_slice:    None,
                        ops: wgpu::Operations {
                            load:  wgpu::LoadOp::Clear(wgpu::Color { r: 0.1, g: 0.1, b: 0.15, a: 1.0 }),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &self.depth_view,
                        depth_ops: Some(wgpu::Operations {
                            load:  wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Discard,
                        }),
                        stencil_ops: None,
                    }),
                    ..Default::default()
                });
                self.scene.render(&self.device, &mut pass, &camera, &());
            }
            self.queue.submit([encoder.finish()]);
        }

        // ---- egui pass — separate encoder ----
        self.egui_winit.handle_platform_output(&self.window, egui_output.platform_output);
        let tris = self.egui_ctx.tessellate(egui_output.shapes, egui_output.pixels_per_point);
        let sd = ScreenDescriptor {
            size_in_pixels:   [self.surface_config.width, self.surface_config.height],
            pixels_per_point: egui_output.pixels_per_point,
        };
        for (id, img) in &egui_output.textures_delta.set {
            self.egui_renderer.update_texture(&self.device, &self.queue, *id, img);
        }
        // egui upload — returns command buffers for any texture uploads
        let mut encoder = self.device.create_command_encoder(&Default::default());
        let egui_commands = self.egui_renderer.update_buffers(
            &self.device, &self.queue, &mut encoder, &tris, &sd
        );

        self.queue.submit(
            std::iter::once(encoder.finish()).chain(egui_commands)
        );

        let view = Arc::new(output.texture.create_view(&Default::default()));

        let mut encoder = self.device.create_command_encoder(&Default::default());
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("egui_pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view:           &view,
                    resolve_target: None,
                    depth_slice:    None,
                    ops: wgpu::Operations {
                        load:  wgpu::LoadOp::Load,
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                ..Default::default()
            }).forget_lifetime();

            self.egui_renderer.render(&mut pass, &tris, &sd);
        }
        self.queue.submit([encoder.finish()]);

        for id in &egui_output.textures_delta.free {
            self.egui_renderer.free_texture(id);
        }

        // view_clone dropped here, output still alive
        output.present();
        self.window.request_redraw();
    }
}

// ---- depth helper ----

fn create_depth(device: &wgpu::Device, w: u32, h: u32) -> (wgpu::Texture, wgpu::TextureView) {
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

// ---- winit ApplicationHandler ----

#[derive(Default)]
struct App {
    state: Option<State>,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Arc::new(
            event_loop.create_window(
                winit::window::WindowAttributes::default()
                    .with_title("Cube Instances")
                    .with_inner_size(winit::dpi::LogicalSize::new(1280u32, 720u32))
            ).unwrap()
        );
        self.state = Some(State::new(window));
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let state = match self.state.as_mut() { Some(s) => s, None => return };

        // feed egui first
        let resp = state.egui_winit.on_window_event(&state.window, &event);
        if resp.consumed { return; }

        match event {
            WindowEvent::CloseRequested => event_loop.exit(),

            WindowEvent::Resized(size) => {
                state.resize(size.width, size.height);
            }

            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: winit::event::MouseButton::Left,
                ..
            } => {
                if !state.mouse_captured {
                    state.set_mouse_captured(true);
                }
            }

            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    physical_key: PhysicalKey::Code(code),
                    state: key_state,
                    ..
                },
                ..
            } => {
                let pressed = key_state == ElementState::Pressed;
                match code {
                    KeyCode::Escape => {
                        if pressed { state.set_mouse_captured(false); }
                    }
                    KeyCode::KeyW     => state.keys.w     = pressed,
                    KeyCode::KeyA     => state.keys.a     = pressed,
                    KeyCode::KeyS     => state.keys.s     = pressed,
                    KeyCode::KeyD     => state.keys.d     = pressed,
                    KeyCode::ShiftLeft | KeyCode::ShiftRight => state.keys.shift = pressed,
                    KeyCode::Space    => state.keys.space = pressed,
                    _ => {}
                }
            }

            WindowEvent::RedrawRequested => {
                state.render();
            }

            _ => {}
        }
    }

    fn device_event(&mut self, _: &ActiveEventLoop, _: DeviceId, event: DeviceEvent) {
        let state = match self.state.as_mut() { Some(s) => s, None => return };
        if !state.mouse_captured { return; }

        if let DeviceEvent::MouseMotion { delta: (dx, dy) } = event {
            state.yaw   += dx as f32 * 0.003;
            state.pitch  = (state.pitch - dy as f32 * 0.003).clamp(-1.5, 1.5);
        }
    }
    fn exiting(&mut self, event_loop: &ActiveEventLoop) {
        self.state = None // prevents a segfault :D
    }
}

fn main() {

    let event_loop = EventLoop::new().unwrap();
    event_loop.run_app(&mut App::default()).unwrap();
}