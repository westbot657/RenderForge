use std::cell::RefCell;
use std::rc;
use std::rc::Rc;
use glow::HasContext;
use crate::geometry::*;
use crate::render::{Renderer, StateController};
use crate::render::shader::{Shader, Uniforms};
use crate::render::state::{GlStateManager, StateSnapshot};

pub trait InstanceData: Sized + Clone {
    fn write(&self, buffer: &mut Vec<f32>);
}

pub trait InstanceLayout: Sized + Clone {
    type Data: InstanceData;
    fn span(&self) -> usize;
    fn alignments(&self) -> impl Iterator<Item = u32>;
}

pub struct InstancedMesh<Geo, GLayout, ILayout>
where
    Geo: GeoUnit<Vert = GLayout::Vert>,
    GLayout: GeoLayout,
    ILayout: InstanceLayout,
{
    pub(crate) inner: Geometry<Geo, GLayout>,
    pub(crate) instance_layout: ILayout,
    pub(crate) data: Vec<ILayout::Data>,
}

impl<Geo, GLayout, ILayout> InstancedMesh<Geo, GLayout, ILayout>
where
    Geo: GeoUnit<Vert = GLayout::Vert>,
    GLayout: GeoLayout,
    ILayout: InstanceLayout + Default,
{
    pub fn new(geometry: Geometry<Geo, GLayout>) -> Self {
        Self::new_with_layout(geometry, ILayout::default())
    }
}

impl<Geo, GLayout, ILayout> InstancedMesh<Geo, GLayout, ILayout>
where
    Geo: GeoUnit<Vert = GLayout::Vert>,
    GLayout: GeoLayout,
    ILayout: InstanceLayout,
{
    pub fn new_with_layout(geometry: Geometry<Geo, GLayout>, instance_layout: ILayout) -> Self {
        Self {
            inner: geometry,
            instance_layout,
            data: Vec::new()
        }
    }
    
    pub fn add_data(&mut self, data: ILayout::Data) {
        self.data.push(data)
    }
    
    pub fn clear_instance_data(&mut self) {
        self.data.clear()
    }
    
    pub fn get_geo_buffer(&self) -> Vec<f32> {
        self.inner.get_buffer()
    }
    
    pub fn get_instance_buffer(&self) -> Vec<f32> {
        let size = self.instance_layout.span()
            * self.data.len();
        let mut buffer = Vec::with_capacity(size);
        
        for data in &self.data {
            data.write(&mut buffer);
        }
        
        buffer
    }
    
}

impl<Geo, GLayout, ILayout> BufferProvider for InstancedMesh<Geo, GLayout, ILayout>
where
    Geo: GeoUnit<Vert = GLayout::Vert>,
    GLayout: GeoLayout,
    ILayout: InstanceLayout
{
    fn get_buffer(&self) -> Vec<f32> {
        self.get_geo_buffer()
    }
}


enum State {
    Initialized {
        vao: glow::VertexArray,
        vbo: glow::Buffer,
        instance_vbo: glow::Buffer,
        vertex_count: usize,
    },
    Uninitialized,
}

/// Note: Instance attributes are laid out before geometry attributes
pub struct InstancedRenderer<Geo, GLayout, ILayout, StateC>
where
    Geo: GeoUnit<Vert = GLayout::Vert>,
    GLayout: GeoLayout,
    ILayout: InstanceLayout,
    StateC: StateController
{
    mesh: Rc<RefCell<InstancedMesh<Geo, GLayout, ILayout>>>,
    state: State,
    uniforms_ref: rc::Weak<RefCell<Uniforms>>,
    program: glow::Program,
    state_controller: StateC
}

pub struct InstancedDrawer<Geo, GLayout, ILayout>
where
    Geo: GeoUnit<Vert = GLayout::Vert>,
    GLayout: GeoLayout,
    ILayout: InstanceLayout,
{
    mesh: rc::Weak<RefCell<InstancedMesh<Geo, GLayout, ILayout>>>
}


impl<Geo, GLayout, ILayout, StateC> InstancedRenderer<Geo, GLayout, ILayout, StateC>
where
    Geo: GeoUnit<Vert = GLayout::Vert>,
    GLayout: GeoLayout,
    ILayout: InstanceLayout,
    StateC: StateController,
{
    pub fn new(
        mesh: InstancedMesh<Geo, GLayout, ILayout>,
        program: glow::Program,
        uniforms_ref: rc::Weak<RefCell<Uniforms>>,
        state_controller: StateC,
    ) -> Self {
        Self {
            mesh: Rc::new(RefCell::new(mesh)),
            state: State::Uninitialized,
            uniforms_ref,
            program,
            state_controller
        }
    }

    pub fn create_drawer(&self) -> InstancedDrawer<Geo, GLayout, ILayout> {
        InstancedDrawer {
            mesh: Rc::downgrade(&self.mesh)
        }
    }

}


