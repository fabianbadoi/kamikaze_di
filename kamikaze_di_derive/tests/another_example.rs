#[macro_use]
extern crate kamikaze_di_derive;
extern crate kamikaze_di;

use kamikaze_di::{Injector, ContainerBuilder, Result};
use std::rc::Rc;

#[derive(Inject, Clone)]
struct Config {
    pub db: String,
}

#[derive(InjectAsRc, Clone)]
struct DatabaseConnection {
    config: Config,
}

#[derive(Inject, Clone)]
struct UserRepository {
    db_connection: Rc<DatabaseConnection>,
}

#[test]
fn test_derive() {
    let mut builder = ContainerBuilder::new();
    builder
        .register::<Config>(Config {
            db: "localhost".to_string(),
        })
        .unwrap();

    let container = builder.build();

    let user_repo_result: Result<UserRepository> = container.inject();

    assert!(user_repo_result.is_ok());

    let _user_repo = user_repo_result.unwrap();
}
