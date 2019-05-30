#[macro_use]
extern crate kamikaze_di_derive;
extern crate kamikaze_di;
#[macro_use]
extern crate log;

use kamikaze_di::{Injector, ContainerBuilder, Result};
use std::rc::Rc;

#[derive(Inject, Clone)]
struct X {
    u: usize,
}

#[derive(Inject, Clone)]
struct Y {
    _x: X,
}

#[derive(InjectAsRc)]
struct Z {
    _x: X,
}

#[test]
fn test_derive() {
    let mut builder = ContainerBuilder::new();
    builder.register::<usize>(42).unwrap();

    let container = builder.build();

    let y: Result<Y> = container.inject();

    assert!(y.is_ok());
}

#[test]
fn test_derive_to_rc() {
    let mut builder = ContainerBuilder::new();
    builder.register::<usize>(42).unwrap();

    let container = builder.build();

    let z: Result<Rc<Z>> = Injector::<Rc<Z>>::inject(&container);

    assert!(z.is_ok());
}
