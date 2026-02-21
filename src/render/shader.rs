use std::collections::HashMap;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex, RwLock};
use shaderc::{Compiler, ShaderKind};
use wgpu::{BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Buffer, BufferUsages, ColorTargetState, DepthStencilState, Device, ErrorFilter, Face, FragmentState, FrontFace, MultisampleState, PipelineLayoutDescriptor, PrimitiveState, Queue, RenderPass, RenderPipeline, RenderPipelineDescriptor, ShaderModule, ShaderModuleDescriptor, ShaderSource, TextureUsages, VertexAttribute, VertexBufferLayout, VertexState, VertexStepMode};
use wgpu::wgt::{BufferDescriptor, SamplerDescriptor, TextureDescriptor, TextureViewDescriptor};
use crate::geometry;
use crate::geometry::{Geometry, GeometryLayout};
use crate::render::{InstanceLayout, PipelineSelector, UniformEntry, UniformHandle, UniformType, UniformsLayout, UniformsSetter};
use crate::render::camera::Camera;
use crate::render::renderer::{BaseRenderer, ImmediateRenderer, InstancedRenderer};

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

#[cfg_attr(feature = "debug", derive(Debug))]
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
    Wgsl {
        module: ShaderModule,
        vert_entry: String,
        frag_entry: String,
    }
}

impl ShaderSet {
    fn vertex_shader(&self) -> &ShaderModule {
        match self {
            Self::Glsl { vsh, .. } => vsh,
            Self::Wgsl { module, .. } => &module,
        }
    }
    fn vertex_entry(&self) -> &str {
        match self {
            Self::Glsl { .. } => "main",
            Self::Wgsl { vert_entry, ..} => vert_entry
        }
    }
    fn fragment_shader(&self) -> &ShaderModule {
        match self {
            Self::Glsl { fsh, .. } => fsh,
            Self::Wgsl { module, .. } => &module,
        }
    }
    fn fragment_entry(&self) -> &str {
        match self {
            Self::Glsl { .. } => "main",
            Self::Wgsl { frag_entry, ..} => frag_entry
        }
    }
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


#[cfg_attr(feature = "debug", derive(Debug))]
pub struct ShaderPipeline<Uniforms, Shared>
where
    Uniforms: UniformsSetter<Shared>,
    Shared: Send + Sync,
{
    pub pipeline: RenderPipeline,
    pub uniforms: Uniforms,
    pub bind_group: BindGroup,
    _phantom: PhantomData<Shared>,
}

#[derive(Clone)]
pub struct PipelineConfig {
    pub cull_mode: Option<Face>,
    pub front_face: FrontFace,
    pub targets: Vec<Option<ColorTargetState>>,
    pub depth_stencil: Option<DepthStencilState>,
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
        vertex_entry: &str,
        fragment_entry: &str,
        layout: ShaderLayout<GLayout, ILayout, ULayout>
    ) -> Result<Self, ShaderError> {
        let module = Self::validate_shader(
            device, source.name,
            ShaderSource::Wgsl(source.source.into())
        )?;

        Ok(Self {
            layout,
            shader_set: Arc::new(ShaderSet::Wgsl {
                module,
                vert_entry: vertex_entry.to_string(),
                frag_entry: fragment_entry.to_string(),
            })
        })
    }

    pub fn create_geometry<Primitive>(&self) -> Geometry<GLayout, Primitive>
    where
        Primitive: geometry::Primitive<Vertex = GLayout::Vertex>
    {
        Geometry::new(self.layout.geometry_layout.clone())
    }

