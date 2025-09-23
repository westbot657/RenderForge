
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BufferRenderError {
    #[error("Data is missing vertices as defined by the buffer layout")]
    IncompleteTriangleData,
    #[error("Data does not align with vertex format")]
    MalformedData,
}

#[derive(Error, Debug)]
pub enum AttributeError {
    #[error("Invalid attribute name: '{0}'")]
    InvalidName(String),
    #[error("Attribute expects a value size of {expected:?}, got {found:?}")]
    ExpectedSize {
        expected: usize,
        found: usize,
    }
}

#[derive(Error, Debug)]
pub enum AtlasError {
    #[error("Could not fit all textures on atlas")]
    TextureOverflow,

    #[error("Cannot add texture with same id twice: '{0}'")]
    DuplicateId(String),
}

