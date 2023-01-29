use std::sync::Arc;

use tokio::sync::Mutex;
use tokio::time;
use tokio::time::Duration;

pub mod repo;

// The service with all the fun logic
pub struct Service {
    repo: Mutex<repo::Repo>,
}

impl Service {
    pub fn new(repo: repo::Repo) -> Self {
        Self {
            repo: Mutex::new(repo),
        }
    }

    // Starts routines for each health check frequency
    //
    // TODO: Need to start for the others
    pub async fn start_check_routines(self) {
        let self_rc = Arc::new(self);

        let fifteen = Arc::new(Mutex::new(Vec::new()));

        // Routine to update the check vectors
        let refresher = self_rc.clone();
        let ref_15 = fifteen.clone();
        let handle = tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_millis(15_000));
            loop {
                interval.tick().await; // Waits until 15s have passed
                let checks = refresher
                    .repo
                    .lock()
                    .await
                    .fetch_checks(15)
                    .expect("error getting 15s checks");

                println!("15s checks: {:?}", checks);

                let mut lock = ref_15.lock().await;
                *lock = checks;
            }
        });

        let call_15 = fifteen.clone();
        tokio::spawn(async move {
            let mut interval = time::interval(Duration::from_millis(15_000));
            loop {
                interval.tick().await; // Waits until 15s have passed

                // Loop through each and try to call out
                let mut lock = call_15.lock().await;
                for check in &*lock {
                    let resp = reqwest::get(&check.url).await;
                    if let Err(err) = resp {
                        println!("{}", format!("error calling to {}: {}", check.url, err));
                        continue;
                    }

                    let response = resp.unwrap();
                    println!("response code was: {:?}", response.status());
                }
            }
        });

        handle.await;
    }
}
