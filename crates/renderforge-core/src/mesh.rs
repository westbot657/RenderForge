use std::ffi::c_void;
use std::thread::panicking;
use gl::types::{GLint, GLsizei, GLuint};
use glam::Mat4;
use crate::data::GlStateManager;
use crate::engine::Engine;

pub struct LayoutMetaData {
    attributes: Vec<(u32, u32)>,
    stride: u32,
}
impl LayoutMetaData {
    pub fn new(alignments: Vec<(u32, u32)>) -> Self {
        let mut stride = 0;
        for (_, s) in &alignments {
            stride += *s;
        }
        Self {
            attributes: alignments,
            stride
        }
    }
}
pub struct MeshLayout {
    mesh_layout: LayoutMetaData,
    instance_layout: LayoutMetaData,
}
impl MeshLayout {
    pub fn new(mesh_layout: LayoutMetaData, instance_layout: LayoutMetaData) -> Self {
        Self {
            mesh_layout,
            instance_layout,
        }
    }
}

pub trait InstancedMeshData {
    fn get_transform(&self) -> &Mat4;
    fn write_data(&self, buffer: &mut Vec<f32>);
    fn write_mesh(buffer: &mut Vec<f32>);
    fn setup_shader(engine: &mut Engine, program: GLuint);
}

pub trait MeshController<T: InstancedMeshData> {
    fn write_mesh(&mut self, buffer: &mut Vec<f32>);
    fn setup_shader(&mut self, engine: &mut Engine, program: GLuint);
}


pub struct InstancedMesh<T: InstancedMeshData, K: MeshController<T>> {
    draws: Vec<T>,
    data_controller: Option<K>,
    shader: GLuint,
    vertex_count: u32,
    layout: MeshLayout,
    vao: GLuint,
    vbo: GLuint,
    indices_vbo: GLuint,
    instance_vbo: GLuint,
    freed: bool,
}

impl<T: InstancedMeshData, K: MeshController<T>> InstancedMesh<T, K> {

    pub fn new(shader_program: GLuint, vertex_count: u32, layout: MeshLayout, data_controller: Option<K>) -> Self {
        unsafe {
            let mut data_controller = data_controller;
            let mut vao = 0;
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);

            let mut vbos: [GLuint; 3] = [0, 0, 0];
            gl::GenBuffers(3, vbos.as_mut_ptr());

            let vbo = vbos[0];
            let indices_vbo = vbos[1];
            let instance_vbo = vbos[2];

            let stride = layout.mesh_layout.stride;
            let mut buffer = Vec::with_capacity(vertex_count as usize * stride as usize);
            if let Some(controller) = &mut data_controller {
                controller.write_mesh(&mut buffer);
            } else {
                T::write_mesh(&mut buffer);
            }
            let f_size = size_of::<f32>();
            let u_size = size_of::<u32>();

            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(gl::ARRAY_BUFFER, (buffer.len() * f_size) as isize, buffer.as_ptr() as *const _, gl::STATIC_DRAW);

            let mut pointer = 0u32;
            for (loc, size) in &layout.mesh_layout.attributes {
                gl::VertexAttribPointer(*loc, *size as GLint, gl::FLOAT, gl::FALSE, (stride * f_size as u32) as GLsizei, (pointer * f_size as u32) as *const c_void);
                gl::EnableVertexAttribArray(*loc);gl::VertexAttribDivisor(*loc, 0);
                pointer += *size;
            }

            let mut indices_buffer = Vec::with_capacity(vertex_count as usize);
            for i in 0..vertex_count {
                indices_buffer.push(i);
            }
            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, indices_vbo);
            gl::BufferData(gl::ELEMENT_ARRAY_BUFFER, (indices_buffer.len() * u_size) as isize, indices_buffer.as_ptr() as *const c_void, gl::STATIC_DRAW);

            gl::BindBuffer(gl::ARRAY_BUFFER, instance_vbo);
            let stride = layout.instance_layout.stride;
            let mut pointer = 0u32;
            for (loc, size) in &layout.instance_layout.attributes {
                gl::VertexAttribPointer(*loc, *size as GLint, gl::FLOAT, gl::FALSE, (stride * f_size as u32) as GLsizei, (pointer * f_size as u32) as *const c_void);
                gl::EnableVertexAttribArray(*loc);
                gl::VertexAttribDivisor(*loc, 1);
                pointer += size;
            }
            gl::BindVertexArray(0);

            Self {
                draws: Vec::new(),
                vertex_count,
                layout,
                data_controller,
                shader: shader_program,
                vao,
                vbo,
                indices_vbo,
                instance_vbo,
                freed: false
            }
        }
    }

    pub fn destroy(&mut self, gl_state: &mut GlStateManager) {
        gl_state.destroy_vbo_box_array(Box::new([self.vbo, self.indices_vbo, self.instance_vbo]));
        gl_state.destroy_vao(self.vao);
        self.freed = true;
    }

    pub fn draw(&mut self, data: T) {
        self.draws.push(data);
    }
    pub fn cancel_draws(&mut self) {
        self.draws.clear();
    }

}

impl<T: InstancedMeshData, K: MeshController<T>> Drop for InstancedMesh<T, K> {
    fn drop(&mut self) {
        if !self.freed {
            if !panicking() {
                let mut state = GlStateManager::new();
                self.destroy(&mut state);
                panic!("Instanced mesh was not destroyed before dropping")
            }
        }
    }
}

