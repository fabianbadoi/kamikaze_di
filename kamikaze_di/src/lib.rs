#![feature(specialization)]

mod container;

pub use container::auto_resolver::{AutoResolver, Resolve};
pub use container::builder::ContainerBuilder;
pub use container::omni_resolver::OmniResolver;
pub use container::Container;

pub type Result<T> = std::result::Result<T, String>;
