use std::sync::Arc;
use shaderc::{ShaderKind};
use wgpu::{Device, RenderPipeline, RenderPipelineDescriptor, ShaderModule};
use crate::geometry::Geometry;
use crate::geometry::layout::{GeometryLayout, InstanceLayout, UniformEntry, UniformKind, UniformsLayout};
use crate::geometry::primitive::Primitive;
use crate::render::PipelineSelector;

pub enum ShaderSet {
    GlslRender {
        vsh: ShaderModule,
        fsh: ShaderModule
    },
    WgslRender {
        module: ShaderModule,
    }
}

pub struct Shader<Geo, Inst, Unis>
where
    Geo: GeometryLayout,
    Inst: InstanceLayout,
    Unis: UniformsLayout,
{
    pub geometry_layout: Geo,
    pub instance_layout: Inst,
    pub uniforms: Unis,
    pub shader_module: Arc<ShaderSet>
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

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum DepthMode {
    WriteAndTest,
    TestOnly,
    Off,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum CullMode {
    Back,
    Front,
    None,
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub enum BlendMode {
    Replace,
    Alpha,
    Additive,
}

impl From<DepthMode> for Option<wgpu::DepthStencilState> {
    fn from(d: DepthMode) -> Self {
        match d {
            DepthMode::Off => None,
            DepthMode::WriteAndTest => Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            DepthMode::TestOnly => Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: false,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: Default::default(),
                bias: Default::default(),
            }),
        }
    }
}

impl From<CullMode> for Option<wgpu::Face> {
    fn from(c: CullMode) -> Self {
        match c {
            CullMode::Back => Some(wgpu::Face::Back),
            CullMode::Front => Some(wgpu::Face::Front),
            CullMode::None => None,
        }
    }
}

impl From<BlendMode> for wgpu::BlendState {
    fn from(b: BlendMode) -> Self {
        match b {
            BlendMode::Replace => wgpu::BlendState::REPLACE,
            BlendMode::Alpha => wgpu::BlendState::ALPHA_BLENDING,
            BlendMode::Additive => wgpu::BlendState::PREMULTIPLIED_ALPHA_BLENDING,
        }
    }
}

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct PipelineConfig {
    pub depth: DepthMode,
    pub cull: CullMode,
    pub blend: BlendMode,
    pub target_format: wgpu::TextureFormat,
}

impl<Geo, Inst, Unis> Shader<Geo, Inst, Unis>
where
    Geo: GeometryLayout,
    Inst: InstanceLayout,
    Unis: UniformsLayout,
{
    pub fn new_glsl(device: &Device, compiler: &shaderc::Compiler, vsh_name: &str, fsh_name: &str, vsh: &str, fsh: &str, geometry_layout: Geo, instance_layout: Inst, uniforms: Unis) -> Result<Self, ShaderError> {
        let vsh = compiler.compile_into_spirv(vsh, ShaderKind::Vertex, vsh_name, "main", None)?;
        let fsh = compiler.compile_into_spirv(fsh, ShaderKind::Fragment, fsh_name, "main", None)?;

        let vsh = Self::create_validated(device, vsh_name, wgpu::util::make_spirv(vsh.as_binary_u8()))?;
        let fsh = Self::create_validated(device, fsh_name, wgpu::util::make_spirv(fsh.as_binary_u8()))?;

        Ok(Self {
            geometry_layout,
            instance_layout,
            uniforms,
            shader_module: Arc::new(ShaderSet::GlslRender { vsh, fsh })
        })
    }

    fn create_validated(
        device: &Device,
        label: &str,
        source: wgpu::ShaderSource,
    ) -> Result<ShaderModule, ShaderError> {
        device.push_error_scope(wgpu::ErrorFilter::Validation);
        let module = device.create_shader_module(
            wgpu::ShaderModuleDescriptor { label: Some(label), source, }
        );
        match pollster::block_on(device.pop_error_scope()) {
            Some(e) => Err(ShaderError::ValidationError(e)),
            None => Ok(module),
        }
    }

    pub fn new_wgsl(device: &Device, name: &str, source: &str, geometry_layout: Geo, instance_layout: Inst, uniforms: Unis) -> Result<Self, ShaderError> {
        let module = Self::create_validated(device, name, wgpu::ShaderSource::Wgsl(source.into()))?;

        Ok(Self {
            geometry_layout,
            instance_layout,
            uniforms,
            shader_module: Arc::new(ShaderSet::WgslRender { module })
        })

    }

    pub fn create_geometry<Prim>(&self) -> Geometry<Prim, Geo>
    where
        Prim: Primitive<Vert = Geo::Vert>
    {
        Geometry::new_with_layout(self.geometry_layout.clone())
    }

    fn build_bind_group_layout(device: &Device, entries: impl Iterator<Item = UniformEntry>) -> wgpu::BindGroupLayout {
        let entries: Vec<wgpu::BindGroupLayoutEntry> = entries.map(|e| {
            wgpu::BindGroupLayoutEntry {
                binding:    e.binding,
                visibility: e.visibility,
                ty: match e.kind {
                    UniformKind::Buffer => wgpu::BindingType::Buffer {
                        ty:                 wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size:   None,
                    },
                    UniformKind::StorageBuffer => wgpu::BindingType::Buffer {
                        ty:                 wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size:   None,
                    },
                    UniformKind::Texture => wgpu::BindingType::Texture {
                        sample_type:    wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled:   false,
                    },
                    UniformKind::Sampler => wgpu::BindingType::Sampler(
                        wgpu::SamplerBindingType::Filtering
                    ),
                },
                count: None,
            }
        }).collect();

        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label:   None,
            entries: &entries,
        })
    }

    pub fn create_pipeline<Prim>(
        &self,
        device: &Device,
        config: &PipelineConfig,
    ) -> Result<RenderPipeline, wgpu::Error>
    where
        Prim: Primitive<Vert = Geo::Vert>
    {
        let geo_attributes: Vec<wgpu::VertexAttribute> = {
            let mut offset = 0u64;
            self.geometry_layout.attributes().map(|(location, format)| {
                let attr = wgpu::VertexAttribute { shader_location: location, offset, format };
                offset += format.size();
                attr
            }).collect()
        };

        let geo_stride: u64 = self.geometry_layout.attributes()
            .map(|(_, fmt)| fmt.size())
            .sum();

        let inst_attributes: Vec<wgpu::VertexAttribute> = {
            let mut offset = 0u64;
            self.instance_layout.attributes().map(|(location, format)| {
                let attr = wgpu::VertexAttribute { shader_location: location, offset, format };
                offset += format.size();
                attr
            }).collect()
        };

        let inst_stride: u64 = self.instance_layout.attributes()
            .map(|(_, fmt)| fmt.size())
            .sum();

        let geo_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: geo_stride,
            step_mode:    wgpu::VertexStepMode::Vertex,
            attributes:   &geo_attributes,
        };

        let inst_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: inst_stride,
            step_mode:    wgpu::VertexStepMode::Instance,
            attributes:   &inst_attributes,
        };

        let vertex_buffers: Vec<wgpu::VertexBufferLayout> = if inst_stride > 0 {
            vec![geo_buffer_layout, inst_buffer_layout]
        } else {
            vec![geo_buffer_layout]
        };

        let (vs_module, vs_entry, fs_module, fs_entry) = match self.shader_module.as_ref() {
            ShaderSet::GlslRender { vsh, fsh } => (vsh, "main", fsh, "main"),
            ShaderSet::WgslRender { module }   => (module, "vs_main", module, "fs_main"),
        };

        let uniforms_layout = Self::build_bind_group_layout(device, self.uniforms.entries());

        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("pipeline_layout"),
            bind_group_layouts: &[&uniforms_layout],
            push_constant_ranges: &[],
        });

        device.push_error_scope(wgpu::ErrorFilter::Validation);

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: Some("pipeline"),
            layout: Some(&layout),

            vertex: wgpu::VertexState {
                module: vs_module,
                entry_point: Some(vs_entry),
                buffers: &vertex_buffers,
                compilation_options: Default::default(),
            },

            fragment: Some(wgpu::FragmentState {
                module: fs_module,
                entry_point: Some(fs_entry),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.target_format,
                    blend: Some(config.blend.into()),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),

            primitive: wgpu::PrimitiveState {
                topology: Prim::TOPOLOGY.into(),
                cull_mode: config.cull.into(),
                front_face: wgpu::FrontFace::Ccw,
                polygon_mode: wgpu::PolygonMode::Fill,
                strip_index_format: None,
                unclipped_depth: false,
                conservative: false,
            },

            depth_stencil: config.depth.into(),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        match pollster::block_on(device.pop_error_scope()) {
            Some(e) => Err(e),
            None => Ok(pipeline),
        }
    }

    pub fn setup_pipelines<Prim>(&self, device: &Device, configs: &[PipelineConfig]) -> Result<Vec<RenderPipeline>, wgpu::Error>
    where
        Prim: Primitive<Vert = Geo::Vert>
    {
        configs.iter().map(|config| self.create_pipeline::<Prim>(device, config)).collect::<Result<Vec<_>, _>>()
    }

    pub fn setup_selector<Sel, Prim, Shared>(&self, device: &Device, configs: &[PipelineConfig], shared: &Shared) -> Result<Sel, wgpu::Error>
    where
        Sel: PipelineSelector<Shared>,
        Prim: Primitive<Vert = Geo::Vert>,
        Shared: Sync + Send
    {
        Ok(Sel::create(self.setup_pipelines::<Prim>(device, configs)?, shared))
    }

}


