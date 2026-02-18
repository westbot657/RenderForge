use glow::Context;
use crate::render::camera::Camera;
use crate::render::Renderer;
use crate::render::state::GlStateManager;

#[derive(Default)]
pub struct Scene<Shared> {
    components: Vec<Box<dyn Renderer<Shared>>>,
    initialized: bool,
}

impl<Shared> Scene<Shared> {
    pub fn new() -> Self {
        Self::with_components(Vec::new())
    }
    pub fn with_components(components: Vec<Box<dyn Renderer<Shared>>>) -> Self {
        Self {
            components,
            initialized: false,
        }
    }
}

impl<Shared> Renderer<Shared> for Scene<Shared> {
    fn setup(&mut self, gl: &Context) -> Result<(), String> {
        if self.initialized { return Ok(()); }
        self.initialized = true;
        for comp in &mut self.components {
            comp.setup(gl)?
        }
        Ok(())
    }
    fn render(&mut self, gl: &Context, state: &mut GlStateManager, camera: &Camera, shared_state: &Shared) {
        if !self.initialized {
            return;
        }
        for comp in &mut self.components {
            comp.render(gl, state, camera, shared_state)
        }
    }
    fn destroy(&mut self, gl: &Context) {
        if !self.initialized { return }
        for comp in &mut self.components {
            comp.destroy(gl)
        }
    }
}
