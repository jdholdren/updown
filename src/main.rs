use clap::{Arg, Command};

use git_version::git_version;
const GIT_VERSION: &'static str = git_version!();

mod health_checks;
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
        .get_matches();

    match matches.subcommand() {
        Some(("start", start_matches)) => {
            let port: u16 = start_matches
                .get_one::<String>("port")
                .unwrap()
                .parse()
                .unwrap();

            server::start_server(port).await;
        }
        _ => unreachable!(),
    }
}
