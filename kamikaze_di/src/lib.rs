#![feature(specialization)]

mod container;

pub use container::injector::{Injector, Inject, InjectAsRc};
pub use container::builder::ContainerBuilder;
pub use container::resolver::Resolver;
pub use container::Container;

pub type Result<T> = std::result::Result<T, String>;
