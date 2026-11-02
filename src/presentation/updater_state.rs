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
    CheckResult(Result<ReleaseInfo, String>),
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
        }
    }

    pub fn trigger_check(&mut self, ctx: egui::Context) {
        if self.status == UpdateStatus::Checking {
            return;
        }
        self.status = UpdateStatus::Checking;
        let tx = self.sender.clone();
        let checker = self.checker.clone();
        let cur_ver = self.current_version.to_string();

        tokio::spawn(async move {
            let res = checker.check_latest_release(&cur_ver).await;
            let _ = tx.send(UpdaterMessage::CheckResult(res));
            ctx.request_repaint();
        });
    }

    pub fn trigger_self_update(&mut self, ctx: egui::Context, dl_url: String) {
        self.status = UpdateStatus::Downloading("Téléchargement de la nouvelle version...".into());
        let tx = self.sender.clone();

        tokio::spawn(async move {
            match SelfUpdateService::download_update(&dl_url).await {
                Ok(temp_path) => {
                    let _ = tx.send(UpdaterMessage::DownloadProgress("Installation et redémarrage automatique...".into()));
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
                UpdaterMessage::CheckResult(Ok(info)) => {
                    if info.is_newer {
                        self.show_modal = true;
                        self.status = UpdateStatus::Available(info);
                    } else {
                        self.status = UpdateStatus::UpToDate(info.tag_name);
                    }
                }
                UpdaterMessage::CheckResult(Err(err)) => {
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

    pub fn render_top_bar_button(&mut self, ui: &mut egui::Ui, ctx: &egui::Context, lang: Language) {
        let t = lang.t().update;
        match &self.status {
            UpdateStatus::Idle => {
                let btn = egui::Button::new(
                    egui::RichText::new(t.btn_check).color(ThemeTokens::CYAN_EDGE).strong(),
                );
                if ui.add(btn).on_hover_text(t.tooltip_check).clicked() {
                    self.trigger_check(ctx.clone());
                }
            }
            UpdateStatus::Checking => {
                ui.label(egui::RichText::new(t.btn_checking).color(ThemeTokens::TEXT_MUTED).size(11.5));
            }
            UpdateStatus::UpToDate(tag) => {
                let text = format!("{} ({})", t.btn_up_to_date, tag);
                let btn = egui::Button::new(egui::RichText::new(text).color(ThemeTokens::SUCCESS).size(11.5));
                if ui.add(btn).on_hover_text(t.tooltip_check).clicked() {
                    self.trigger_check(ctx.clone());
                }
            }
            UpdateStatus::Available(info) => {
                let text = format!("{} {}", t.btn_available, info.tag_name);
                let btn = egui::Button::new(
                    egui::RichText::new(text).color(ThemeTokens::GOLD_5090).strong(),
                );
                if ui.add(btn).on_hover_text(t.tooltip_check).clicked() {
                    self.show_modal = true;
                }
            }
            UpdateStatus::Downloading(msg) => {
                ui.label(egui::RichText::new(format!("MàJ: {msg}")).color(ThemeTokens::CYAN_EDGE).size(11.0));
            }
            UpdateStatus::Error(err) => {
                let btn = egui::Button::new(
                    egui::RichText::new(format!("[!] {}", t.btn_check)).color(ThemeTokens::WARNING),
                );
                if ui.add(btn).on_hover_text(format!("{}: {err}", t.err_check)).clicked() {
                    self.trigger_check(ctx.clone());
                }
            }
        }
    }

    pub fn render_modal(&mut self, ctx: &egui::Context, lang: Language) {
        if !self.show_modal {
            return;
        }

        let t = lang.t().update;
        let mut is_open = self.show_modal;
        let mut trigger_url_to_install = None;
        let mut close_requested = false;

        egui::Window::new(t.modal_title)
            .open(&mut is_open)
            .resizable(false)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .fixed_size([500.0, 340.0])
            .show(ctx, |ui| {
                ui.add_space(4.0);

                match &self.status {
                    UpdateStatus::Downloading(msg) => {
                        ui.vertical_centered(|ui| {
                            ui.add_space(20.0);
                            ui.spinner();
                            ui.add_space(10.0);
                            ui.label(egui::RichText::new(msg).color(ThemeTokens::CYAN_EDGE).size(14.0).strong());
                            ui.add_space(8.0);
                            ui.label(
                                egui::RichText::new("Le binaire se met à jour en arrière-plan.\nL'application va se relancer automatiquement dans un instant.")
                                    .color(ThemeTokens::TEXT_SECONDARY)
                                    .size(12.0)
                            );
                            ui.add_space(20.0);
                        });
                    }
                    UpdateStatus::Available(info) => {
                        ui.horizontal(|ui| {
                            ui.label(egui::RichText::new(t.current_version).color(ThemeTokens::TEXT_MUTED));
                            ui.label(egui::RichText::new(format!("v{}", self.current_version)).strong());
                            ui.label("->");
                            ui.label(egui::RichText::new(t.new_version).color(ThemeTokens::GOLD_5090).strong());
                            ui.label(egui::RichText::new(&info.tag_name).color(ThemeTokens::GOLD_5090).size(14.0).strong());
                        });

                        if let Some(date) = &info.published_at {
                            ui.label(egui::RichText::new(format!("Date : {date}")).size(11.0).color(ThemeTokens::TEXT_MUTED));
                        }

                        ui.separator();
                        ui.label(egui::RichText::new(t.release_notes_title).strong());

                        let body_text = info.body.as_deref().unwrap_or(t.no_notes);
                        egui::ScrollArea::vertical().max_height(130.0).show(ui, |ui| {
                            egui::Frame::group(ui.style())
                                .fill(ThemeTokens::BG_PANEL)
                                .show(ui, |ui| {
                                    ui.label(egui::RichText::new(body_text).size(11.5));
                                });
                        });

                        ui.add_space(10.0);
                        let dl_url = info.download_url.as_deref().unwrap_or(&info.html_url);
                        let is_direct_binary = dl_url.ends_with(".exe") || dl_url.ends_with(".AppImage") || dl_url.ends_with(".bin");

                        ui.horizontal(|ui| {
                            if is_direct_binary {
                                let update_btn = egui::Button::new(
                                    egui::RichText::new(" Mettre à jour en 1 Clic (Auto-Redémarrage)")
                                        .color(ThemeTokens::CYAN_EDGE)
                                        .strong()
                                );
                                if ui.add(update_btn).clicked() {
                                    trigger_url_to_install = Some(dl_url.to_string());
                                }
                            }

                            if ui.button(egui::RichText::new(" Télécharger via Navigateur").color(ThemeTokens::TEXT_MUTED)).clicked() {
                                let _ = open::that(dl_url);
                            }

                            if ui.button(t.btn_close).clicked() {
                                close_requested = true;
                            }
                        });
                    }
                    UpdateStatus::Error(err) => {
                        ui.label(egui::RichText::new(format!("Erreur lors de la mise à jour: {err}")).color(ThemeTokens::WARNING));
                        ui.add_space(8.0);
                        if let UpdateStatus::Available(info) = &self.status {
                            let dl_url = info.download_url.as_deref().unwrap_or(&info.html_url);
                            if ui.button(egui::RichText::new("Télécharger manuellement via le navigateur").color(ThemeTokens::CYAN_EDGE)).clicked() {
                                let _ = open::that(dl_url);
                            }
                        }
                        if ui.button(t.btn_close).clicked() {
                            close_requested = true;
                        }
                    }
                    _ => {
                        if ui.button(t.btn_close).clicked() {
                            close_requested = true;
                        }
                    }
                }
            });

        self.show_modal = if close_requested { false } else { is_open };

        if let Some(url) = trigger_url_to_install {
            self.trigger_self_update(ctx.clone(), url);
        }
    }
}
