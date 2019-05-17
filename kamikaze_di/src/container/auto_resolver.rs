use crate::Result;
use crate::container::{Container};

/// This trait allows the container to resolve some types without
/// them having to be registered beforehand.
///
/// See the Resolve trait for examples.
pub trait AutoResolver<T> {
    fn resolve(&self) -> Result<T>;
}

/// Allows the type to be resolved by the container without having to
/// register it beforehand.
///
/// # Examples
///
/// ```
/// use kamikaze_di::{Result, Container, ContainerBuilder, Resolve, AutoResolver};
///
/// #[derive(Clone)]
/// struct Point { x: i32, y: i32 }
///
/// impl Resolve for Point {
///     fn resolve(container: &Container) -> Result<Self> {
///         // You can use the container here.
///         // As long as the compile can figure out the type you want,
///         // it will do the right thing.
///         Ok(Point { x: container.resolve()?, y: 5 })
///     }
/// }
///
/// let mut container_builder = ContainerBuilder::new();
/// container_builder.register::<i32>(42);
///
/// let container = container_builder.build();
/// 
/// let point: Point = container.resolve().unwrap();
/// 
/// assert_eq!(42, point.x);
/// assert_eq!( 5, point.y);
/// ```
pub trait Resolve where Self: Sized {
    fn resolve(container: &Container) -> Result<Self>;
}

impl<T> AutoResolver<T> for Container where T: Clone + 'static {
    default fn resolve(&self) -> Result<T> {
        self.get()
    }
}

// This would be amazing
//use std::convert::TryFrom;
//impl<T> TryFrom<Container> for T where T: Resolve {
//    fn from(other: &Container) -> Result<T> {
//        AutoResolver::<T>::resolve(other)
//    }
//}

impl<T> AutoResolver<T> for Container where T: Resolve + Clone + 'static {
    fn resolve(&self) -> Result<T> {
        if !self.has::<T>() {
            let item = T::resolve(self)?;

            use super::Resolver;
            let resolver = Resolver::Shared(Box::new(item));

            self.insert::<T>(resolver)?;
        }

        self.get()
    }
}

#[cfg(test)]
mod tests {
    use crate::{Result, ContainerBuilder, Container};
    use super::{Resolve, AutoResolver};

    #[derive(Clone)]
    struct X { inner: i32 }
    #[derive(Clone)]
    struct Y { x: X }

    impl Resolve for Y {
        fn resolve(container: &Container) -> Result<Self> {
            Ok(Y { x: container.resolve()? })
        }
    }

    impl Resolve for X {
        fn resolve(_: &Container) -> Result<Self> {
            Ok(X { inner: 42 })
        }
    }

    #[test]
    fn container_can_resolve_resolvables_automatically() {
        let container = ContainerBuilder::new().build();

        let x: X = container.resolve().expect("expected a value for X");

        assert_eq!(42, x.inner);
    }

    #[test]
    fn auto_resolvables_can_get_chained()
    {
        let container = ContainerBuilder::new().build();

        let y: Y = container.resolve().expect("expected a value for Y");

        assert_eq!(42, y.x.inner);
    }
    #[test]
    fn resolvables_get_stored() {
        use std::rc::Rc;

        #[derive(Clone)]
        struct A { inner: Rc<usize> }
        impl Resolve for A {
            fn resolve(container: &Container) -> Result<A> {
                Ok(A { inner: container.resolve()? })
            }
        }

        let mut builder = ContainerBuilder::new();
        builder.register::<Rc<usize>>(Rc::new(42)).unwrap();

        let container = builder.build();

        let a1: A = container.resolve().unwrap();
        let _a2: A = container.resolve().unwrap();


		// the inner rc was cloned:
		// - once in resolve() when calling resolve() => strong count of inner is 2
		// - once more because more when cloning A on the first resolve => 3
		// - a third time when resolving A again => 4
        let a1_was_cloned = Rc::strong_count(&a1.inner) == 4;
        assert!(a1_was_cloned);
    }

    #[test]
    fn test_resolvable_interaction_with_auto_factory() {
        use std::rc::Rc;

        #[derive(Clone)]
        struct A { inner: Rc<usize> }
        impl Resolve for A {
            fn resolve(container: &Container) -> Result<A> {
                Ok(A { inner: container.resolve()? })
            }
        }

        let mut builder = ContainerBuilder::new();
        builder.register::<Rc<usize>>(Rc::new(42)).unwrap();
        builder.register_automatic_factory::<A>().unwrap();

        let container = builder.build();

        let a1: A = container.resolve().unwrap();
        let _a2: A = container.resolve().unwrap();


		// the inner rc was cloned:
		// - once in resolve() when calling resolve() => strong count of inner is 2
		// - once more because more when cloning A on the first resolve => 3
        let a1_was_not_cloned = Rc::strong_count(&a1.inner) == 3;
        assert!(a1_was_not_cloned);
    }
}
