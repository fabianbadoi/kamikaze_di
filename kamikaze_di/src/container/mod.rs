mod resolver;

use std::any::{Any, TypeId};
use std::collections::HashMap;
use std::cell::RefCell;

use crate::cycle::CycleStopper;
pub use resolver::*;

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
/// # use kamikaze_di::{Container, ContainerBuilder, DependencyResolver};
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
/// # use kamikaze_di::{Container, ContainerBuilder, DependencyResolver};
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
pub struct ContainerBuilder {
    resolvers: HashMap<TypeId, Resolver>,
}

pub struct Container {
    resolvers: RefCell<HashMap<TypeId, Resolver>>,
    cycle_stopper: CycleStopper,
}

// TODO these can be trait aliases, once that feature becomes stable
/// Factories can be called multiple times
pub type Factory<T> = FnMut(&Container) -> T;
/// Builders will only be called once
pub type Builder<T> = FnOnce(&Container) -> T;

impl ContainerBuilder {
    pub fn new() -> ContainerBuilder {
        ContainerBuilder {
            resolvers: Default::default()
        }
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
    /// # use kamikaze_di::{Container, ContainerBuilder, DependencyResolver};
    ///
    /// let mut builder = ContainerBuilder::new();
    /// let result = builder.register::<u32>(42);
    ///
    /// assert!(result.is_ok());
    /// ```
    pub fn register<T: 'static>(&mut self, item: T) -> Result<()> {
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
    /// # use kamikaze_di::{Container, ContainerBuilder, DependencyResolver};
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
        where F: (FnMut(&Container) -> T) + 'static,
              T: 'static
    {
        // We use double boxes so we can downcast to the inner box type.
        // you can only downcast to Sized types, that's why we need an inner box
        // see call_factory() for use.
        let boxed = Box::new(factory) as Box<(FnMut(&Container) -> T) + 'static>;
        let boxed = Box::new(boxed) as Box<Any>;
        let resolver = Resolver::Factory(RefCell::new(boxed));

        self.insert::<T>(resolver)
    }

    pub fn register_automatic_factory<T: 'static>(&mut self) -> Result<()> {
        //let resolver = Resolver::Factory(Box::new(|container| auto_factory::<T>(container)));

        //self.insert::<T>(resolver)
        unimplemented!("This will be implemented later")
    }

    /// Registers a builder.
    ///
    /// The dependency is created only when needed and after that
    /// it behaves as if registered via register(item).
    ///
    /// # Examples
    ///
    /// ```
    /// # use kamikaze_di::{Container, ContainerBuilder, DependencyResolver};
    ///
    /// let mut builder = ContainerBuilder::new();
    /// builder.register::<i16>(43);
    ///
    /// builder.register_builder::<i32, _>(|container| {
    ///     let base: i16 = container.resolve().unwrap();
    ///     let base: i32 = base.into();
    ///     base - 1
    /// });
    ///
    /// builder.register_builder::<i64, _>(|container| {
    ///     let base: i32 = container.resolve().unwrap();
    ///     let base: i64 = base.into();
    ///     base - 1
    /// });
    ///
    /// let container = builder.build();
    ///
    /// let forty_one: i64 = container.resolve().unwrap();
    /// let forty_two: i32 = container.resolve().unwrap();
    ///
    /// assert_eq!(forty_one, 41);
    /// assert_eq!(forty_two, 42);
    /// ```
    pub fn register_builder<T, B>(&mut self, builder: B) -> Result<()>
        where B: (FnOnce(&Container) -> T) + 'static,
              T: 'static
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
    /// # use kamikaze_di::{Container, ContainerBuilder, DependencyResolver};
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

impl Container {
    fn has<T: 'static>(&self) -> bool {
        let type_id = TypeId::of::<T>();

        self.resolvers.borrow().contains_key(&type_id)
    }

