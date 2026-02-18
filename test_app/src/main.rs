use std::sync::{Arc, RwLock};
use std::time::Instant;
use eframe::{CreationContext, Frame};
use egui::Ui;
use glam::{Mat4, Vec3};
use glow::Context;
use renderforge::builtin;
use renderforge::builtin::meshes::pos_color::{Vertex as PCVert};
use renderforge::builtin::components::basic::BackgroundColor;
use renderforge::geometry::{GeoUnit, Quad};
use renderforge::render::camera::Camera;
use renderforge::render::scene::Scene;
use renderforge::render::{CameraUniformsStateController, EmptyStateController, MultiStateController, Renderer};
use renderforge::render::instanced::InstancedDrawer;
use renderforge::render::shader::Shader;
use renderforge::render::state::GlStateManager;


struct ShaderLib {
    pos_col: Shader<builtin::meshes::pos_color::Layout, builtin::instanced::pos::Layout>
}
impl ShaderLib {
    fn new(gl: &Context) -> Result<Self, String> {
        Ok(Self {
            pos_col: Shader::new_instanced(
                gl,
                include_str!("../assets/shaders/instanced/pos_col.vsh"),
                include_str!("../assets/shaders/instanced/pos_col.fsh"),
                Default::default(),
                Default::default()
            )?
        })
    }
}

struct SharedState {
    primary_camera: Camera,
    fov: f32,
    shader_lib: ShaderLib,
    last_time: Instant,
    dt: f32,
    cube_spin_speed: f32,
}
impl SharedState {
    fn new(gl: &Context) -> Result<Self, String> {
        Ok(Self {
            primary_camera: Camera::default(),
            fov: 100f32.to_radians(),
            shader_lib: ShaderLib::new(gl)?,
            last_time: Instant::now(),
            dt: 0.,
            cube_spin_speed: 0.5,
        })
    }
}

struct TopBar {

}
impl TopBar {
    fn new() -> Self {
        Self {}
    }
    fn show(&mut self, ctx: &egui::Context, ui: &mut Ui) {
        ui.heading("Test App - Spinning Cube");
        let (rect, res) = ui.allocate_exact_size(
            ui.available_size(),
            egui::Sense::all()
        );
    }
}

struct BottomBar {

}
impl BottomBar {
    fn new() -> Self {
        Self {}
    }
    fn show(&mut self, ctx: &egui::Context, ui: &mut Ui) {
        let (rect, res) = ui.allocate_exact_size(
            ui.available_size(),
            egui::Sense::all()
        );
    }
}

struct LeftPanel {

}
impl LeftPanel {
    fn new() -> Self {
        Self {}
    }
    fn show(&mut self, ctx: &egui::Context, ui: &mut Ui) {
        let (rect, res) = ui.allocate_exact_size(
            ui.available_size(),
            egui::Sense::all()
        );
    }
}

struct RightPanel {
    shared: Arc<RwLock<SharedState>>
}
impl RightPanel {
    fn new(shared: Arc<RwLock<SharedState>>) -> Self {
        Self { shared }
    }
    fn show(&mut self, ctx: &egui::Context, ui: &mut Ui) {

        let _ = ui.allocate_exact_size(
            egui::Vec2::new(ui.available_size().x, 20.),
            egui::Sense::empty()
        );

        let mut speed = self.shared.read().unwrap().cube_spin_speed;

        ui.add(egui::Slider::new(&mut speed, 0.0..=50.0)
            .step_by(0.25)
            .text("Cube Spin Speed"));

        self.shared.write().unwrap().cube_spin_speed = speed;

        let (rect, res) = ui.allocate_exact_size(
            ui.available_size(),
            egui::Sense::all()
        );


    }
}

struct TestDrawer<Geo>
where
    Geo: GeoUnit<Vert = PCVert>
{
    drawer: InstancedDrawer<Geo, builtin::meshes::pos_color::Layout, builtin::instanced::pos::Layout>,
    pos: Mat4
}
impl<Geo> Renderer<Arc<RwLock<SharedState>>> for TestDrawer<Geo>
where
    Geo: GeoUnit<Vert = PCVert>
{
    fn render(&mut self, _: &Context, _: &mut GlStateManager, _: &Camera, state: &Arc<RwLock<SharedState>>) {
        let y= {
            let s = state.read().unwrap();
            s.dt * s.cube_spin_speed
        };

        let rotation = Mat4::from_rotation_y(y);
        self.pos *= rotation;

        self.drawer.draw(builtin::instanced::pos::Data::new(self.pos)).unwrap()
    }
}

