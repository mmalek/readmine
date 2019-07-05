use crate::constants::DATE_FORMAT;
use crate::error::Error;
use crate::result::Result;
use crate::response;
use crate::serialization_formats::*;
use chrono::{Datelike, Duration, Local, NaiveDate};
use reqwest::Client;
use rpassword::read_password_from_tty;
use serde::{Serialize, Deserialize};
use std::io::{self, Write};

pub fn login(url: &String, login_name: Option<String>) -> Result<response::User> {
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
    let mut res = client.get(&url).basic_auth(&login_name, Some(&password)).send()?;
    let status = res.status();
    if status == reqwest::StatusCode::OK {
        response::parse_user(&res.text()?)
    } else {
        Err(Error::RequestFailed(status))
    }
}

pub fn user(url: &String, api_key: &Option<String>) -> Result<response::User> {
    let url = format!("{}/users/current.json", url);
    let client = Client::new();
    let mut request_builder = client.get(&url);
    if let Some(api_key) = api_key {
        request_builder = request_builder.header("X-Redmine-API-Key", api_key.clone());
    }
    let mut res = request_builder.send()?;
    let status = res.status();
    if status == reqwest::StatusCode::OK {
        response::parse_user(&res.text()?)
    } else {
        Err(Error::RequestFailed(status))
    }
}

pub fn time(url: &String, api_key: &Option<String>) -> Result<Vec<response::TimeEntry>> {
    let today = Local::today();
    let from = today - Duration::days(today.weekday().num_days_from_monday() as i64);
    let to = from + Duration::days(6);

    let from = from.format(DATE_FORMAT).to_string();
    let to = to.format(DATE_FORMAT).to_string();
    let url = format!("{}/time_entries.json?user_id=me&from={}&to={}", url, from, to);

    let client = Client::new();
    let mut request_builder = client.get(&url);
    if let Some(api_key) = api_key {
        request_builder = request_builder.header("X-Redmine-API-Key", api_key.clone());
    }
    let mut res = request_builder.send()?;
    let status = res.status();
    if status == reqwest::StatusCode::OK {
        response::parse_time_entries(&res.text()?)
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

pub fn time_add(url: &String, api_key: &Option<String>, time_entry: TimeEntry) -> Result<()> {
    let url = format!("{}/time_entries.json", url);
    let time_entry_request = TimeEntryRequest{time_entry};
    let client = Client::new();
    let mut request_builder = client.post(&url).json(&time_entry_request);
    if let Some(api_key) = api_key {
        request_builder = request_builder.header("X-Redmine-API-Key", api_key.clone());
    }
    let res = request_builder.send()?;
    let status = res.status();
    if status == reqwest::StatusCode::CREATED {
        Ok(())
    } else {
        Err(Error::RequestFailed(status))
    }
}
