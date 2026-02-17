use glam::Mat4;

pub struct Camera {
    view: Mat4,
    projection: Mat4,
}

impl Camera {
    pub fn new(view: Mat4, projection: Mat4) -> Self {
        Self { view, projection }
    }

    pub fn new_perspective(
        fov_y_radians: f32,
        aspect: f32,
        near: f32,
        far: f32,
    ) -> Self {
        Self::new(
            Mat4::IDENTITY,
            Mat4::perspective_lh(
                fov_y_radians, aspect,
                near, far
            )
        )
    }

    pub fn view(&self) -> &Mat4 {
        &self.view
    }

    pub fn view_mut(&mut self) -> &mut Mat4 {
        &mut self.view
    }

    pub fn projection(&self) -> &Mat4 {
        &self.projection
    }

}