struct View3d {
    gl: Arc<Context>,
    shared: Arc<RwLock<SharedState>>,
    scene: Arc<RwLock<Scene<Arc<RwLock<SharedState>>>>>,
}
impl View3d {
    fn new(gl: Arc<Context>, shared: Arc<RwLock<SharedState>>) -> Self {

        let mut s = shared.write().unwrap();

        s.primary_camera = Camera::look_at(
            Vec3::new(0.0, 150.0, -200.0),
            Vec3::new(0.0, 0.0, 0.0),
            Vec3::Y
        );

        let mut cube = s.shader_lib.pos_col.create_geometry::<Quad<PCVert>>();
        let vertices = [
            Vec3::new(-1., -1., -1.),
            Vec3::new(-1., -1., 1.),
            Vec3::new(-1., 1., -1.),
            Vec3::new(-1., 1., 1.),

            Vec3::new(1., -1., -1.),
            Vec3::new(1., -1., 1.),
            Vec3::new(1., 1., -1.),
            Vec3::new(1., 1., 1.),
        ];

        cube.add_quad(Quad::new(
            PCVert::new(vertices[0], ((vertices[0] + 1.) / 2.).extend(1.)),
            PCVert::new(vertices[2], ((vertices[2] + 1.) / 2.).extend(1.)),
            PCVert::new(vertices[3], ((vertices[3] + 1.) / 2.).extend(1.)),
            PCVert::new(vertices[1], ((vertices[1] + 1.) / 2.).extend(1.)),
        ));
        cube.add_quad(Quad::new(
            PCVert::new(vertices[5], ((vertices[5] + 1.) / 2.).extend(1.)),
            PCVert::new(vertices[7], ((vertices[7] + 1.) / 2.).extend(1.)),
            PCVert::new(vertices[6], ((vertices[6] + 1.) / 2.).extend(1.)),
            PCVert::new(vertices[4], ((vertices[4] + 1.) / 2.).extend(1.)),
        ));
        cube.add_quad(Quad::new(
            PCVert::new(vertices[0], ((vertices[0] + 1.) / 2.).extend(1.)),
            PCVert::new(vertices[1], ((vertices[1] + 1.) / 2.).extend(1.)),
            PCVert::new(vertices[5], ((vertices[5] + 1.) / 2.).extend(1.)),
            PCVert::new(vertices[4], ((vertices[4] + 1.) / 2.).extend(1.)),
        ));
        cube.add_quad(Quad::new(
            PCVert::new(vertices[2], ((vertices[2] + 1.) / 2.).extend(1.)),
            PCVert::new(vertices[6], ((vertices[6] + 1.) / 2.).extend(1.)),
            PCVert::new(vertices[7], ((vertices[7] + 1.) / 2.).extend(1.)),
            PCVert::new(vertices[3], ((vertices[3] + 1.) / 2.).extend(1.)),
        ));
        cube.add_quad(Quad::new(
            PCVert::new(vertices[4], ((vertices[4] + 1.) / 2.).extend(1.)),
            PCVert::new(vertices[6], ((vertices[6] + 1.) / 2.).extend(1.)),
            PCVert::new(vertices[2], ((vertices[2] + 1.) / 2.).extend(1.)),
            PCVert::new(vertices[0], ((vertices[0] + 1.) / 2.).extend(1.)),
        ));
        cube.add_quad(Quad::new(
            PCVert::new(vertices[1], ((vertices[1] + 1.) / 2.).extend(1.)),
            PCVert::new(vertices[3], ((vertices[3] + 1.) / 2.).extend(1.)),
            PCVert::new(vertices[7], ((vertices[7] + 1.) / 2.).extend(1.)),
            PCVert::new(vertices[5], ((vertices[5] + 1.) / 2.).extend(1.)),
        ));

        let cube = s.shader_lib.pos_col.create_instanced_mesh(cube);
        let cube = s.shader_lib.pos_col.create_instanced_renderer(
            cube, MultiStateController::new() + CameraUniformsStateController::<Arc<RwLock<SharedState>>>::new("viewMat", "projMat") + EmptyStateController::new()
        );

        let pos = Mat4::from_scale(Vec3::splat(50.));

        let cube_inst = TestDrawer { drawer: cube.create_drawer(), pos };

        drop(s);

        let mut scene = Scene::with_components(vec![
            Box::new(BackgroundColor::splat_rgb(0.5)),
            Box::new(cube_inst),
            Box::new(cube)
        ]);

        scene.setup(gl.as_ref()).unwrap();

        Self {
            gl,
            shared,
            scene: Arc::new(RwLock::new(scene))
        }
    }
    fn show(&mut self, ctx: &egui::Context, ui: &mut Ui) {
        let (rect, res) = ui.allocate_exact_size(
            ui.available_size(),
            egui::Sense::all()
        );

        let shared = Arc::clone(&self.shared);
        let scene = Arc::clone(&self.scene);
        let camera = {
            let mut sh = shared.write().unwrap();
            let fov = sh.fov;
            sh.primary_camera.update_perspective_projection(fov, rect.aspect_ratio(), 0.1, 4000.);
            sh.primary_camera.clone()
        };

        ui.painter().add(egui::PaintCallback {
            rect,
            callback: Arc::new(egui_glow::CallbackFn::new(move |_info, painter| {
                let mut default_state = GlStateManager::new();
                let gl = painter.gl().as_ref();
                default_state.depth_test(gl, true);
                default_state.depth_mask(gl, true);
                default_state.culling(gl, true);
                scene.write().unwrap().render(gl, &mut default_state, &camera, &shared);


            }))
        });

    }
}


