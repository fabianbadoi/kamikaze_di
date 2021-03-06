use super::private::Sealed;
use super::*;

/// Resolves dependencies.
///
/// Dependencies have to be registered beforehand, how you do
/// that depends on the implementing type.
///
/// Dependencies can be shared across multiple use points. In
/// garbage collected languages, these dependencies would
/// naturally live on the heap and the garbage collector would
/// take care of deallocating them.
///
/// All dependencies will be cloned when resolving. If you would
/// like to have a shared dependency, use Rc<T>.
///
/// # Using a shared dependency
///
/// ```
/// # use std::rc::Rc;
/// # use kamikaze_di::{Container, ContainerBuilder, Resolver};
/// #
/// # fn main() -> std::result::Result<(), String> {
/// #
/// // does not implement Clone or Copy
/// struct Keeper { x: i32 }
///
/// let mut builder = ContainerBuilder::new();
/// builder.register::<Rc<Keeper>>(Rc::new(Keeper{ x: 42 }));
///
/// let container = builder.build();
///
/// let resolved = container.resolve::<Rc<Keeper>>()?;
/// assert_eq!((*resolved).x, 42);
/// #
/// # Ok(())
/// # }
/// ```
///
///
/// # If you need to resolve a trait, use Rc<Trait>.
///
/// ```
/// # use std::rc::Rc;
/// # use kamikaze_di::{Container, ContainerBuilder, Resolver};
/// #
/// # fn main() -> std::result::Result<(), String> {
/// #
/// // does not implement Clone or Copy
/// struct Keeper { x: i32 }
/// trait XKeeper { fn get_x(&self) -> i32; }
/// impl XKeeper for Keeper { fn get_x(&self) -> i32 { self.x } }
///
/// let mut builder = ContainerBuilder::new();
/// builder.register::<Rc<XKeeper>>(Rc::new(Keeper{ x: 42 }));
///
/// let container = builder.build();
///
/// let resolved = container.resolve::<Rc<XKeeper>>()?;
/// assert_eq!(resolved.get_x(), 42);
/// #
/// # Ok(())
/// # }
/// ```
pub trait Resolver: Sealed {
    /// Resolve a dependency.
    ///
    /// # Examples
    ///
    /// ```
    /// # use kamikaze_di::{Container, ContainerBuilder, Resolver};
    /// #
    /// # fn main() -> std::result::Result<(), String> {
    /// #
    /// let mut builder = ContainerBuilder::new();
    /// builder.register::<u32>(42);
    ///
    /// let container = builder.build();
    ///
    /// let resolved: u32 = container.resolve()?;
    /// assert_eq!(resolved, 42);
    /// #
    /// # Ok(())
    /// # }
    /// ```
    fn resolve<T: Clone + 'static>(&self) -> Result<T>;

    /// Returns true if a dependency is registered.
    ///
    /// # Examples
    ///
    /// ```
    /// # use kamikaze_di::{Container, ContainerBuilder, Resolver};
    /// #
    /// let mut builder = ContainerBuilder::new();
    /// builder.register::<i16>(43);
    /// let container = builder.build();
    ///
    /// assert!(container.has::<i16>());
    /// assert!(!container.has::<i32>());
    /// ```
    fn has<T: 'static>(&self) -> bool;
}

impl Resolver for Container {
    fn resolve<T: Clone + 'static>(&self) -> Result<T> {
        self.get::<T>()
    }

    fn has<T: 'static>(&self) -> bool {
        self.has::<T>()
    }
}
