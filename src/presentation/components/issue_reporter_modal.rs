use eframe::egui;
use crate::presentation::i18n::Language;
use crate::presentation::theme::ThemeTokens;

pub fn render_sync_error_modal(
    ctx: &egui::Context,
    sync_error: &mut Option<String>,
    lang: Language,
) {
    let err_msg = match sync_error {
        Some(msg) => msg.clone(),
        None => return,
    };

    let t = lang.sync_error();
    let mut win_open = true;
    let mut should_close = false;

    egui::Window::new(egui::RichText::new(t.dialog_title).strong().color(ThemeTokens::DANGER))
        .open(&mut win_open)
        .collapsible(false)
        .resizable(true)
        .default_size(egui::vec2(540.0, 390.0))
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.label(egui::RichText::new(t.dialog_desc).color(ThemeTokens::TEXT_PRIMARY).size(12.0));
            ui.add_space(8.0);

            ui.label(egui::RichText::new(t.trace_label).strong().color(ThemeTokens::TEXT_SECONDARY).size(11.0));
            egui::Frame::group(ui.style())
                .fill(egui::Color32::from_rgb(12, 16, 24))
                .inner_margin(8.0)
                .rounding(4.0)
                .show(ui, |ui| {
                    egui::ScrollArea::vertical().max_height(140.0).show(ui, |ui| {
                        ui.label(egui::RichText::new(&err_msg).monospace().size(10.5).color(ThemeTokens::WARNING));
                    });
                });

            ui.add_space(8.0);
            ui.label(egui::RichText::new("ℹ️ En cliquant ci-dessous, la page GitHub s'ouvre avec le rapport d'incident 100% pré-rempli. Vous n'avez qu'à cliquer sur « Submit new issue ».").size(11.0).color(ThemeTokens::TEXT_MUTED));
            ui.add_space(8.0);

            let copy_key = egui::Id::new("copy_sync_report_time");
            let last_copy = ui.ctx().memory_mut(|m| m.data.get_temp::<f64>(copy_key)).unwrap_or(0.0);
            let is_recent = ui.input(|i| i.time) - last_copy < 2.5;

            let report_body = format!(
                "**Sync Incident Report**\n- Version: v{}\n- OS: {}\n- Timestamp: {}\n- Trace:\n```\n{}\n```",
                env!("CARGO_PKG_VERSION"),
                std::env::consts::OS,
                chrono::Local::now().to_rfc3339(),
                err_msg
            );

            ui.horizontal(|ui| {
                if ui.button(egui::RichText::new(t.btn_github_issue).strong().color(ThemeTokens::GOLD_5090)).clicked() {
                    let title = format!("[Sync Incident] Data refresh failure: {}", err_msg.lines().next().unwrap_or("API error"));
                    let url = format!(
                        "https://github.com/Hidr4c/benchmark-llm-gui/issues/new?title={}&body={}",
                        url_encode(&title),
                        url_encode(&report_body)
                    );
                    ui.ctx().copy_text(report_body.clone());
                    let _ = open::that(url);
                }

                let btn_text = if is_recent { t.copied_notice } else { t.btn_copy_report };
                let btn_color = if is_recent { ThemeTokens::SUCCESS } else { ThemeTokens::CYAN_EDGE };
                if ui.button(egui::RichText::new(btn_text).color(btn_color)).clicked() {
                    ui.ctx().copy_text(report_body);
                    ui.ctx().memory_mut(|m| m.data.insert_temp(copy_key, ui.input(|i| i.time)));
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(t.btn_close).clicked() {
                        should_close = true;
                    }
                });
            });
        });

    if should_close || !win_open {
        *sync_error = None;
    }
}

