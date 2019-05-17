#[macro_use]
extern crate kamikaze_di_derive;
extern crate kamikaze_di;

use kamikaze_di::{AutoResolver, ContainerBuilder, Result};

#[derive(Resolve, Clone)]
struct X {
    u: usize,
}

#[derive(Resolve, Clone)]
struct Y {
    x: X,
}

#[test]
fn test_derive() {
    let mut builder = ContainerBuilder::new();
    builder.register::<usize>(42).unwrap();

    let container = builder.build();

    let y: Result<Y> = container.resolve();

    assert!(y.is_ok());
}
