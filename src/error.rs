use std::fmt;
use std::io;
use std::num::{ParseFloatError, ParseIntError};
use std::str::Utf8Error;
use chrono;
use reqwest;
use serde_json;
use term;
use toml;
use url;

#[derive(Debug)]
pub enum Error {
    Io(io::Error),
    ProjectDirs,
    ConfigLoad(toml::de::Error),
    ConfigSave(toml::ser::Error),
    UrlParse(url::ParseError),
    Reqwest(reqwest::Error),
    RequestFailed(reqwest::StatusCode),
    JsonParse(serde_json::Error),
    FloatParse(ParseFloatError),
    IntParse(ParseIntError),
    Utf8Parse(Utf8Error),
    CannotOpenTerminal,
    Terminal(term::Error),
    ChronoParse(chrono::ParseError),
}

impl std::error::Error for Error {}

impl From<io::Error> for Error {
    fn from(error: io::Error) -> Self {
        Error::Io(error)
    }
}

impl From<toml::de::Error> for Error {
    fn from(error: toml::de::Error) -> Self {
        Error::ConfigLoad(error)
    }
}

impl From<toml::ser::Error> for Error {
    fn from(error: toml::ser::Error) -> Self {
        Error::ConfigSave(error)
    }
}

impl From<url::ParseError> for Error {
    fn from(error: url::ParseError) -> Self {
        Error::UrlParse(error)
    }
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        Error::Reqwest(error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::JsonParse(error)
    }
}

impl From<ParseFloatError> for Error {
    fn from(error: ParseFloatError) -> Self {
        Error::FloatParse(error)
    }
}

impl From<ParseIntError> for Error {
    fn from(error: ParseIntError) -> Self {
        Error::IntParse(error)
    }
}

impl From<Utf8Error> for Error {
    fn from(error: Utf8Error) -> Self {
        Error::Utf8Parse(error)
    }
}

impl From<term::Error> for Error {
    fn from(error: term::Error) -> Self {
        Error::Terminal(error)
    }
}

impl From<chrono::ParseError> for Error {
    fn from(error: chrono::ParseError) -> Self {
        Error::ChronoParse(error)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::Io(error) => write!(f, "IO error: {}", error),
            Error::ProjectDirs => write!(f, "Cannot locate application config directory"),
            Error::ConfigLoad(error) => write!(f, "Config reading error: {}", error),
            Error::ConfigSave(error) => write!(f, "Config writing error: {}", error),
            Error::UrlParse(error) => write!(f, "Incorrect URL: {}", error),
            Error::Reqwest(error) => write!(f, "Web request failed: {}", error),
            Error::RequestFailed(status) => write!(f, "Request failed ({})", status),
            Error::JsonParse(error) => write!(f, "JSON parse error: {}", error),
            Error::FloatParse(error) => write!(f, "Float parse error: {}", error),
            Error::IntParse(error) => write!(f, "Int parse error: {}", error),
            Error::Utf8Parse(error) => write!(f, "UTF-8 parse error: {}", error),
            Error::CannotOpenTerminal => write!(f, "Cannot open terminal interface"),
            Error::Terminal(error) => write!(f, "Terminal error: {}", error),
            Error::ChronoParse(error) => write!(f, "Date/time parse error: {}", error),
        }
    }
}
