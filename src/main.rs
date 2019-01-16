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
