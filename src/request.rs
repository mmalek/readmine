use crate::constants::DATE_FORMAT;
use crate::error::Error;
use crate::response;
use crate::result::Result;
use crate::serialization_formats::*;
use crate::time_range::TimeRange;
use chrono::NaiveDate;
use reqwest::Client;
use rpassword::read_password_from_tty;
use serde::{Deserialize, Serialize};
use std::io::{self, Write};

pub async fn login(url: &str, login_name: Option<String>) -> Result<response::User> {
    let login_name = if let Some(name) = login_name {
        name
    } else {
        print!("Login: ");
        io::stdout().flush()?;
        let mut login_name = String::new();
        io::stdin().read_line(&mut login_name)?;
        login_name.retain(|c| c != '\n' && c != '\r');
        login_name
    };

    let password = read_password_from_tty(Some("Password: "))?;
    println!();

    let url = format!("{}/users/current.json", url);
    let client = Client::new();
    let res = client
        .get(&url)
        .basic_auth(&login_name, Some(&password))
        .send()
        .await?;
    let status = res.status();
    if status == reqwest::StatusCode::OK {
        response::parse_user(&res.text().await?)
    } else {
        Err(Error::RequestFailed(status))
    }
}

pub async fn user(url: &str, api_key: &Option<String>) -> Result<response::User> {
    let url = format!("{}/users/current.json", url);
    let client = Client::new();
    let mut request_builder = client.get(&url);
    if let Some(api_key) = api_key {
        request_builder = request_builder.header("X-Redmine-API-Key", api_key.clone());
    }
    let res = request_builder.send().await?;
    let status = res.status();
    if status == reqwest::StatusCode::OK {
        response::parse_user(&res.text().await?)
    } else {
        Err(Error::RequestFailed(status))
    }
}

pub async fn time(
    url: &str,
    api_key: &Option<String>,
    range: &TimeRange,
) -> Result<Vec<response::TimeEntry>> {
    let from = range.from.format(DATE_FORMAT).to_string();
    let to = range.to.format(DATE_FORMAT).to_string();
    let url = format!(
        "{}/time_entries.json?user_id=me&from={}&to={}",
        url, from, to
    );

    let client = Client::new();
    let mut request_builder = client.get(&url);
    if let Some(api_key) = api_key {
        request_builder = request_builder.header("X-Redmine-API-Key", api_key.clone());
    }
    let res = request_builder.send().await?;
    let status = res.status();
    if status == reqwest::StatusCode::OK {
        response::parse_time_entries(&res.text().await?)
    } else {
        Err(Error::RequestFailed(status))
    }
}

pub async fn activities(
    url: &str,
    api_key: &Option<String>,
) -> Result<Vec<response::TimeEntryActivity>> {
    let url = format!("{}/enumerations/time_entry_activities.json", url);
    let client = Client::new();
    let mut request_builder = client.get(&url);
    if let Some(api_key) = api_key {
        request_builder = request_builder.header("X-Redmine-API-Key", api_key.clone());
    }
    let res = request_builder.send().await?;
    let status = res.status();
    if status == reqwest::StatusCode::OK {
        response::parse_time_entry_activities(&res.text().await?)
    } else {
        Err(Error::RequestFailed(status))
    }
}

#[derive(Serialize, Deserialize)]
pub struct TimeEntryRequest {
    pub time_entry: TimeEntry,
}

#[derive(Serialize, Deserialize)]
pub struct TimeEntry {
    pub issue_id: i32,
    #[serde(with = "redmine_date_format")]
    pub spent_on: NaiveDate,
    pub hours: f32,
    pub activity_id: i32,
    pub comments: Option<String>,
}

pub async fn time_add(url: &str, api_key: &Option<String>, time_entry: TimeEntry) -> Result<()> {
    let url = format!("{}/time_entries.json", url);
    let time_entry_request = TimeEntryRequest { time_entry };
    let client = Client::new();
    let mut request_builder = client.post(&url).json(&time_entry_request);
    if let Some(api_key) = api_key {
        request_builder = request_builder.header("X-Redmine-API-Key", api_key.clone());
    }
    let res = request_builder.send().await?;
    let status = res.status();
    if status == reqwest::StatusCode::CREATED {
        Ok(())
    } else {
        Err(Error::RequestFailed(status))
    }
}
