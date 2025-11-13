use std::collections::HashMap;
use std::marker::PhantomData;
use std::mem;
use std::os::raw::c_void;
use std::thread::panicking;
use anyhow::Result;
use gl::types::{GLint, GLsizei, GLuint};
use glam::{Vec2, Vec3, Mat4};
use crate::data::*;
use crate::engine::Engine;
use crate::errors::{AttributeError, BufferRenderError};

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

pub trait InstancedMeshTrait {

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

impl<T: InstancedMeshData, K: MeshController<T>> InstancedMeshTrait for InstancedMesh<T, K> {

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
        if !self.freed && !panicking() {
            let mut state = GlStateManager::new();
            self.destroy(&mut state);
            panic!("Instanced mesh was not destroyed before dropping")
        }
    }
}


pub trait VertexRenderController {
    fn set_uniforms(program: GLuint);
}

pub struct VertexRenderer<T: VertexRenderController> {
    buffer: Vec<f32>,
    layout: LayoutMetaData,
    vao: GLuint,
    vbo: GLuint,
    program: GLuint,
    _implicit: PhantomData<T>,
}

impl<T: VertexRenderController> VertexRenderer<T> {

    pub fn new(layout: LayoutMetaData, program: GLuint) -> Self {
        unsafe {
            let mut vao = 0;
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);
            let mut vbo = 0;
            gl::GenBuffers(1, &mut vbo);

            Self {
                buffer: Vec::new(),
                layout,
                vao,
                vbo,
                program,
                _implicit: PhantomData,
            }
        }
    }

    pub fn set_shader(&mut self, program: GLuint) {
        self.program = program;
    }

    pub fn put(&mut self, value: f32) -> &mut Self {
        self.buffer.push(value);
        self
    }

    pub fn put2(&mut self, v0: f32, v1: f32) -> &mut Self {
        self.buffer.push(v0);
        self.buffer.push(v1);
        self
    }

    pub fn put3(&mut self, v0: f32, v1: f32, v2: f32) -> &mut Self {
        self.buffer.push(v0);
        self.buffer.push(v1);
        self.buffer.push(v2);
        self
    }

    pub fn put4(&mut self, v0: f32, v1: f32, v2: f32, v3: f32) -> &mut Self {
        self.buffer.push(v0);
        self.buffer.push(v1);
        self.buffer.push(v2);
        self.buffer.push(v3);
        self
    }

    pub fn put_mat4(&mut self, mat: Mat4) -> &mut Self {
        let m = mat.to_cols_array();
        self.put4(m[0], m[1], m[2], m[3])
            .put4(m[4], m[5], m[6], m[7])
            .put4(m[8], m[9], m[10], m[11])
            .put4(m[12], m[13], m[14], m[15])
    }

    pub fn render(&mut self, gl_state: &mut GlStateManager) -> Result<()> {

        let buf = mem::take(&mut self.buffer);

        if !(buf.len() as u32).is_multiple_of(self.layout.stride) {
            return Err(BufferRenderError::MalformedData.into());
        }
        if !(buf.len() as u32 / self.layout.stride).is_multiple_of(3) {
            return Err(BufferRenderError::IncompleteTriangleData.into());
        }

        let f_size = size_of::<f32>();

        gl_state.bind_vao(self.vao);
        gl_state.use_program(self.program);

        T::set_uniforms(self.program);

        unsafe {
            gl::BindBuffer(gl::ARRAY_BUFFER, self.vbo);
            gl::BufferData(gl::ARRAY_BUFFER, (buf.len() * f_size) as isize, buf.as_ptr() as *const _, gl::STREAM_DRAW);

            let mut pointer = 0u32;

            for (loc, size) in &self.layout.attributes {
                gl::EnableVertexAttribArray(*loc);
                gl::VertexAttribPointer(*loc, *size as GLint, gl::FLOAT, gl::FALSE, (self.layout.stride * f_size as u32) as GLsizei, (pointer * f_size as u32) as *const c_void);
                pointer += *size;
            }

            gl::DrawArrays(gl::TRIANGLES, 0, (buf.len() as u32 / self.layout.stride) as GLsizei);

            for (loc, _) in &self.layout.attributes {
                gl::DisableVertexAttribArray(*loc);
            }


        }

        Ok(())

    }

}

