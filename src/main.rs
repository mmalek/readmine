mod config;
mod error;
mod request;
mod response;
mod result;

use crate::config::Config;
use crate::result::Result;
use clap::{App, AppSettings, Arg, SubCommand};

enum Command {
    Server{url: String},
    Login{email: Option<String>},
    Logout,
    User,
    Time,
}

fn main() {
    let matches = App::new("readmine")
        .setting(AppSettings::SubcommandRequired)
        .version("0.1")
        .about("Redmine client")
        .subcommand(SubCommand::with_name("server")
                    .about("sets up Redmine server details")
                    .arg(Arg::with_name("url")
                        .help("Full address of the Redmine server, e.g. \"http://www.redmine.org\"")
                        .index(1)
                        .required(true)))
        .subcommand(SubCommand::with_name("login")
                    .about("login to the Redmine server")
                    .arg(Arg::with_name("name")
                        .help("user login name")
                        .index(1)))
        .subcommand(SubCommand::with_name("logout")
                    .about("log out of the Redmine server"))
        .subcommand(SubCommand::with_name("user")
                    .about("show user info"))
        .subcommand(SubCommand::with_name("time")
                    .about("show time entries"))
        .get_matches();

    let command = if let Some(matches) = matches.subcommand_matches("server") {
        let url = matches.value_of("url").expect("missing \"url\" parameter in \"server\" command").to_string();
        Command::Server{url}
    } else if let Some(matches) = matches.subcommand_matches("login") {
        let email = matches.value_of("email").map(str::to_string);
        Command::Login{email}
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
        Command::Server{url} => {
            config.url = Some(url);
            config.save()
        }
        Command::Login{email} => {
            if let Some(url) = &config.url {
                let user = request::login(url, email)?;
                config.api_key = Some(user.api_key);
                config.save()
            } else {
                println!("Server details not set. Please use \"server\" command first.");
                Ok(())
            }
        }
        Command::Logout => {
            config.api_key = None;
            config.save()
        }
        Command::User => {
            if let Some(url) = &config.url {
                let user = request::user(url, &config.api_key)?;
                println!("id: {}\nlogin: {}\nfirst name: {}\nlast name: {}\nmail: {}\ncreated on: {}\nlast login on: {}\napi key: {}",
                    user.id, user.login, user.firstname, user.lastname, user.mail, user.created_on, user.last_login_on, user.api_key)
            } else {
                println!("Server details not set. Please use \"server\" command first.");
            };
            Ok(())
        }
        Command::Time => {
            if let Some(url) = &config.url {
                let time_entries = request::time(url, &config.api_key)?;
                for time_entry in time_entries {
                    println!("{} - {:.1} - {} - {}", time_entry.spent_on, time_entry.hours, time_entry.project.name, time_entry.issue.id);
                }
            } else {
                println!("Server details not set. Please use \"server\" command first.");
            };
            Ok(())
        }
    }
}
