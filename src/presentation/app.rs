use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use eframe::egui;
use crate::domain::hardware_detector::{DetectedHardware, IHardwareDetector};
use crate::domain::model::ModelBenchmark;
use crate::domain::simulator::HardwareRigConfig;
use crate::domain::traits::{IBenchmarkRepository, IModelFetcher, SnapshotMeta};
use crate::infrastructure::hardware::system_detector::SystemHardwareDetector;
use crate::presentation::components::detail_view::render_detail_panel;
use crate::presentation::components::filter_bar::{render_filter_bar, FilterState};
use crate::presentation::components::graph_view::{render_graph_view, GraphViewState};
use crate::presentation::components::simulator_view::render_simulator_view;
use crate::presentation::components::sources_view::render_sources_view;
use crate::presentation::components::table_view::render_table;
use crate::presentation::components::top_bar::{render_top_header, AppTab};
use crate::presentation::components::tour_overlay::{render_tour_overlay, TourState};
use crate::presentation::components::tutorial_view::render_tutorial_view;
use crate::presentation::filtering::FilterService;
use crate::presentation::i18n::Language;

pub struct BenchmarkApp {
    repo: Box<dyn IBenchmarkRepository>,
    fetcher: Arc<dyn IModelFetcher>,
    detector: Box<dyn IHardwareDetector>,
    detected_hardware: Option<DetectedHardware>,
    sim_config: HardwareRigConfig,
    snapshots: Vec<SnapshotMeta>,
    selected_snapshot_id: Option<i64>,
    all_models: Vec<ModelBenchmark>,
    filters: FilterState,
    selected_model_id: Option<String>,
    active_tab: AppTab,
    graph_state: GraphViewState,
    tour_state: TourState,
    is_syncing: bool,
    status_message: Option<(String, bool)>,
    sync_sender: Sender<Result<Vec<ModelBenchmark>, String>>,
    sync_receiver: Receiver<Result<Vec<ModelBenchmark>, String>>,
    pub language: Language,
}

impl BenchmarkApp {
    pub fn new(repo: Box<dyn IBenchmarkRepository>, fetcher: Arc<dyn IModelFetcher>) -> Self {
        let (tx, rx) = channel();

        let (snapshots, selected_id, models, initial_msg) = match repo.load_latest_snapshot() {
            Ok(Some((latest, models))) => {
                let snaps = repo.list_snapshots().unwrap_or_default();
                let msg = format!("Active snapshot: {} ({} models in local database).", latest.created_at, latest.model_count);
                (snaps, Some(latest.id), models, (msg, false))
            }
            Ok(None) => {
                (Vec::new(), None, Vec::new(), ("Empty database. Click 'Refresh' to fetch models.".to_string(), false))
            }
            Err(e) => {
                (Vec::new(), None, Vec::new(), (format!("Database error: {e}"), true))
            }
        };

        Self {
            repo,
            fetcher,
            detector: Box::new(SystemHardwareDetector::new()),
            detected_hardware: None,
            sim_config: HardwareRigConfig::default(),
            snapshots,
            selected_snapshot_id: selected_id,
            all_models: models,
            filters: FilterState::default(),
            selected_model_id: None,
            active_tab: AppTab::Table,
            graph_state: GraphViewState::default(),
            tour_state: TourState::default(),
            is_syncing: false,
            status_message: Some(initial_msg),
            sync_sender: tx,
            sync_receiver: rx,
            language: Language::French,
        }
    }

    pub fn trigger_live_sync(&mut self, ctx: egui::Context) {
        if self.is_syncing { return; }
        self.is_syncing = true;
        self.status_message = Some(("Synchronizing with Hugging Face, Benchmarks & LiteLLM...".into(), false));

        let tx = self.sync_sender.clone();
        let fetcher = self.fetcher.clone();

        tokio::spawn(async move {
            let res = fetcher.fetch_and_unify().await;
            let _ = tx.send(res);
            ctx.request_repaint();
        });
    }

    pub fn switch_snapshot(&mut self, snapshot_id: i64) {
        match self.repo.load_snapshot_models(snapshot_id) {
            Ok(models) => {
                self.selected_snapshot_id = Some(snapshot_id);
                self.all_models = models;
                if let Some(s) = self.snapshots.iter().find(|s| s.id == snapshot_id) {
                    self.status_message = Some((format!("History: Snapshot from {} ({} models).", s.created_at, s.model_count), false));
                }
            }
            Err(e) => { self.status_message = Some((format!("Error: {e}"), true)); }
        }
    }

    pub fn export_indices_to_csv(&mut self, indices: &[usize]) {
        if let Ok(mut wtr) = csv::Writer::from_path("benchmarks_export.csv") {
            let _ = wtr.write_record(["Name", "Family", "Year", "Size_GB", "VRAM_GB", "Cross_Score", "Speed_tok_s", "Cloud"]);
            for &idx in indices {
                let m = &self.all_models[idx];
                let year_str = m.release_year_label();
                let size_str = format!("{:.2}", m.size_gb);
                let vram_str = format!("{:.2}", m.vram_rec_gb);
                let score_str = format!("{:.1}", m.score_for_sources(&self.filters.sources).unwrap_or(0.0));
                let tps_str = format!("{:.0}", m.tokens_per_sec.unwrap_or(0.0));
                let cloud_str = if m.is_cloud { "Yes" } else { "No" };
                let _ = wtr.write_record([
                    m.name.as_str(),
                    m.family.as_str(),
                    year_str.as_str(),
                    size_str.as_str(),
                    vram_str.as_str(),
                    score_str.as_str(),
                    tps_str.as_str(),
                    cloud_str,
                ]);
            }
            let _ = wtr.flush();
            self.status_message = Some(("CSV export successful ('benchmarks_export.csv')!".into(), false));
        }
    }
}

