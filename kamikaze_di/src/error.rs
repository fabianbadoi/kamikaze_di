/// Error type
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Default)]
pub struct Error {
    message: String,
}

impl From<String> for Error {
    fn from(message: String) -> Error {
        Error { message }
    }
}

impl From<&str> for Error {
    fn from(message: &str) -> Error {
        let message = message.to_string();
        Error { message }
    }
}

impl From<Error> for String {
    fn from(error: Error) -> String {
        error.message
    }
}

impl std::error::Error for Error {
    fn description(&self) -> &str {
        &self.message
    }

    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.message.fmt(f)
    }
}

impl Error {
    pub fn with_message(message: &str) -> Error {
        message.into()
    }
}

#[cfg(test)]
mod tests {
    use super::Error;

    #[test]
    fn test_send() {
        fn assert_send<T: Send>() {}
        assert_send::<Error>();
    }

    #[test]
    fn test_sync() {
        fn assert_sync<T: Sync>() {}
        assert_sync::<Error>();
    }
}
