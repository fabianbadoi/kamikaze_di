use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;

use super::injector::Inject;
use super::cycle::CycleStopper;
use crate::Result;

use super::{Container, Resolver};

/// Dependency container builder
///
/// You can register shared dependencies (they will act like singletons)
/// with the register() and register_builder() functions.
///
/// You can register factories for dependencies (each request for them
/// will produce a new instance) with the register_factory() and
/// register_automatic_factory() functions.
///
/// Register fuctions return an Err(String) when trying to register the same
/// dependency twice.
///
/// # Examples
///
/// ```
/// # use kamikaze_di::{Container, ContainerBuilder, Resolver};
///
/// let mut builder = ContainerBuilder::new();
/// let result_1 = builder.register::<u32>(42);
/// let result_2 = builder.register::<u32>(43);
///
/// assert!(result_1.is_ok());
/// assert!(result_2.is_err());
///
/// let container = builder.build();
/// assert_eq!(container.resolve::<u32>().unwrap(), 42);
/// ```
///
/// Circular dependencies will cause continer.resolve() to panic:
/// ```should_panic
/// # use kamikaze_di::{Container, ContainerBuilder, Resolver};
///
/// let mut builder = ContainerBuilder::new();
///
/// builder.register_factory::<i32, _>(|container| {
///     use std::convert::TryInto;
///
///     let base: i64 = container.resolve().unwrap();
///     let base: i32 = base.try_into().unwrap();
///     base - 1
/// });
///
/// builder.register_factory::<i64, _>(|container| {
///     let base: i32 = container.resolve().unwrap();
///     let base: i64 = base.into();
///     base - 1
/// });
///
/// let container = builder.build();
///
/// let forty_one: i64 = container.resolve().unwrap();
/// ```
#[derive(Default)]
pub struct ContainerBuilder {
    resolvers: HashMap<TypeId, Resolver>,
}

impl ContainerBuilder {
    pub fn new() -> ContainerBuilder {
        Default::default()
    }

    pub fn build(self) -> Container {
        Container {
            resolvers: RefCell::new(self.resolvers),
            cycle_stopper: CycleStopper::default(),
        }
    }

    /// Registeres a dependency directly
    ///
    /// # Examples
    ///
    /// ```
    /// # use kamikaze_di::{Container, ContainerBuilder, Resolver};
    ///
    /// let mut builder = ContainerBuilder::new();
    /// let result = builder.register::<u32>(42);
    ///
    /// assert!(result.is_ok());
    /// ```
    pub fn register<T: 'static>(&mut self, item: T) -> Result<()> {
        // shared resolvers hold Box<Any>
        let resolver = Resolver::Shared(Box::new(item));

