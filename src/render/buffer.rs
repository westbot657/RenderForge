use std::cell::RefCell;
use std::rc;
use std::rc::Rc;
use glow::{Context, HasContext};
use crate::geometry::*;
use crate::render::shader::{NullInstanceLayout, Shader, Uniforms};
use crate::render::{Renderer, StateController};
use crate::render::camera::Camera;
use crate::render::state::{GlStateManager, StateSnapshot};

pub struct Buffer<Geo, Layout>
where
    Geo: GeoUnit<Vert = Layout::Vert>,
    Layout: GeoLayout
{
    inner: Geometry<Geo, Layout>
}

impl<Geo, Layout> Buffer<Geo, Layout>
where
    Geo: GeoUnit<Vert = Layout::Vert>,
    Layout: GeoLayout + Default
{
    pub fn new() -> Self {
        Self::new_with_layout(Layout::default())
    }
}

impl<Geo, Layout> Buffer<Geo, Layout>
where
    Geo: GeoUnit<Vert = Layout::Vert>,
    Layout: GeoLayout
{
    pub fn new_with_layout(layout: Layout) -> Self {
        Self {
            inner: Geometry::new_with_layout(layout)
        }
    }

    pub fn clear(&mut self) {
        self.inner.geometry.clear();
    }

}

impl<Geo, Layout> BufferProvider for Buffer<Geo, Layout>
where
    Geo: GeoUnit<Vert = Layout::Vert>,
    Layout: GeoLayout
{
    fn get_buffer(&self) -> Vec<f32> {
        self.inner.get_buffer()
    }
}


impl<Layout> Buffer<Quad<Layout::Vert>, Layout>
where
    Layout: GeoLayout
{
    pub fn add_quad(&mut self, quad: Quad<Layout::Vert>) {
        self.inner.add_quad(quad)
    }
}

impl<Layout> Buffer<Tri<Layout::Vert>, Layout>
where
    Layout: GeoLayout
{
    pub fn add_tri(&mut self, tri: Tri<Layout::Vert>) {
        self.inner.add_tri(tri)
    }
}


pub trait RenderableBuffer : Sized + BufferProvider {
    type Geo: GeoUnit<Vert = <Self::Layout as GeoLayout>::Vert>;
    type Layout: GeoLayout;
    fn layout(&self) -> &Self::Layout;
}

impl<Geo, Layout> RenderableBuffer for Buffer<Geo, Layout>
where
    Geo: GeoUnit<Vert = Layout::Vert>,
    Layout: GeoLayout
{
    type Geo = Geo;
    type Layout = Layout;
    fn layout(&self) -> &Self::Layout {
        &self.inner.layout
    }
}

enum State {
    Initialized {
        vao: glow::VertexArray,
        vbo: glow::Buffer,
    },
    Uninitialized,
}

pub struct BufferRenderer<Geo, Layout, StateC, BufferT>
where
    Geo: GeoUnit<Vert = Layout::Vert> + Sized,
    Layout: GeoLayout + Sized,
    StateC: StateController,
    BufferT: RenderableBuffer<Geo = Geo, Layout = Layout>
{
    buffer: BufferT,
    state: State,
    state_controller: StateC,
    uniforms_ref: rc::Weak<RefCell<Uniforms>>,
    program: glow::Program,
}

impl<Geo, Layout, StateC, BufferT> BufferRenderer<Geo, Layout, StateC, BufferT>
where
    Geo: GeoUnit<Vert = Layout::Vert> + Sized,
    Layout: GeoLayout + Sized,
    StateC: StateController,
    BufferT: RenderableBuffer<Geo = Geo, Layout = Layout>
{
    pub fn new(
        buffer: BufferT,
        program: glow::Program,
        uniforms_ref: rc::Weak<RefCell<Uniforms>>,
        state_controller: StateC,
    ) -> Self {
        Self {
            buffer,
            state: State::Uninitialized,
            uniforms_ref,
            state_controller,
            program
        }
    }
}

impl<Geo, Layout, StateC, BufferT> Renderer for BufferRenderer<Geo, Layout, StateC, BufferT>
where
    Geo: GeoUnit<Vert = Layout::Vert> + Sized,
    Layout: GeoLayout + Sized,
    StateC: StateController,
    BufferT: RenderableBuffer<Geo = Geo, Layout = Layout>
{
    fn setup(&mut self, gl: &Context) -> Result<(), String> {
        unsafe {
            let vao = gl.create_vertex_array()?;
            gl.bind_vertex_array(Some(vao));
            let vbo = gl.create_buffer()?;

            gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));

            let span = (self.buffer.layout().span() * size_of::<f32>()) as i32;
            let mut attrib = 0u32;
            let mut offset = 0i32;

            for alignment in self.buffer.layout().alignments() {
                gl.vertex_attrib_pointer_f32(
                    attrib,
                    alignment as i32,
                    glow::FLOAT,
                    false,
                    span,
                    offset,
                );
                gl.enable_vertex_attrib_array(attrib);
                gl.vertex_attrib_divisor(attrib, 0);

                offset += (alignment as usize * size_of::<f32>()) as i32;
                attrib += 1;
            }

            gl.bind_buffer(glow::ARRAY_BUFFER, None);
            gl.bind_vertex_array(None);

            self.state = State::Initialized { vao, vbo }
        }
        Ok(())
    }

    fn render(
        &mut self,
        gl: &Context,
        state: &mut GlStateManager,
        camera: &Camera,
    ) {
        match self.state {
            State::Initialized { vao, vbo } => {
                let geo_buffer = self.buffer.get_buffer();
                if geo_buffer.is_empty() { return }

                unsafe {
                    gl.use_program(Some(self.program));

                    gl.bind_vertex_array(Some(vao));
                    gl.bind_buffer(glow::ARRAY_BUFFER, Some(vbo));
                    gl.buffer_data_u8_slice(
                        glow::ARRAY_BUFFER,
                        bytemuck::cast_slice(geo_buffer.as_slice()),
                        glow::DYNAMIC_DRAW,
                    );

                    let snap = StateSnapshot::new(state);
                    self.state_controller.set_state(state, &self.uniforms_ref, camera);

                    let vertex_count = (geo_buffer.len() / self.buffer.layout().span()) as i32;
                    gl.draw_arrays(
                        Geo::MODE,
                        0,
                        vertex_count,
                    );

                    gl.bind_buffer(glow::ARRAY_BUFFER, None);
                    gl.bind_vertex_array(None);
                    gl.use_program(None);

                    snap.restore(state, gl);
                }
            }
            State::Uninitialized => {
                eprintln!("Attempted to render uninitialized BufferRenderer");
            }
        }
    }
    fn destroy(self, gl: &Context) {
        match self.state {
            State::Initialized { vao, vbo } => {
                unsafe {
                    gl.delete_buffer(vbo);
                    gl.delete_vertex_array(vao);
                }
            }
            State::Uninitialized => {}
        }
    }
}

impl<GLayout> Shader<GLayout, NullInstanceLayout>
where
    GLayout: GeoLayout
{
    pub fn create_buffer_renderer<Geo, BufferT, StateC>(
        &self,
        buffer: BufferT,
        state_controller: StateC,
    ) -> BufferRenderer<Geo, GLayout, StateC, BufferT>
    where
        BufferT: RenderableBuffer<Geo = Geo, Layout = GLayout>,
        Geo: GeoUnit<Vert = GLayout::Vert>,
        StateC: StateController
    {
        BufferRenderer::new(buffer, self.program, Rc::downgrade(&self.uniforms), state_controller)
    }
}




