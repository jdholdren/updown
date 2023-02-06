use std::time::SystemTime;

use anyhow::{bail, Result};

use tokio::time;
use tokio::time::Duration;

use serde::{Deserialize, Serialize};

pub mod repo;
use repo::Repo;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Config {
    endpoints: Vec<Endpoint>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Endpoint {
    series: String,
    url: String,
    interval: u16,
}

// The service with all the fun logic
pub struct Service {
    endpoints_15: Vec<Endpoint>,
    repo: Repo,
}

impl Service {
    pub fn new(cfg: Config, repo: repo::Repo) -> Self {
        // TODO: Ensure that each endpoint matches a supported interval
        Self {
            endpoints_15: cfg.endpoints,
            repo,
        }
    }

    // Starts routines for each health check frequency
    pub async fn start_check_routines(self) -> Result<()> {
        println!("{:?}", self.endpoints_15);

        let handle_15 = tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_millis(15_000));
            loop {
                interval.tick().await; // Waits until 15s have passed

                // Loop through each and try to call out
                for check in &self.endpoints_15 {
                    let resp = match reqwest::get(&check.url).await {
                        Ok(v) => v,
                        Err(err) => {
                            println!("error checking {}: {}", check.url, err);
                            continue;
                        }
                    };

                    // Insert into the db
                    self.repo
                        .store_status(&repo::Status {
                            id: String::new(),
                            series: check.series.clone(),
                            status: resp.status().as_u16(),
                            time: SystemTime::now()
                                .duration_since(SystemTime::UNIX_EPOCH)?
                                .as_secs(),
                        })
                        .await?;
                }
            }

            // For infering the result type
            #[allow(unreachable_code)]
            Ok::<(), anyhow::Error>(())
        });

        tokio::join!(handle_15)
            .0
            .expect("Did not want a join error")
            .unwrap();

        Ok(())
    }
}
