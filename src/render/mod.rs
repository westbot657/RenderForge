pub mod shader;
pub mod camera;
pub mod renderer;

use std::collections::HashMap;
use wgpu::{AddressMode, BindGroup, BindingType, BufferBindingType, BufferSize, CompareFunction, Device, Extent3d, FilterMode, Queue, SamplerBindingType, SamplerBorderColor, ShaderStages, TextureFormat, TextureSampleType, TextureViewDimension, VertexFormat};
use crate::render::camera::Camera;
use crate::render::shader::ShaderPipeline;
use crate::SizedThreadSafe;

pub trait Data: SizedThreadSafe {
    fn write(&self, buffer: &mut Vec<u8>);
}

pub trait InstanceLayout: SizedThreadSafe + Clone {
    type Data: Data;
    /// Returns an iterator over attributes' layout location and format
    /// make sure the locations don't conflict with geometry layout locations
    fn attributes(&self) -> impl Iterator<Item=(u32, VertexFormat)>;
    /// There is almost no reason to override this function
    fn span(&self) -> u64 { self.attributes().map(|(_, format)| format.size()).sum() }
    /// Do not override this, it only exists so that unit type can skip instance setup
    unsafe fn is_instanced() -> bool { true }
}

impl Data for () {
    fn write(&self, _: &mut Vec<u8>) {}
}

impl InstanceLayout for () {
    type Data = ();
    fn attributes(&self) -> impl Iterator<Item=(u32, VertexFormat)> { [].into_iter() }
    unsafe fn is_instanced() -> bool { false }
}

#[derive(Copy, Clone)]
#[cfg_attr(feature = "debug", derive(Debug))]
pub enum UniformType {
    Buffer {
        has_dynamic_offset: bool,
        min_binding_size: Option<BufferSize>,
        size: u64,
    },
    Sampler {
        ty: SamplerBindingType,
        min_filter: FilterMode,
        mag_filter: FilterMode,
        mipmap_filter: FilterMode,
        address_mode_u: AddressMode,
        address_mode_v: AddressMode,
        address_mode_w: AddressMode,
        lod_min_clamp: f32,
        lod_max_clamp: f32,
        compare: Option<CompareFunction>,
        anisotropy_clamp: u16,
        border_color: Option<SamplerBorderColor>
    },
    Texture {
        sample_type: TextureSampleType,
        dimension: TextureViewDimension,
        format: TextureFormat,
        multisampled: bool,
        mip_level_count: u32,
        sample_count: u32,
        size: Extent3d,
    }
}

impl UniformType {
    pub fn binding_type(&self) -> BindingType {
        match *self {
            Self::Buffer {
                has_dynamic_offset,
                min_binding_size,
                ..
            } => BindingType::Buffer { ty: BufferBindingType::Uniform, has_dynamic_offset, min_binding_size },
            Self::Sampler {
                ty,
                ..
            } => BindingType::Sampler(ty),
            Self::Texture {
                sample_type,
                dimension: view_dimension,
                multisampled,
                ..
            } => BindingType::Texture { sample_type, view_dimension, multisampled }
        }
    }
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub struct UniformEntry {
    pub name: String,
    pub location: u32,
    pub visibility: ShaderStages,
    pub uniform_type: UniformType,
}

#[cfg_attr(feature = "debug", derive(Debug))]
pub enum UniformHandle {
    Buffer(wgpu::Buffer),
    Texture(wgpu::Texture, wgpu::TextureView),
    Sampler(wgpu::Sampler),
}

impl UniformHandle {
    pub fn as_binding_resource(&self) -> wgpu::BindingResource<'_> {
        match self {
            Self::Buffer(buf) => buf.as_entire_binding(),
            Self::Texture(_, view) => wgpu::BindingResource::TextureView(view),
            Self::Sampler(sampler) => wgpu::BindingResource::Sampler(sampler)
        }
    }
}

/// This object only needs to be clonable *before* any uniforms are bound
/// so implementing an improper clone function that doesn't clone
/// uniform-related data will work as intended
pub trait UniformsSetter<Shared: Send + Sync> : SizedThreadSafe + Clone {
    /// Will give the setter a map of uniforms to bind with
    fn bind(&mut self, device: &Device, queue: &Queue, uniforms: HashMap<String, UniformHandle>) -> Result<(), String>;
    fn set(&self, device: &Device, queue: &Queue, camera: &Camera, shared: &Shared);
}

pub trait UniformsLayout: SizedThreadSafe + Clone {
    fn entries(&self) -> impl Iterator<Item=UniformEntry>;
}

impl<Shared: Send + Sync> UniformsSetter<Shared> for () {
    fn bind(&mut self, _: &Device, _: &Queue, _: HashMap<String, UniformHandle>) -> Result<(), String> { Ok(()) }
    fn set(&self, _: &Device, _: &Queue, _: &Camera, _: &Shared) {}
}

impl UniformsLayout for () {
    fn entries(&self) -> impl Iterator<Item=UniformEntry> { [].into_iter() }
}

pub trait PipelineSelector<Uniforms, Shared> : Sized + Send + Sync
where
    Uniforms: UniformsSetter<Shared>,
    Shared: Send + Sync,
{
    fn create(pipelines: Vec<ShaderPipeline<Uniforms, Shared>>) -> Result<Self, String>;
    fn select(&self, shared: &Shared) -> &ShaderPipeline<Uniforms, Shared>;
}


#[cfg_attr(feature = "debug", derive(Debug))]
pub struct SinglePipelineSelector
<Uniforms, Shared>
(ShaderPipeline<Uniforms, Shared>)
where
    Uniforms: UniformsSetter<Shared>,
    Shared: Send + Sync,
;

impl<Uniforms, Shared> PipelineSelector<Uniforms, Shared> for SinglePipelineSelector<Uniforms, Shared>
where
    Uniforms: UniformsSetter<Shared>,
    Shared: Send + Sync,
{
    fn create(pipelines: Vec<ShaderPipeline<Uniforms, Shared>>) -> Result<Self, String> {
        let a = pipelines
            .into_iter()
            .next()
            .ok_or_else(|| String::from("No pipeline was supplied for SinglePipelineSelector"))?;

        Ok(Self(a))
    }
    fn select(&self, _: &Shared) -> &ShaderPipeline<Uniforms, Shared> {
        &self.0
    }
}



