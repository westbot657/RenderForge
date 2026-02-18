use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use glow::{Context, HasContext, Program, UniformLocation};
use crate::geometry;
use crate::geometry::{GeoLayout};
use crate::render::GlData;
use crate::render::instanced::{InstanceData, InstanceLayout};


impl InstanceData for () {
    fn write(&self, _: &mut Vec<f32>) {}
}
impl InstanceLayout for () {
    type Data = ();
    fn span(&self) -> usize { 0 }
    fn alignments(&self) -> impl Iterator<Item = u32> { [].into_iter() }
}

impl geometry::Vertex for () {
    fn write(&self, _: &mut Vec<f32>) {}
}

impl GeoLayout for () {
    type Vert = ();
    fn span(&self) -> usize { 0 }
    fn alignments(&self) -> impl Iterator<Item=u32> { [].into_iter() }
}


pub struct Uniforms {
    values: HashMap<String, (UniformLocation, Vec<f32>)>,
    pub program: Program,
}

pub trait UniformUploader where Self: Sized {
    fn upload(gl: &Context, program: Program, loc: UniformLocation, val: &[Self]);
    fn cast(f: &f32) -> Self;
    fn uncast(self) -> f32;
}

impl UniformUploader for f32 {
    fn upload(gl: &Context, program: Program, loc: UniformLocation, val: &[Self]) {
        unsafe {
            match val.len() {
                1 => gl.program_uniform_1_f32_slice(program, Some(&loc), val),
                2 => gl.program_uniform_2_f32_slice(program, Some(&loc), val),
                3 => gl.program_uniform_3_f32_slice(program, Some(&loc), val),
                4 => gl.program_uniform_4_f32_slice(program, Some(&loc), val),
                16 => gl.program_uniform_matrix_4_f32_slice(program, Some(&loc), false, val),
                _ => {}
            }
        }
    }
    fn cast(f: &f32) -> Self {
        *f
    }
    fn uncast(self) -> f32 {
        self
    }
}

impl UniformUploader for u32 {
    fn upload(gl: &Context, program: Program, loc: UniformLocation, val: &[Self]) {
        unsafe {
            match val.len() {
                1 => gl.program_uniform_1_u32_slice(program, Some(&loc), val),
                2 => gl.program_uniform_2_u32_slice(program, Some(&loc), val),
                3 => gl.program_uniform_3_u32_slice(program, Some(&loc), val),
                4 => gl.program_uniform_4_u32_slice(program, Some(&loc), val),
                _ => {}
            }
        }
    }
    fn cast(f: &f32) -> Self {
        f.to_bits()
    }
    fn uncast(self) -> f32 {
        f32::from_bits(self)
    }
}

impl UniformUploader for i32 {
    fn upload(gl: &Context, program: Program, loc: UniformLocation, val: &[Self]) {
        unsafe {
            match val.len() {
                1 => gl.program_uniform_1_i32_slice(program, Some(&loc), val),
                2 => gl.program_uniform_2_i32_slice(program, Some(&loc), val),
                3 => gl.program_uniform_3_i32_slice(program, Some(&loc), val),
                4 => gl.program_uniform_4_i32_slice(program, Some(&loc), val),
                _ => {}
            }
        }
    }
    fn cast(f: &f32) -> Self {
        f.to_bits() as i32
    }
    fn uncast(self) -> f32 {
        f32::from_bits(self as u32)
    }
}

impl Uniforms {
    pub fn new(program: Program) -> Self {
        Self {
            values: HashMap::new(),
            program,
        }
    }

    fn set_inner<T: UniformUploader>(gl: &Context, program: Program, loc: UniformLocation, val: &[T]) {
        T::upload(gl, program, loc, val)
    }

    /// This will fail silently if the uniform name is not in the program or if the data does not fit for the given type
    pub fn set<T: UniformUploader + PartialEq>(&mut self, gl: &Context, name: &str, value: &dyn GlData<DataType = T>) {
        let mut val = Vec::with_capacity(value.size());
        value.write(&mut val);
        if let Some((loc, current)) = self.values.get_mut(name) {
            let curr = current.iter().map(T::cast).collect::<Vec<T>>();
            if curr != val {
                *current = val.into_iter().map(T::uncast).collect();
                Self::set_inner(gl, self.program, *loc, current.as_slice())
            }
        } else if let Some(loc) = unsafe { gl.get_uniform_location(self.program, name) } {
            Self::set_inner(gl, self.program, loc, val.as_slice());
            self.values.insert(name.to_string(), (loc, val.into_iter().map(T::uncast).collect()));
        }
    }

}

pub struct Shader<GLayout, ILayout>
where
    GLayout: GeoLayout,
    ILayout: InstanceLayout,
{
    pub program: Program,
    pub(crate) layout: GLayout,
    pub(crate) instance_layout: Option<ILayout>,
    pub uniforms: Arc<RwLock<Uniforms>>,
}

impl<GLayout, ILayout> Shader<GLayout, ILayout>
where
    GLayout: GeoLayout,
    ILayout: InstanceLayout,
{
    pub fn new_instanced(gl: &Context, vsh: &str, fsh: &str, layout: GLayout, instance_layout: ILayout) -> Result<Self, String> {
        Self::new_inner(gl, vsh, fsh, layout, Some(instance_layout))
    }

    fn compile_shader(gl: &Context, vsh: &str, fsh: &str) -> Result<Program, String> {
        unsafe {
            let program = gl.create_program()?;

            let vert = gl.create_shader(glow::VERTEX_SHADER)?;
            gl.shader_source(vert, vsh);
            gl.compile_shader(vert);

            let frag = gl.create_shader(glow::FRAGMENT_SHADER)?;
            gl.shader_source(frag, fsh);
            gl.compile_shader(frag);

            gl.attach_shader(program, vert);
            gl.attach_shader(program, frag);
            gl.link_program(program);

            gl.delete_shader(vert);
            gl.delete_shader(frag);

            Ok(program)
        }

    }

    fn new_inner(gl: &Context, vsh: &str, fsh: &str, layout: GLayout, instance_layout: Option<ILayout>) -> Result<Self, String> {
        let program = Self::compile_shader(gl, vsh, fsh)?;
        Ok(Self {
            program,
            layout,
            instance_layout,
            uniforms: Arc::new(RwLock::new(Uniforms::new(program))),
        })
    }

}

impl<GLayout> Shader<GLayout, ()>
where
    GLayout: GeoLayout,
{
    pub fn new(gl: &Context, vsh: &str, fsh: &str, layout: GLayout) -> Result<Self, String> {
        Self::new_inner(gl, vsh, fsh, layout, None)
    }
}