        self.insert::<T>(resolver)
    }

    /// Registers a factory.
    ///
    /// Every call to get() will return a new dependency.
    ///
    /// # Examples
    ///
    /// ```
    /// # use kamikaze_di::{Container, ContainerBuilder, Resolver};
    ///
    /// let mut builder = ContainerBuilder::new();
    /// builder.register::<i16>(43);
    ///
    /// let mut i = 0;
    /// builder.register_factory::<i32, _>(move |container| {
    ///     i += 1;
    ///     let base: i16 = container.resolve().unwrap();
    ///     let base: i32 = base.into();
    ///     base - i
    /// });
    ///
    /// let container = builder.build();
    ///
    /// let forty_two: i32 = container.resolve().unwrap();
    /// let forty_one: i32 = container.resolve().unwrap();
    ///
    /// assert_eq!(forty_two, 42);
    /// assert_eq!(forty_one, 41);
    /// ```
    pub fn register_factory<T, F>(&mut self, factory: F) -> Result<()>
    where
        F: (FnMut(&Container) -> T) + 'static,
        T: 'static,
    {
        // We use double boxes so we can downcast to the inner box type.
        // you can only downcast to Sized types, that's why we need an inner box
        // see call_factory() for use.
        let boxed = Box::new(factory) as Box<(FnMut(&Container) -> T) + 'static>;
        let boxed = Box::new(boxed) as Box<Any>;
        let resolver = Resolver::Factory(RefCell::new(boxed));

        self.insert::<T>(resolver)
    }

    /// Makes the container **not** reuse this type. Every call to get() will
    /// give you a new instance.
    ///
    /// # Examples
    ///
    /// ```
    /// # use kamikaze_di::{Container, ContainerBuilder, Resolver, Inject, Result};
    /// # use std::rc::Rc;
    /// #
    /// #[derive(Clone)]
    /// struct X {}
    /// impl Inject for X {
    ///     fn resolve(container: &Container) -> Result<Self> {
    ///         Ok(X {})
    ///     }
    /// }
    /// let mut builder = ContainerBuilder::new();
    /// builder.register::<Rc<usize>>(Rc::new(42));
    /// builder.register_automatic_factory::<X>();
    ///
    /// let container = builder.build();
    ///
    /// let x1 = container.resolve::<X>().unwrap();
    /// let x2 = container.resolve::<X>().unwrap();
    /// ```
    pub fn register_automatic_factory<T: Inject + 'static>(&mut self) -> Result<()> {
        self.register_factory(auto_factory::<T>)
    }

    /// Registers a builder.
    ///
    /// The dependency is created only when needed and after that
    /// it behaves as if registered via register(item).
    ///
    /// # Examples
    ///
    /// ```
    /// # use kamikaze_di::{Container, ContainerBuilder, Resolver};
    ///
    /// let mut builder = ContainerBuilder::new();
    /// builder.register::<i16>(43);
    ///
    /// builder.register_builder::<i32, _>(|container| {
    ///     let base = container.resolve::<i16>().unwrap();
    ///     let base: i32 = base.into();
    ///     base - 1
    /// });
    ///
    /// builder.register_builder::<i64, _>(|container| {
    ///     let base = container.resolve::<i32>().unwrap();
    ///     let base: i64 = base.into();
    ///     base - 1
    /// });
    ///
    /// let container = builder.build();
    ///
    /// let forty_one = container.resolve::<i64>().unwrap();
    /// let forty_two = container.resolve::<i32>().unwrap();
    ///
    /// assert_eq!(forty_one, 41);
    /// assert_eq!(forty_two, 42);
    /// ```
    pub fn register_builder<T, B>(&mut self, builder: B) -> Result<()>
    where
        B: (FnOnce(&Container) -> T) + 'static,
        T: 'static,
    {
        // We use double boxes so we can downcast to the inner box type.
        // you can only downcast to Sized types, that's why we need an inner box
        // see consume_builder() for use.
        let boxed = Box::new(builder) as Box<(FnOnce(&Container) -> T) + 'static>;
        let boxed = Box::new(boxed) as Box<Any>;
        let resolver = Resolver::Builder(boxed);

        self.insert::<T>(resolver)
    }

    /// Returns true if a dependency is registered
    ///
    /// # Examples
    ///
    /// ```
    /// # use kamikaze_di::{Container, ContainerBuilder, Resolver};
    ///
    /// let mut builder = ContainerBuilder::new();
    /// builder.register::<i16>(43);
    ///
    /// assert!(builder.has::<i16>());
    /// assert!(!builder.has::<i32>());
    /// ```
    pub fn has<T: 'static>(&self) -> bool {
        let type_id = TypeId::of::<T>();

        self.resolvers.contains_key(&type_id)
    }

    fn insert<T: 'static>(&mut self, resolver: Resolver) -> Result<()> {
        let type_id = TypeId::of::<T>();

        if self.has::<T>() {
            return Err(format!("Container already has {:?}", type_id));
        }

        self.resolvers.insert(type_id, resolver);

        Ok(())
    }
}

fn auto_factory<T: Inject>(container: &Container) -> T {
    T::resolve(container).unwrap()
}
