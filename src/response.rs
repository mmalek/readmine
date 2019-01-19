use crate::result::Result;
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
    pub spent_on: String,
    pub created_on: String,
    pub updated_on: String,
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
