use async_trait::async_trait;
use crate::domain::harness::HarnessSnapshotData;
use crate::domain::runtime::LocalRuntimeProfile;
use crate::domain::traits::IEcosystemFetcher;
use super::harness_adapter::HarnessAdapter;
use super::runtime_adapter::RuntimeAdapter;

pub struct LiveEcosystemFetcher {
    harness_adapter: HarnessAdapter,
    runtime_adapter: RuntimeAdapter,
}

impl Default for LiveEcosystemFetcher {
    fn default() -> Self {
        let client = reqwest::Client::builder()
            .user_agent("BenchmarkLLM/2.0 (Ecosystem Explorer)")
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .expect("Failed to initialize ecosystem HTTP client");

        Self {
            harness_adapter: HarnessAdapter::new(client.clone()),
            runtime_adapter: RuntimeAdapter::new(client),
        }
    }
}

#[async_trait]
impl IEcosystemFetcher for LiveEcosystemFetcher {
    async fn fetch_harnesses(&self) -> Result<HarnessSnapshotData, String> {
        self.harness_adapter.fetch_all().await
    }

    async fn fetch_runtimes(&self) -> Result<Vec<LocalRuntimeProfile>, String> {
        self.runtime_adapter.fetch_runtimes().await
    }
}
