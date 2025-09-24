# renderforge.rs

A wrapper around openGL that aims to provide simplicity for 2D and 3D graphics,
including built-in support for instanced, batched, and immediate rendering,
along with tools for textures, texture atlases, fonts and text, and gui systems


## Important Core systems

### Engine
Holds gl and app state
Controls the event loop

### Meshes

#### InstancedMesh
Holds the necessary gl state objects for rendering instanced meshes.  
Due to being in control of the gl state, this class is typically unusable except with Rc<RefCell<>>, Arc<Mutex<>>, or an ECS.

#### BufferBuilder

Simplifies immediate/batched rendering, useful for dynamic geometry or debug rendering.  

- BufferBuilder<ArbitraryBufferFormat>
Requires you to define both the layout, and the shaders.  

- BufferBuilder<SimpleBufferFormat>
Preset with a layout of: `position[, color][, normal][, uv]`  
There are pre-defined shaders for every combination of attributes that can be used, but you can also write your own shaders for it.  



