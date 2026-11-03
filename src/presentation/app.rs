use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use eframe::egui;
use crate::domain::hardware_detector::{DetectedHardware, IHardwareDetector};
use crate::domain::model::ModelBenchmark;
use crate::domain::simulator::HardwareRigConfig;
use crate::domain::traits::{IBenchmarkRepository, IModelFetcher, IPreferencesRepository, SnapshotMeta};
use crate::infrastructure::hardware::system_detector::SystemHardwareDetector;
use crate::infrastructure::updater::GitHubReleaseAdapter;
use crate::presentation::components::comparator_view::{render_comparator_view, ComparatorState};
use crate::presentation::components::detail_view::render_detail_panel;
use crate::presentation::components::domains_view::render_domains_view;
use crate::presentation::components::filter_bar::{render_filter_bar, FilterState};
use crate::presentation::components::graph_view::{render_graph_view, GraphViewState};
use crate::presentation::components::issue_reporter_modal::{render_contact_modal, render_sync_error_modal};
use crate::presentation::components::simulator_view::render_simulator_view;
use crate::presentation::components::sources_view::render_sources_view;
use crate::presentation::components::table_view::render_table;
use crate::presentation::components::top_bar::{render_top_header, AppTab};
use crate::presentation::components::tour_overlay::{render_tour_overlay, TourState};
use crate::presentation::components::tutorial_view::render_tutorial_view;
use crate::presentation::filtering::FilterService;
use crate::presentation::i18n::Language;
use crate::presentation::updater_state::UpdaterState;

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
    comp_state: ComparatorState,
    graph_state: GraphViewState,
    tour_state: TourState,
    is_syncing: bool,
    status_message: Option<(String, bool)>,
    sync_sender: Sender<Result<Vec<ModelBenchmark>, String>>,
    sync_receiver: Receiver<Result<Vec<ModelBenchmark>, String>>,
    pub zoom_factor: f32,
    pub language: Language,
    pub updater: UpdaterState,
    pub sync_error_dialog: Option<String>,
    pub contact_dialog_open: bool,
    last_saved_prefs: Option<crate::domain::table_preferences::TablePreferences>,
}

impl BenchmarkApp {
    pub fn new(repo: Box<dyn IBenchmarkRepository>, fetcher: Arc<dyn IModelFetcher>) -> Self {
        let (tx, rx) = channel();
        let (snapshots, selected_id, models, initial_msg) = match repo.load_latest_snapshot() {
            Ok(Some((latest, models))) => (repo.list_snapshots().unwrap_or_default(), Some(latest.id), models, None),
            Ok(None) => (Vec::new(), None, Vec::new(), Some(("Empty database. Click 'Refresh' to fetch models.".into(), false))),
            Err(e) => (Vec::new(), None, Vec::new(), Some((format!("Database error: {e}"), true))),
        };

        let mut filters = FilterState::default();
        if let Ok(pref_repo) = crate::infrastructure::preferences::SqlitePreferencesRepository::open("benchmark.db") {
            if let Ok(Some(prefs)) = pref_repo.load_table_preferences() {
                filters.sort_field = prefs.sort_field;
                filters.sort_order = prefs.sort_order;
                filters.table_prefs = prefs;
            }
        }
        let last_saved_prefs = Some(filters.table_prefs.clone());

        Self {
            repo, fetcher, detector: Box::new(SystemHardwareDetector::new()), detected_hardware: None,
            sim_config: HardwareRigConfig::default(), snapshots, selected_snapshot_id: selected_id,
            all_models: models, filters, selected_model_id: None, active_tab: AppTab::Table,
            comp_state: ComparatorState::default(), graph_state: GraphViewState::default(),
            tour_state: TourState::default(), is_syncing: false, status_message: initial_msg,
            sync_sender: tx, sync_receiver: rx, zoom_factor: 1.0, language: Language::French,
            updater: UpdaterState::new(Arc::new(GitHubReleaseAdapter::new("Hidr4c", "benchmark-llm-gui"))),
            sync_error_dialog: None, contact_dialog_open: false, last_saved_prefs,
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
            Ok(models) => { self.selected_snapshot_id = Some(snapshot_id); self.all_models = models; self.status_message = None; }
            Err(e) => { self.status_message = Some((format!("Error: {e}"), true)); }
        }
    }

