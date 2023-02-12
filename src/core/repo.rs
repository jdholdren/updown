use anyhow::Result;
use rusqlite::{params, Connection};
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Debug)]
pub struct HealthCheck {
    pub id: String,
    pub frequency: u32,
    pub url: String,
}

pub struct Repo {
    conn: Mutex<Connection>,
}

pub struct Status {
    pub id: String,
    pub series: String,
    pub status: u16,
    pub time: u64,
}

impl Repo {
    pub fn new(conn: Connection) -> Repo {
        Repo {
            conn: Mutex::new(conn),
        }
    }

    // Returns the id of the inserted status
    pub async fn store_status(&self, status: &Status) -> Result<String> {
        let lock = self.conn.lock().await;
        let id = Uuid::new_v4().to_string();

        let mut stmt =
            lock.prepare("INSERT INTO statuses (id, series, status, time) VALUES (?, ?, ?, ?)")?;
        stmt.execute(params![id, status.series, status.status, status.time])?;

        Ok(id)
    }
}
