use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use eframe::egui;
use crate::domain::updater::{IUpdateChecker, ReleaseInfo};
use crate::infrastructure::updater::SelfUpdateService;
use crate::presentation::i18n::Language;
use crate::presentation::theme::ThemeTokens;

#[derive(Debug, Clone, PartialEq)]
pub enum UpdateStatus {
    Idle,
    Checking,
    UpToDate(String),
    Available(ReleaseInfo),
    Downloading(String),
    Error(String),
}

pub enum UpdaterMessage {
    CheckResult { res: Result<ReleaseInfo, String>, open_modal_if_newer: bool },
    DownloadProgress(String),
    InstallError(String),
}

pub struct UpdaterState {
    checker: Arc<dyn IUpdateChecker>,
    pub status: UpdateStatus,
    pub show_modal: bool,
    sender: Sender<UpdaterMessage>,
    receiver: Receiver<UpdaterMessage>,
    pub current_version: &'static str,
    auto_checked_startup: bool,
}

impl UpdaterState {
    pub const APP_VERSION: &'static str = env!("CARGO_PKG_VERSION");

    pub fn new(checker: Arc<dyn IUpdateChecker>) -> Self {
        let (sender, receiver) = channel();
        Self {
            checker,
            status: UpdateStatus::Idle,
            show_modal: false,
            sender,
            receiver,
            current_version: Self::APP_VERSION,
            auto_checked_startup: false,
        }
    }

    pub fn ensure_startup_check(&mut self, ctx: egui::Context) {
        if self.auto_checked_startup {
            return;
        }
        self.auto_checked_startup = true;
        self.trigger_background_check(ctx);
    }

    /// Silent check invoked automatically at startup and on clicking Refresh (does not pop modal uninvited).
    pub fn trigger_background_check(&mut self, ctx: egui::Context) {
        self.spawn_check(ctx, false);
    }

    /// Interactive check invoked when user explicitly clicks the version badge.
    pub fn trigger_check(&mut self, ctx: egui::Context) {
        self.spawn_check(ctx, true);
    }

    fn spawn_check(&mut self, ctx: egui::Context, open_modal_if_newer: bool) {
        if matches!(self.status, UpdateStatus::Checking | UpdateStatus::Downloading(_)) {
            return;
        }
        if tokio::runtime::Handle::try_current().is_err() {
            return;
        }
        self.status = UpdateStatus::Checking;
        let tx = self.sender.clone();
        let checker = self.checker.clone();
        let cur_ver = self.current_version.to_string();

        tokio::spawn(async move {
            let res = checker.check_latest_release(&cur_ver).await;
            let _ = tx.send(UpdaterMessage::CheckResult { res, open_modal_if_newer });
            ctx.request_repaint();
        });
    }

    pub fn trigger_self_update(&mut self, ctx: egui::Context, dl_url: String, is_fr: bool) {
        if tokio::runtime::Handle::try_current().is_err() {
            return;
        }
        let dl_msg = if is_fr {
            "Téléchargement de la nouvelle version..."
        } else {
            "Downloading new release..."
        };
        let inst_msg = if is_fr {
            "Installation et redémarrage automatique..."
        } else {
            "Installing & restarting application..."
        };
        self.status = UpdateStatus::Downloading(dl_msg.into());
        let tx = self.sender.clone();

        tokio::spawn(async move {
            match SelfUpdateService::download_update(&dl_url).await {
                Ok(temp_path) => {
                    let _ = tx.send(UpdaterMessage::DownloadProgress(inst_msg.into()));
                    ctx.request_repaint();
                    if let Err(e) = SelfUpdateService::apply_and_restart(&temp_path) {
                        let _ = tx.send(UpdaterMessage::InstallError(e));
                        ctx.request_repaint();
                    }
                }
                Err(err) => {
                    let _ = tx.send(UpdaterMessage::InstallError(err));
                    ctx.request_repaint();
                }
            }
        });
    }

