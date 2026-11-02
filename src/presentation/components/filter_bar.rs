use eframe::egui;
use crate::domain::hardware::HardwarePlatform;
use crate::domain::license::LicenseCategory;
use crate::domain::modality::ModelModality;
use crate::presentation::components::filter_bar_advanced::{render_advanced_panel, render_contextual_hardware_advice};
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
        // --- Row 1: Primary Controls (Direct Access to Grouping & Evaluation Mode) ---
        ui.horizontal_wrapped(|ui| {
            // 1. Scope dropdown
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

            // 2. Search input
            ui.add(
                egui::TextEdit::singleline(&mut filters.search_text)
                    .hint_text(t.search_placeholder)
                    .desired_width(150.0)
            ).on_hover_text(t.search_tooltip);

            if !filters.search_text.is_empty() && ui.button("✖").clicked() {
                filters.search_text.clear();
            }

            ui.separator();

            // 3. Hardware Dropdown & Auto-detect (Always visible to maintain rock-solid UI stability)
            let hw_label = if filters.hardware == HardwarePlatform::All { t.hw_all } else { filters.hardware.label() };
            let hw_resp = egui::ComboBox::from_id_salt("hardware_combo")
                .selected_text(hw_label)
                .width(145.0)
                .show_ui(ui, |ui| {
                    let gpus = [HardwarePlatform::All, HardwarePlatform::Gpu8Gb, HardwarePlatform::Gpu12Gb, HardwarePlatform::Gpu16Gb, HardwarePlatform::Gpu24Gb, HardwarePlatform::Gpu32Gb, HardwarePlatform::Workstation];
                    for hw in gpus {
                        let lbl = if hw == HardwarePlatform::All { t.hw_all } else { hw.label() };
                        if ui.selectable_label(filters.hardware == hw, lbl).clicked() {
                            filters.hardware = hw;
                            if filters.scope == ModelScope::CloudOnly { filters.scope = ModelScope::All; }
                        }
                    }
                    ui.separator();
                    let edges = [HardwarePlatform::MicroIoT, HardwarePlatform::RaspberryPi4, HardwarePlatform::RaspberryPi5, HardwarePlatform::JetsonOrinNano, HardwarePlatform::MiniPcCpu];
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
            }

            ui.separator();

            // 4. Advanced Filters Toggle Button
            let adv_text = if filters.show_advanced { t.advanced_less } else { t.advanced_more };
            if ui.selectable_label(filters.show_advanced, adv_text).on_hover_text(t.advanced_tooltip).clicked() {
                filters.show_advanced = !filters.show_advanced;
            }

            if ui.button("↺").on_hover_text(t.reset_tooltip).clicked() {
                filters.reset();
            }

            ui.separator();

            // 5. Counter badge (with tour result anchor)
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

        // --- Row 2: Secondary / Advanced filters (only when expanded) ---
        if filters.show_advanced {
            ui.add_space(3.0);
            ui.horizontal_wrapped(|ui| {
                // License Dropdown
                let lic_lbl = match filters.license {
                    LicenseCategory::All => t.license_all,
                    LicenseCategory::PermissiveCommercial => t.license_permissive,
                    LicenseCategory::CommunityCommercial => t.license_community,
                    LicenseCategory::CloudProprietary => t.license_cloud,
                };
                egui::ComboBox::from_id_salt("license_combo")
                    .selected_text(lic_lbl)
                    .width(140.0)
                    .show_ui(ui, |ui| {
                        if ui.selectable_label(filters.license == LicenseCategory::All, t.license_all).clicked() { filters.license = LicenseCategory::All; }
                        if ui.selectable_label(filters.license == LicenseCategory::PermissiveCommercial, t.license_permissive).clicked() { filters.license = LicenseCategory::PermissiveCommercial; }
                        if ui.selectable_label(filters.license == LicenseCategory::CommunityCommercial, t.license_community).clicked() { filters.license = LicenseCategory::CommunityCommercial; }
                        if ui.selectable_label(filters.license == LicenseCategory::CloudProprietary, t.license_cloud).clicked() { filters.license = LicenseCategory::CloudProprietary; }
                    }).response.on_hover_text(t.license_tooltip);

                ui.separator();

                // Modality Dropdown
                let mod_lbl = match filters.modality {
                    ModalityFilter::All => t.mod_all,
                    ModalityFilter::Specific(m) => m.label(),
                };
                egui::ComboBox::from_id_salt("modality_filter_combo")
                    .selected_text(mod_lbl)
                    .width(130.0)
                    .show_ui(ui, |ui| {
                        if ui.selectable_label(filters.modality == ModalityFilter::All, t.mod_all).clicked() { filters.modality = ModalityFilter::All; }
                        for m in ModelModality::all() {
                            if ui.selectable_label(filters.modality == ModalityFilter::Specific(m), m.label()).clicked() { filters.modality = ModalityFilter::Specific(m); }
                        }
                    }).response.on_hover_text(t.mod_tooltip);

                ui.separator();

                // Benchmark Dropdown
                egui::ComboBox::from_id_salt("benchmark_source_combo")
                    .selected_text(filters.sources.label())
                    .width(150.0)
                    .show_ui(ui, |ui| {
                        if ui.selectable_label(filters.sources.is_all(), t.bench_avg).clicked() {
                            filters.sources.open_llm = true; filters.sources.coding = true; filters.sources.agentic = true; filters.sources.real_world = true;
                        }
                        if ui.selectable_label(filters.sources.is_open_llm_only(), t.bench_general).clicked() {
                            filters.sources.open_llm = true; filters.sources.coding = false; filters.sources.agentic = false; filters.sources.real_world = false;
                        }
                        if ui.selectable_label(filters.sources.is_coding_only(), t.bench_coding).clicked() {
                            filters.sources.open_llm = false; filters.sources.coding = true; filters.sources.agentic = false; filters.sources.real_world = false;
                        }
                        if ui.selectable_label(filters.sources.is_agentic_only(), t.bench_agentic).clicked() {
                            filters.sources.open_llm = false; filters.sources.coding = false; filters.sources.agentic = true; filters.sources.real_world = false;
                        }
                        if ui.selectable_label(filters.sources.is_real_world_only(), t.bench_real_world).clicked() {
                            filters.sources.open_llm = false; filters.sources.coding = false; filters.sources.agentic = false; filters.sources.real_world = true;
                        }
                    }).response.on_hover_text(t.bench_tooltip);

                ui.separator();

                // Priority Dropdown
                let priority_lbl = match filters.priority {
                    UserPriority::Intelligence => t.priority_intel,
                    UserPriority::Efficiency => t.priority_efficiency,
                    UserPriority::Speed => t.priority_speed,
                    UserPriority::Compactness => t.priority_compact,
                    UserPriority::RealWorld => t.priority_real_world,
                };
                egui::ComboBox::from_id_salt("priority_combo")
                    .selected_text(priority_lbl)
                    .width(130.0)
                    .show_ui(ui, |ui| {
                        if ui.selectable_label(filters.priority == UserPriority::Intelligence, t.priority_intel).clicked() { filters.set_priority(UserPriority::Intelligence); }
                        if ui.selectable_label(filters.priority == UserPriority::Efficiency, t.priority_efficiency).clicked() { filters.set_priority(UserPriority::Efficiency); }
                        if ui.selectable_label(filters.priority == UserPriority::Speed, t.priority_speed).clicked() { filters.set_priority(UserPriority::Speed); }
                        if ui.selectable_label(filters.priority == UserPriority::Compactness, t.priority_compact).clicked() { filters.set_priority(UserPriority::Compactness); }
                        if ui.selectable_label(filters.priority == UserPriority::RealWorld, t.priority_real_world).clicked() { filters.set_priority(UserPriority::RealWorld); }
                    }).response.on_hover_text(t.priority_tooltip);
            });

            render_contextual_hardware_advice(ui, filters);
            render_advanced_panel(ui, filters, on_export);
        }
    });
}
