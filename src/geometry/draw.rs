
pub trait Data: Sized + Sync + Send {
    fn write(&self, buffer: &mut Vec<u8>);
}