    fn create_pipeline<Primitive, Uniforms, Shared>(
        &self,
        device: &Device,
        queue: &Queue,
        config: &PipelineConfig,
        mut uniforms: Uniforms,
    ) -> Result<ShaderPipeline<Uniforms, Shared>, String>
    where
        Primitive: geometry::Primitive<Vertex=GLayout::Vertex>,
        Uniforms: UniformsSetter<Shared>,
        Shared: Send + Sync,
    {
        let mut uniform_bindings = HashMap::new();
        let mut locations = HashMap::new();

        let mut layout_entries = Vec::new();
        let mut entries = Vec::new();

        for UniformEntry {
            name, location,
            visibility, uniform_type,
        } in self.layout.uniforms_layout.entries() {
            layout_entries.push(BindGroupLayoutEntry {
                binding: location,
                visibility,
                ty: uniform_type.binding_type(),
                count: None,
            });

            locations.insert(name.clone(), location);

            let handle = match uniform_type {
                UniformType::Buffer {
                    size,
                    ..
                } => {
                    let buf = device.create_buffer(&BufferDescriptor {
                        label: Some(&name),
                        size,
                        usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
                        mapped_at_creation: false,
                    });
                    UniformHandle::Buffer(buf)
                },
                UniformType::Sampler {
                    min_filter, mag_filter, mipmap_filter,
                    address_mode_u, address_mode_v, address_mode_w,
                    lod_min_clamp, lod_max_clamp,
                    compare, anisotropy_clamp, border_color,
                    ..
                } => {
                    let sampler = device.create_sampler(&SamplerDescriptor {
                        label: Some(&name),
                        mag_filter, min_filter, mipmap_filter,
                        address_mode_u, address_mode_v, address_mode_w,
                        lod_min_clamp, lod_max_clamp,
                        compare, anisotropy_clamp, border_color,
                    });
                    UniformHandle::Sampler(sampler)
                },
                UniformType::Texture {
                    size, dimension,
                    format, mip_level_count, sample_count,
                    ..
                } => {
                    let texture = device.create_texture(&TextureDescriptor {
                        label: Some(&name),
                        size, mip_level_count, sample_count, format,
                        dimension: dimension.compatible_texture_dimension(),
                        usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
                        view_formats: &[],
                    });
                    let view = texture.create_view(&Default::default());

                    UniformHandle::Texture(texture, view)
                }
            };

            uniform_bindings.insert(name.clone(), handle);

        }

        for (name, handle) in &uniform_bindings {
            entries.push(BindGroupEntry {
                binding: *locations.get(name).unwrap(),
                resource: handle.as_binding_resource(),
            })
        }


        let bgl = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: layout_entries.as_slice(),
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: None,
            layout: &bgl,
            entries: entries.as_slice()
        });

        uniforms.bind(device, queue, uniform_bindings)?;

        let pll = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bgl],
            push_constant_ranges: &[],
        });

        let mut geo_attrs = Vec::new();
        let mut offset = 0;
        for (loc, format) in self.layout.geometry_layout.attributes() {
            geo_attrs.push(VertexAttribute {
                shader_location: loc,
                offset,
                format,
            });
            offset += format.size();
        }

        let mut buffers = vec![VertexBufferLayout {
            array_stride: self.layout.geometry_layout.span(),
            step_mode: VertexStepMode::Vertex,
            attributes: geo_attrs.as_slice(),
        }];

        let mut inst_attrs = Vec::new();
        // Safety: This function is marked unsafe to discourage trait
        // implementers from overriding it, there's nothing inherently
        // unsafe about it.
        if unsafe { ILayout::is_instanced() } {
            let mut offset = 0;
            for (loc, format) in self.layout.instance_layout.attributes() {
                inst_attrs.push(VertexAttribute {
                    shader_location: loc,
                    offset,
                    format,
                });
                offset += format.size();
            }

            let inst_layout = VertexBufferLayout {
                array_stride: self.layout.instance_layout.span(),
                step_mode: VertexStepMode::Instance,
                attributes: inst_attrs.as_slice()
            };

            buffers.push(inst_layout);
        }

        let pipeline = device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&pll),
            vertex: VertexState {
                module: self.shader_set.vertex_shader(),
                entry_point: Some(self.shader_set.vertex_entry()),
                buffers: buffers.as_slice(),
                compilation_options: Default::default(),
            },
            fragment: Some(FragmentState {
                module: self.shader_set.fragment_shader(),
                entry_point: Some(self.shader_set.fragment_entry()),
                targets: config.targets.as_slice(),
                compilation_options: Default::default(),
            }),
            primitive: PrimitiveState {
                topology: Primitive::TOPOLOGY,
                cull_mode: config.cull_mode,
                front_face: config.front_face,
                ..Default::default()
            },
            depth_stencil: config.depth_stencil.clone(),
            multisample: MultisampleState::default(),
            multiview: None,
            cache: None,
        });


        Ok(ShaderPipeline {
            pipeline,
            uniforms,
            bind_group,
            _phantom: PhantomData
        })
    }

    fn create_pipelines<Primitive, Uniforms, Shared>(
        &self,
        device: &Device,
        queue: &Queue,
        configs: &[PipelineConfig],
        uniforms: Uniforms,
    ) -> Result<Vec<ShaderPipeline<Uniforms, Shared>>, String>
    where
        Primitive: geometry::Primitive<Vertex=GLayout::Vertex>,
        Uniforms: UniformsSetter<Shared>,
        Shared: Send + Sync,
    {
        if configs.is_empty() {
            return Err(String::from("No configs provided"))
        }
        let mut pipelines = Vec::new();
        for config in &configs[..configs.len()-1] {
            pipelines.push(
                self.create_pipeline::<Primitive, _, _>(device, queue, config, uniforms.clone())?
            )
        }
        pipelines.push(self.create_pipeline::<Primitive, _, _>(device, queue, &configs.last().unwrap(), uniforms)?);
        Ok(pipelines)
    }


    fn create_base_renderer
    <Primitive, Selector, Shared, Uniforms>
    (
        &self,
        device: &Device,
        queue: &Queue,
        geometry: Geometry<GLayout, Primitive>,
        configs: &[PipelineConfig],
        uniforms: Uniforms,
        geometry_size: u64,
        vertex_init: bool,
    ) -> Result<BaseRenderer<GLayout, Primitive, Selector, Uniforms, Shared>, String>
    where
        Primitive: geometry::Primitive<Vertex=GLayout::Vertex>,
        Selector: PipelineSelector<Uniforms, Shared>,
        Uniforms: UniformsSetter<Shared>,
        Shared: Send + Sync,
    {
        let selector = Selector::create(self.create_pipelines::<Primitive, _, _>(
            device, queue, configs, uniforms
        )?)?;

        let vertex_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("vertex buffer"),
            size: geometry_size,
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: vertex_init,
        });

        Ok(BaseRenderer {
            geometry,
            selector,
            vertex_buffer,
            vertex_count: 0,
            geometry_dirty: true,
            _phantom: PhantomData,
        })
    }


}


