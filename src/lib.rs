pub mod geometry;
pub mod render;

#[cfg(feature = "debug")]
pub trait SizedThreadSafe: Sized + Sync + Send + std::fmt::Debug {}
#[cfg(not(feature = "debug"))]
pub trait SizedThreadSafe: Sized + Sync + Send {}

#[cfg(feature = "debug")]
impl<T> SizedThreadSafe for T where T: Sized + Sync + Send + std::fmt::Debug {}
#[cfg(not(feature = "debug"))]
impl<T> SizedThreadSafe for T where T: Sized + Sync + Send {}