    pub fn poll(&mut self) {
        while let Ok(msg) = self.receiver.try_recv() {
            match msg {
                UpdaterMessage::CheckResult { res: Ok(info), open_modal_if_newer } => {
                    if info.is_newer {
                        if open_modal_if_newer {
                            self.show_modal = true;
                        }
                        self.status = UpdateStatus::Available(info);
                    } else {
                        self.status = UpdateStatus::UpToDate(info.tag_name);
                    }
                }
                UpdaterMessage::CheckResult { res: Err(err), .. } => {
                    self.status = UpdateStatus::Error(err);
                }
                UpdaterMessage::DownloadProgress(msg) => {
                    self.status = UpdateStatus::Downloading(msg);
                }
                UpdaterMessage::InstallError(err) => {
                    self.status = UpdateStatus::Error(err);
                }
            }
        }
    }

    pub fn format_version_tag(tag: &str) -> String {
        let trimmed = tag.trim();
        if trimmed.starts_with('v') || trimmed.starts_with('V') {
            format!("v{}", &trimmed[1..])
        } else {
            format!("v{trimmed}")
        }
    }

    pub fn render_top_bar_button(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, lang: Language) {
        let t = lang.t().update;
        let cur_tag = Self::format_version_tag(self.current_version);
        match &self.status {
            UpdateStatus::Available(info) => {
                let next_tag = Self::format_version_tag(&info.tag_name);
                let text = format!("{} {next_tag}", t.btn_available);
                let btn = egui::Button::new(egui::RichText::new(text).color(ThemeTokens::GOLD_5090).strong().size(11.5))
                    .fill(egui::Color32::from_rgb(48, 36, 12))
                    .stroke(egui::Stroke::new(1.2, ThemeTokens::GOLD_5090));
                let tip = format!("{} {cur_tag} -> {next_tag}\n{}", t.current_version, t.tooltip_check);
                if ui.add(btn).on_hover_text(tip).clicked() {
                    self.show_modal = true;
                }
            }
            UpdateStatus::Checking => {
                ui.label(egui::RichText::new(format!("{cur_tag} ({})", t.btn_checking)).color(ThemeTokens::TEXT_MUTED).size(11.5))
                    .on_hover_text(t.tooltip_check);
            }
            UpdateStatus::UpToDate(tag) => {
                let tip = format!("{} {cur_tag} ({} : {tag})\n{}", t.current_version, t.btn_up_to_date, t.tooltip_check);
                let btn = egui::Button::new(egui::RichText::new(&cur_tag).color(ThemeTokens::SUCCESS).size(11.5)).frame(false);
                if ui.add(btn).on_hover_text(tip).clicked() {
                    self.trigger_check(ctx.clone());
                }
            }
            UpdateStatus::Downloading(msg) => {
                ui.label(egui::RichText::new(format!("{cur_tag} - {msg}")).color(ThemeTokens::CYAN_EDGE).size(11.0));
            }
            UpdateStatus::Idle | UpdateStatus::Error(_) => {
                let tip = match &self.status {
                    UpdateStatus::Error(err) => format!("{cur_tag} | {}: {err}\n{}", t.err_check, t.tooltip_check),
                    _ => format!("{} {cur_tag}\n{}", t.current_version, t.tooltip_check),
                };
                let btn = egui::Button::new(egui::RichText::new(&cur_tag).color(ThemeTokens::TEXT_SECONDARY).size(11.5)).frame(false);
                if ui.add(btn).on_hover_text(tip).clicked() {
                    self.trigger_check(ctx.clone());
                }
            }
        }
    }

    fn is_installable_asset(dl_url: &str) -> bool {
        let low = dl_url.to_lowercase();
        low.ends_with(".exe") || low.ends_with(".zip") || low.ends_with(".tar.gz")
            || low.ends_with(".tgz") || low.ends_with(".appimage") || low.ends_with(".bin")
    }

