use glam::Mat4;

#[derive(Copy, Clone)]
pub struct Camera {
    view: Mat4,
    proj: Mat4,
}

impl Camera {
    pub fn new(view: Mat4, proj: Mat4) -> Self {
        Self { view, proj }
    }
}

impl Default for Camera {
    fn default() -> Self {
        Self::new(Mat4::IDENTITY, Mat4::perspective_lh(90., 1., 0.1, 4000.))
    }
}
