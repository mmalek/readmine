use crate::error::Error;
use crate::result::Result;
use quick_xml::Reader;
use quick_xml::events::Event;

pub struct User {
    pub id: i32,
    pub login: String,
    pub first_name: String,
    pub last_name: String,
    pub mail: String,
    pub created_on: String,
    pub last_login_on: String,
    pub api_key: String,
}

pub fn parse_user(text: &str) -> Result<User> {
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

pub struct TimeEntry {
    pub id: i32,
    pub project: (i32, String),
    pub issue_id: i32,
    pub user: (i32, String),
    pub activity: (i32, String),
    pub hours: String,
    pub comments: String,
    pub spent_on: String,
    pub created_on: String,
    pub updated_on: String,
}

pub fn parse_time_entries(text: &str) -> Result<Vec<TimeEntry>> {
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