impl<GLayout, ILayout, ULayout> Shader<GLayout, ILayout, ULayout>
where
    GLayout: GeometryLayout,
    ILayout: InstanceLayout,
    ULayout: UniformsLayout,
{
    pub fn create_instanced_renderer
    <Primitive, Selector, Uniforms, Shared>
    (
        &self,
        device: &Device,
        queue: &Queue,
        configs: &[PipelineConfig],
        geometry: Geometry<GLayout, Primitive>,
        uniforms_setter: Uniforms,
        initial_instance_max_count: u64,
    ) -> Result<InstancedRenderer<GLayout, ILayout, Primitive, Selector, Uniforms, Shared>, String>
    where
        Primitive: geometry::Primitive<Vertex=GLayout::Vertex>,
        Selector: PipelineSelector<Uniforms, Shared>,
        Uniforms: UniformsSetter<Shared>,
        Shared: Send + Sync,
    {

        // Safety: This function is marked unsafe to discourage trait
        // implementers from overriding it, there's nothing inherently
        // unsafe about it.
        if !unsafe { ILayout::is_instanced() } {
            return Err(String::from("Layout is not instanced"))
        }

        let vertex_count = geometry.primitives.len() as u32 * Primitive::VERTICES;
        let size = vertex_count as u64 * self.layout.geometry_layout.span();

        let mut base = self.create_base_renderer::<Primitive, _, _, _>(
            device, queue, geometry, configs, uniforms_setter, size, true
        )?;

        let mut bytes = Vec::new();
        base.geometry.write(&mut bytes);
        {
            let mut view = base.vertex_buffer.slice(..).get_mapped_range_mut();
            view[..bytes.len()].copy_from_slice(&bytes);
        }
        base.vertex_buffer.unmap();
        base.vertex_count = vertex_count;
        base.geometry_dirty = false;

        let instance_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("vertex buffer"),
            size: initial_instance_max_count * self.layout.instance_layout.span(),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Ok(InstancedRenderer {
            base,
            draw_calls: Arc::new(Mutex::new(Vec::new())),
            instance_buffer,
            instance_count: 0
        })
    }
}


impl<GLayout, ULayout> Shader<GLayout, (), ULayout>
where
    GLayout: GeometryLayout,
    ULayout: UniformsLayout,
{
    pub fn create_immediate_renderer
    <Primitive, Selector, Uniforms, Shared>
    (
        &self,
        device: &Device,
        queue: &Queue,
        configs: &[PipelineConfig],
        geometry: Geometry<GLayout, Primitive>,
        uniforms_setter: Uniforms,
    ) -> Result<ImmediateRenderer<GLayout, Primitive, Selector, Uniforms, Shared>, String>
    where
        Primitive: geometry::Primitive<Vertex=GLayout::Vertex>,
        Selector: PipelineSelector<Uniforms, Shared>,
        Uniforms: UniformsSetter<Shared>,
        Shared: Send + Sync,
    {
        let size = geometry.primitives.len() as u64 * Primitive::VERTICES as u64 * self.layout.geometry_layout.span();

        Ok(ImmediateRenderer {
            base: self.create_base_renderer(device, queue, geometry, configs, uniforms_setter, size, false)?
        })

    }
}


impl<Uniforms, Shared>
ShaderPipeline<Uniforms, Shared>
where
    Uniforms: UniformsSetter<Shared>,
    Shared: Send + Sync,
{
    pub(crate) fn setup(&self, device: &Device, queue: &Queue, camera: &Camera, shared: &Shared) {
        self.uniforms.set(device, queue, camera, shared)
    }
    pub(crate) fn render(&self, pass: &mut RenderPass) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
    }
}



