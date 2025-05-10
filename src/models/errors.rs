// var (
//     ErrNoRecord = errors.New("models: no matching record found")

//     // Add a new ErrInvalidCredentials error. We'll use this later if a user
//     // tries to login with an incorrect email address or password.
//     ErrInvalidCredentials = errors.New("models: invalid credentials")

//     // Add a new ErrDuplicateEmail error. We'll use this later if a user
//     // tries to signup with an email address that's already in use.
//     ErrDuplicateEmail = errors.New("models: duplicate email")
// )

// struct UserErrors {
//     err_no_record:
// }

use std::{
    error::Error,
    fmt::{self, Debug},
};

// #[derive(Debug)]
// struct ErrDuplicateEmail;
// impl Display for ErrDuplicateEmail {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         Ok(())
//     }
// }
// impl Error for ErrDuplicateEmail {}

// enum DBErrors

#[derive(Debug, Clone)]
pub struct ErrInvalidCredentials;

// Generation of an error is completely separate from how it is displayed.
// There's no need to be concerned about cluttering complex logic with the display style.
//
// Note that we don't store any extra info about the errors. This means we can't state
// which string failed to parse without modifying our types to carry that information.
impl fmt::Display for ErrInvalidCredentials {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "invalid credentials")
    }
}

unsafe impl Send for ErrInvalidCredentials {}

impl Error for ErrInvalidCredentials {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        None
    }

    // fn description(&self) -> &str {
    //     "description() is deprecated; use Display"
    // }

    fn cause(&self) -> Option<&dyn Error> {
        self.source()
    }

    // fn provide<'a>(&'a self, request: &mut std::error::Request<'a>) {

    // }
}
