
pub trait Vertex: Sized + Clone + Send + Sync {
    fn write(&self, buffer: &mut Vec<u8>);
}



