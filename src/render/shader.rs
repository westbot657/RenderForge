use glow::HasContext;
use crate::geometry::{GeoLayout};
use crate::render::instanced::{InstanceData, InstanceLayout};

#[derive(Copy, Clone, Default)]
pub(crate) struct NullInstanceLayout;
#[derive(Copy, Clone, Default)]
pub(crate) struct NullInstanceData;

impl InstanceData for NullInstanceData {
    fn write(&self, _: &mut Vec<f32>) {}
}
impl InstanceLayout for NullInstanceLayout {
    type Data = NullInstanceData;
    fn span(&self) -> usize { 0 }
}


pub struct Shader<GLayout, ILayout>
where
    GLayout: GeoLayout,
    ILayout: InstanceLayout,
{
    pub(crate) program: glow::Program,
    pub(crate) layout: GLayout,
    pub(crate) instance_layout: Option<ILayout>
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
            instance_layout
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




impl<Layout> Shader<Layout, NullInstanceLayout>
where
    Layout: GeoLayout
{
    pub fn from_instanced_shader<GLayout, ILayout>(shader: &Shader<GLayout, ILayout>) -> Result<Self, String>
    where
        GLayout: GeoLayout,
        ILayout: InstanceLayout,
        Layout: TryFrom<(GLayout, ILayout), Error = String>
    {
        let layout = Layout::try_from((shader.layout.clone(), shader.instance_layout.clone().unwrap()))?;
        Ok(Self {
            program: shader.program,
            layout,
            instance_layout: None
        })
    }
}

