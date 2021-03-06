pub mod builder;
pub mod injector;
pub mod resolver;

mod cycle;

use std::any::{Any, TypeId};
use std::cell::RefCell;
use std::collections::HashMap;

use crate::Result;
use cycle::CycleStopper;

/// Dependency container. Can be used with Resolver or Injector.
///
/// See [Injector](trait.Injector.html) and [Resolver](trait.Resolver.html) on how to use.
/// Use the [ContainerBuilder](struct.ContainerBuilder.html) to set up containers.
#[derive(Debug)]
pub struct Container {
    resolvers: RefCell<HashMap<TypeId, Resolver>>,
    cycle_stopper: CycleStopper,
}

// TODO these can be trait aliases, once that feature becomes stable
/// Factories can be called multiple times
pub type Factory<T> = dyn FnMut(&Container) -> T;
/// Builders will only be called once
pub type Builder<T> = dyn FnOnce(&Container) -> T;

impl Container {
    /// Creates an empty container.
    ///
    /// Even though it will be empty, it will still resolve
    /// dependencies via the [Injector](trait.Injector.html) trait.
    ///
    /// # Examples
    /// ```
    /// use kamikaze_di::{Container, Inject, Injector, Result};
    ///
    /// # fn main() -> std::result::Result<(), String> {
    /// #
    /// #[derive(Clone)]
    /// struct X {
    ///     inner: usize,
    /// };
    ///
    /// impl Inject for X {
    ///     fn resolve(container: &Container) -> Result<X> {
    ///         Ok(X { inner: 42 })
    ///     }
    /// }
    ///
    /// let container: Container = Container::new();
    /// let x: X = container.inject()?;
    ///
    /// assert_eq!(42, x.inner);
    /// #
    /// # Ok(())
    /// # }
    /// ```
    pub fn new() -> Container {
        Container {
            resolvers: RefCell::new(Default::default()),
            cycle_stopper: Default::default(),
        }
    }

    fn has<T: 'static>(&self) -> bool {
        debug!("has called");

        let type_id = TypeId::of::<T>();

        self.resolvers.borrow().contains_key(&type_id)
    }

    fn get<T: Clone + 'static>(&self) -> Result<T> {
        debug!("resolving type via .get()");

        let type_id = TypeId::of::<T>();
        let _guard = self.cycle_stopper.track(type_id);

        let resolver_type = self.get_resolver_type(type_id);
        debug!("resolving via {:?}", resolver_type);

        match resolver_type {
            Some(ResolverType::Factory) => self.call_factory::<T>(type_id),
            Some(ResolverType::Builder) => {
                self.consume_builder::<T>()?;
                self.get_shared(type_id)
            }
            Some(ResolverType::Shared) => self.get_shared(type_id),
            None => Err(format!("Type not registered: {:?}", type_id).into()),
        }
    }

    fn get_resolver_type(&self, type_id: TypeId) -> Option<ResolverType> {
        self.resolvers.borrow().get(&type_id).map(|r| r.into())
    }

    fn call_factory<T: 'static>(&self, type_id: TypeId) -> Result<T> {
        if let Resolver::Factory(cell) = self
            .resolvers
            .borrow()
            .get(&type_id)
            .expect("could not find a registered factory")
        {
            let mut boxed = cell.borrow_mut();
            let factory = boxed
                .downcast_mut::<Box<Factory<T>>>()
                .expect("could not downcast factory");

            let item = factory(self);

            return Ok(item);
        }

        panic!("Type {:?} not registered as factory", type_id)
    }

    fn consume_builder<T: 'static>(&self) -> Result<()> {
        let type_id = TypeId::of::<T>();

        let builder = if let Resolver::Builder(boxed) = self
            .resolvers
            .borrow_mut()
            .remove(&type_id)
            .expect("could not find a registered resolver")
        {
            boxed
                .downcast::<Box<Builder<T>>>()
                .expect("could not downcast builder")
        } else {
            panic!("Type {:?} not registered as builder", type_id)
        };

        let item = builder(self);
        let resolver = Resolver::Shared(Box::new(item));

        self.insert::<T>(resolver)
    }

    fn get_shared<T: Clone + 'static>(&self, type_id: TypeId) -> Result<T> {
        if let Resolver::Shared(boxed_any) = self
            .resolvers
            .borrow()
            .get(&type_id)
            .expect("could not find a registered type")
        {
            use std::borrow::Borrow;

            let borrowed_any: &dyn Any = boxed_any.borrow();
            let borrowed_item: &T = borrowed_any
                .downcast_ref()
                .expect("could not downcast shared object");

            return Ok(borrowed_item.clone());
        }

        panic!("Type {:?} not registered as shared dependency", type_id)
    }

    fn insert<T: 'static>(&self, resolver: Resolver) -> Result<()> {
        debug!("inerting new type");

        let type_id = TypeId::of::<T>();

        if self.has::<T>() {
            return Err(format!("Container already has {:?}", type_id).into());
        }

        self.resolvers.borrow_mut().insert(type_id, resolver);

        Ok(())
    }
}

impl Default for Container {
    fn default() -> Container {
        Container::new()
    }
}

#[derive(Debug)]
enum Resolver {
    /// Factories get called multiple times
    ///
    /// Factories are called by the container, and they themselves will
    /// call container.resolve() as they see fit. This means we can't
    /// own a mutable borrow to the resolvers collection during the
    /// calls. Thus we must use RefCell.
    Factory(RefCell<Box<dyn Any>>),
    Builder(Box<dyn Any>),
    Shared(Box<dyn Any>),
}

#[derive(Debug)]
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
    use super::builder::ContainerBuilder;
    use crate::Resolver;

    #[test]
    #[should_panic(expected = "Circular dependency")]
    fn panics_on_circular_dendencies() {
        let mut builder = ContainerBuilder::new();

        builder
            .register_factory::<i32, _>(|container| {
                use std::convert::TryInto;

                let base: i64 = container.resolve().unwrap();
                let base: i32 = base.try_into().unwrap();
                base - 1
            })
            .unwrap();

        builder
            .register_factory::<i64, _>(|container| {
                let base: i32 = container.resolve().unwrap();
                let base: i64 = base.into();
                base - 1
            })
            .unwrap();

        let container = builder.build();

        container.resolve::<i32>().unwrap();
    }
}

// Prevent users from implementing Injector and Resolver
mod private {
    pub trait Sealed {}

    impl Sealed for super::Container {}
}
