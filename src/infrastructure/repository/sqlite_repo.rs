use std::sync::Mutex;
use chrono::Local;
use rusqlite::{params, Connection};
use crate::domain::economic::EconomicScores;
use crate::domain::modality::ModelModality;
use crate::domain::model::{BenchmarkScores, ModelBenchmark};
use crate::domain::traits::{IBenchmarkRepository, SnapshotMeta};

pub const DEFAULT_DB_PATH: &str = "benchmark.db";

pub struct SqliteRepository {
    conn: Mutex<Connection>,
}

impl SqliteRepository {
    pub fn open(path: &str) -> Result<Self, String> {
        let conn = Connection::open(path).map_err(|e| e.to_string())?;
        let repo = Self { conn: Mutex::new(conn) };
        repo.init_schema()?;
        Ok(repo)
    }

    fn init_schema(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS snapshots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                created_at TEXT NOT NULL,
                model_count INTEGER NOT NULL,
                summary TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS models (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                snapshot_id INTEGER NOT NULL REFERENCES snapshots(id) ON DELETE CASCADE,
                repo_id TEXT NOT NULL,
                name TEXT NOT NULL,
                family TEXT NOT NULL,
                param_count_b REAL,
                size_gb REAL NOT NULL,
                vram_rec_gb REAL NOT NULL,
                tokens_per_sec REAL,
                tps_hardware TEXT,
                score_average REAL,
                score_ifeval REAL,
                score_bbh REAL,
                score_math REAL,
                score_gpqa REAL,
                score_mmlu_pro REAL,
                score_musr REAL,
                score_open_llm REAL,
                score_coding REAL,
                score_agentic REAL,
                downloads INTEGER NOT NULL,
                likes INTEGER NOT NULL,
                ollama_tag TEXT,
                hf_url TEXT NOT NULL,
                is_moe INTEGER NOT NULL,
                context_k INTEGER NOT NULL,
                is_cloud INTEGER NOT NULL DEFAULT 0,
                provider TEXT,
                pricing_input_usd REAL,
                pricing_output_usd REAL,
                release_year INTEGER,
                modality TEXT DEFAULT 'text'
            );

            CREATE INDEX IF NOT EXISTS idx_models_snapshot ON models(snapshot_id);
            ",
        ).map_err(|e| e.to_string())?;

        // Graceful migrations for existing DB
        let _ = conn.execute("ALTER TABLE models ADD COLUMN score_open_llm REAL;", []);
        let _ = conn.execute("ALTER TABLE models ADD COLUMN score_coding REAL;", []);
        let _ = conn.execute("ALTER TABLE models ADD COLUMN score_agentic REAL;", []);
        let _ = conn.execute("ALTER TABLE models ADD COLUMN is_cloud INTEGER NOT NULL DEFAULT 0;", []);
        let _ = conn.execute("ALTER TABLE models ADD COLUMN provider TEXT;", []);
        let _ = conn.execute("ALTER TABLE models ADD COLUMN pricing_input_usd REAL;", []);
        let _ = conn.execute("ALTER TABLE models ADD COLUMN pricing_output_usd REAL;", []);
        let _ = conn.execute("ALTER TABLE models ADD COLUMN release_year INTEGER;", []);
        let _ = conn.execute("ALTER TABLE models ADD COLUMN modality TEXT DEFAULT 'text';", []);
        let _ = conn.execute("ALTER TABLE models ADD COLUMN economic_data TEXT;", []);

        Ok(())
    }
}

impl IBenchmarkRepository for SqliteRepository {
    fn save_snapshot(&mut self, models: &[ModelBenchmark], summary: &str) -> Result<i64, String> {
        let mut conn = self.conn.lock().map_err(|e| e.to_string())?;
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

        tx.execute(
            "INSERT INTO snapshots (created_at, model_count, summary) VALUES (?1, ?2, ?3)",
            params![now, models.len() as i64, summary],
        ).map_err(|e| e.to_string())?;

        let snapshot_id = tx.last_insert_rowid();

        {
            let mut stmt = tx.prepare(
                "INSERT INTO models (
                    snapshot_id, repo_id, name, family, param_count_b, size_gb, vram_rec_gb,
                    tokens_per_sec, tps_hardware, score_average, score_ifeval, score_bbh,
                    score_math, score_gpqa, score_mmlu_pro, score_musr, score_open_llm,
                    score_coding, score_agentic, downloads, likes, ollama_tag, hf_url, is_moe, context_k,
                    is_cloud, provider, pricing_input_usd, pricing_output_usd, release_year, modality, economic_data
                ) VALUES (
                    ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16,
                    ?17, ?18, ?19, ?20, ?21, ?22, ?23, ?24, ?25, ?26, ?27, ?28, ?29, ?30, ?31, ?32
                )",
            ).map_err(|e| e.to_string())?;

            for m in models {
                let econ_json = m.economic.as_ref().and_then(|e| serde_json::to_string(e).ok());
                stmt.execute(params![
                    snapshot_id,
                    m.id,
                    m.name,
                    m.family,
                    m.param_count_b,
                    m.size_gb,
                    m.vram_rec_gb,
                    m.tokens_per_sec,
                    m.tps_hardware,
                    m.scores.average,
                    m.scores.ifeval,
                    m.scores.bbh,
                    m.scores.math_lvl5,
                    m.scores.gpqa,
                    m.scores.mmlu_pro,
                    m.scores.musr,
                    m.scores.open_llm,
                    m.scores.coding,
                    m.scores.agentic,
                    m.downloads as i64,
                    m.likes as i64,
                    m.ollama_tag,
                    m.hf_url,
                    if m.is_moe { 1 } else { 0 },
                    m.context_k as i64,
                    if m.is_cloud { 1 } else { 0 },
                    m.provider,
                    m.pricing_input_usd,
                    m.pricing_output_usd,
                    m.release_year.map(|y| y as i64),
                    m.modality.as_str(),
                    econ_json,
                ]).map_err(|e| e.to_string())?;
            }
        }

