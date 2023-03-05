use std::sync::Arc;
use std::{error::Error, fs};

use anyhow::{anyhow, Context, Result};
use clap::{Arg, Command};
use rusqlite::params;
use serde::{Deserialize, Serialize};

use git_version::git_version;
const GIT_VERSION: &str = git_version!();

mod core;
mod migrate;
mod server;

#[tokio::main]
async fn main() -> Result<()> {
    let matches = Command::new("updown")
        .bin_name("updown")
        .subcommand_required(true)
        .subcommand(
            Command::new("httpd")
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
        Some(("httpd", start_matches)) => {
            let port: u16 = start_matches
                .get_one::<String>("port")
                .ok_or_else(|| anyhow!("port is required"))?
                .parse()?;

            // Get the config to pass to the service
            let config_location = start_matches
                .get_one::<String>("config")
                .ok_or_else(|| anyhow!("config location required"))?;
            let yaml = fs::read_to_string(config_location).context("unable to read config file")?;
            let cfg: Config = serde_yaml::from_str(&yaml).context("error deserializing config")?;

            // Create the service
            let conn = connect_to_db("./db.sqlite").expect("error opening db");
            let repo = core::Repo::new(conn);

            let service = Arc::new(core::Service::new(repo, cfg.endpoints));

            let server_service = service.clone();
            tokio::spawn(async move {
                server::start_server(port, server_service).await;
            });

            service.clone().start_check_routines().await;
        }
        // When you're trying to migrate
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

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    endpoints: Vec<Endpoint>,
}

impl Config {
    pub fn validate(self) -> Result<ValidatedConfig> {
        // TODO
        Ok(ValidatedConfig(self))
    }
}

#[derive(Debug, PartialEq, Serialize, Deserialize, Clone)]
pub struct Endpoint {
    series: String,
    url: String,
    interval: u16,
}

pub struct ValidatedConfig(Config);
