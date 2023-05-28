pub mod users;

pub mod services {
    use std::{error, fmt};

    pub trait AuthenticateService {
        type Credentials;

        fn authenticate(&self, credentials: Self::Credentials) -> Result<(), AuthenticateError>;
    }

    #[derive(Debug)]
    pub struct AuthenticateError {}

    impl fmt::Display for AuthenticateError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "authenticate error")
        }
    }

    impl error::Error for AuthenticateError {}
}
