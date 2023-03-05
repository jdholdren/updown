use std::sync::Arc;
use std::time::SystemTime;

use anyhow::Result;

use tokio::time;
use tokio::time::Duration;

use crate::core::Repo;
use crate::Endpoint;

use super::repo::CreateStatus;
use super::time_wheel::TimingWheel;
use super::Status;

// The service with all the fun logic
pub struct Service {
    repo: Repo,
    pub endpoints: Vec<Endpoint>,
}

impl Service {
    pub fn new(repo: Repo, endpoints: Vec<Endpoint>) -> Self {
        Self { repo, endpoints }
    }

    // Starts routines for each health check frequency
    pub async fn start_check_routines(self: Arc<Self>) {
        // Set up the initial set of endpoints in the timing wheel
        let mut tw: TimingWheel<Endpoint> = Default::default();
        schedule_endpoints(&mut tw, self.endpoints.to_vec());

        let mut interval = time::interval(Duration::from_secs(1));
        loop {
            interval.tick().await; // Waits until 15s have passed

            // Tick the wheel forward. It'll return any endpoints that we need to check.
            // Since these are continuously being checked at an interval, immediately reschedule
            // them according to their interval.
            let to_run = tw.tick();

            // Schedule a task for each endpoint
            for endpoint in to_run.iter() {
                let service = self.clone();
                let endpoint = endpoint.clone();
                tokio::spawn(
                    async move { service.execute_check(endpoint.series, endpoint.url).await },
                );
            }

            // Put them back
            schedule_endpoints(&mut tw, to_run);
        }
    }

    // Given a list of checks, it runs them each in a new task
    async fn execute_check(&self, series_name: String, url: String) {
        let start = SystemTime::now();

        let resp = match reqwest::get(url).await {
            Err(err) => {
                tracing::error!("error performing request: {}", err);
                return;
            }
            Ok(resp) => resp,
        };
        let end = SystemTime::now();
        let duration_ms = end
            .duration_since(start)
            .expect("we went backwards in time")
            .as_millis() as u64;

        let status = CreateStatus {
            series: series_name,
            status: resp.status().into(),
            start,
            duration_ms,
        };
        if let Err(err) = self.repo.store_status(status).await {
            tracing::error!("error storing status: {}", err);
        }
    }

    // Lists statuses from the given time until the to time
    pub async fn list_statuses(self: Arc<Self>, from: u64, to: u64) -> Result<Vec<Status>> {
        self.repo.fetch_statuses(from, to).await
    }
}

fn schedule_endpoints(tw: &mut TimingWheel<Endpoint>, endpoints: Vec<Endpoint>) {
    for endpoint in endpoints {
        let seconds_from_now = endpoint.interval as usize; // Getting around a move below
        tw.add(endpoint, seconds_from_now);
    }
}
