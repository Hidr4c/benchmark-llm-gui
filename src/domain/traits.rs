use async_trait::async_trait;
use crate::domain::model::ModelBenchmark;

#[derive(Debug, Clone)]
pub struct SnapshotMeta {
    pub id: i64,
    pub created_at: String,
    pub model_count: usize,
    #[allow(dead_code)]
    pub summary: String,
}

/// Inversion of Control Port: Data Persistence Contract
pub trait IBenchmarkRepository: Send + Sync {
    fn save_snapshot(&mut self, models: &[ModelBenchmark], summary: &str) -> Result<i64, String>;
    fn list_snapshots(&self) -> Result<Vec<SnapshotMeta>, String>;
    fn load_snapshot_models(&self, snapshot_id: i64) -> Result<Vec<ModelBenchmark>, String>;
    fn load_latest_snapshot(&self) -> Result<Option<(SnapshotMeta, Vec<ModelBenchmark>)>, String>;
}

/// Inversion of Control Port: Network Fetching Contract
#[async_trait]
pub trait IModelFetcher: Send + Sync {
    async fn fetch_and_unify(&self) -> Result<Vec<ModelBenchmark>, String>;
}

/// Inversion of Control Port: User Settings and Table Preferences Persistence
pub trait IPreferencesRepository: Send + Sync {
    fn load_table_preferences(&self) -> Result<Option<crate::domain::table_preferences::TablePreferences>, String>;
    fn save_table_preferences(&self, prefs: &crate::domain::table_preferences::TablePreferences) -> Result<(), String>;
}

/// Inversion of Control Port: Ecosystem Persistence (Coding Harnesses & Local Runtimes)
pub trait IEcosystemRepository: Send + Sync {
    fn save_harness_snapshot(&mut self, data: &crate::domain::harness::HarnessSnapshotData) -> Result<(), String>;
    fn load_latest_harnesses(&self) -> Result<crate::domain::harness::HarnessSnapshotData, String>;
    fn save_runtimes_snapshot(&mut self, runtimes: &[crate::domain::runtime::LocalRuntimeProfile]) -> Result<(), String>;
    fn load_latest_runtimes(&self) -> Result<Vec<crate::domain::runtime::LocalRuntimeProfile>, String>;
}

/// Inversion of Control Port: Ecosystem Network Fetchers
#[async_trait]
pub trait IEcosystemFetcher: Send + Sync {
    async fn fetch_harnesses(&self) -> Result<crate::domain::harness::HarnessSnapshotData, String>;
    async fn fetch_runtimes(&self) -> Result<Vec<crate::domain::runtime::LocalRuntimeProfile>, String>;
}
