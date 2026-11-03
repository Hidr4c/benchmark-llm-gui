use eframe::egui;
use crate::domain::license::LicenseCategory;
use crate::domain::modality::ModelModality;
use crate::presentation::filtering::{ConfidenceFilter, FilterState, ModalityFilter};
use crate::presentation::i18n::Language;
use crate::presentation::theme::ThemeTokens;

pub fn render_advanced_panel(
    ui: &mut egui::Ui,
    filters: &mut FilterState,
    on_export: &mut bool,
    lang: Language,
) {
    if !filters.show_advanced {
        return;
    }

    let t = lang.t();
    let adv = lang.adv();
    ui.add_space(3.0);

    egui::Frame::group(ui.style())
        .fill(ThemeTokens::BG_CARD)
        .stroke(egui::Stroke::new(1.0, ThemeTokens::BG_PANEL))
        .inner_margin(egui::Margin::symmetric(12.0, 7.0))
        .rounding(6.0)
        .show(ui, |ui| {
            // Header with title, CSV export and explicit Reset button
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new(adv.panel_title)
                        .strong()
                        .size(11.0)
                        .color(ThemeTokens::PRIMARY),
                );

                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.small_button(egui::RichText::new(adv.btn_reset).color(ThemeTokens::WARNING)).clicked() {
                        filters.reset();
                    }
                    if ui.small_button(egui::RichText::new(adv.btn_export).color(ThemeTokens::CYAN_EDGE).strong()).on_hover_text(adv.tooltip_export).clicked() {
                        *on_export = true;
                    }
                });
            });

            ui.add_space(3.0);
            ui.separator();
            ui.add_space(3.0);

            // Unified controls row
            ui.horizontal_wrapped(|ui| {
                // 1. Licences
                ui.label(egui::RichText::new(adv.lbl_license).size(11.5).color(ThemeTokens::TEXT_MUTED));
                let lic_lbl = match filters.license {
                    LicenseCategory::All => t.license_all,
                    LicenseCategory::PermissiveCommercial => t.license_permissive,
                    LicenseCategory::CommunityCommercial => t.license_community,
                    LicenseCategory::CloudProprietary => t.license_cloud,
                };
                egui::ComboBox::from_id_salt("adv_license_combo")
                    .selected_text(lic_lbl)
                    .width(135.0)
                    .show_ui(ui, |ui| {
                        if ui.selectable_label(filters.license == LicenseCategory::All, t.license_all).clicked() { filters.license = LicenseCategory::All; }
                        if ui.selectable_label(filters.license == LicenseCategory::PermissiveCommercial, t.license_permissive).clicked() { filters.license = LicenseCategory::PermissiveCommercial; }
                        if ui.selectable_label(filters.license == LicenseCategory::CommunityCommercial, t.license_community).clicked() { filters.license = LicenseCategory::CommunityCommercial; }
                        if ui.selectable_label(filters.license == LicenseCategory::CloudProprietary, t.license_cloud).clicked() { filters.license = LicenseCategory::CloudProprietary; }
                    });

                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);

                // 2. Modality
                ui.label(egui::RichText::new(adv.lbl_modality).size(11.5).color(ThemeTokens::TEXT_MUTED));
                let mod_lbl = match filters.modality {
                    ModalityFilter::All => t.mod_all,
                    ModalityFilter::Specific(m) => m.label(),
                };
                egui::ComboBox::from_id_salt("adv_modality_combo")
                    .selected_text(mod_lbl)
                    .width(125.0)
                    .show_ui(ui, |ui| {
                        if ui.selectable_label(filters.modality == ModalityFilter::All, t.mod_all).clicked() { filters.modality = ModalityFilter::All; }
                        for m in ModelModality::all() {
                            if ui.selectable_label(filters.modality == ModalityFilter::Specific(m), m.label()).clicked() { filters.modality = ModalityFilter::Specific(m); }
                        }
                    });


                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);

                // 4. Family / Architecture
                ui.label(egui::RichText::new(adv.lbl_family).size(11.5).color(ThemeTokens::TEXT_MUTED));
                let families = ["All", "Mistral", "Qwen", "Llama", "DeepSeek", "SmolLM", "Gemma", "Phi"];
                egui::ComboBox::from_id_salt("adv_family_combo")
                    .selected_text(&filters.selected_family)
                    .width(105.0)
                    .show_ui(ui, |ui| {
                        for fam in families {
                            if ui.selectable_label(filters.selected_family == fam, fam).clicked() {
                                filters.selected_family = fam.to_string();
                            }
                        }
                    });

                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);

                // 5. Maximum RAM Required Slider
                ui.label(egui::RichText::new(adv.lbl_max_ram).size(11.5).strong().color(ThemeTokens::CYAN_EDGE));
                ui.add(
                    egui::Slider::new(&mut filters.max_size_gb, 0.5..=100.0)
                        .suffix(adv.suffix_ram)
                        .logarithmic(true)
                ).on_hover_text(adv.tooltip_max_ram);

                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);

                // 6. Score Min Slider
                ui.label(egui::RichText::new(adv.lbl_min_score).size(11.5).strong().color(ThemeTokens::GOLD_5090));
                ui.add(
                    egui::Slider::new(&mut filters.min_intelligence, 0.0..=100.0)
                        .suffix(" pts")
                ).on_hover_text(adv.tooltip_min_score);

                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);

                // 7. Score Confidence / Reliability Level
                ui.label(egui::RichText::new("Fiabilité :").size(11.5).color(ThemeTokens::TEXT_MUTED));
                if ui.selectable_label(filters.confidence_level == ConfidenceFilter::All, "Tous").on_hover_text("Tous les scores (y compris source unique)").clicked() {
                    filters.confidence_level = ConfidenceFilter::All;
                }
                if ui.selectable_label(filters.confidence_level == ConfidenceFilter::MediumPlus, "Moyenne & +").on_hover_text("Scores consolidés (au moins 2 sources)").clicked() {
                    filters.confidence_level = ConfidenceFilter::MediumPlus;
                }
                if ui.selectable_label(filters.confidence_level == ConfidenceFilter::HighOnly, "Haute (3+)").on_hover_text("Haute fiabilité uniquement (au moins 3 sources croisées)").clicked() {
                    filters.confidence_level = ConfidenceFilter::HighOnly;
                }

                ui.add_space(6.0);
                ui.separator();
                ui.add_space(6.0);

                // 8. Colonnes & Affichage du tableau
                filters.show_advanced_columns = filters.table_prefs.are_advanced_columns_visible();
                filters.show_raw_sources = filters.table_prefs.are_raw_sources_visible();

                let chk_adv = ui.checkbox(&mut filters.show_advanced_columns, egui::RichText::new("Avancé").size(11.5).strong())
                    .on_hover_text("Afficher les colonnes détaillées : Année, Efficacité, Valeur Réelle / Éco, Popularité");
                if chk_adv.changed() {
                    filters.table_prefs.set_advanced_columns_visible(filters.show_advanced_columns);
                }

                let chk_raw = ui.checkbox(&mut filters.show_raw_sources, egui::RichText::new("Sources Brutes").size(11.5).strong())
                    .on_hover_text("Afficher les scores bruts par benchmark : MMLU-Pro, SWE-bench, Tool-Calling, OpenLLM");
                if chk_raw.changed() {
                    filters.table_prefs.set_raw_sources_visible(filters.show_raw_sources);
                }

                let col_btn = ui.button(egui::RichText::new(t.table_menu.btn_column_chooser).size(11.5))
                    .on_hover_text(t.table_menu.menu_column_chooser);
                if col_btn.clicked() {
                    filters.column_chooser_open = true;
                }

                let cols_rect = chk_adv.rect.union(chk_raw.rect).union(col_btn.rect);
                ui.ctx().memory_mut(|m| m.data.insert_temp(egui::Id::new("tour_target_adv_cols"), cols_rect));
            });
        });
}