impl eframe::App for BenchmarkApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        if let Ok(result) = self.sync_receiver.try_recv() {
            self.is_syncing = false;
            match result {
                Ok(new_models) => {
                    let count = new_models.len();
                    match self.repo.save_snapshot(&new_models, "Live Multi-Source Sync") {
                        Ok(new_id) => {
                            self.snapshots = self.repo.list_snapshots().unwrap_or_default();
                            self.selected_snapshot_id = Some(new_id);
                            self.all_models = new_models;
                            self.status_message = Some((format!("Success: {count} models synchronized with cross scores!"), false));
                        }
                        Err(e) => {
                            self.all_models = new_models;
                            self.status_message = Some((format!("Synchronized with database notice: {e}"), true));
                        }
                    }
                }
                Err(err) => {
                    self.status_message = Some((format!("Download error: {err}"), true));
                }
            }
        }

        let filtered_indices = FilterService::compute_filtered_indices(&self.all_models, &self.filters);
        let total_count = self.all_models.len();
        let matching_count = filtered_indices.len();

        let mut trigger_sync = false;
        let mut trigger_auto_detect = false;
        let mut trigger_export = false;
        let mut trigger_switch_snapshot = None;
        let mut trigger_launch_tour = false;

        egui::TopBottomPanel::top("top_header_panel").show(ctx, |ui| {
            render_top_header(
                ui,
                &mut self.active_tab,
                &mut self.language,
                self.is_syncing,
                &self.snapshots,
                self.selected_snapshot_id,
                self.status_message.as_ref(),
                &mut trigger_sync,
                &mut trigger_switch_snapshot,
                &mut trigger_launch_tour,
            );

            if self.active_tab == AppTab::Table || self.active_tab == AppTab::Graph {
                ui.add_space(2.0);
                ui.separator();
                ui.add_space(2.0);

                render_filter_bar(
                    ui, &mut self.filters, total_count, matching_count,
                    &mut trigger_auto_detect, &mut trigger_export, self.language,
                );
            }
            ui.add_space(3.0);
        });

        if trigger_launch_tour { self.tour_state.start(); }
        if let Some(snap_id) = trigger_switch_snapshot { self.switch_snapshot(snap_id); }
        if trigger_sync { self.trigger_live_sync(ctx.clone()); }
        if trigger_export { self.export_indices_to_csv(&filtered_indices); }
        if trigger_auto_detect {
            match self.detector.detect() {
                Ok(hw) => {
                    self.filters.hardware = hw.matched_platform;
                    if let Some(vram) = hw.gpu_vram_gb {
                        self.graph_state.vram_budget_gb = vram;
                    }
                    let msg = format!("{}: {}", self.language.t().btn_auto_detect, hw.summary());
                    self.status_message = Some((msg, false));
                    self.detected_hardware = Some(hw);
                }
                Err(e) => {
                    self.status_message = Some((format!("Detection notice: {e}"), true));
                }
            }
            self.tour_state.on_auto_detect_clicked();
        }

        let selected_model = self.selected_model_id.as_ref().and_then(|id| {
            self.all_models.iter().find(|m| &m.id == id).cloned()
        });

        if let Some(model) = selected_model {
            egui::TopBottomPanel::bottom("detail_panel").resizable(true).default_height(240.0).min_height(180.0).show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    render_detail_panel(ui, &model, &mut self.selected_model_id, self.language);
                });
            });
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let t = self.language.t();
            if self.all_models.is_empty() && !self.is_syncing && self.active_tab != AppTab::Sources && self.active_tab != AppTab::Tutorial {
                ui.vertical_centered(|ui| {
                    ui.add_space(60.0);
                    ui.heading("👋 LLM Benchmark Explorer");
                    ui.add_space(10.0);
                    ui.label(t.no_matching_models);
                    ui.add_space(15.0);
                    if ui.button(egui::RichText::new(format!("🚀 {}", t.sync_refresh)).size(15.0).strong()).clicked() {
                        self.trigger_live_sync(ctx.clone());
                    }
                });
            } else {
                match self.active_tab {
                    AppTab::Table => {
                        render_table(ui, &self.all_models, &filtered_indices, &mut self.filters, &mut self.selected_model_id, self.language);
                    }
                    AppTab::Graph => {
                        render_graph_view(ui, &mut self.graph_state, &self.all_models, &filtered_indices, &self.filters, &mut self.selected_model_id, self.language);
                    }
                    AppTab::Simulator => {
                        egui::ScrollArea::vertical().show(ui, |ui| {
                            render_simulator_view(
                                ui,
                                &mut self.sim_config,
                                self.detected_hardware.as_ref(),
                                &self.all_models,
                                &self.filters,
                                &mut self.selected_model_id,
                                self.language,
                            );
                        });
                    }
                    AppTab::Sources => {
                        render_sources_view(ui, self.language);
                    }
                    AppTab::Tutorial => {
                        render_tutorial_view(ui, &mut self.tour_state, self.language);
                    }
                }
            }
        });

        // Interactive guided walkthrough tour overlay
        render_tour_overlay(
            ctx,
            &mut self.tour_state,
            &mut self.active_tab,
            &self.filters,
            self.selected_model_id.as_deref(),
            self.language,
        );
    }
}
