use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Result;
use rusqlite::{params, Connection};
use tokio::sync::Mutex;
use uuid::Uuid;

use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub struct HealthCheck {
    pub id: String,
    pub frequency: u32,
    pub url: String,
}

pub struct Repo {
    conn: Arc<Mutex<Connection>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Status {
    pub id: String,
    pub series: String,
    pub status: u16,
    pub start: SystemTime,
    pub duration_ms: u64,
}

pub struct CreateStatus {
    pub series: String,
    pub status: u16,
    pub start: SystemTime,
    pub duration_ms: u64,
}

impl Repo {
    pub fn new(conn: Connection) -> Repo {
        Repo {
            conn: Arc::new(Mutex::new(conn)),
        }
    }

    // Returns the id of the inserted status
    pub async fn store_status(&self, status: CreateStatus) -> Result<String> {
        let id = Uuid::new_v4().to_string();

        let lock = self.conn.lock().await;
        let mut stmt = lock.prepare(
            "INSERT INTO statuses (id, series, status, start, duration_ms) VALUES (?, ?, ?, ?, ?);",
        )?;
        stmt.execute(params![
            id,
            status.series,
            status.status,
            status.start.duration_since(UNIX_EPOCH)?.as_secs(),
            status.duration_ms
        ])?;

        Ok(id)
    }

    pub async fn fetch_statuses(&self, from: u64, to: u64) -> Result<Vec<Status>> {
        let lock = self.conn.lock().await;
        let mut stmt = lock.prepare(
            "SELECT id, series, status, start, duration_ms FROM statuses WHERE start >= ? AND start <= ?",
        )?;
        let statuses = stmt.query_map(params![from, to], |row| {
            Ok(Status {
                id: row.get(0)?,
                series: row.get(1)?,
                status: row.get(2)?,
                start: UNIX_EPOCH + Duration::from_secs(row.get(3)?),
                duration_ms: row.get(4)?,
            })
        })?;

        let mut ret = Vec::new();
        for status_res in statuses {
            ret.push(status_res?);
        }

        Ok(ret)
    }
}
