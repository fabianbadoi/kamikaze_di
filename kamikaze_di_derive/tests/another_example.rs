#[macro_use]
extern crate kamikaze_di_derive;
extern crate kamikaze_di;

use kamikaze_di::{AutoResolver, ContainerBuilder, Result};

#[derive(Resolve, Clone)]
struct Config {
    pub db: String,
}

#[derive(Resolve, Clone)]
struct DatabaseConnection {
    config: Config,
}

#[derive(Resolve, Clone)]
struct UserRepository {
    // TODO RC<>
    db_connection: DatabaseConnection,
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

    let user_repo_result: Result<UserRepository> = container.resolve();

    assert!(user_repo_result.is_ok());

    let _user_repo = user_repo_result.unwrap();
}
