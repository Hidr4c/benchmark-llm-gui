use std::sync::Mutex;
use chrono::Local;
use rusqlite::{params, Connection};
use crate::domain::harness::HarnessSnapshotData;
use crate::domain::runtime::LocalRuntimeProfile;
use crate::domain::traits::IEcosystemRepository;

pub struct SqliteEcosystemRepository {
    conn: Mutex<Connection>,
}

impl SqliteEcosystemRepository {
    pub fn open(path: &str) -> Result<Self, String> {
        let conn = Connection::open(path).map_err(|e| e.to_string())?;
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS ecosystem_snapshots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                kind TEXT NOT NULL,
                created_at TEXT NOT NULL,
                payload_json TEXT NOT NULL
            );
            CREATE INDEX IF NOT EXISTS idx_eco_kind ON ecosystem_snapshots(kind, id);
            ",
        ).map_err(|e| e.to_string())?;
        Ok(Self { conn: Mutex::new(conn) })
    }

    fn save_payload(&self, kind: &str, json: &str) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        conn.execute(
            "INSERT INTO ecosystem_snapshots (kind, created_at, payload_json) VALUES (?1, ?2, ?3)",
            params![kind, now, json],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }

    fn load_latest_payload(&self, kind: &str) -> Result<Option<String>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare(
            "SELECT payload_json FROM ecosystem_snapshots WHERE kind = ?1 ORDER BY id DESC LIMIT 1",
        ).map_err(|e| e.to_string())?;
        let mut rows = stmt.query(params![kind]).map_err(|e| e.to_string())?;
        if let Some(row) = rows.next().map_err(|e| e.to_string())? {
            let val: String = row.get(0).map_err(|e| e.to_string())?;
            return Ok(Some(val));
        }
        Ok(None)
    }
}

impl IEcosystemRepository for SqliteEcosystemRepository {
    fn save_harness_snapshot(&mut self, data: &HarnessSnapshotData) -> Result<(), String> {
        let json = serde_json::to_string(data).map_err(|e| e.to_string())?;
        self.save_payload("harnesses", &json)
    }

    fn load_latest_harnesses(&self) -> Result<HarnessSnapshotData, String> {
        match self.load_latest_payload("harnesses")? {
            Some(json) => serde_json::from_str(&json).map_err(|e| e.to_string()),
            None => Ok(HarnessSnapshotData::default()),
        }
    }

    fn save_runtimes_snapshot(&mut self, runtimes: &[LocalRuntimeProfile]) -> Result<(), String> {
        let json = serde_json::to_string(runtimes).map_err(|e| e.to_string())?;
        self.save_payload("runtimes", &json)
    }

    fn load_latest_runtimes(&self) -> Result<Vec<LocalRuntimeProfile>, String> {
        match self.load_latest_payload("runtimes")? {
            Some(json) => serde_json::from_str(&json).map_err(|e| e.to_string()),
            None => Ok(Vec::new()),
        }
    }
}
