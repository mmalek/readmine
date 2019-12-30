use clap::{App, AppSettings, Arg, SubCommand};

pub fn build_cli() -> App<'static, 'static> {
    App::new("readmine")
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
                    .about("show/add time entries")
                    .arg(Arg::with_name("range")
                        .help("time range for showing time ranges: 2019-01-23..2019-05-09, \
                               week (current week), \
                               month (current month), \
                               week-1 (last week), week-2 (the week before last),
                               month-1 (last month),
                               month-1..week-1 (from the beginning of last month to the end of last week) etc.")
                        .default_value("week")
                        .index(1))
                    .subcommand(SubCommand::with_name("add")
                        .arg(Arg::with_name("date").index(1).required(true))
                        .arg(Arg::with_name("hours").index(2).required(true))
                        .arg(Arg::with_name("issue_id").index(3).required(true))
                        .arg(Arg::with_name("activity").index(4).required(true))
                        .arg(Arg::with_name("comment").index(5))))
}
