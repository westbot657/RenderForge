use hecs::World;

use crate::data::GlStateManager;
use crate::registry::Registry;

pub struct Engine {
    pub gl_state: GlStateManager,
    pub registry: Registry,
    pub ecs: World,
    pub running: bool,
}



impl Engine {
    
    pub fn new() -> Self {

        Self {
            gl_state: GlStateManager::new(),
            registry: Registry::new(),
            ecs: World::new(),
            running: true,
        }
    }

    pub fn run(self) {
        
        'mainloop: loop {
            
        }
    }

}