    fn get<T: Clone + 'static>(&self) -> Result<T> {
        self.resolve_as_any::<T>()
    }

    fn resolve_as_any<T: Clone + 'static>(&self) -> Result<T> {
        let type_id = TypeId::of::<T>();
        let _guard = self.cycle_stopper.track(type_id);

        let resolver_type = self.get_resolver_type(&type_id);

        match resolver_type {
            Some(ResolverType::Factory) => self.call_factory::<T>(&type_id),
            Some(ResolverType::Builder) => {
                self.consume_builder::<T>()?;
                self.get_shared(&type_id)
            },
            Some(ResolverType::Shared) => self.get_shared(&type_id),
            None => Err(format!("Type not registered: {:?}", type_id)),
        }
    }

    fn get_resolver_type(&self, type_id: &TypeId) -> Option<ResolverType> {
        self.resolvers.borrow()
            .get(type_id)
            .map(|r| r.into())
    }

    fn call_factory<T: 'static>(&self, type_id: &TypeId) -> Result<T> {
        if let Resolver::Factory(cell) = self.resolvers.borrow().get(&type_id).unwrap() {
            let mut boxed = cell.borrow_mut();
            let factory = boxed.downcast_mut::<Box<Factory<T>>>().unwrap();

            let item = factory(self);

            return Ok(item);
        }

        panic!("Type {:?} not registered as factory", type_id)
    }

    fn consume_builder<T: 'static>(&self) -> Result<()> {
        let type_id = TypeId::of::<T>();

        let builder = if let Resolver::Builder(boxed) = self.resolvers.borrow_mut().remove(&type_id).unwrap() {
            boxed.downcast::<Box<Builder<T>>>().unwrap()
        } else {
            panic!("Type {:?} not registered as builder", type_id)
        };

        let item = builder(self);
        let resolver = Resolver::Shared(Box::new(item));

        return self.insert::<T>(resolver);
    }

    fn get_shared<T: Clone + 'static>(&self, type_id: &TypeId) -> Result<T> {
        if let Resolver::Shared(boxed_any) = self.resolvers.borrow().get(&type_id).unwrap() {
            use std::borrow::Borrow;

            let borrowed_any: &Any = boxed_any.borrow();
            let borrowed_item: &T = borrowed_any.downcast_ref().unwrap();

            return Ok(borrowed_item.clone());
        }

        panic!("Type {:?} not registered as shared dependency", type_id)
    }

    fn insert<T: 'static>(&self, resolver: Resolver) -> Result<()> {
        let type_id = TypeId::of::<T>();

        if self.has::<T>() {
            return Err(format!("Container already has {:?}", type_id));
        }

        self.resolvers.borrow_mut().insert(type_id, resolver);

        Ok(())
    }
}

fn auto_factory<T>(container: &Container) -> Box<T> {
    unimplemented!()
}

enum Resolver {
    /// Factories get called multiple times
    ///
    /// Factories are called by the container, and they themselves will
    /// call container.resolve() as they see fit. This means we can't
    /// own a mutable borrow to the resolvers collection during the
    /// calls. Thus we must use RefCell.
    Factory(RefCell<Box<Any>>),
    Builder(Box<Any>),
    Shared(Box<Any>),
}

pub type Result<T> = std::result::Result<T, String>;

enum ResolverType {
    Factory,
    Builder,
    Shared,
}

impl From<&Resolver> for ResolverType {
    fn from(other: &Resolver) -> Self {
        use ResolverType::*;

        match other {
            Resolver::Factory(_) => Factory,
            Resolver::Builder(_) => Builder,
            Resolver::Shared(_) => Shared,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic(expected = "Circular dependency")]
    fn panics_on_circular_dendencies() {
        let mut builder = ContainerBuilder::new();

        builder.register_factory::<i32, _>(|container| {
            use std::convert::TryInto;

            let base: i64 = container.resolve().unwrap();
            let base: i32 = base.try_into().unwrap();
            base - 1
        }).unwrap();

        builder.register_factory::<i64, _>(|container| {
            let base: i32 = container.resolve().unwrap();
            let base: i64 = base.into();
            base - 1
        }).unwrap();

        let container = builder.build();

        container.resolve::<i32>().unwrap();
    }
}
