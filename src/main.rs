use std::{error::Error, fs};

use anyhow::{anyhow, Context, Result};
use clap::{Arg, Command};
use rusqlite::params;

use git_version::git_version;
const GIT_VERSION: &str = git_version!();

mod health_checks;
mod migrate;
mod server;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = Command::new("updown")
        .bin_name("updown")
        .subcommand_required(true)
        .subcommand(
            Command::new("start")
                .about("starts the server")
                .arg(
                    Arg::new("port")
                        .short('p')
                        .long("port")
                        .help("what port the server should listen on")
                        .default_value("4444")
                        .num_args(1),
                )
                .arg(
                    Arg::new("config")
                        .short('c')
                        .help("location of config yaml")
                        .num_args(1)
                        .required(true),
                ),
        )
        .subcommand(Command::new("migrate").about("migrates the db"))
        .get_matches();

    match matches.subcommand() {
        Some(("start", start_matches)) => {
            // let port: u16 = start_matches
            //     .get_one::<String>("port")
            //     .ok_or_else(|| anyhow!("port is required"))?
            //     .parse()?;

            // Get the config to pass to the service
            let config_location = start_matches
                .get_one::<String>("config")
                .ok_or_else(|| anyhow!("config location required"))?;
            let yaml = fs::read_to_string(config_location).context("unable to read config file")?;
            let cfg: health_checks::Config =
                serde_yaml::from_str(&yaml).context("error deserializing config")?;

            // tokio::spawn(async move {
            //     server::start_server(port).await;
            // });

            let conn = connect_to_db("./db.sqlite").expect("error opening db");
            let repo = health_checks::repo::Repo::new(conn);

            let service = health_checks::Service::new(cfg, repo);
            service.start_check_routines().await?;
        }
        Some(("migrate", _)) => {
            let conn = connect_to_db("./db.sqlite").unwrap();

            for query in migrate::up::MIGRATIONS {
                conn.execute(query, params![]).unwrap();
            }
        }
        _ => unreachable!(),
    }

    Ok(())
}

fn connect_to_db(file_name: &str) -> Result<rusqlite::Connection, Box<dyn Error>> {
    Ok(rusqlite::Connection::open(file_name)?)
}
