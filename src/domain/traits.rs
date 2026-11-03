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