        tx.commit().map_err(|e| e.to_string())?;
        Ok(snapshot_id)
    }

    fn list_snapshots(&self) -> Result<Vec<SnapshotMeta>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare(
            "SELECT id, created_at, model_count, summary FROM snapshots ORDER BY id DESC",
        ).map_err(|e| e.to_string())?;

        let rows = stmt.query_map([], |row| {
            Ok(SnapshotMeta {
                id: row.get(0)?,
                created_at: row.get(1)?,
                model_count: row.get::<_, i64>(2)? as usize,
                summary: row.get(3)?,
            })
        }).map_err(|e| e.to_string())?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| e.to_string())?);
        }
        Ok(list)
    }

    fn load_snapshot_models(&self, snapshot_id: i64) -> Result<Vec<ModelBenchmark>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare(
            "SELECT repo_id, name, family, param_count_b, size_gb, vram_rec_gb,
                    tokens_per_sec, tps_hardware, score_average, score_ifeval, score_bbh,
                    score_math, score_gpqa, score_mmlu_pro, score_musr, downloads, likes,
                    ollama_tag, hf_url, is_moe, context_k, score_open_llm, score_coding, score_agentic,
                    is_cloud, provider, pricing_input_usd, pricing_output_usd, release_year, modality, economic_data
             FROM models WHERE snapshot_id = ?1 ORDER BY id ASC",
        ).map_err(|e| e.to_string())?;

        let rows = stmt.query_map([snapshot_id], |row| {
            let econ: Option<EconomicScores> = row.get::<_, Option<String>>(30).ok().flatten()
                .and_then(|s| serde_json::from_str(&s).ok());
            let rw_score = econ.as_ref().and_then(|e| e.composite_score());

            Ok(ModelBenchmark {
                id: row.get(0)?,
                name: row.get(1)?,
                family: row.get(2)?,
                param_count_b: row.get(3)?,
                size_gb: row.get(4)?,
                vram_rec_gb: row.get(5)?,
                tokens_per_sec: row.get(6)?,
                tps_hardware: row.get(7)?,
                scores: BenchmarkScores {
                    average: row.get(8)?,
                    ifeval: row.get(9)?,
                    bbh: row.get(10)?,
                    math_lvl5: row.get(11)?,
                    gpqa: row.get(12)?,
                    mmlu_pro: row.get(13)?,
                    musr: row.get(14)?,
                    open_llm: row.get(21).ok(),
                    coding: row.get(22).ok(),
                    agentic: row.get(23).ok(),
                    real_world: rw_score,
                },
                downloads: row.get::<_, i64>(15)? as u64,
                likes: row.get::<_, i64>(16)? as u64,
                ollama_tag: row.get(17)?,
                hf_url: row.get(18)?,
                is_moe: row.get::<_, i64>(19)? == 1,
                context_k: row.get::<_, i64>(20)? as u32,
                is_cloud: row.get::<_, i64>(24).unwrap_or(0) == 1,
                provider: row.get(25).ok(),
                pricing_input_usd: row.get(26).ok(),
                pricing_output_usd: row.get(27).ok(),
                release_year: row.get::<_, Option<i64>>(28).ok().flatten().map(|y| y as u16),
                modality: row.get::<_, Option<String>>(29).ok().flatten().map(|s| ModelModality::from_str_lenient(&s)).unwrap_or_default(),
                economic: econ,
            })
        }).map_err(|e| e.to_string())?;

        let mut models = Vec::new();
        for r in rows {
            models.push(r.map_err(|e| e.to_string())?);
        }
        Ok(models)
    }

    fn load_latest_snapshot(&self) -> Result<Option<(SnapshotMeta, Vec<ModelBenchmark>)>, String> {
        let snapshots = self.list_snapshots()?;
        if let Some(latest) = snapshots.first() {
            let models = self.load_snapshot_models(latest.id)?;
            Ok(Some((latest.clone(), models)))
        } else {
            Ok(None)
        }
    }
}
