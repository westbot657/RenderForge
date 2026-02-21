// src/main.rs

use egui::{Frame, Sense};
use glam::{Mat4, Vec3, Vec4};
use wgpu::{Backends, ColorTargetState, CompareFunction, DepthStencilState, Face, FrontFace, TextureFormat};
use winit::{
    event::{DeviceEvent, ElementState, KeyEvent, MouseButton, WindowEvent},
    keyboard::{KeyCode, PhysicalKey},
    window::CursorGrabMode,
};
use renderforge::{quad, Core, App, Renderable, Viewport};
use renderforge::builtin::{geometry, instanced};
use renderforge::builtin::uniforms::CameraUniformLayout;
use renderforge::geometry::primitive::Quad;
use renderforge::render::camera::Camera;
use renderforge::render::renderer::InstanceDrawer;
use renderforge::render::shader::{PipelineConfig, Shader, ShaderLayout, Source};
use renderforge::render::{Scene, SinglePipelineSelector};
use wgpu::{Device, Queue};

// ---- Input state ----

#[derive(Default)]
struct Keys {
    w: bool,
    a: bool,
    s: bool,
    d: bool,
    shift: bool,
    space: bool,
}

// ---- CubeDrawer ----

struct CubeDrawer {
    drawer: InstanceDrawer<instanced::pos::Layout>,
    pos: Mat4,
}

impl<Shared: Sync + Send> Renderable<Shared> for CubeDrawer {
    fn pre_render(&mut self, _: &Device, _: &Queue, _: &Camera, _: &Shared) {
        self.drawer.draw(instanced::pos::Data(self.pos))
    }
}

// ---- Game ----

struct CubeGame {
    scene: Scene<()>,
    pos: Vec3,
    yaw: f32,
    pitch: f32,
    keys: Keys,
    viewport: Viewport,
    mouse_captured: bool,
}

impl CubeGame {
    fn view_matrix(&self) -> Mat4 {
        let dir = Vec3::new(
            self.yaw.sin() * self.pitch.cos(),
            self.pitch.sin(),
            self.yaw.cos() * self.pitch.cos(),
        ).normalize();
        Mat4::look_at_lh(self.pos, self.pos + dir, Vec3::Y)
    }

    fn set_mouse_captured(&mut self, core: &Core, captured: bool) {
        self.mouse_captured = captured;
        let window = core.window();
        if captured {
            let _ = window.set_cursor_grab(CursorGrabMode::Locked)
                .or_else(|_| window.set_cursor_grab(CursorGrabMode::Confined));
            window.set_cursor_visible(false);
        } else {
            let _ = window.set_cursor_grab(CursorGrabMode::None);
            window.set_cursor_visible(true);
        }
    }
}

impl App for CubeGame {
    fn backends() -> Backends {
        Backends::VULKAN | Backends::GL
    }

    fn new(core: &mut Core) -> Self {
        let fmt = core.surface_format();

        let uniforms_layout = CameraUniformLayout {
            name: String::from("Camera"),
            location: 0,
        };
        let camera_setter = uniforms_layout.create_setter();

        let cube_shader = Shader::new_wgsl(
            core.device(),
            Source { name: Some("cube.wgsl"), source: include_str!("cube.wgsl") },
            "vs_main", "fs_main",
            ShaderLayout {
                geometry_layout: geometry::pos_color::Layout(0),
                instance_layout: instanced::pos::Layout(2),
                uniforms_layout,
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
                Vec3::new( 1., 1., -1.), Vec4::new(1., 1., 0., 1.);
                Vec3::new( 1., 1.,  1.), Vec4::new(1., 1., 1., 1.);
                Vec3::new(-1., 1.,  1.), Vec4::new(0., 1., 1., 1.);
                Vec3::new(-1., 1., -1.), Vec4::new(0., 1., 0., 1.);
            ],
            quad![geometry::pos_color::Vertex::new:
                Vec3::new(-1.,  1., 1.), Vec4::new(0., 1., 1., 1.);
                Vec3::new( 1.,  1., 1.), Vec4::new(1., 1., 1., 1.);
                Vec3::new( 1., -1., 1.), Vec4::new(1., 0., 1., 1.);
                Vec3::new(-1., -1., 1.), Vec4::new(0., 0., 1., 1.);
            ],
            quad![geometry::pos_color::Vertex::new:
                Vec3::new( 1.,  1., -1.), Vec4::new(1., 1., 0., 1.);
                Vec3::new(-1.,  1., -1.), Vec4::new(0., 1., 0., 1.);
                Vec3::new(-1., -1., -1.), Vec4::new(0., 0., 0., 1.);
                Vec3::new( 1., -1., -1.), Vec4::new(1., 0., 0., 1.);
            ],
            quad![geometry::pos_color::Vertex::new:
                Vec3::new( 1.,  1.,  1.), Vec4::new(1., 1., 1., 1.);
                Vec3::new( 1.,  1., -1.), Vec4::new(1., 1., 0., 1.);
                Vec3::new( 1., -1., -1.), Vec4::new(1., 0., 0., 1.);
                Vec3::new( 1., -1.,  1.), Vec4::new(1., 0., 1., 1.);
            ],
            quad![geometry::pos_color::Vertex::new:
                Vec3::new(-1.,  1., -1.), Vec4::new(0., 1., 0., 1.);
                Vec3::new(-1.,  1.,  1.), Vec4::new(0., 1., 1., 1.);
                Vec3::new(-1., -1.,  1.), Vec4::new(0., 0., 1., 1.);
                Vec3::new(-1., -1., -1.), Vec4::new(0., 0., 0., 1.);
            ],
        ]);

