use std::{fmt,error};

#[derive(Debug)]
pub enum Error {
    Message(String),
    Error(Box<error::Error>),
}


impl Error {

    pub fn message<M: Into<String>>(msg: M) -> Self { Error::Message(msg.into()) }
}


impl fmt::Display for Error {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Message(msg) => f.write_str(msg),
            Error::Error(err) => err.fmt(f),
        }
    }
}


impl<T> From<T> for Error where T: error::Error + 'static {

    fn from(err: T) -> Self { Error::Error(Box::new(err)) }
}

