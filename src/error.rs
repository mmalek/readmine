use crate::response::TimeEntryActivity;
use chrono;
use reqwest;
use serde_json;
use std::fmt;
use std::io;
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
    InvalidTimeLogHours(String),
    InvalidIssueId(String),
    CannotOpenTerminal,
    Terminal(term::Error),
    ChronoParse(chrono::ParseError),
    InvalidActivityName(String, Vec<TimeEntryActivity>),
    InvalidTimeRangeFormat(String),
    InvalidMonthOffset(i32),
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
            Error::InvalidTimeLogHours(arg) => write!(f, "Invalid hours time log entry: '{}'", arg),
            Error::InvalidIssueId(arg) => write!(f, "Invalid issue id entry: '{}'", arg),
            Error::CannotOpenTerminal => write!(f, "Cannot open terminal interface"),
            Error::Terminal(error) => write!(f, "Terminal error: {}", error),
            Error::ChronoParse(error) => write!(f, "Date/time parse error: {}", error),
            Error::InvalidActivityName(provided_name, activities) => {
                let first_name = activities
                    .first()
                    .map(|a| a.name.clone())
                    .unwrap_or_else(String::new);
                let names = activities
                    .iter()
                    .skip(1)
                    .fold(first_name, |names, a| format!("{}, {}", names, a.name));
                write!(
                    f,
                    "Invalid activity name \"{}\". Available values: {}",
                    provided_name, names
                )
            }
            Error::InvalidTimeRangeFormat(input) => {
                write!(f, "Invalid format of time range \"{}\"", input)
            }
            Error::InvalidMonthOffset(offset) => {
                write!(f, "Cannot use month+{} in time range", offset)
            }
        }
    }
}
