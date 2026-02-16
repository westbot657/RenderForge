pub mod batched;
pub mod immediate;
pub mod instanced;
mod shader;

pub trait Renderable: Sized {
    fn setup(&mut self) {}
    fn render(&mut self);
}

