use eframe::egui;
use crate::domain::comparator::ModelHardwareMetrics;
use crate::domain::model::{ModelBenchmark, SourceSelection};
use crate::domain::simulator::HardwareRigConfig;
use crate::presentation::components::comparator_verdict::render_model_hardware_specs_grid;
use crate::presentation::i18n::Language;
use crate::presentation::theme::ThemeTokens;

pub fn render_graph_search_picker(
    ui: &mut egui::Ui,
    search_query: &mut String,
    all_models: &[ModelBenchmark],
    selected_model_id: &mut Option<String>,
    lang: Language,
) {
    let t_hint = if lang == Language::French {
        "Rechercher un modèle sur le graphe (nom, taille)..."
    } else {
        "Search a model on plot (name, size)..."
    };

    ui.horizontal(|ui| {
        ui.label(egui::RichText::new("Recherche :").strong().size(12.0));
        ui.add(
            egui::TextEdit::singleline(search_query)
                .id_source("graph_search_model_input")
                .hint_text(t_hint)
                .desired_width(340.0),
        );
        if !search_query.is_empty() && ui.small_button("Effacer").clicked() {
            search_query.clear();
        }
    });

    if search_query.trim().len() >= 2 {
        let q = search_query.trim().to_lowercase();
        let matches: Vec<&ModelBenchmark> = all_models
            .iter()
            .filter(|m| m.name.to_lowercase().contains(&q) || m.family.to_lowercase().contains(&q))
            .take(6)
            .collect();

        if !matches.is_empty() {
            egui::Frame::group(ui.style())
                .fill(ThemeTokens::BG_CARD)
                .stroke(egui::Stroke::new(1.0, ThemeTokens::CYAN_EDGE))
                .inner_margin(egui::Margin::symmetric(8.0, 4.0))
                .show(ui, |ui| {
                    ui.label(
                        egui::RichText::new(format!("Résultats ({}) - Cliquez pour localiser :", matches.len()))
                            .size(11.0)
                            .color(ThemeTokens::TEXT_MUTED),
                    );
                    for m in matches {
                        ui.horizontal(|ui| {
                            let is_sel = selected_model_id.as_deref() == Some(&m.id);
                            let score_str = format!("{:.1} pts", m.score_for_sources(&SourceSelection::default()).unwrap_or(0.0));
                            let size_str = if m.is_cloud { "Cloud API".into() } else { format!("{:.1} Go", m.size_gb) };
                            let col = if is_sel { ThemeTokens::GOLD_5090 } else { ThemeTokens::TEXT_PRIMARY };
                            ui.label(egui::RichText::new(&m.name).strong().size(11.5).color(col));
                            ui.label(egui::RichText::new(format!("({}, {})", size_str, score_str)).size(11.0).color(ThemeTokens::TEXT_MUTED));
                            if ui.button(egui::RichText::new("Localiser").strong().color(ThemeTokens::CYAN_EDGE)).clicked() {
                                *selected_model_id = Some(m.id.clone());
                            }
                        });
                    }
                });
        }
    }
}

pub fn render_slot_model_card(
    ui: &mut egui::Ui,
    slot_num: usize,
    slot_id: &mut Option<String>,
    query: &mut String,
    all_models: &[ModelBenchmark],
    metrics: Option<&ModelHardwareMetrics>,
    rig: &HardwareRigConfig,
    color: egui::Color32,
    _lang: Language,
) {
    let current_model = slot_id.as_ref().and_then(|id| all_models.iter().find(|m| &m.id == id));

    egui::Frame::group(ui.style())
        .fill(ThemeTokens::BG_CARD)
        .stroke(egui::Stroke::new(1.5, color))
        .inner_margin(egui::Margin::symmetric(10.0, 7.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new(format!("[Modèle #{}]", slot_num)).color(color).strong().size(13.0));
                if let Some(m) = current_model {
                    ui.label(egui::RichText::new(&m.name).strong().size(12.5).color(egui::Color32::WHITE));
                } else {
                    ui.label(egui::RichText::new("Aucun modèle sélectionné").italics().size(12.0).color(ThemeTokens::TEXT_MUTED));
                }

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if current_model.is_some() && ui.small_button("Effacer").clicked() {
                        *slot_id = None;
                        query.clear();
                    }
                });
            });

            ui.add_space(3.0);

            // Search input field with auto-completion
            ui.horizontal(|ui| {
                ui.label(egui::RichText::new("Recherche :").size(11.0).color(ThemeTokens::TEXT_MUTED));
                ui.add(
                    egui::TextEdit::singleline(query)
                        .id_source(egui::Id::new("comparator_slot_search").with(slot_num))
                        .hint_text("Taper un nom ou taille...")
                        .desired_width(240.0),
                );
                if !query.is_empty() && ui.small_button("X").clicked() {
                    query.clear();
                }
            });

            // Autocomplete suggestions popup
            if query.trim().len() >= 2 {
                let q = query.trim().to_lowercase();
                let matches: Vec<&ModelBenchmark> = all_models
                    .iter()
                    .filter(|m| m.name.to_lowercase().contains(&q) || m.family.to_lowercase().contains(&q))
                    .take(6)
                    .collect();

                if !matches.is_empty() {
                    egui::Frame::group(ui.style())
                        .fill(ThemeTokens::BG_PANEL)
                        .stroke(egui::Stroke::new(1.0, color))
                        .inner_margin(egui::Margin::symmetric(6.0, 3.0))
                        .show(ui, |ui| {
                            ui.label(egui::RichText::new(format!("Cliquez pour choisir ({}) :", matches.len())).size(10.5).color(ThemeTokens::TEXT_MUTED));
                            for m in matches {
                                ui.horizontal(|ui| {
                                    let score_str = format!("{:.1} pts", m.score_for_sources(&SourceSelection::default()).unwrap_or(0.0));
                                    let size_str = if m.is_cloud { "Cloud API".into() } else { format!("{:.1} Go", m.size_gb) };
                                    if ui.button(egui::RichText::new(&m.name).strong().size(11.0).color(color)).clicked() {
                                        *slot_id = Some(m.id.clone());
                                        query.clear();
                                    }
                                    ui.label(egui::RichText::new(format!("({}, {})", size_str, score_str)).size(10.5).color(ThemeTokens::TEXT_MUTED));
                                });
                            }
                        });
                }
            }

            // Specs metadata badge of selected model
            if let Some(m) = current_model {
                ui.add_space(3.0);
                let size_text = if m.is_cloud { "Cloud API".into() } else { format!("{:.1} Go", m.size_gb) };
                let score = m.score_for_sources(&SourceSelection::default()).unwrap_or(0.0);
                let year = m.release_year_label();
                let text = format!("Empreinte : {}  |  Année : {}  |  Famille : {}  |  Score : {:.1} pts", size_text, year, m.family, score);
                ui.label(egui::RichText::new(text).size(10.5).color(ThemeTokens::TEXT_SECONDARY));

                // Unified Hardware Diagnostic Specs
                if let Some(met) = metrics {
                    ui.add_space(3.0);
                    render_model_hardware_specs_grid(ui, met, rig, color);
                }
            }
        });
}