struct TestApp {
    top: TopBar,
    bottom: BottomBar,
    left: LeftPanel,
    right: RightPanel,
    view3d: View3d,
    shared: Arc<RwLock<SharedState>>
}

impl TestApp {
    fn new(cc: &CreationContext) -> Result<Self, String> {
        let gl = Arc::clone(&cc.gl.as_ref().expect("GL context is not set properly"));

        let shared = Arc::new(RwLock::new(
            SharedState::new(gl.as_ref())?
        ));

        Ok(Self {
            top: TopBar::new(),
            bottom: BottomBar::new(),
            left: LeftPanel::new(),
            right: RightPanel::new(Arc::clone(&shared)),
            view3d: View3d::new(gl, Arc::clone(&shared)),
            shared,
        })
    }

}

impl eframe::App for TestApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut Frame) {

        {
            let mut s = self.shared.write().unwrap();
            let now = Instant::now();
            s.dt = s.last_time.elapsed().as_secs_f32();
            s.last_time = now;
        }

        egui::TopBottomPanel::top("top")
            .show(ctx, |ui| {
                self.top.show(ctx, ui)
            }
        );

        egui::TopBottomPanel::bottom("bottom")
            .show(ctx, |ui| {
                self.bottom.show(ctx, ui)
            }
        );

        egui::SidePanel::left("left")
            .resizable(true)
            .default_width(200.)
            .show(ctx, |ui| {
                self.left.show(ctx, ui)
            }
        );

        egui::SidePanel::right("right")
            .resizable(true)
            .default_width(200.)
            .show(ctx, |ui| {
                self.right.show(ctx, ui)
            }
        );

        egui::CentralPanel::default()
            .show(ctx, |ui| {
                self.view3d.show(ctx, ui)
            }
        );

        ctx.request_repaint()
    }

    fn on_exit(&mut self, gl: Option<&Context>) {
        self.view3d.scene.write().unwrap().destroy(gl.as_ref().unwrap())
    }
}

pub fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800., 600.]),
        multisampling: 4,
        renderer: eframe::Renderer::Glow,
        ..Default::default()
    };

    eframe::run_native(
        "Test App",
        options,
        Box::new(|cc| Ok(Box::new(TestApp::new(cc)?)))
    )

}