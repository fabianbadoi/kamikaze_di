#![feature(specialization)]

mod container;

pub use container::auto_resolver::{AutoResolver, Resolve, ResolveToRc};
pub use container::builder::ContainerBuilder;
pub use container::resolver::Resolver;
pub use container::Container;

pub type Result<T> = std::result::Result<T, String>;