        let instanced = cube_shader.create_instanced_renderer::<SinglePipelineSelector<_, ()>, _, _, _>(
            core.device(), core.queue(),
            &[PipelineConfig {
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
                }),
            }],
            cube, camera_setter, 20,
        ).unwrap();

        let drawer = instanced.create_drawer();

        let scene = Scene::with_components(vec![
            Box::new(CubeDrawer {
                drawer: drawer.clone(),
                pos: Mat4::IDENTITY,
            }),
            Box::new(CubeDrawer {
                drawer,
                pos: Mat4::from_translation(Vec3::new(3., 0., 0.))
                    * Mat4::from_scale(Vec3::splat(0.5)),
            }),
            Box::new(instanced),
        ]);

        let viewport = core.create_viewport(wgpu::Color::BLACK);

        Self {
            scene,
            pos: Vec3::new(0., 0., -5.),
            yaw: 0.,
            pitch: 0.,
            keys: Keys::default(),
            viewport,
            mouse_captured: false,
        }
    }

    fn on_event(&mut self, core: &mut Core, event: &WindowEvent) -> bool {
        match event {
            WindowEvent::MouseInput {
                state: ElementState::Pressed,
                button: MouseButton::Left,
                ..
            } => {
                if !self.mouse_captured {
                    self.set_mouse_captured(core, true);
                }
                true
            }

            WindowEvent::KeyboardInput {
                event: KeyEvent {
                    physical_key: PhysicalKey::Code(code),
                    state: key_state,
                    ..
                },
                ..
            } => {
                let pressed = *key_state == ElementState::Pressed;
                match code {
                    KeyCode::Escape => {
                        if pressed { self.set_mouse_captured(core, false); }
                        true
                    }
                    KeyCode::KeyW => { self.keys.w = pressed; true }
                    KeyCode::KeyA => { self.keys.a = pressed; true }
                    KeyCode::KeyS => { self.keys.s = pressed; true }
                    KeyCode::KeyD => { self.keys.d = pressed; true }
                    KeyCode::ShiftLeft | KeyCode::ShiftRight => { self.keys.shift = pressed; true }
                    KeyCode::Space => { self.keys.space = pressed; true }
                    _ => false,
                }
            }

            _ => false,
        }
    }

    fn on_device_event(&mut self, _core: &mut Core, event: &DeviceEvent) {
        if !self.mouse_captured { return; }
        if let DeviceEvent::MouseMotion { delta: (dx, dy) } = event {
            self.yaw += *dx as f32 * 0.003;
            self.pitch = (self.pitch - *dy as f32 * 0.003).clamp(-1.5, 1.5);
        }
    }

    fn update(&mut self, core: &mut Core, dt: f32) {
        let speed = 8.0 * dt;
        let forward = Vec3::new( self.yaw.sin(), 0., self.yaw.cos());
        let right = Vec3::new( self.yaw.cos(), 0., -self.yaw.sin());

        if self.keys.w { self.pos += forward * speed; }
        if self.keys.s { self.pos -= forward * speed; }
        if self.keys.a { self.pos -= right * speed; }
        if self.keys.d { self.pos += right * speed; }
        if self.keys.shift { self.pos.y -= speed; }
        if self.keys.space { self.pos.y += speed; }

        let w = core.surface_size().0 as f32;
        let h = core.surface_size().1 as f32;
        let camera = Camera {
            view: self.view_matrix(),
            proj: Mat4::perspective_lh(60f32.to_radians(), w / h.max(1.0), 0.1, 1000.0),
        };

        self.scene.pre_render(core.device(), core.queue(), &camera, &());
        *core.camera_mut() = camera;
    }

    fn ui(&mut self, core: &mut Core, ctx: &egui::Context) {

        egui::CentralPanel::default()
            .frame(Frame::NONE)
            .show(ctx, |ui| {
                let (rect, _) = ui.allocate_exact_size(
                    ui.available_size(),
                    Sense::empty()
                );
                core.render_to_rect(ui, rect, &mut self.viewport, &mut self.scene, &());
            });

        egui::Window::new("Controls").show(ctx, |ui| {
            ui.label(if self.mouse_captured { "ESC — release mouse" } else { "Click to capture mouse" });
            ui.label("WASD — move  |  Shift/Space — down/up");
        });

        egui::Window::new("2")
            .default_pos((300., 50.))
            .default_size((500., 500.))
            .show(ctx, |ui| {
                let (rect, _) = ui.allocate_exact_size(
                    ui.available_size(),
                    Sense::empty()
                );
                core.render_to_rect(ui, rect, &mut self.viewport, &mut self.scene, &());
            });
    }
}

fn main() {
    renderforge::run::<CubeGame>().unwrap();
}