use std::sync::Arc;
use shaderc::{Compiler, ShaderKind};
use wgpu::{Device, ErrorFilter, ShaderModule, ShaderModuleDescriptor, ShaderSource};
use crate::geometry;
use crate::geometry::{Geometry, GeometryLayout};
use crate::render::{InstanceLayout, UniformsLayout};

pub fn create_glsl_compiler() -> Result<Compiler, shaderc::Error> {
    Compiler::new()
}

#[derive(thiserror::Error, Debug)]
pub enum ShaderError {
    #[error("GLSL compile error: {0}")]
    GlslError(shaderc::Error),
    #[error("SPIR-V validation error: {0}")]
    ValidationError(wgpu::Error)
}

impl From<shaderc::Error> for ShaderError {
    fn from(value: shaderc::Error) -> Self {
        Self::GlslError(value)
    }
}
impl From<wgpu::Error> for ShaderError {
    fn from(value: wgpu::Error) -> Self {
        Self::ValidationError(value)
    }
}

pub struct Source<'s> {
    pub name: Option<&'s str>,
    pub source: &'s str,
}

#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum ShaderSet {
    Glsl {
        vsh: ShaderModule,
        fsh: ShaderModule,
    },
    Wgsl(ShaderModule)
}

#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct ShaderLayout<GLayout, ILayout, ULayout>
where
    GLayout: GeometryLayout,
    ILayout: InstanceLayout,
    ULayout: UniformsLayout,
{
    pub geometry_layout: GLayout,
    pub instance_layout: ILayout,
    pub uniforms_layout: ULayout,
}

#[derive(Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub struct Shader<GLayout, ILayout, ULayout>
where
    GLayout: GeometryLayout,
    ILayout: InstanceLayout,
    ULayout: UniformsLayout,
{
    pub layout: ShaderLayout<GLayout, ILayout, ULayout>,
    pub shader_set: Arc<ShaderSet>,
}

impl<GLayout, ILayout, ULayout> Shader<GLayout, ILayout, ULayout>
where
    GLayout: GeometryLayout,
    ILayout: InstanceLayout,
    ULayout: UniformsLayout,
{
    fn validate_shader(device: &Device, label: Option<&str>, source: ShaderSource) -> Result<ShaderModule, ShaderError> {
        device.push_error_scope(ErrorFilter::Validation);

        let module = device.create_shader_module(ShaderModuleDescriptor {
            label,
            source
        });

        match pollster::block_on(device.pop_error_scope()) {
            Some(e) => Err(ShaderError::ValidationError(e)),
            None => Ok(module)
        }
    }

    pub fn new_glsl(
        device: &Device,
        compiler: &Compiler,
        vsh: Source,
        fsh: Source,
        layout: ShaderLayout<GLayout, ILayout, ULayout>
    ) -> Result<Self, ShaderError> {

        let v = compiler.compile_into_spirv(
            vsh.source, ShaderKind::Vertex,
            vsh.name.unwrap_or("<vertex-shader>"),
            "main", None
        )?;
        let f = compiler.compile_into_spirv(
            fsh.source, ShaderKind::Fragment,
            fsh.name.unwrap_or("<fragment-shader>"),
            "main", None
        )?;

        let vsh = Self::validate_shader(
            device, vsh.name,
            wgpu::util::make_spirv(v.as_binary_u8())
        )?;
        let fsh = Self::validate_shader(
            device, fsh.name,
            wgpu::util::make_spirv(f.as_binary_u8())
        )?;

        Ok(Self {
            layout,
            shader_set: Arc::new(ShaderSet::Glsl { vsh, fsh })
        })

    }

    pub fn new_wgsl(
        device: &Device,
        source: Source,
        layout: ShaderLayout<GLayout, ILayout, ULayout>
    ) -> Result<Self, ShaderError> {
        let module = Self::validate_shader(
            device, source.name,
            ShaderSource::Wgsl(source.source.into())
        )?;

        Ok(Self {
            layout,
            shader_set: Arc::new(ShaderSet::Wgsl(module))
        })
    }

    pub fn create_geometry<Primitive>(&self) -> Geometry<GLayout, Primitive>
    where
        Primitive: geometry::Primitive<Vertex = GLayout::Vertex>
    {
        Geometry::new(self.layout.geometry_layout.clone())
    }

}


