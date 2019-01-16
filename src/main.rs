mod config;
mod error;
mod result;

use crate::config::Config;
use crate::error::Error;
use crate::result::Result;
use clap::{App, AppSettings, Arg, SubCommand};
use std::io::{self, Write};
use quick_xml::Reader;
use quick_xml::events::Event;
use reqwest::Client;
use rpassword::read_password_from_tty;

enum Command {
    Server{url: String},
    Login{email: Option<String>},
    Logout,
    User,
    Time,
}

fn main() -> Result<()> {
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
        return Err(Error::MissingCommand);
    };

    let mut config = Config::load()?;

    match command {
        Command::Server{url} => {
            config.url = Some(url);
            config.save()?;
        }
        Command::Login{email} => {
            if let Some(url) = &config.url {
                let email = email
                    .map(|s| -> Result<String> { Ok(s) })
                    .unwrap_or_else(|| {
                        print!("Email: ");
                        io::stdout().flush()?;
                        let mut email = String::new();
                        io::stdin().read_line(&mut email)?;
                        email.retain(|c| c != '\n' && c != '\r');
                        Ok(email)
                    })?;

                let password = read_password_from_tty(Some("Password: "))?;
                println!();

                let url = format!("{}/users/current.xml", url);
                let client = Client::new();
                let mut res = client.get(&url).basic_auth(&email, Some(&password)).send()?;
                let status = res.status();
                if status == reqwest::StatusCode::OK {
                    let user = parse_user(&res.text()?)?;
                    config.api_key = Some(user.api_key);
                    config.save()?;
                } else {
                    return Err(Error::RequestFailed(status));
                }
            } else {
                println!("Server details not set. Please use \"server\" command first.")
            }
        }
        Command::Logout => {
            config.api_key = None;
            config.save()?;
        }
        Command::User => {
            if let Some(url) = config.url {
                let url = format!("{}/users/current.xml", url);
                let client = Client::new();
                let mut request_builder = client.get(&url);
                if let Some(api_key) = config.api_key {
                    request_builder = request_builder.header("X-Redmine-API-Key", api_key);
                }
                let mut res = request_builder.send()?;
                let status = res.status();
                if status == reqwest::StatusCode::OK {
                    let user = parse_user(&res.text()?)?;
                    println!("id: {}\nlogin: {}\nfirst name: {}\nlast name: {}\nmail: {}\ncreated on: {}\nlast login on: {}\napi key: {}",
                        user.id, user.login, user.first_name, user.last_name, user.mail, user.created_on, user.last_login_on, user.api_key);
                } else {
                    return Err(Error::RequestFailed(status));
                }
            } else {
                println!("Server details not set. Please use \"server\" command first.")
            }
        }
        Command::Time => {
            if let Some(url) = config.url {
                let url = format!("{}/time_entries.xml?user_id=me", url);
                let client = Client::new();
                let mut request_builder = client.get(&url);
                if let Some(api_key) = config.api_key {
                    request_builder = request_builder.header("X-Redmine-API-Key", api_key);
                }
                let mut res = request_builder.send()?;
                let status = res.status();
                if status == reqwest::StatusCode::OK {
                    let time_entries = parse_time_entries(&res.text()?)?;
                    for time_entry in time_entries {
                        println!("{} - {} - {} - {}", time_entry.spent_on, time_entry.hours, time_entry.project.1, time_entry.issue_id);
                    }
                } else {
                    return Err(Error::RequestFailed(status));
                }
            } else {
                println!("Server details not set. Please use \"server\" command first.")
            }
        }
    }

    Ok(())
}

struct User {
    id: i32,
    login: String,
    first_name: String,
    last_name: String,
    mail: String,
    created_on: String,
    last_login_on: String,
    api_key: String,
}

fn parse_user(text: &str) -> Result<User> {
    let mut reader = Reader::from_str(text);
    reader.trim_text(true);

    let mut buf = Vec::new();

    let mut user: Option<User> = None;
    let mut element = Vec::<u8>::new();

    loop {
        match reader.read_event(&mut buf)? {
            Event::Start(ref e) => {
                match e.name() {
                    b"user" => {
                        element.clear();
                        user = Some(User{
                            id: -1,
                            login: String::new(),
                            first_name: String::new(),
                            last_name: String::new(),
                            mail: String::new(),
                            created_on: String::new(),
                            last_login_on: String::new(),
                            api_key: String::new(),
                        })
                    }
                    _ => element = e.name().to_owned(),
                }
            },
            Event::Text(e) => {
                let contents = e.unescape_and_decode(&reader)?;
                if let Some(ref mut user) = user {
                    match &element[..] {
                        b"id" =>  user.id = contents.parse()?,
                        b"login" => user.login = contents,
                        b"firstname" => user.first_name = contents,
                        b"lastname" => user.last_name = contents,
                        b"mail" => user.mail = contents,
                        b"created_on" => user.created_on = contents,
                        b"last_login_on" => user.last_login_on = contents,
                        b"api_key" => user.api_key = contents,
                        _ => {}
                    }
                }
            }
            Event::Eof => {
                break; // exits the loop when reaching end of file
            }
            _ => (), // There are several other `Event`s we do not consider here
        }
    }

    user.ok_or(Error::XmlNotParsed)
}

