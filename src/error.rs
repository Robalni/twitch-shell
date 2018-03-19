use std;
use std::error::Error;
use std::fmt;
use json;
extern crate curl;

#[derive(Debug)]
pub enum MyError {
    IoError(::std::io::Error),
    FromUtf8Error(::std::string::FromUtf8Error),
    JsonError(json::Error),
    ParseIntError(std::num::ParseIntError),
    NetworkError(curl::Error),
    Other(String),
}

impl From<std::io::Error> for MyError {
    fn from(e: std::io::Error) -> Self {
        MyError::IoError(e)
    }
}

impl From<std::string::FromUtf8Error> for MyError {
    fn from(e: std::string::FromUtf8Error) -> Self {
        MyError::FromUtf8Error(e)
    }
}

impl From<json::Error> for MyError {
    fn from(e: json::Error) -> Self {
        MyError::JsonError(e)
    }
}

impl From<std::num::ParseIntError> for MyError {
    fn from(e: std::num::ParseIntError) -> Self {
        MyError::ParseIntError(e)
    }
}

impl From<String> for MyError {
    fn from(s: String) -> Self {
        MyError::Other(s)
    }
}

impl<'a> From<&'a str> for MyError {
    fn from(s: &str) -> Self {
        MyError::Other(s.to_owned())
    }
}

impl Error for MyError {
    fn description(&self) -> &str {
        use MyError::*;
        match *self {
            IoError(ref e) => e.description(),
            FromUtf8Error(ref e) => e.description(),
            JsonError(ref e) => e.description(),
            ParseIntError(ref e) => e.description(),
            NetworkError(ref e) => e.description(),
            Other(ref s) => &s,
        }
    }
}

impl fmt::Display for MyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        use MyError::*;
        match *self {
            IoError(ref e) => fmt::Display::fmt(&e, f),
            FromUtf8Error(ref e) => fmt::Display::fmt(&e, f),
            JsonError(ref e) => fmt::Display::fmt(&e, f),
            ParseIntError(ref e) => fmt::Display::fmt(&e, f),
            NetworkError(ref e) => fmt::Display::fmt(&e, f),
            Other(ref s) => fmt::Display::fmt(&s, f),
        }
    }
}