pub struct Vertex {
    parts: Vec<(u8, u8, Vec<f32>)>
}

pub trait BufferFormat {
    fn get_vertex(&self) -> Vertex;
    fn stride(&self) -> usize;
    fn get_sizes(&self) -> Vec<u8>;
}

#[derive(Copy, Clone, Debug)]
pub struct SimpleBufferFormat {
    uv: bool,
    color: bool,
    normal: bool,
}

pub struct ArbitraryBufferFormat {
    /// name: String, idx: u8, size: u8
    attributes: Vec<(String, u8, u8)>,
}


impl BufferFormat for SimpleBufferFormat {
    fn get_vertex(&self) -> Vertex {
        let mut parts = Vec::new();
        parts.push((0, 3, Vec::new()));

        if self.color {
            parts.push((1, 4, Vec::new()))
        }
        if self.normal {
            parts.push((2, 3, Vec::new()))
        }
        if self.uv {
            parts.push((3, 2, Vec::new()))
        }

        Vertex {
            parts
        }
    }

    fn stride(&self) -> usize {
        (if self.uv { 2 } else { 0 }) +
        (if self.normal { 3 } else { 0 }) +
        (if self.color { 4 } else { 0 }) +
        3
    }

    fn get_sizes(&self) -> Vec<u8> {
        let mut v = Vec::new();
        v.push(3);

        if self.color { v.push(4) }
        if self.normal { v.push(3) }
        if self.uv { v.push(2) }

        v
    }

}
impl BufferFormat for ArbitraryBufferFormat {
    fn get_vertex(&self) -> Vertex {
        let mut parts = Vec::new();

        for (_, idx, count) in &self.attributes {
            parts.push((*idx, *count, Vec::new()));
        }

        Vertex {
            parts
        }
    }

    fn stride(&self) -> usize {
        let mut l = 0;

        for (_, _, c) in &self.attributes {
            l += *c as usize
        }

        l
    }

    fn get_sizes(&self) -> Vec<u8> {
        let mut v = Vec::new();

        for (_, _, sz) in &self.attributes {
            v.push(*sz)
        }

        v
    }
}

impl Vertex {
    fn set_property(&mut self, property: u8, values: Vec<f32>) {
        for part in &mut self.parts {
            if part.0 == property && part.1 as usize == values.len() && part.2.is_empty() {
                part.2 = values;
                return;
            }
        }
        panic!("Unable to set property of render buffer")
    }

    fn is_complete(&self) -> bool {
        for (_, count, values) in &self.parts {
            if (*count as usize) != values.len() {
                return false;
            }
        }
        true
    }

    fn is_started(&self) -> bool {
        for (_, _, v) in &self.parts {
            if !v.is_empty() {
                return true;
            }
        }
        false
    }

    fn pack(&mut self) -> Vec<f32> {
        let mut buf = Vec::new();

        for (_, _, data) in &mut self.parts {
            buf.append(data);
        }

        buf
    }

}

pub struct BufferBuilder<F: BufferFormat> {
    format: F,
    current_vertex: Vertex,
    data: Vec<f32>,
    shader: GLuint,
    uniforms: HashMap<String, GLUniform>,
    samplers: HashMap<String, (u32, GLuint)>,
}

impl<F: BufferFormat> BufferBuilder<F> {
    fn new_internal(format: F, shader: GLuint) -> Self {
        let vert = format.get_vertex();
        Self {
            format,
            current_vertex: vert,
            data: Vec::new(),
            shader,
            uniforms: HashMap::new(),
            samplers: HashMap::new(),
        }
    }

    fn push_vertex(&mut self) {
        if self.current_vertex.is_started() && self.current_vertex.is_complete() {
            let mut data = self.current_vertex.pack();
            self.data.append(&mut data);
        }
    }

    pub fn set_uniform(&mut self, name: impl ToString, value: GLUniform) {
        self.uniforms.insert(name.to_string(), value);
    }
    
