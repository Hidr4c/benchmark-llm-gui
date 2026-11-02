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
        .default_size(egui::vec2(540.0, 380.0))
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
                    egui::ScrollArea::vertical().max_height(160.0).show(ui, |ui| {
                        ui.label(egui::RichText::new(&err_msg).monospace().size(10.5).color(ThemeTokens::WARNING));
                    });
                });

            ui.add_space(10.0);

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
        .default_size(egui::vec2(500.0, 360.0))
        .anchor(egui::Align2::CENTER_CENTER, egui::vec2(0.0, 0.0))
        .show(ctx, |ui| {
            ui.label(egui::RichText::new(t.dialog_desc).color(ThemeTokens::TEXT_PRIMARY).size(12.0));
            ui.add_space(8.0);

            // GitHub section
            ui.label(egui::RichText::new(t.section_github).strong().color(ThemeTokens::GOLD_5090).size(12.0));
            ui.horizontal(|ui| {
                if ui.button(egui::RichText::new(t.btn_suggest_feature).color(ThemeTokens::CYAN_EDGE)).clicked() {
                    let title = "[Feature / Model Request] ";
                    let body = "**Description de l'idée / Nouveau modèle :**\n\n**Bénéfice utilisateur :**\n";
                    let url = format!(
                        "https://github.com/Hidr4c/benchmark-llm-gui/issues/new?title={}&body={}",
                        url_encode(title),
                        url_encode(body)
                    );
                    let _ = open::that(url);
                }

                if ui.button(egui::RichText::new(t.btn_report_bug).color(ThemeTokens::DANGER)).clicked() {
                    let title = "[Bug Report] ";
                    let body = format!(
                        "**Comportement attendu :**\n\n**Comportement observé :**\n\n- Version: v{}\n- OS: {}\n",
                        env!("CARGO_PKG_VERSION"),
                        std::env::consts::OS
                    );
                    let url = format!(
                        "https://github.com/Hidr4c/benchmark-llm-gui/issues/new?title={}&body={}",
                        url_encode(title),
                        url_encode(&body)
                    );
                    let _ = open::that(url);
                }
            });

            ui.add_space(10.0);
            ui.separator();
            ui.add_space(10.0);

            // Anonymous feedback section (without GitHub account)
            ui.label(egui::RichText::new(t.section_anonymous).strong().color(ThemeTokens::SUCCESS).size(12.0));
            ui.label(egui::RichText::new(t.anonymous_desc).color(ThemeTokens::TEXT_SECONDARY).size(11.0));
            ui.add_space(6.0);

            let copy_key = egui::Id::new("copy_feedback_time");
            let last_copy = ui.ctx().memory_mut(|m| m.data.get_temp::<f64>(copy_key)).unwrap_or(0.0);
            let is_recent = ui.input(|i| i.time) - last_copy < 2.5;

            let feedback_template = format!(
                "--- Retour Utilisateur Benchmark LLM ---\nDate: {}\nVersion: v{}\nOS: {}\n\n[Votre message / idée / retour ici]\n",
                chrono::Local::now().to_rfc3339(),
                env!("CARGO_PKG_VERSION"),
                std::env::consts::OS
            );

            ui.horizontal(|ui| {
                let btn_text = if is_recent { t.copied_feedback } else { t.btn_copy_anonymous_feedback };
                let btn_color = if is_recent { ThemeTokens::SUCCESS } else { ThemeTokens::PRIMARY };
                if ui.button(egui::RichText::new(btn_text).color(btn_color).strong()).clicked() {
                    ui.ctx().copy_text(feedback_template);
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
