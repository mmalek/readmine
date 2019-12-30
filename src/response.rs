use crate::result::Result;
use crate::serialization_formats::*;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
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

#[derive(Clone, Debug, Serialize, Deserialize)]
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
    pub time_entries: Vec<TimeEntry>,
}

pub fn parse_time_entries(text: &str) -> Result<Vec<TimeEntry>> {
    let response: TimeEntriesResponse = serde_json::from_str(text)?;
    Ok(response.time_entries)
}

#[derive(Serialize, Deserialize)]
pub struct TimeEntryResponse {
    pub time_entry_activities: Vec<TimeEntryActivity>,
}

pub fn parse_time_entry_activities(text: &str) -> Result<Vec<TimeEntryActivity>> {
    let response: TimeEntryResponse = serde_json::from_str(text)?;
    Ok(response.time_entry_activities)
}
