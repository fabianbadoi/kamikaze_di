use std::rc::Rc;

use super::private::Sealed;
use crate::container::Container;
use crate::Result;

/// Resolves dependencies automatically.
///
/// This trait allows the container to resolve some types without
/// them having to be registered beforehand.
///
/// See the [Inject](trait.Inject.html) trait for examples.
pub trait Injector<T>: Sealed {
    /// Produces T.
    fn inject(&self) -> Result<T>;
}

/// Resolves itself from a container.
///
/// Allows the type to be resolved by the container without having to
/// register it beforehand. If you don't want to also implement Clone,
/// which this trait requires, use [InjectAsRc](trait.InjectAsRc.html).
///
/// # Examples
///
/// ```
/// use kamikaze_di::{Result, Container, ContainerBuilder, Inject, Injector};
///
/// # fn main() -> std::result::Result<(), String> {
/// #
/// #[derive(Clone)]
/// struct Point { x: i32, y: i32 }
///
/// impl Inject for Point {
///     fn resolve(container: &Container) -> Result<Self> {
///         // You can use the container here.
///         // As long as the compile can figure out the type you want,
///         // it will do the right thing.
///         Ok(Point { x: container.inject()?, y: 5 })
///     }
/// }
///
/// let mut container_builder = ContainerBuilder::new();
/// container_builder.register::<i32>(42);
///
/// let container = container_builder.build();
///
/// let point: Point = container.inject()?;
///
/// assert_eq!(42, point.x);
/// assert_eq!( 5, point.y);
/// #
/// # Ok(())
/// # }
/// ```
pub trait Inject
where
    Self: Sized,
{
    /// Resolve Self from a Container.
    fn resolve(container: &Container) -> Result<Self>;
}

/// Resolves itself from a container as a Rc<T>.
///
/// Allows the type to be resolved by the container without having to
/// register it beforehand. Use this if you don't want your type to
/// implement Clone.
///
/// # Examples
///
/// ```
/// use std::rc::Rc;
/// use kamikaze_di::{Result, Container, ContainerBuilder, InjectAsRc, Injector};
///
/// # fn main() -> std::result::Result<(), String> {
/// #
/// struct Point { x: i32, y: i32 }
///
/// impl InjectAsRc for Point {
///     fn resolve(container: &Container) -> Result<Self> {
///         // You can use the container here.
///         // As long as the compile can figure out the type you want,
///         // it will do the right thing.
///         Ok(Point { x: container.inject()?, y: 5 })
///     }
/// }
///
/// let mut container_builder = ContainerBuilder::new();
/// container_builder.register::<i32>(42);
///
/// let container = container_builder.build();
///
/// let point: Rc<Point> = container.inject()?;
///
/// assert_eq!(42, point.x);
/// assert_eq!( 5, point.y);
/// assert_eq!(2, Rc::strong_count(&point));
/// #
/// # Ok(())
/// # }
/// ```
pub trait InjectAsRc
where
    Self: Sized,
{
    /// Resolve Self from a Container.
    ///
    /// The object will be Rc-ed inside the container.
    fn resolve(container: &Container) -> Result<Self>;
}

impl<T> Injector<T> for Container
where
    T: Clone + 'static,
{
    default fn inject(&self) -> Result<T> {
        debug!("injecting registered type");
        self.get()
    }
}

// This would be amazing
//use std::convert::TryFrom;
//impl<T> TryFrom<Container> for T where T: Inject {
//    fn from(other: &Container) -> Result<T> {
//        Injector::<T>::inject(other)
//    }
//}

impl<T> Injector<T> for Container
where
    T: Inject + Clone + 'static,
{
    fn inject(&self) -> Result<T> {
        debug!("injecting Inject type");

        if !self.has::<T>() {
            debug!("Inject type not known, auto-resolving");
            let item = T::resolve(self)?;

            use super::Resolver;
            let resolver = Resolver::Shared(Box::new(item));

            self.insert::<T>(resolver)?;
        }

        self.get()
    }
}

impl<T> Injector<Rc<T>> for Container
where
    T: InjectAsRc + 'static,
{
    fn inject(&self) -> Result<Rc<T>> {
        debug!("injecting InjectAsRc type");

        if !self.has::<Rc<T>>() {
            debug!("InjectAsRc type not known, auto-resolving");

            let item = T::resolve(self)?;

            use super::Resolver;
            let resolver = Resolver::Shared(Box::new(Rc::new(item)));

            self.insert::<Rc<T>>(resolver)?;
        }

        self.get()
    }
}
#[cfg(test)]
mod tests {
    use super::{Inject, Injector};
    use crate::{Container, ContainerBuilder, Result};

    #[derive(Clone)]
    struct X {
        inner: i32,
    }
    #[derive(Clone)]
    struct Y {
        x: X,
    }

    impl Inject for Y {
        fn resolve(container: &Container) -> Result<Self> {
            Ok(Y {
                x: container.inject()?,
            })
        }
    }

    impl Inject for X {
        fn resolve(_: &Container) -> Result<Self> {
            Ok(X { inner: 42 })
        }
    }

    #[test]
    fn container_can_resolve_resolvables_automatically() {
        let container = ContainerBuilder::new().build();

        let x: X = container.inject().expect("expected a value for X");

        assert_eq!(42, x.inner);
    }

    #[test]
    fn auto_resolvables_can_get_chained() {
        let container = ContainerBuilder::new().build();

        let y: Y = container.inject().expect("expected a value for Y");

        assert_eq!(42, y.x.inner);
    }

    #[test]
    fn resolvables_get_stored() {
        use std::rc::Rc;

        #[derive(Clone)]
        struct A {
            inner: Rc<usize>,
        }
        impl Inject for A {
            fn resolve(container: &Container) -> Result<A> {
                Ok(A {
                    inner: container.inject()?,
                })
            }
        }

        let mut builder = ContainerBuilder::new();
        builder.register::<Rc<usize>>(Rc::new(42)).unwrap();

        let container = builder.build();

        let a1: A = container.inject().unwrap();
        let _a2: A = container.inject().unwrap();

        // the inner rc was cloned:
        // - once in inject() when calling resolve() => strong count of inner is 2
        // - once more  when cloning A on the first inject() => 3
        // - a third time when inject()-ing A again => 4
        let a1_was_cloned = Rc::strong_count(&a1.inner) == 4;
        assert!(a1_was_cloned);
    }

    #[test]
    fn test_resolvable_interaction_with_auto_factory() {
        use std::rc::Rc;

        #[derive(Clone)]
        struct A {
            inner: Rc<usize>,
        }
        impl Inject for A {
            fn resolve(container: &Container) -> Result<A> {
                Ok(A {
                    inner: container.inject()?,
                })
            }
        }

        let mut builder = ContainerBuilder::new();
        builder.register::<Rc<usize>>(Rc::new(42)).unwrap();
        builder.register_automatic_factory::<A>().unwrap();

        let container = builder.build();

        let a1: A = container.inject().unwrap();
        let _a2: A = container.inject().unwrap();

        // the inner rc was cloned:
        // - once in inject() when calling resolve() => strong count of inner is 2
        // - once  more when cloning A on the first inject() => 3
        let a1_was_not_cloned = Rc::strong_count(&a1.inner) == 3;
        assert!(a1_was_not_cloned);
    }
}
