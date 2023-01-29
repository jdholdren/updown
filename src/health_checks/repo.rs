use anyhow::Result;
use rusqlite::{params, Connection};

#[derive(Debug)]
pub struct HealthCheck {
    pub id: String,
    pub frequency: u32,
    pub url: String,
}

pub struct Repo {
    conn: Connection,
}

impl Repo {
    pub fn new(conn: Connection) -> Repo {
        Repo { conn }
    }

    pub fn fetch_checks(&self, frequency: u32) -> Result<Vec<HealthCheck>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, frequency, url FROM checks WHERE frequency = ?;")?;
        let check_iter = stmt.query_map(params![frequency], |row| {
            Ok(HealthCheck {
                id: row.get(0)?,
                frequency: row.get(1)?,
                url: row.get(2)?,
            })
        })?;

        let mut ret = Vec::new();
        for check in check_iter {
            match check {
                Err(err) => return Err(err.into()),
                Ok(hc) => ret.push(hc),
            }
        }

        Ok(ret)
    }
}
