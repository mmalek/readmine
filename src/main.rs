mod cli;
mod config;
mod constants;
mod error;
mod request;
mod response;
mod result;
mod serialization_formats;
mod time_range;

use crate::config::Config;
use crate::constants::DATE_FORMAT;
use crate::error::Error;
use crate::result::Result;
use crate::time_range::TimeRange;
use chrono::prelude::*;
use term;

enum Command {
    Login { url: String, email: Option<String> },
    Logout,
    User,
    Time(TimeRange),
    TimeAdd(TimeEntry),
}

pub struct TimeEntry {
    pub issue_id: i32,
    pub spent_on: NaiveDate,
    pub hours: f32,
    pub activity_name: String,
    pub comments: Option<String>,
}

impl TimeEntry {
    fn into_request(
        self,
        activities: &Vec<response::TimeEntryActivity>,
    ) -> Result<request::TimeEntry> {
        activities
            .iter()
            .find(|activity| self.activity_name == activity.name)
            .ok_or_else(|| {
                error::Error::InvalidActivityName(self.activity_name.clone(), activities.clone())
            })
            .map(|activity| request::TimeEntry {
                issue_id: self.issue_id,
                spent_on: self.spent_on,
                hours: self.hours,
                comments: self.comments,
                activity_id: activity.id,
            })
    }
}

#[tokio::main]
async fn main() {
    if let Err(error) = just_run().await {
        eprintln!("{}", error);
        std::process::exit(1);
    }
}

async fn just_run() -> Result<()> {
    let matches = cli::build_cli().get_matches();

    let command = if let Some(matches) = matches.subcommand_matches("login") {
        let url = matches
            .value_of("url")
            .expect("missing \"url\" parameter in \"server\" command")
            .to_string();
        let email = matches.value_of("name").map(str::to_string);
        Command::Login { url, email }
    } else if matches.subcommand_matches("logout").is_some() {
        Command::Logout
    } else if matches.subcommand_matches("user").is_some() {
        Command::User
    } else if let Some(matches) = matches.subcommand_matches("time") {
        if let Some(matches) = matches.subcommand_matches("add") {
            let spent_on = matches
                .value_of("date")
                .expect("missing \"date\" parameter in \"time add\" command");
            let spent_on = NaiveDate::parse_from_str(spent_on, DATE_FORMAT)?;
            let hours: f32 = matches
                .value_of("hours")
                .expect("missing \"hours\" parameter in \"time add\" command")
                .parse()?;
            let issue_id: i32 = matches
                .value_of("issue_id")
                .expect("missing \"issue_id\" parameter in \"time add\" command")
                .parse()?;
            let activity_name = matches
                .value_of("activity")
                .expect("missing \"activity\" parameter in \"time add\" command")
                .to_string();
            let comments = matches.value_of("comment").map(str::to_string);
            Command::TimeAdd(TimeEntry {
                issue_id,
                spent_on,
                hours,
                activity_name,
                comments,
            })
        } else {
            Command::Time(TimeRange::parse(
                matches
                    .value_of("range")
                    .expect("missing \"range\" parameter in \"time\" command"),
            )?)
        }
    } else {
        unreachable!();
    };

    run_command(command).await
}

async fn run_command(command: Command) -> Result<()> {
    let mut config = Config::load()?;

    match command {
        Command::Login { url, email } => {
            let user = request::login(&url, email).await?;
            config.url = Some(url);
            config.api_key = Some(user.api_key);
            config.save()
        }
        Command::Logout => {
            config.url = None;
            config.api_key = None;
            config.save()
        }
        Command::User => {
            if let Some(url) = &config.url {
                let user = request::user(url, &config.api_key).await?;
                println!("id: {}\nlogin: {}\nfirst name: {}\nlast name: {}\nmail: {}\ncreated on: {}\nlast login on: {}\napi key: {}",
                    user.id, user.login, user.firstname, user.lastname, user.mail, user.created_on, user.last_login_on, user.api_key)
            } else {
                println!("Server details not set. Please use \"login\" command first.");
            };
            Ok(())
        }
        Command::Time(range) => {
            if let Some(url) = &config.url {
                let mut t = term::stdout().ok_or(Error::CannotOpenTerminal)?;
                let time_entries = request::time(url, &config.api_key, &range).await?;
                let total = time_entries
                    .iter()
                    .fold(0.0, |sum, entry| sum + entry.hours);
                let max_project_title_len = time_entries
                    .iter()
                    .map(|entry| entry.project.name.len())
                    .max()
                    .unwrap_or(0);
                for entry in time_entries {
                    t.fg(term::color::WHITE)?;
                    write!(t, "{}   ", entry.spent_on)?;
                    t.attr(term::Attr::Bold)?;
                    t.fg(term::color::WHITE)?;
                    write!(t, "{}", entry.hours)?;
                    t.reset()?;
                    write!(
                        t,
                        "h   #{} {:width$}   ",
                        entry.issue.id,
                        entry.project.name,
                        width = max_project_title_len
                    )?;
                    t.fg(term::color::YELLOW)?;
                    writeln!(t, "{}", entry.comments)?;
                }
                t.fg(term::color::WHITE)?;
                write!(t, "Total time: ")?;
                t.attr(term::Attr::Bold)?;
                t.fg(term::color::WHITE)?;
                write!(t, "{}", total)?;
                t.reset()?;
                writeln!(t, "h")?;
            } else {
                println!("Server details not set. Please use \"login\" command first.");
            };
            Ok(())
        }
        Command::TimeAdd(time_entry) => {
            if let Some(url) = &config.url {
                let activities = request::activities(url, &config.api_key).await?;
                let time_entry = time_entry.into_request(&activities)?;
                request::time_add(url, &config.api_key, time_entry).await
            } else {
                println!("Server details not set. Please use \"login\" command first.");
                Ok(())
            }
        }
    }
}
