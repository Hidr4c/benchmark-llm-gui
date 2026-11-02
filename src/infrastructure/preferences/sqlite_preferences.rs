use std::sync::Mutex;
use rusqlite::{params, Connection};
use crate::domain::table_preferences::TablePreferences;
use crate::domain::traits::IPreferencesRepository;

pub struct SqlitePreferencesRepository {
    conn: Mutex<Connection>,
}

impl SqlitePreferencesRepository {
    pub fn open(path: &str) -> Result<Self, String> {
        let conn = Connection::open(path).map_err(|e| e.to_string())?;
        let repo = Self { conn: Mutex::new(conn) };
        repo.init_schema()?;
        Ok(repo)
    }

    fn init_schema(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS user_preferences (
                pref_key TEXT PRIMARY KEY,
                pref_val TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );"
        ).map_err(|e| e.to_string())?;
        Ok(())
    }
}

impl IPreferencesRepository for SqlitePreferencesRepository {
    fn load_table_preferences(&self) -> Result<Option<TablePreferences>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn
            .prepare("SELECT pref_val FROM user_preferences WHERE pref_key = 'table_preferences'")
            .map_err(|e| e.to_string())?;
        let mut rows = stmt.query([]).map_err(|e| e.to_string())?;

        if let Some(row) = rows.next().map_err(|e| e.to_string())? {
            let json_str: String = row.get(0).map_err(|e| e.to_string())?;
            let prefs: TablePreferences = serde_json::from_str(&json_str)
                .map_err(|e| e.to_string())?;
            Ok(Some(prefs))
        } else {
            Ok(None)
        }
    }

    fn save_table_preferences(&self, prefs: &TablePreferences) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let json_str = serde_json::to_string(prefs).map_err(|e| e.to_string())?;
        let now = chrono::Local::now().to_rfc3339();
        conn.execute(
            "INSERT INTO user_preferences (pref_key, pref_val, updated_at) VALUES ('table_preferences', ?1, ?2)
             ON CONFLICT(pref_key) DO UPDATE SET pref_val = excluded.pref_val, updated_at = excluded.updated_at",
            params![json_str, now],
        ).map_err(|e| e.to_string())?;
        Ok(())
    }
}
