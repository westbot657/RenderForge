use glam::{Mat4, Vec3};

#[derive(Copy, Clone)]
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

    pub fn look_at(eye: Vec3, target: Vec3, up: Vec3) -> Self {
        Self::new(
            Mat4::look_at_lh(eye, target, up),
            Mat4::IDENTITY,
        )
    }

    pub fn update_perspective_projection(&mut self, fov_y_radians: f32, aspect: f32, near: f32, far: f32) {
        self.projection = Mat4::perspective_lh(
            fov_y_radians, aspect,
            near, far
        );
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

impl Default for Camera {
    fn default() -> Self {
        Self::new_perspective(100., 1., 0.1, 4000.)
    }
}
