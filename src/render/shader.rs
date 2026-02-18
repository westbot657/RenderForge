use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use glow::HasContext;
use crate::geometry::{GeoLayout};
use crate::render::GlData;
use crate::render::instanced::{InstanceData, InstanceLayout};

#[derive(Copy, Clone, Default)]
pub struct NullInstanceLayout;
#[derive(Copy, Clone, Default)]
pub struct NullInstanceData;

impl InstanceData for NullInstanceData {
    fn write(&self, _: &mut Vec<f32>) {}
}
impl InstanceLayout for NullInstanceLayout {
    type Data = NullInstanceData;
    fn span(&self) -> usize { 0 }
    fn alignments(&self) -> impl Iterator<Item = u32> {
        [].into_iter()
    }
}


pub struct Uniforms {
    values: HashMap<String, (glow::UniformLocation, Vec<f32>)>,
    program: glow::Program,
}

impl Uniforms {
    pub fn new(program: glow::Program) -> Self {
        Self {
            values: HashMap::new(),
            program,
        }
    }

    fn set_inner(gl: &glow::Context, program: glow::Program, loc: glow::UniformLocation, val: &[f32]) {
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

    /// This will fail silently if the uniform name is not in the program or if the data does not fit in 1, 2, 3, 4, or 16 f32s
    pub fn set(&mut self, gl: &glow::Context, name: &str, value: &dyn GlData) {
        let mut val = Vec::with_capacity(value.size());
        value.write(&mut val);
        if let Some((loc, current)) = self.values.get_mut(name) {
            if *current != val {
                *current = val;
                Self::set_inner(gl, self.program, *loc, current.as_slice())
            }
        } else if let Some(loc) = unsafe { gl.get_uniform_location(self.program, name) } {
            Self::set_inner(gl, self.program, loc, val.as_slice());
            self.values.insert(name.to_string(), (loc, val));
        }
    }

}

pub struct Shader<GLayout, ILayout>
where
    GLayout: GeoLayout,
    ILayout: InstanceLayout,
{
    pub(crate) program: glow::Program,
    pub(crate) layout: GLayout,
    pub(crate) instance_layout: Option<ILayout>,
    pub(crate) uniforms: Arc<RwLock<Uniforms>>,
}

impl<GLayout, ILayout> Shader<GLayout, ILayout>
where
    GLayout: GeoLayout,
    ILayout: InstanceLayout,
{
    pub fn new_instanced(gl: &glow::Context, vsh: &str, fsh: &str, layout: GLayout, instance_layout: ILayout) -> Result<Self, String> {
        Self::new_inner(gl, vsh, fsh, layout, Some(instance_layout))
    }

    fn compile_shader(gl: &glow::Context, vsh: &str, fsh: &str) -> Result<glow::Program, String> {
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

    fn new_inner(gl: &glow::Context, vsh: &str, fsh: &str, layout: GLayout, instance_layout: Option<ILayout>) -> Result<Self, String> {
        let program = Self::compile_shader(gl, vsh, fsh)?;
        Ok(Self {
            program,
            layout,
            instance_layout,
            uniforms: Arc::new(RwLock::new(Uniforms::new(program))),
        })
    }

}

impl<GLayout> Shader<GLayout, NullInstanceLayout>
where
    GLayout: GeoLayout,
{
    pub fn new(gl: &glow::Context, vsh: &str, fsh: &str, layout: GLayout) -> Result<Self, String> {
        Self::new_inner(gl, vsh, fsh, layout, None)
    }
}


