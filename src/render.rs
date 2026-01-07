use std::marker::PhantomData;
use crate::types::{Renderable, Shared, WeakShared};

pub struct InstancedRenderer<T: Renderable + Sized> {
    renderable: T,
    instance_data: Shared<Vec<DrawCall<T>>>
}

pub struct DrawCall<T: Renderable> {
    attributes: Vec<Vec<f32>>,
    _phantom: PhantomData<T>
}

pub struct Instance<T: Renderable> {
    instance_data: WeakShared<Vec<DrawCall<T>>>
}

impl<T: Renderable + Sized> InstancedRenderer<T> {

    pub fn new(renderable: T) -> Self {
        Self {
            renderable,
            instance_data: Shared::new(Vec::new()),
        }
    }

    pub fn draw_all(&mut self) {

    }

    pub fn create_instance(&self) -> Instance<T> {
        Instance {
            instance_data: (&*self.instance_data).into()
        }
    }

}


