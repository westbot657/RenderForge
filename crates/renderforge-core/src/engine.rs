use hecs::World;

use crate::data::GlStateManager;
use crate::registry::Registry;

pub struct Engine {
    pub gl_state: GlStateManager,
    pub registry: Registry,
    pub ecs: World
}



impl Engine {
    
    pub fn new() -> Self {

        Self {
            gl_state: GlStateManager::new(),
            registry: Registry::new(),
            ecs: World::new(),
        }
    }

    pub fn run(self) {
        
        'mainloop: loop {

            break 'mainloop;
        }
    }

}











