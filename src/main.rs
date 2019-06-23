mod config;
mod constants;
mod error;
mod request;
mod response;
mod result;

use crate::config::Config;
use crate::error::Error;
use crate::result::Result;
use clap::{App, AppSettings, Arg, SubCommand};
use term;

enum Command {
    Login{url: String, email: Option<String>},
    Logout,
    User,
    Time,
}

fn main() {
    let matches = App::new("readmine")
        .setting(AppSettings::SubcommandRequired)
        .version("0.1")
        .about("Redmine client")
        .subcommand(SubCommand::with_name("login")
                    .about("login to the Redmine server")
                    .arg(Arg::with_name("url")
                        .help("Full address of the Redmine server, e.g. \"http://www.redmine.org\"")
                        .index(1)
                        .required(true))
                    .arg(Arg::with_name("name")
                        .help("user login name")
                        .index(2)))
        .subcommand(SubCommand::with_name("logout")
                    .about("log out of the Redmine server"))
        .subcommand(SubCommand::with_name("user")
                    .about("show user info"))
        .subcommand(SubCommand::with_name("time")
                    .about("show time entries"))
        .get_matches();

    let command = if let Some(matches) = matches.subcommand_matches("login") {
        let url = matches.value_of("url").expect("missing \"url\" parameter in \"server\" command").to_string();
        let email = matches.value_of("name").map(str::to_string);
        Command::Login{url, email}
    } else if matches.subcommand_matches("logout").is_some() {
        Command::Logout
    } else if matches.subcommand_matches("user").is_some() {
        Command::User
    } else if matches.subcommand_matches("time").is_some() {
        Command::Time
    } else {
        unreachable!();
    };

    if let Err(error) = run_command(command) {
        eprintln!("{}", error);
        std::process::exit(1);
    }
}

fn run_command(command: Command) -> Result<()> {
    let mut config = Config::load()?;

    match command {
        Command::Login{url, email} => {
            let user = request::login(&url, email)?;
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
                let user = request::user(url, &config.api_key)?;
                println!("id: {}\nlogin: {}\nfirst name: {}\nlast name: {}\nmail: {}\ncreated on: {}\nlast login on: {}\napi key: {}",
                    user.id, user.login, user.firstname, user.lastname, user.mail, user.created_on, user.last_login_on, user.api_key)
            } else {
                println!("Server details not set. Please use \"login\" command first.");
            };
            Ok(())
        }
        Command::Time => {
            if let Some(url) = &config.url {
                let mut t = term::stdout().ok_or(Error::CannotOpenTerminal)?;
                let time_entries = request::time(url, &config.api_key)?;
                let total = time_entries.iter().fold(0.0, |sum, entry| sum + entry.hours);
                let max_project_title_len = time_entries.iter()
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
                    write!(t, "h   #{} {:width$}   ", entry.issue.id, entry.project.name, width=max_project_title_len)?;
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
    }
}
