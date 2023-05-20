use std::error::Error as StdErr;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum Error {
    Parse(toml::de::Error),
    Io(io::Error),
    Utf8(std::str::Utf8Error),
    Other(String),
}

impl StdErr for Error {
    fn source(&self) -> Option<&(dyn StdErr + 'static)> {
        match *self {
            Error::Parse(ref err) => Some(err),
            Error::Io(ref err) => Some(err),
            Error::Utf8(ref err) => Some(err),
            Error::Other(_) => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Parse(ref err) => err.fmt(f),
            Error::Io(ref err) => err.fmt(f),
            Error::Utf8(ref err) => err.fmt(f),
            Error::Other(msg) => f.write_str(msg),
        }
    }
}

impl Clone for Error {
    fn clone(&self) -> Self {
        match self {
            Error::Parse(ref err) => Error::Parse(err.clone()),
            Error::Io(ref err) => Error::Io(io::Error::new(err.kind(), err.to_string())),
            Error::Utf8(ref err) => Error::Utf8(*err),
            Error::Other(msg) => Error::Other(msg.clone()),
        }
    }
}

impl From<toml::de::Error> for Error {
    fn from(o: toml::de::Error) -> Self {
        Error::Parse(o)
    }
}

impl From<io::Error> for Error {
    fn from(o: io::Error) -> Self {
        Error::Io(o)
    }
}

impl From<std::str::Utf8Error> for Error {
    fn from(o: std::str::Utf8Error) -> Self {
        Error::Utf8(o)
    }
}