    pub fn render_modal(&mut self, ctx: &egui::Context, lang: Language) {
        if !self.show_modal {
            return;
        }
        let t = lang.t().update;
        let is_fr = lang == Language::French;
        let mut is_open = self.show_modal;
        let mut trigger_url_to_install = None;
        let mut close_requested = false;

        egui::Window::new(t.modal_title)
            .open(&mut is_open)
            .resizable(false)
            .collapsible(false)
            .default_pos(ctx.screen_rect().center())
            .pivot(egui::Align2::CENTER_CENTER)
            .fixed_size([520.0, 340.0])
            .show(ctx, |ui| {
                ui.add_space(4.0);
                match &self.status {
                    UpdateStatus::Downloading(msg) => {
                        ui.vertical_centered(|ui| {
                            ui.add_space(24.0);
                            ui.spinner();
                            ui.add_space(10.0);
                            ui.label(egui::RichText::new(msg).color(ThemeTokens::CYAN_EDGE).size(14.0).strong());
                            ui.add_space(8.0);
                            let sub = if is_fr {
                                "Le binaire se met à jour et l'application va redémarrer automatiquement."
                            } else {
                                "Updating executable and restarting automatically..."
                            };
                            ui.label(egui::RichText::new(sub).color(ThemeTokens::TEXT_SECONDARY).size(12.0));
                        });
                    }
                    UpdateStatus::Available(info) => {
                        let cur_tag = Self::format_version_tag(self.current_version);
                        let next_tag = Self::format_version_tag(&info.tag_name);
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(t.current_version).color(ThemeTokens::TEXT_MUTED));
                            ui.label(egui::RichText::new(cur_tag).strong());
                            ui.label("->");
                            ui.label(egui::RichText::new(t.new_version).color(ThemeTokens::GOLD_5090).strong());
                            ui.label(egui::RichText::new(next_tag).color(ThemeTokens::GOLD_5090).size(14.0).strong());
                        });
                        if let Some(date) = &info.published_at {
                            ui.label(egui::RichText::new(format!("Date : {date}")).size(11.0).color(ThemeTokens::TEXT_MUTED));
                        }
                        ui.separator();
                        ui.label(egui::RichText::new(t.release_notes_title).strong());
                        let body_text = info.body.as_deref().unwrap_or(t.no_notes);
                        egui::ScrollArea::vertical().max_height(135.0).show(ui, |ui| {
                            egui::Frame::group(ui.style()).fill(ThemeTokens::BG_PANEL).show(ui, |ui| {
                                ui.label(egui::RichText::new(body_text).size(11.5));
                            });
                        });
                        ui.add_space(10.0);
                        let dl_url = info.download_url.as_deref().unwrap_or(&info.html_url);
                        ui.horizontal_wrapped(|ui| {
                            if Self::is_installable_asset(dl_url) {
                                let update_btn = egui::Button::new(egui::RichText::new(t.btn_download).color(ThemeTokens::CYAN_EDGE).strong());
                                if ui.add(update_btn).clicked() {
                                    trigger_url_to_install = Some(dl_url.to_string());
                                }
                            }
                            let web_lbl = if is_fr { "Ouvrir sur GitHub" } else { "Open on GitHub" };
                            if ui.button(egui::RichText::new(web_lbl).color(ThemeTokens::TEXT_MUTED)).clicked() {
                                let _ = open::that(&info.html_url);
                            }
                            if ui.button(t.btn_close).clicked() {
                                close_requested = true;
                            }
                        });
                    }
                    UpdateStatus::Error(err) => {
                        ui.label(egui::RichText::new(format!("{}: {err}", t.err_check)).color(ThemeTokens::WARNING));
                        ui.add_space(8.0);
                        if ui.button(t.btn_close).clicked() { close_requested = true; }
                    }
                    _ => {
                        if ui.button(t.btn_close).clicked() { close_requested = true; }
                    }
                }
            });

        self.show_modal = if close_requested { false } else { is_open };
        if let Some(url) = trigger_url_to_install {
            self.trigger_self_update(ctx.clone(), url, is_fr);
        }
    }
}
