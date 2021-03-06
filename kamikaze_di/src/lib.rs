//! # Kamikaze DI
//! Dependency Injection framework.
//!
//! Have a look at [the readme](https://github.com/fabianbadoi/kamikaze_di) file in the git repo for a more in depth discussion.
//!
//! # In use
//! ```rust,ignore
//! extern crate kamikaze_di;
//! #[macro_use] extern crate kamikaze_di_derive;
//!
//! use kamikaze_di::{Injector, ContainerBuilder, Result};
//! use std::rc::Rc;
//!
//! #[derive(Inject, Clone)]
//! struct Config {
//!    pub db: String,
//! }
//!
//! #[derive(InjectAsRc, Clone)]
//! struct DatabaseConnection {
//!    config: Config,
//! }
//!
//! #[derive(Inject, Clone)]
//! struct UserRepository {
//!    db_connection: Rc<DatabaseConnection>,
//! }
//!
//! # fn main() -> std::result::Result<(), String> {
//! #
//! let mut builder = ContainerBuilder::new();
//! builder
//!    .register::<Config>(Config {
//!        db: "localhost".to_string(),
//!    })?;
//!
//! let container = builder.build();
//!
//! let user_repo_result: Result<UserRepository> = container.inject();
//!
//! assert!(user_repo_result.is_ok());
//!
//! let _user_repo = user_repo_result?;
//! #
//! # Ok(())
//! # }
//! ```
#![doc(html_root_url = "https://docs.rs/kamikaze_di/0.1.0")]
#![feature(specialization)]
#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code,
    unused_import_braces,
    unused_qualifications
)]

#[macro_use]
extern crate log;

mod container;
mod error;

pub use container::builder::ContainerBuilder;
pub use container::injector::{Inject, InjectAsRc, Injector};
pub use container::resolver::Resolver;
pub use container::Container;
pub use error::Error;

/// Result type
pub type Result<T> = std::result::Result<T, Error>;