pub fn render_contact_modal(
    ctx: &egui::Context,
    contact_open: &mut bool,
    lang: Language,
) {
    if !*contact_open {
        return;
    }

    let t = lang.contact();
    let mut win_open = true;
    let mut should_close = false;

    egui::Window::new(egui::RichText::new(t.dialog_title).strong().color(ThemeTokens::CYAN_EDGE))
        .open(&mut win_open)
        .collapsible(false)
        .resizable(true)
        .default_size(egui::vec2(520.0, 420.0))
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.label(egui::RichText::new(t.dialog_desc).color(ThemeTokens::TEXT_PRIMARY).size(12.0));
            ui.add_space(8.0);

            // In-app text draft
            let draft_key = egui::Id::new("contact_feedback_draft_text");
            let mut draft_text = ui.ctx().memory_mut(|m| m.data.get_temp::<String>(draft_key)).unwrap_or_default();

            ui.label(egui::RichText::new("Rédigez votre message ou idée directement ici :").strong().size(11.5).color(ThemeTokens::GOLD_5090));
            let edit_resp = ui.add(
                egui::TextEdit::multiline(&mut draft_text)
                    .hint_text("Décrivez votre idée, nouveau modèle souhaité ou anomalie rencontrée...")
                    .desired_rows(5)
                    .desired_width(f32::INFINITY)
            );
            if edit_resp.changed() {
                ui.ctx().memory_mut(|m| m.data.insert_temp(draft_key, draft_text.clone()));
            }

            ui.add_space(6.0);
            ui.label(egui::RichText::new("ℹ️ En cliquant ci-dessous, la page GitHub s'ouvre avec votre message déjà 100% pré-rempli. Vous n'avez qu'à vous connecter et cliquer sur « Submit new issue ».").size(11.0).color(ThemeTokens::TEXT_MUTED));
            ui.add_space(10.0);

            // Action buttons
            let full_body = format!(
                "{}\n\n---\n- Application: Benchmark LLM Explorer v{}\n- OS: {}\n- Date: {}",
                if draft_text.trim().is_empty() { "(Aucun message rédigé)" } else { draft_text.trim() },
                env!("CARGO_PKG_VERSION"),
                std::env::consts::OS,
                chrono::Local::now().to_rfc3339()
            );

            ui.horizontal(|ui| {
                if ui.button(egui::RichText::new("🐙 Ouvrir sur GitHub (Pré-rempli)").strong().color(ThemeTokens::CYAN_EDGE)).clicked() {
                    let title = if draft_text.trim().is_empty() {
                        "[Retour Utilisateur / Suggestion] ".to_string()
                    } else {
                        format!("[Retour] {}", draft_text.lines().next().unwrap_or("Suggestion"))
                    };
                    let url = format!(
                        "https://github.com/Hidr4c/benchmark-llm-gui/issues/new?title={}&body={}",
                        url_encode(&title),
                        url_encode(&full_body)
                    );
                    ui.ctx().copy_text(full_body.clone());
                    let _ = open::that(url);
                }

                let copy_key = egui::Id::new("copy_feedback_time");
                let last_copy = ui.ctx().memory_mut(|m| m.data.get_temp::<f64>(copy_key)).unwrap_or(0.0);
                let is_recent = ui.input(|i| i.time) - last_copy < 2.5;

                let btn_text = if is_recent { t.copied_feedback } else { "📋 Copier dans le presse-papier" };
                let btn_color = if is_recent { ThemeTokens::SUCCESS } else { ThemeTokens::PRIMARY };
                if ui.button(egui::RichText::new(btn_text).color(btn_color)).clicked() {
                    ui.ctx().copy_text(full_body);
                    ui.ctx().memory_mut(|m| m.data.insert_temp(copy_key, ui.input(|i| i.time)));
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(t.btn_close).clicked() {
                        should_close = true;
                    }
                });
            });
        });

    if should_close || !win_open {
        *contact_open = false;
    }
}

fn url_encode(s: &str) -> String {
    let mut encoded = String::new();
    for b in s.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => encoded.push(b as char),
            b' ' => encoded.push_str("%20"),
            _ => {
                encoded.push_str(&format!("%{:02X}", b));
            }
        }
    }
    encoded
}
