use std::sync::Arc;
use eframe::{CreationContext, Frame};
use egui::Ui;
use glow::Context;

struct SharedState {
    
}


struct TestApp {
    gl: Arc<Context>
}

impl TestApp {
    fn new(cc: &CreationContext) -> Self {
        let gl = Arc::clone(&cc.gl.as_ref().expect("GL context is not set properly"));

        Self { gl }
    }

    pub fn show(&mut self, ui: &mut Ui) {

    }

}

impl eframe::App for TestApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            self.show(ui)
        });

        ctx.request_repaint()
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
        Box::new(|cc| Ok(Box::new(TestApp::new(cc))))
    )

}