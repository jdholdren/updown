use std::sync::Arc;
use std::time::SystemTime;

use anyhow::Result;

use tokio::time;
use tokio::time::Duration;

use crate::core;
use crate::{Endpoint, ValidatedConfig};

// The service with all the fun logic
pub struct Service {
    endpoints_15: Vec<Endpoint>,
    repo: core::Repo,
}

impl Service {
    pub fn new(cfg: ValidatedConfig, repo: core::Repo) -> Self {
        Self {
            endpoints_15: cfg
                .0
                .endpoints
                .into_iter()
                .filter(|e| e.interval == 15)
                .collect(),
            repo,
        }
    }

    // Starts routines for each health check frequency
    pub async fn start_check_routines(self: Arc<Self>) -> Result<()> {
        let handle_15 = tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_secs(1));
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
                        .store_status(&core::Status {
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

    // Given a list of checks, it runs them each in a new task
    async fn execute_checks(checks: Vec<Endpoint>) {}
}
