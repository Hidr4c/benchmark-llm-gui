use eframe::egui;
use crate::domain::hardware::HardwarePlatform;
use crate::presentation::components::filter_bar_advanced::render_advanced_panel;
pub use crate::presentation::filtering::{FilterState, ModalityFilter, ModelScope, SizeCategory, TableColumnMode, UserPriority};
use crate::presentation::i18n::Language;
use crate::presentation::theme::ThemeTokens;

pub fn render_filter_bar(
    ui: &mut egui::Ui,
    filters: &mut FilterState,
    total_count: usize,
    matching_count: usize,
    on_auto_detect: &mut bool,
    on_export: &mut bool,
    lang: Language,
) {
    let t = lang.t();
    ui.vertical(|ui| {
        // --- Row 1: Central Core Actions (Exact Order Requested) ---
        ui.horizontal_wrapped(|ui| {
            // 1. Choix du type de LLM (Périmètre : Local vs Cloud vs Tous)
            let scope_label = match filters.scope {
                ModelScope::LocalOnly => t.scope_local,
                ModelScope::CloudOnly => t.scope_cloud,
                ModelScope::All => t.scope_all,
            };
            let scope_resp = egui::ComboBox::from_id_salt("scope_combo")
                .selected_text(scope_label)
                .width(135.0)
                .show_ui(ui, |ui| {
                    if ui.selectable_label(filters.scope == ModelScope::All, t.scope_all).clicked() { filters.scope = ModelScope::All; }
                    if ui.selectable_label(filters.scope == ModelScope::LocalOnly, t.scope_local).clicked() { filters.scope = ModelScope::LocalOnly; }
                    if ui.selectable_label(filters.scope == ModelScope::CloudOnly, t.scope_cloud).clicked() { filters.scope = ModelScope::CloudOnly; }
                }).response;
            ui.ctx().memory_mut(|m| m.data.insert_temp(egui::Id::new("tour_target_scope"), scope_resp.rect));
            scope_resp.on_hover_text(t.scope_tooltip);

            ui.separator();

            // 2. Choix des Benchmarks (Sources croisées / Agentic / Code / etc.)
            egui::ComboBox::from_id_salt("benchmark_source_combo")
                .selected_text(filters.sources.label())
                .width(160.0)
                .show_ui(ui, |ui| {
                    if ui.selectable_label(filters.sources.is_all(), t.bench_avg).clicked() {
                        filters.sources.open_llm = true; filters.sources.coding = true; filters.sources.agentic = true; filters.sources.real_world = true;
                    }
                    if ui.selectable_label(filters.sources.is_agentic_only(), t.bench_agentic).clicked() {
                        filters.sources.open_llm = false; filters.sources.coding = false; filters.sources.agentic = true; filters.sources.real_world = false;
                    }
                    if ui.selectable_label(filters.sources.is_coding_only(), t.bench_coding).clicked() {
                        filters.sources.open_llm = false; filters.sources.coding = true; filters.sources.agentic = false; filters.sources.real_world = false;
                    }
                    if ui.selectable_label(filters.sources.is_real_world_only(), t.bench_real_world).clicked() {
                        filters.sources.open_llm = false; filters.sources.coding = false; filters.sources.agentic = false; filters.sources.real_world = true;
                    }
                    if ui.selectable_label(filters.sources.is_open_llm_only(), t.bench_general).clicked() {
                        filters.sources.open_llm = true; filters.sources.coding = false; filters.sources.agentic = false; filters.sources.real_world = false;
                    }
                }).response.on_hover_text(t.bench_tooltip);

            ui.separator();

            // 3. Choix des Critères / Tri (Plus Intelligent, Plus Rapide, etc.)
            let priority_lbl = match filters.priority {
                UserPriority::Intelligence => t.priority_intel,
                UserPriority::Efficiency => t.priority_efficiency,
                UserPriority::Speed => t.priority_speed,
                UserPriority::Compactness => t.priority_compact,
                UserPriority::RealWorld => t.priority_real_world,
            };
            egui::ComboBox::from_id_salt("priority_combo")
                .selected_text(priority_lbl)
                .width(145.0)
                .show_ui(ui, |ui| {
                    if ui.selectable_label(filters.priority == UserPriority::Intelligence, t.priority_intel).clicked() { filters.set_priority(UserPriority::Intelligence); }
                    if ui.selectable_label(filters.priority == UserPriority::Speed, t.priority_speed).clicked() { filters.set_priority(UserPriority::Speed); }
                    if ui.selectable_label(filters.priority == UserPriority::Efficiency, t.priority_efficiency).clicked() { filters.set_priority(UserPriority::Efficiency); }
                    if ui.selectable_label(filters.priority == UserPriority::RealWorld, t.priority_real_world).clicked() { filters.set_priority(UserPriority::RealWorld); }
                    if ui.selectable_label(filters.priority == UserPriority::Compactness, t.priority_compact).clicked() { filters.set_priority(UserPriority::Compactness); }
                }).response.on_hover_text(t.priority_tooltip);

            ui.separator();

            // 4. Champ de Recherche de Modèle
            ui.add(
                egui::TextEdit::singleline(&mut filters.search_text)
                    .hint_text(t.search_placeholder)
                    .desired_width(140.0)
            ).on_hover_text(t.search_tooltip);

            if !filters.search_text.is_empty() && ui.button("X").clicked() {
                filters.search_text.clear();
            }

            ui.separator();

            // 5. Choix du Matériel & Détecter le PC
            let hw_label = if filters.hardware == HardwarePlatform::All { t.hw_all } else { filters.hardware.label() };
            let hw_resp = egui::ComboBox::from_id_salt("hardware_combo")
                .selected_text(hw_label)
                .width(155.0)
                .show_ui(ui, |ui| {
                    ui.label(egui::RichText::new("--- Smartphones & Mobiles ---").color(ThemeTokens::TEXT_MUTED).size(10.5));
                    let mobiles = [
                        HardwarePlatform::SmartphoneMid,
                        HardwarePlatform::SmartphoneHigh,
                        HardwarePlatform::SmartphonePro,
                    ];
                    for hw in mobiles {
                        if ui.selectable_label(filters.hardware == hw, hw.label()).clicked() {
                            filters.hardware = hw;
                            if filters.scope == ModelScope::CloudOnly { filters.scope = ModelScope::All; }
                        }
                    }

                    ui.separator();
                    ui.label(egui::RichText::new("--- Cartes Graphiques PC ---").color(ThemeTokens::TEXT_MUTED).size(10.5));
                    let gpus = [
                        HardwarePlatform::All,
                        HardwarePlatform::Gpu8Gb,
                        HardwarePlatform::Gpu12Gb,
                        HardwarePlatform::Gpu16Gb,
                        HardwarePlatform::Gpu24Gb,
                        HardwarePlatform::Gpu32Gb,
                        HardwarePlatform::Workstation,
                    ];
                    for hw in gpus {
                        let lbl = if hw == HardwarePlatform::All { t.hw_all } else { hw.label() };
                        if ui.selectable_label(filters.hardware == hw, lbl).clicked() {
                            filters.hardware = hw;
                            if filters.scope == ModelScope::CloudOnly { filters.scope = ModelScope::All; }
                        }
                    }

                    ui.separator();
                    ui.label(egui::RichText::new("--- Systemes Embarques & IoT ---").color(ThemeTokens::TEXT_MUTED).size(10.5));
                    let edges = [
                        HardwarePlatform::RaspberryPi5,
                        HardwarePlatform::JetsonOrinNano,
                        HardwarePlatform::RaspberryPi4,
                        HardwarePlatform::MiniPcCpu,
                        HardwarePlatform::MicroIoT,
                    ];
                    for hw in edges {
                        if ui.selectable_label(filters.hardware == hw, hw.label()).clicked() {
                            filters.hardware = hw;
                            if filters.scope == ModelScope::CloudOnly { filters.scope = ModelScope::All; }
                        }
                    }
                }).response;
            ui.ctx().memory_mut(|m| m.data.insert_temp(egui::Id::new("tour_target_hardware"), hw_resp.rect));
            hw_resp.on_hover_text(t.hw_tooltip);

            let detect_btn = egui::Button::new(
                egui::RichText::new(t.btn_auto_detect).color(ThemeTokens::CYAN_EDGE).strong()
            );
            let detect_resp = ui.add(detect_btn).on_hover_text(t.tooltip_auto_detect);
            ui.ctx().memory_mut(|m| m.data.insert_temp(egui::Id::new("tour_target_detect"), detect_resp.rect));
            if detect_resp.clicked() {
                *on_auto_detect = true;
                if filters.scope == ModelScope::CloudOnly { filters.scope = ModelScope::All; }
                ui.ctx().memory_mut(|m| m.data.insert_temp(egui::Id::new("detect_pc_click_time"), ui.input(|i| i.time)));
            }
            let last_detect = ui.ctx().memory_mut(|m| m.data.get_temp::<f64>(egui::Id::new("detect_pc_click_time"))).unwrap_or(0.0);
            if ui.input(|i| i.time) - last_detect < 3.0 && filters.hardware != HardwarePlatform::All {
                ui.label(egui::RichText::new(format!("[✓ {}]", filters.hardware.label())).color(ThemeTokens::SUCCESS).strong().size(11.0));
            }

            ui.separator();

            // 6. Bouton Filtres Avancés
            let adv_text = if filters.show_advanced { "Masquer Filtres ^" } else { "Filtres Avances v" };
            let adv_resp = ui.button(egui::RichText::new(adv_text).color(if filters.show_advanced { ThemeTokens::CYAN_EDGE } else { ThemeTokens::TEXT_PRIMARY }));
            ui.ctx().memory_mut(|m| m.data.insert_temp(egui::Id::new("tour_target_adv_filters"), adv_resp.rect));
            if adv_resp.clicked() {
                filters.show_advanced = !filters.show_advanced;
            }

            ui.separator();

            // 7. Checkboxes Colonnes du Tableau
            let chk_adv = ui.checkbox(&mut filters.show_advanced_columns, egui::RichText::new("Avancé").size(11.5).strong())
                .on_hover_text("Afficher les colonnes détaillées : Année, Efficacité, Valeur Réelle / Éco, Popularité");
            let chk_raw = ui.checkbox(&mut filters.show_raw_sources, egui::RichText::new("Sources Brutes").size(11.5).strong())
                .on_hover_text("Afficher les scores bruts par benchmark : MMLU-Pro, SWE-bench, Tool-Calling, OpenLLM");
            let cols_rect = chk_adv.rect.union(chk_raw.rect);
            ui.ctx().memory_mut(|m| m.data.insert_temp(egui::Id::new("tour_target_adv_cols"), cols_rect));

            ui.separator();

            // 8. Compteur visible
            let counter_resp = ui.label(
                egui::RichText::new(format!("{matching_count} / {total_count}"))
                    .color(if matching_count > 0 { ThemeTokens::SUCCESS } else { ThemeTokens::WARNING })
                    .strong()
            );
            ui.ctx().memory_mut(|m| {
                m.data.insert_temp(egui::Id::new("tour_result_scope"), counter_resp.rect);
                m.data.insert_temp(egui::Id::new("tour_result_hardware"), counter_resp.rect);
            });
        });

        // --- Row 2: Unified Ergonomic Advanced Filters Card ---
        render_advanced_panel(ui, filters, on_export, lang);
    });
}
