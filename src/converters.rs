use crate::builtin::{
    instanced::dynamic as dyn_inst,
    meshes::dynamic as dyn_mesh
};
use crate::builtin::meshes::dynamic::DynamicGeometryError;

impl dyn_inst::Layout {
    fn as_geo_layout(&self) -> Result<dyn_mesh::Layout, DynamicGeometryError> {
        let mut layout = dyn_mesh::Layout::new();
        
        for attr in &self.attrs {
            layout.add_attribute(attr.name.clone(), attr.span)?;
        }
        
        layout.build(None, None)
    }
}

impl TryFrom<(dyn_mesh::Layout, dyn_inst::Layout)> for dyn_mesh::Layout {
    type Error = String;
    fn try_from(value: (dyn_mesh::Layout, dyn_inst::Layout)) -> Result<Self, Self::Error> {
        let (msh, ins) = value;
        let mut base = ins.as_geo_layout().map_err(|e| format!("{e}"))?;
        let base_len = base.attrs.len() as u16;
        
        base.position_marker = msh.position_marker.map(|x| x + base_len);
        base.normal_marker = msh.normal_marker.map(|x| x + base_len);
        base.attrs.extend_from_slice(msh.attrs.as_slice());
        Ok(base)
    }
}