    pub fn export_indices_to_csv(&mut self, indices: &[usize]) {
        if let Ok(mut w) = csv::Writer::from_path("benchmarks_export.csv") {
            let _ = w.write_record(["Name", "Family", "Year", "Size_GB", "VRAM_GB", "Cross_Score", "Speed_tok_s", "Cloud"]);
            for &idx in indices {
                let m = &self.all_models[idx];
                let sc = format!("{:.1}", m.score_for_sources(&self.filters.sources).unwrap_or(0.0));
                let _ = w.write_record([m.name.as_str(), m.family.as_str(), m.release_year_label().as_str(), &format!("{:.2}", m.size_gb), &format!("{:.2}", m.vram_rec_gb), &sc, &format!("{:.0}", m.tokens_per_sec.unwrap_or(0.0)), if m.is_cloud { "Yes" } else { "No" }]);
            }
            let _ = w.flush();
            self.status_message = Some(("CSV export successful ('benchmarks_export.csv')!".into(), false));
        }
    }
}

impl eframe::App for BenchmarkApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.set_zoom_factor(self.zoom_factor);
        self.updater.poll();

        if let Ok(result) = self.sync_receiver.try_recv() {
            self.is_syncing = false;
            match result {
                Ok(new_models) => {
                    let count = new_models.len();
                    let (sid, is_err, msg) = match self.repo.save_snapshot(&new_models, "Live Multi-Source Sync") {
                        Ok(nid) => (Some(nid), false, format!("Success: {count} models synchronized with cross scores!")),
                        Err(e) => (self.selected_snapshot_id, true, format!("Synchronized with database notice: {e}")),
                    };
                    self.snapshots = self.repo.list_snapshots().unwrap_or_default();
                    self.selected_snapshot_id = sid;
                    self.all_models = new_models;
                    self.status_message = Some((msg, is_err));
                }
                Err(err) => {
                    self.sync_error_dialog = Some(err.clone());
                    self.status_message = Some((format!("Download error: {err}"), true));
                }
            }
        }

        let filtered_indices = FilterService::compute_filtered_indices(&self.all_models, &self.filters);
        let total_count = self.all_models.len();
        let matching_count = filtered_indices.len();
        let (mut trigger_sync, mut trigger_auto_detect, mut trigger_export, mut trigger_launch_tour) = (false, false, false, false);
        let mut trigger_switch_snapshot = None;

        egui::TopBottomPanel::top("top_header_panel").show(ctx, |ui| {
            render_top_header(
                ui, &mut self.active_tab, &mut self.language, &mut self.zoom_factor, self.is_syncing,
                &self.snapshots, self.selected_snapshot_id, self.status_message.as_ref(), &mut self.updater,
                &mut trigger_sync, &mut trigger_switch_snapshot, &mut trigger_launch_tour, &mut self.contact_dialog_open,
            );
            if self.active_tab == AppTab::Table || self.active_tab == AppTab::Graph {
                ui.add_space(2.0); ui.separator(); ui.add_space(2.0);
                render_filter_bar(ui, &mut self.filters, total_count, matching_count, &mut trigger_auto_detect, &mut trigger_export, self.language);
            }
            ui.add_space(3.0);
        });

        if trigger_launch_tour { self.tour_state.start(); }
        if self.tour_state.request_clear_selection {
            self.selected_model_id = None;
            self.tour_state.request_clear_selection = false;
        }
        if let Some(snap_id) = trigger_switch_snapshot { self.switch_snapshot(snap_id); }
        if trigger_sync { self.trigger_live_sync(ctx.clone()); }
        if trigger_export { self.export_indices_to_csv(&filtered_indices); }
        if ctx.memory_mut(|m| m.data.get_temp::<bool>(egui::Id::new("req_tab_switch_comparator"))).unwrap_or(false) {
            ctx.memory_mut(|m| m.data.insert_temp(egui::Id::new("req_tab_switch_comparator"), false));
            if let Some(id0) = self.filters.compared_model_ids.first() { self.comp_state.slots[0] = Some(id0.clone()); }
            if let Some(id1) = self.filters.compared_model_ids.get(1) { self.comp_state.slots[1] = Some(id1.clone()); }
            self.active_tab = AppTab::Comparator;
        }
        if trigger_auto_detect {
            let detect_res = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.detector.detect()));
            match detect_res {
                Ok(Ok(hw)) => {
                    self.filters.hardware = hw.matched_platform;
                    if let Some(vram) = hw.gpu_vram_gb { self.graph_state.vram_budget_gb = vram; }
                    self.comp_state.target_platform = hw.matched_platform;
                    self.comp_state.rig = crate::domain::simulator::HardwareRigConfig::from_platform(hw.matched_platform);
                    self.comp_state.rig.ram_gb = hw.total_ram_gb;
                    self.status_message = None;
                    self.detected_hardware = Some(hw);
                }
                Ok(Err(e)) => self.status_message = Some((format!("Detection notice: {e}"), true)),
                Err(_) => self.status_message = Some(("Avertissement: Echec securise lors de la detection materielle".into(), true)),
            }
            self.tour_state.on_auto_detect_clicked();
        }

        let selected_model = self.selected_model_id.as_ref().and_then(|id| {
            self.all_models.iter().find(|m| &m.id == id).cloned()
        });
        if let Some(model) = selected_model {
            let panel_resp = egui::TopBottomPanel::bottom("detail_panel").resizable(true).default_height(240.0).min_height(180.0).show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    render_detail_panel(ui, &model, &mut self.selected_model_id, self.language);
                });
            });
            ctx.memory_mut(|m| m.data.insert_temp(egui::Id::new("tour_result_detail_panel"), panel_resp.response.rect));
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let t = self.language.t();
            if self.all_models.is_empty() && !self.is_syncing && self.active_tab != AppTab::Sources && self.active_tab != AppTab::Tutorial {
                ui.vertical_centered(|ui| {
                    ui.add_space(60.0); ui.heading("LLM Benchmark Explorer"); ui.add_space(10.0);
                    ui.label(t.no_matching_models); ui.add_space(15.0);
                    if ui.button(egui::RichText::new(t.sync_refresh.to_string()).size(15.0).strong()).clicked() {
                        self.trigger_live_sync(ctx.clone());
                    }
                });
            } else {
                match self.active_tab {
                    AppTab::Table => render_table(ui, &self.all_models, &filtered_indices, &mut self.filters, &mut self.selected_model_id, self.language),
                    AppTab::Graph => {
                        render_graph_view(ui, &mut self.graph_state, &self.all_models, &filtered_indices, &self.filters, &mut self.selected_model_id, self.language);
                        if ctx.memory_mut(|m| m.data.remove_temp::<bool>(egui::Id::new("tour_evt_recenter_clicked"))).unwrap_or(false) {
                            self.tour_state.on_recenter_clicked();
                        }
                    }
                    AppTab::Comparator => {
                        render_comparator_view(ui, &mut self.comp_state, self.detected_hardware.as_ref(), &self.all_models, &self.filters, &mut self.active_tab, self.language);
                        self.tour_state.check_and_update_comp_progress(&self.comp_state.slots);
                    }
                    AppTab::Simulator => {
                        egui::ScrollArea::vertical().show(ui, |ui| render_simulator_view(ui, &mut self.sim_config, self.detected_hardware.as_ref(), &self.all_models, &self.filters, &mut self.selected_model_id, self.language));
                        self.tour_state.check_and_update_sim_progress(&self.sim_config);
                    }
                    AppTab::Domains => render_domains_view(ui, &self.all_models, &mut self.filters, &mut self.active_tab, &mut self.selected_model_id, self.language),
                    AppTab::Sources => render_sources_view(ui, self.language),
                    AppTab::Tutorial => render_tutorial_view(ui, &mut self.tour_state, &mut self.active_tab, &mut self.filters, self.language),
                }
            }
        });

        if self.last_saved_prefs.as_ref() != Some(&self.filters.table_prefs) {
            self.last_saved_prefs = Some(self.filters.table_prefs.clone());
            if let Ok(pref_repo) = crate::infrastructure::preferences::SqlitePreferencesRepository::open("benchmark.db") {
                let _ = pref_repo.save_table_preferences(&self.filters.table_prefs);
            }
        }

        self.updater.render_modal(ctx, self.language);
        render_sync_error_modal(ctx, &mut self.sync_error_dialog, self.language);
        render_contact_modal(ctx, &mut self.contact_dialog_open, self.language);

        render_tour_overlay(
            ctx, &mut self.tour_state, &mut self.active_tab, &self.filters,
            self.selected_model_id.as_deref(), self.detected_hardware.as_ref(),
            &self.all_models, matching_count, total_count, self.language,
        );
    }
}
