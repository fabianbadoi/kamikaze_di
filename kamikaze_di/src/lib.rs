#![feature(specialization)]

mod container;

pub use container::Container;
pub use container::builder::ContainerBuilder;
pub use container::auto_resolver::{Resolve, AutoResolver};
pub use container::omni_resolver::OmniResolver;

pub type Result<T> = std::result::Result<T, String>;
