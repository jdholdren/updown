use clap::{Arg, Command};
use rusqlite::params;
use std::error::Error;

use git_version::git_version;
const GIT_VERSION: &'static str = git_version!();

mod health_checks;
mod migrate;
mod server;

#[tokio::main]
async fn main() {
    let matches = Command::new("updown")
        .bin_name("updown")
        .subcommand_required(true)
        .subcommand(
            Command::new("start").about("starts the server").arg(
                Arg::new("port")
                    .short('p')
                    .long("port")
                    .help("what port the server should listen on")
                    .default_value("4444")
                    .num_args(1),
            ),
        )
        .subcommand(Command::new("migrate").about("migrates the db"))
        .get_matches();

    match matches.subcommand() {
        Some(("start", start_matches)) => {
            let port: u16 = start_matches
                .get_one::<String>("port")
                .unwrap()
                .parse()
                .unwrap();

            tokio::spawn(async move {
                server::start_server(port).await;
            });

            let conn = connect_to_db("./db.sqlite").expect("error opening db");
            let repo = health_checks::repo::Repo::new(conn);

            let service = health_checks::Service::new(repo);
            service.start_check_routines().await;
        }
        Some(("migrate", _)) => {
            let conn = connect_to_db("./db.sqlite").unwrap();

            for query in migrate::up::MIGRATIONS {
                conn.execute(query, params![]).unwrap();
            }
        }
        _ => unreachable!(),
    }
}

fn connect_to_db(file_name: &str) -> Result<rusqlite::Connection, Box<dyn Error>> {
    Ok(rusqlite::Connection::open(file_name)?)
}