struct TimeEntry {
    id: i32,
    project: (i32, String),
    issue_id: i32,
    user: (i32, String),
    activity: (i32, String),
    hours: String,
    comments: String,
    spent_on: String,
    created_on: String,
    updated_on: String,
}

fn parse_time_entries(text: &str) -> Result<Vec<TimeEntry>> {
    let mut reader = Reader::from_str(text);
    reader.trim_text(true);

    let mut buf = Vec::new();

    let mut time_entries: Option<Vec<TimeEntry>> = None;
    let mut time_entry: Option<TimeEntry> = None;
    let mut element = Vec::<u8>::new();

    loop {
        match reader.read_event(&mut buf)? {
            Event::Start(ref e) => {
                match e.name() {
                    b"time_entries" => time_entries = Some(Vec::new()),
                    b"time_entry" => {
                        element.clear();
                        time_entry = Some(TimeEntry{
                            id: -1,
                            project: (-1, String::new()),
                            issue_id: -1,
                            user: (-1, String::new()),
                            activity: (-1, String::new()),
                            hours: String::new(),
                            comments: String::new(),
                            spent_on: String::new(),
                            created_on: String::new(),
                            updated_on: String::new(),
                        })
                    }
                    _ => element = e.name().to_owned(),
                }
            }
            Event::End(ref e) => {
                match e.name() {
                    b"time_entry" => {
                        element.clear();
                        if let Some(time_entry) = time_entry {
                            if let Some(ref mut time_entries) = time_entries {
                                time_entries.push(time_entry);
                            }
                        }
                        time_entry = None;
                    }
                    _ => {}
                }
            }
            Event::Empty(ref e) => {
                match e.name() {
                    b"project" => if let Some(ref mut time_entry) = time_entry { time_entry.project = get_id_and_name_attrs(e.attributes())?; },
                    b"user" => if let Some(ref mut time_entry) = time_entry { time_entry.user = get_id_and_name_attrs(e.attributes())?; },
                    b"activity" => if let Some(ref mut time_entry) = time_entry { time_entry.activity = get_id_and_name_attrs(e.attributes())?; },
                    b"issue" => if let Some(ref mut time_entry) = time_entry { time_entry.issue_id = get_id_attr(e.attributes())?; },
                    _ => {}
                }
            }
            Event::Text(e) => {
                let contents = e.unescape_and_decode(&reader)?;
                if let Some(ref mut time_entry) = time_entry {
                    match &element[..] {
                        b"id" =>  time_entry.id = contents.parse()?,
                        b"hours" => time_entry.hours = contents,
                        b"comments" => time_entry.comments = contents,
                        b"spent_on" => time_entry.spent_on = contents,
                        b"created_on" => time_entry.created_on = contents,
                        b"updated_on" => time_entry.updated_on = contents,
                        _ => {}
                    }
                }
            }
            Event::Eof => {
                break; // exits the loop when reaching end of file
            }
            _ => (), // There are several other `Event`s we do not consider here
        }
    }

    time_entries.ok_or(Error::XmlNotParsed)
}

fn get_id_and_name_attrs(attributes: quick_xml::events::attributes::Attributes) -> Result<(i32, String)> {
    let mut id: i32 = -1;
    let mut name = String::new();
    for attr in attributes {
        let attr = attr?;
        let value = std::str::from_utf8(&attr.value[..])?;
        match attr.key {
            b"id" => id = value.parse()?,
            b"name" => name = value.to_owned(),
            _ => {}
        }
    }
    Ok((id, name))
}

fn get_id_attr(attributes: quick_xml::events::attributes::Attributes) -> Result<i32> {
    for attr in attributes {
        let attr = attr?;
        let value = std::str::from_utf8(&attr.value[..])?;
        match attr.key {
            b"id" => return Ok(value.parse()?),
            _ => {}
        }
    }

    Ok(-1)
}
