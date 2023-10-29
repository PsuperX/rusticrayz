#[cfg(feature = "dynamic")]
#[allow(unused_imports)]
#[allow(clippy::single_component_path_imports)]
use link_dynamic;

mod app;
mod layer;
mod raytracer;

pub use app::Application;
