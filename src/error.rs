use std::fmt;
use std::io;
use std::num::ParseIntError;
use std::str::Utf8Error;
use quick_xml;
use reqwest;
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
    XmlParse(quick_xml::Error),
    XmlNotParsed,
    IntParse(ParseIntError),
    Utf8Parse(Utf8Error),
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

impl From<quick_xml::Error> for Error {
    fn from(error: quick_xml::Error) -> Self {
        Error::XmlParse(error)
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
            Error::XmlParse(error) => write!(f, "XML parse error: {}", error),
            Error::XmlNotParsed => write!(f, "XML response has not been parsed correctly"),
            Error::IntParse(error) => write!(f, "Int parse error: {}", error),
            Error::Utf8Parse(error) => write!(f, "UTF-8 parse error: {}", error),
        }
    }
}
