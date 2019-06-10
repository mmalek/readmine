use crate::result::Result;
use chrono::prelude::*;
use serde::{Serialize, Deserialize};
use serde_json;

#[derive(Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub login: String,
    pub firstname: String,
    pub lastname: String,
    pub mail: String,
    pub created_on: String,
    pub last_login_on: String,
    pub api_key: String,
}

#[derive(Serialize, Deserialize)]
struct UserResponse {
    user: User,
}

pub fn parse_user(text: &str) -> Result<User> {
    let user_response: UserResponse = serde_json::from_str(text)?;
    Ok(user_response.user)
}

#[derive(Serialize, Deserialize)]
pub struct TimeEntry {
    pub id: i32,
    pub project: TimeEntryProject,
    pub issue: TimeEntryIssue,
    pub user: TimeEntryUser,
    pub activity: TimeEntryActivity,
    pub hours: f32,
    pub comments: String,
    #[serde(with = "redmine_date_format")]
    pub spent_on: NaiveDate,
    #[serde(with = "redmine_datetime_format")]
    pub created_on: NaiveDateTime,
    #[serde(with = "redmine_datetime_format")]
    pub updated_on: NaiveDateTime,
}

#[derive(Serialize, Deserialize)]
pub struct TimeEntryProject {
    pub id: i32,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct TimeEntryIssue {
    pub id: i32,
}

#[derive(Serialize, Deserialize)]
pub struct TimeEntryActivity {
    pub id: i32,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct TimeEntryUser {
    pub id: i32,
    pub name: String,
}

#[derive(Serialize, Deserialize)]
pub struct TimeEntriesResponse {
    pub time_entries: Vec<TimeEntry>
}

pub fn parse_time_entries(text: &str) -> Result<Vec<TimeEntry>> {
    let response: TimeEntriesResponse = serde_json::from_str(text)?;
    Ok(response.time_entries)
}

mod redmine_date_format {
    use chrono::NaiveDate;
    use serde::{self, Deserialize, Serializer, Deserializer};

    const FORMAT: &'static str = "%Y-%m-%d";

    pub fn serialize<S>(
        date: &NaiveDate,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<NaiveDate, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDate::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}

mod redmine_datetime_format {
    use chrono::NaiveDateTime;
    use serde::{self, Deserialize, Serializer, Deserializer};

    const FORMAT: &'static str = "%Y-%m-%dT%H:%M:%SZ";

    pub fn serialize<S>(
        date: &NaiveDateTime,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", date.format(FORMAT));
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<NaiveDateTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        NaiveDateTime::parse_from_str(&s, FORMAT).map_err(serde::de::Error::custom)
    }
}