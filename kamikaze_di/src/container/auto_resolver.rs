use crate::Result;
use crate::container::{Container};

/// This trait allows the container to resolve some types without
/// them having to be registered beforehand.
///
/// See the Resolvable trait for examples.
pub trait AutoResolver<T> {
    fn resolve(&self) -> Result<T>;
}

/// Allows the type to be resolved by the container without having to
/// register it beforehand.
///
/// # Examples
///
/// ```
/// use kamikaze_di::{Result, Container, ContainerBuilder, Resolvable, AutoResolver};
///
/// #[derive(Clone)]
/// struct Point { x: i32, y: i32 }
///
/// impl Resolvable for Point {
///     fn resolve_from(container: &Container) -> Result<Self> {
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
pub trait Resolvable where Self: Sized {
    fn resolve_from(container: &Container) -> Result<Self>;
}

impl<T> AutoResolver<T> for Container where T: Clone + 'static {
    default fn resolve(&self) -> Result<T> {
        self.get()
    }
}

impl<T> AutoResolver<T> for Container where T: Resolvable + Clone + 'static {
    fn resolve(&self) -> Result<T> {
        T::resolve_from(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::Result;
    use crate::container::{ContainerBuilder, Container};
    use super::{Resolvable, AutoResolver};

    #[derive(Clone)]
    struct X { inner: i32 }
    #[derive(Clone)]
    struct Y { x: X }

    impl Resolvable for Y {
        fn resolve_from(container: &Container) -> Result<Self> {
            Ok(Y { x: container.resolve()? })
        }
    }

    impl Resolvable for X {
        fn resolve_from(_: &Container) -> Result<Self> {
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
}
