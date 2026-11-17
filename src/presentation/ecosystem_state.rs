use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use eframe::egui;
use crate::domain::harness::HarnessSnapshotData;
use crate::domain::runtime::LocalRuntimeProfile;
use crate::domain::traits::{IEcosystemFetcher, IEcosystemRepository};
use crate::infrastructure::fetcher::ecosystem_fetcher::LiveEcosystemFetcher;
use crate::infrastructure::repository::sqlite_ecosystem_repo::SqliteEcosystemRepository;

pub struct EcosystemState {
    repo: Option<SqliteEcosystemRepository>,
    fetcher: Arc<dyn IEcosystemFetcher>,
    pub harnesses: HarnessSnapshotData,
    pub runtimes: Vec<LocalRuntimeProfile>,
    pub is_syncing: bool,
    tx: Sender<Result<(HarnessSnapshotData, Vec<LocalRuntimeProfile>), String>>,
    rx: Receiver<Result<(HarnessSnapshotData, Vec<LocalRuntimeProfile>), String>>,
}

impl EcosystemState {
    pub fn load_or_empty(db_path: &str) -> Self {
        let (tx, rx) = channel();
        let repo = SqliteEcosystemRepository::open(db_path).ok();
        let (harnesses, runtimes) = if let Some(ref r) = repo {
            (
                r.load_latest_harnesses().unwrap_or_default(),
                r.load_latest_runtimes().unwrap_or_default(),
            )
        } else {
            (HarnessSnapshotData::default(), Vec::new())
        };

        Self {
            repo,
            fetcher: Arc::new(LiveEcosystemFetcher::default()),
            harnesses,
            runtimes,
            is_syncing: false,
            tx,
            rx,
        }
    }

    pub fn spawn_sync(&mut self, ctx: egui::Context) {
        if self.is_syncing {
            return;
        }
        self.is_syncing = true;
        let tx = self.tx.clone();
        let fetcher = self.fetcher.clone();
        tokio::spawn(async move {
            let (h_res, r_res) = tokio::join!(fetcher.fetch_harnesses(), fetcher.fetch_runtimes());
            let res = match (h_res, r_res) {
                (Ok(h), Ok(r)) => Ok((h, r)),
                (Err(e), _) | (_, Err(e)) => Err(e),
            };
            let _ = tx.send(res);
            ctx.request_repaint();
        });
    }

    pub fn poll(&mut self) -> Option<Result<String, String>> {
        if let Ok(res) = self.rx.try_recv() {
            self.is_syncing = false;
            match res {
                Ok((h, r)) => {
                    if let Some(ref mut repo) = self.repo {
                        let _ = repo.save_harness_snapshot(&h);
                        let _ = repo.save_runtimes_snapshot(&r);
                    }
                    let summary = format!("{} harnesses, {} runtimes synced", h.profiles.len(), r.len());
                    self.harnesses = h;
                    self.runtimes = r;
                    return Some(Ok(summary));
                }
                Err(e) => return Some(Err(e)),
            }
        }
        None
    }
}
