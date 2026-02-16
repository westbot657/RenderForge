use crate::geometry::*;
use crate::render::shader::Shader;

pub trait InstanceData: Sized + Clone {
    fn write(&self, buffer: &mut Vec<f32>);
}

pub trait InstanceLayout: Sized + Clone {
    type Data: InstanceData;
    fn span(&self) -> usize;
}

pub struct InstancedMesh<Geo, GLayout, ILayout>
where
    Geo: GeoUnit<Vert = GLayout::Vert>,
    GLayout: GeoLayout,
    ILayout: InstanceLayout,
{
    pub(crate) inner: Geometry<Geo, GLayout>,
    pub(crate) instance_layout: ILayout,
    pub(crate) data: Vec<ILayout::Data>,
}

impl<Geo, GLayout, ILayout> InstancedMesh<Geo, GLayout, ILayout>
where
    Geo: GeoUnit<Vert = GLayout::Vert>,
    GLayout: GeoLayout,
    ILayout: InstanceLayout + Default,
{
    pub fn new(geometry: Geometry<Geo, GLayout>) -> Self {
        Self::new_with_layout(geometry, ILayout::default())
    }
}

impl<Geo, GLayout, ILayout> InstancedMesh<Geo, GLayout, ILayout>
where
    Geo: GeoUnit<Vert = GLayout::Vert>,
    GLayout: GeoLayout,
    ILayout: InstanceLayout,
{
    pub fn new_with_layout(geometry: Geometry<Geo, GLayout>, instance_layout: ILayout) -> Self {
        Self {
            inner: geometry,
            instance_layout,
            data: Vec::new()
        }
    }
    
    pub fn add_data(&mut self, data: ILayout::Data) {
        self.data.push(data)
    }
    
    pub fn clear_data(&mut self) {
        self.data.clear()
    }
    
    pub fn get_geo_buffer(&self) -> Vec<f32> {
        self.inner.get_buffer()
    }
    
    pub fn get_instance_buffer(&self) -> Vec<f32> {
        let size = self.instance_layout.span()
            * self.data.len();
        let mut buffer = Vec::with_capacity(size);
        
        for data in &self.data {
            data.write(&mut buffer);
        }
        
        buffer
    }
    
}

impl<Geo, GLayout, ILayout> BufferProvider for InstancedMesh<Geo, GLayout, ILayout>
where
    Geo: GeoUnit<Vert = GLayout::Vert>,
    GLayout: GeoLayout,
    ILayout: InstanceLayout
{
    fn get_buffer(&self) -> Vec<f32> {
        self.get_geo_buffer()
    }
}


impl<GLayout, ILayout> Shader<GLayout, ILayout>
where
    GLayout: GeoLayout,
    ILayout: InstanceLayout
{
    pub fn get_instanced_renderer<Geo>(&self, geometry: Geometry<Geo, GLayout>) -> InstancedMesh<Geo, GLayout, ILayout>
    where
        Geo: GeoUnit<Vert = GLayout::Vert>
    {
        InstancedMesh::new_with_layout(geometry, self.instance_layout.clone().unwrap())
    }
}



