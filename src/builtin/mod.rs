use crate::geometry::layout::{UniformEntry, UniformKind, UniformsLayout};

pub mod simple;
pub mod instanced;
pub mod geometry;
pub mod renderer;

#[derive(Clone)]
pub struct DynamicUniforms {
    uniforms: Vec<UniformEntry>
}

impl DynamicUniforms {
    pub fn new() -> Self {
        Self::of(Vec::new())
    }

    pub fn of(uniforms: Vec<UniformEntry>) -> Self {
        Self { uniforms }
    }

    pub fn add_uniform(&mut self, name: &str, binding: u32, visibility: wgpu::ShaderStages, kind: UniformKind) -> &mut Self {
        self.uniforms.push(UniformEntry {
            name: name.to_string(),
            binding,
            visibility,
            kind,
        });
        self
    }

    pub fn with_uniform(mut self, name: &str, binding: u32, visibility: wgpu::ShaderStages, kind: UniformKind) -> Self {
        self.uniforms.push(UniformEntry {
            name: name.to_string(),
            binding,
            visibility,
            kind,
        });
        self
    }

}

impl UniformsLayout for DynamicUniforms {
    fn entries(&self) -> impl Iterator<Item=UniformEntry> {
        self.uniforms.clone().into_iter()
    }
}

impl UniformsLayout for () {
    fn entries(&self) -> impl Iterator<Item=UniformEntry> {
        [].into_iter()
    }
}