impl<Geo, GLayout, ILayout, StateC> Renderer for InstancedRenderer<Geo, GLayout, ILayout, StateC>
where
    Geo: GeoUnit<Vert = GLayout::Vert>,
    GLayout: GeoLayout,
    ILayout: InstanceLayout,
    StateC: StateController,
{
    fn setup(&mut self, gl: &glow::Context) -> Result<(), String> {

        let (geometry_buffer, primitive_count) = {
            let b = self.mesh.borrow();
            let buf = b.get_geo_buffer();
            let count = b.inner.geometry.len();
            (buf, count)
        };

        if primitive_count == 0 {
            return Err(String::from("Instanced mesh is empty"))
        } else if primitive_count < Geo::MIN_PRIMITIVE_COUNT {
            return Err(String::from("Mesh data does not include enough data to draw"))
        }

        let vertex_count = primitive_count * Geo::VERTEX_COUNT;

        unsafe {

            let mesh = self.mesh.borrow();

            let vao = gl.create_vertex_array()?;
            gl.bind_vertex_array(Some(vao));
            let vbo = gl.create_buffer()?;
            let instance_vbo = gl.create_buffer()?;

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
            gl.buffer_data_u8_slice(
                glow::ARRAY_BUFFER,
                bytemuck::cast_slice(geometry_buffer.as_slice()),
                glow::STATIC_DRAW,
            );

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(instance_vbo));

            let instance_span = (mesh.instance_layout.span() * size_of::<f32>()) as i32;
            let mut instance_attrib = 0u32;
            let mut instance_offset = 0i32;

            for alignment in mesh.instance_layout.alignments() {
                gl.vertex_attrib_pointer_f32(
                    instance_attrib,
                    alignment as i32,
                    glow::FLOAT,
                    false,
                    instance_span,
                    instance_offset,
                );
                gl.enable_vertex_attrib_array(instance_attrib);
                gl.vertex_attrib_divisor(instance_attrib, 1);

                instance_offset += (alignment as usize * size_of::<f32>()) as i32;
                instance_attrib += 1;
            }

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));

            let geo_span = (mesh.inner.layout.span() * size_of::<f32>()) as i32;
            let mut geo_attrib = instance_attrib;
            let mut geo_offset = 0i32;

            for alignment in mesh.inner.layout.alignments() {
                gl.vertex_attrib_pointer_f32(
                    geo_attrib,
                    alignment as i32,
                    glow::FLOAT,
                    false,
                    geo_span,
                    geo_offset,
                );
                gl.enable_vertex_attrib_array(geo_attrib);
                gl.vertex_attrib_divisor(geo_attrib, 0);

                geo_offset += (alignment as usize * size_of::<f32>()) as i32;
                geo_attrib += 1;
            }

            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.bind_vertex_array(None);

            self.state = State::Initialized { vao, vbo, instance_vbo, vertex_count }
        }
        Ok(())
    }

    fn render(&mut self, gl: &glow::Context, state: &mut GlStateManager) {
        match self.state {
            State::Initialized {
                vao, vbo: _,
                instance_vbo, vertex_count
            } => {

                let (instance_buffer, instance_count) = {
                    let mut b = self.mesh.borrow_mut();
                    let buf = b.get_instance_buffer();
                    let count = b.data.len();
                    b.clear_instance_data();
                    (buf, count)
                };
                if instance_count == 0 { return }

                unsafe {
                    gl.use_program(Some(self.program));

                    gl.bind_vertex_array(Some(vao));
                    gl.bind_buffer(glow::ARRAY_BUFFER, Some(instance_vbo));
                    gl.buffer_data_u8_slice(
                        glow::ARRAY_BUFFER,
                        bytemuck::cast_slice(instance_buffer.as_slice()),
                        glow::DYNAMIC_DRAW,
                    );

                    let snap = StateSnapshot::new(state);
                    self.state_controller.set_state(state, &self.uniforms_ref);

                    gl.draw_arrays_instanced(
                        Geo::MODE,
                        0,
                        vertex_count as i32,
                        instance_count as i32,
                    );

                    gl.bind_buffer(glow::ARRAY_BUFFER, None);
                    gl.bind_vertex_array(None);
                    gl.use_program(None);

                    snap.restore(state, gl)

                }

            }
            State::Uninitialized => {
                eprintln!("Attempted to render uninitialized instanced mesh")
            }
        }
    }

    fn destroy(self, gl: &glow::Context) {
        match self.state {
            State::Initialized {
                vao, vbo,
                instance_vbo, vertex_count: _
            } => {
                unsafe {
                    gl.delete_buffer(vbo);
                    gl.delete_buffer(instance_vbo);
                    gl.delete_vertex_array(vao);
                }
            }
            State::Uninitialized => {}
        }
    }

}


impl<Geo, GLayout, ILayout> InstancedDrawer<Geo, GLayout, ILayout>
where
    Geo: GeoUnit<Vert = GLayout::Vert>,
    GLayout: GeoLayout,
    ILayout: InstanceLayout
{
    /// Returns an Err if the Mesh has been dropped
    pub fn draw(&self, data: ILayout::Data) -> Result<(), String> {
        if let Some(mesh) = self.mesh.upgrade() {
            mesh.borrow_mut().add_data(data);
            Ok(())
        } else {
            Err(String::from("InstancedDrawer mesh was dropped"))
        }
    }
}


impl<GLayout, ILayout> Shader<GLayout, ILayout>
where
    GLayout: GeoLayout,
    ILayout: InstanceLayout,
{
    pub fn create_instanced_renderer<Geo, StateC>(
        &self,
        mesh: InstancedMesh<Geo, GLayout, ILayout>,
        state_controller: StateC,
    ) -> InstancedRenderer<Geo, GLayout, ILayout, StateC>
    where
        Geo: GeoUnit<Vert = GLayout::Vert>,
        StateC: StateController,
    {
        InstancedRenderer::new(mesh, self.program, Rc::downgrade(&self.uniforms), state_controller)
    }
}