    pub fn set_sampler(&mut self, name: impl ToString, slot: u32, tex: GLuint) {
        self.samplers.insert(name.to_string(), (slot, tex));
    }

    pub fn render(&mut self, gl_state: &mut GlStateManager) -> Result<()> {

        self.push_vertex();

        let stride = self.format.stride();
        if !(self.data.len() / stride).is_multiple_of(3) {
            return Err(BufferRenderError::IncompleteTriangleData.into())
        }

        unsafe {
            let mut vao = 0;
            let mut vbo = 0;
            gl::GenVertexArrays(1, &mut vao);
            gl::GenBuffers(1, &mut vbo);

            gl::BindVertexArray(vao);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);

            gl::BufferData(
                gl::ARRAY_BUFFER,
                (self.data.len() * size_of::<f32>()) as isize,
                self.data.as_ptr() as *const _,
                gl::STREAM_DRAW
            );

            let mut offset = 0;
            for (i, sz) in self.format.get_sizes().iter().enumerate() {
                gl::EnableVertexAttribArray(i as GLuint);
                gl::VertexAttribPointer(
                    i as GLuint,
                    *sz as GLint,
                    gl::FLOAT,
                    gl::FALSE,
                    (stride * size_of::<f32>()) as GLint,
                    (offset * size_of::<f32>()) as *const _,
                );
                offset += *sz as usize;
            }


            gl_state.use_program(self.shader);
            for (name, uni) in &self.uniforms {
                gl_state.set_uniform(name, *uni);
            }
            
            for (name, (slot, tex)) in &self.samplers {
                gl_state.set_uniform(name, GLUniform::I32(*slot as i32));
                gl_state.bind_texture(*slot, *tex);
            }

            gl::DrawArrays(gl::TRIANGLES, 0, (self.data.len() / stride) as GLsizei);

            gl::BindVertexArray(0);
            gl::DeleteBuffers(1, &vbo);
            gl::DeleteVertexArrays(1, &vao);
        }
        self.data.clear();
        self.current_vertex = self.format.get_vertex();
        Ok(())
    }

}

impl BufferBuilder<SimpleBufferFormat> {
    pub fn new(shader: GLuint, uses_color: bool, uses_normal: bool, uses_uv: bool) -> Self {
        let f = SimpleBufferFormat {
            color: uses_color,
            normal: uses_normal,
            uv: uses_uv,
        };
        Self::new_internal(f, shader)
    }

    pub fn add_vertex(&mut self, vertex: Vec3) -> &mut Self {
        self.push_vertex();

        let mut v = Vec::new();
        vertex.upload_gl(&mut v);
        self.current_vertex.set_property(0, v);
        self
    }

    pub fn set_color(&mut self, color: Color) -> &mut Self {
        let mut v = Vec::new();
        color.upload_gl(&mut v);
        self.current_vertex.set_property(1, v);
        self
    }

    pub fn set_normal(&mut self, normal: Vec3) -> &mut Self {
        let mut v = Vec::new();
        normal.upload_gl(&mut v);
        self.current_vertex.set_property(2, v);
        self
    }

    pub fn set_uv(&mut self, uv: Vec2) -> &mut Self {
        let mut v = Vec::new();
        uv.upload_gl(&mut v);
        self.current_vertex.set_property(3, v);
        self
    }

}


impl BufferBuilder<ArbitraryBufferFormat> {

    pub fn new(shader: GLuint, vertex_format: ArbitraryBufferFormat) -> Self {
        Self::new_internal(vertex_format, shader)
    }

    pub fn add_vertex(&mut self) {
        self.push_vertex();
    }

    pub fn set_value(&mut self, attr: impl ToString, value: Vec<f32>) -> Result<()> {
        let attr = attr.to_string();
        for (name, idx, count) in &self.format.attributes {
            if *name == attr {
                if value.len() == *count as usize {
                    self.current_vertex.set_property(*idx, value);
                    return Ok(());
                } else {
                    return Err(AttributeError::ExpectedSize { expected: *count as usize, found: value.len() }.into())
                }
            }
        }
        Err(AttributeError::InvalidName(attr).into())
    }

}





