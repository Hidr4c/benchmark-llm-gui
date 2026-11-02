use eframe::egui;
use crate::domain::comparator::HardwareComparator;
use crate::domain::hardware::HardwarePlatform;
use crate::domain::hardware_detector::DetectedHardware;
use crate::domain::model::ModelBenchmark;
use crate::domain::simulator::HardwareRigConfig;
use crate::presentation::components::comparator_bars::render_thematic_dual_bars;
use crate::presentation::components::comparator_modes::{apply_detected_to_rig, apply_preset_fast_vs_deep};
use crate::presentation::components::comparator_verdict::render_comparator_verdict;
use crate::presentation::components::model_search_picker::render_slot_model_card;
use crate::presentation::components::top_bar::AppTab;
use crate::presentation::filtering::FilterState;
use crate::presentation::i18n::Language;
use crate::presentation::theme::ThemeTokens;

#[derive(Debug, Clone)]
pub struct ComparatorState {
    pub rig: HardwareRigConfig,
    pub target_platform: HardwarePlatform,
    pub slots: [Option<String>; 2],
    pub search_query_1: String,
    pub search_query_2: String,
}

impl Default for ComparatorState {
    fn default() -> Self {
        Self {
            rig: HardwareRigConfig::default(),
            target_platform: HardwarePlatform::Gpu32Gb,
            slots: [None, None],
            search_query_1: String::new(),
            search_query_2: String::new(),
        }
    }
}

pub fn render_comparator_view(
    ui: &mut egui::Ui,
    state: &mut ComparatorState,
    detected_hw: Option<&DetectedHardware>,
    all_models: &[ModelBenchmark],
    filters: &FilterState,
    active_tab: &mut AppTab,
    lang: Language,
) {
    let t = lang.t();
    if state.slots[0].is_none() && state.slots[1].is_none() && !all_models.is_empty() {
        apply_preset_fast_vs_deep(&mut state.slots, all_models);
    }

    ui.add_space(2.0);

    // 1. Top Header with Back Button & Hardware Target Selector
    render_header_and_hardware(ui, state, active_tab, detected_hw, lang);

    ui.add_space(6.0);

    // 2. Side-by-Side Model Pickers (Model #1 and Model #2)
    ui.columns(2, |cols| {
        cols[0].vertical(|ui| {
            render_slot_model_card(
                ui,
                1,
                &mut state.slots[0],
                &mut state.search_query_1,
                all_models,
                ThemeTokens::CYAN_EDGE,
                lang,
            );
        });
        cols[1].vertical(|ui| {
            render_slot_model_card(
                ui,
                2,
                &mut state.slots[1],
                &mut state.search_query_2,
                all_models,
                ThemeTokens::GOLD_5090,
                lang,
            );
        });
    });

    ui.add_space(6.0);

    // 3. Gather Metrics for Selected Models
    let m1_obj = state.slots[0].as_ref().and_then(|id| all_models.iter().find(|m| &m.id == id));
    let m2_obj = state.slots[1].as_ref().and_then(|id| all_models.iter().find(|m| &m.id == id));

    let m1_metrics = m1_obj.map(|m| HardwareComparator::compute_metrics(m, &state.rig, &filters.sources));
    let m2_metrics = m2_obj.map(|m| HardwareComparator::compute_metrics(m, &state.rig, &filters.sources));

    // 4. Thematic Dual Bars (2 columns per theme)
    let p1 = m1_obj.zip(m1_metrics.as_ref());
    let p2 = m2_obj.zip(m2_metrics.as_ref());

    let scroll_resp = egui::ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            render_thematic_dual_bars(ui, p1, p2, &state.rig);

            ui.add_space(6.0);

            // 5. Machine Fitness & Usage Verdict
            let mut metrics_list = Vec::new();
            if let Some(ref met) = m1_metrics {
                metrics_list.push(met.clone());
            }
            if let Some(ref met) = m2_metrics {
                metrics_list.push(met.clone());
            }

            render_comparator_verdict(ui, &metrics_list, &state.rig);

            // 6. Return Button at the Bottom
            ui.add_space(14.0);
            ui.vertical_centered(|ui| {
                let back_btn = egui::Button::new(
                    egui::RichText::new(format!("<- {}", t.comp.btn_back))
                        .strong()
                        .color(ThemeTokens::CYAN_EDGE)
                        .size(13.0),
                )
                .fill(egui::Color32::from_rgb(18, 28, 44))
                .stroke(egui::Stroke::new(1.2, ThemeTokens::CYAN_EDGE))
                .rounding(8.0)
                .min_size(egui::vec2(280.0, 36.0));

                if ui.add(back_btn).on_hover_text("Revenir a la vue principale du classement des modeles").clicked() {
                    *active_tab = AppTab::Table;
                }
            });
            ui.add_space(16.0);
        });
    ui.ctx().memory_mut(|m| m.data.insert_temp(egui::Id::new("tour_result_comp_bars"), scroll_resp.inner_rect));
}

fn render_header_and_hardware(
    ui: &mut egui::Ui,
    state: &mut ComparatorState,
    active_tab: &mut AppTab,
    detected_hw: Option<&DetectedHardware>,
    lang: Language,
) {
    let t = lang.t();
    let header_resp = ui.horizontal(|ui| {
        let top_back = egui::Button::new(
            egui::RichText::new("<- Retour").strong().color(ThemeTokens::CYAN_EDGE).size(12.0)
        )
        .fill(egui::Color32::from_rgb(18, 28, 44))
        .stroke(egui::Stroke::new(1.0, ThemeTokens::CYAN_EDGE))
        .rounding(6.0);

        if ui.add(top_back).on_hover_text("Revenir au classement des modeles").clicked() {
            *active_tab = AppTab::Table;
        }

        ui.heading(
            egui::RichText::new("Comparateur Materiel Face-a-Face (2 Modeles)")
                .color(ThemeTokens::PRIMARY)
                .size(16.0)
                .strong(),
        );

        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if let Some(hw) = detected_hw {
                let btn = egui::Button::new(
                    egui::RichText::new("Mon PC Detecte").strong().color(ThemeTokens::CYAN_EDGE),
                );
                if ui.add(btn).clicked() {
                    apply_detected_to_rig(&mut state.rig, hw);
                    state.target_platform = hw.matched_platform;
                }
            }

            egui::ComboBox::from_id_salt("comp_gpu_selector")
                .selected_text(
                    egui::RichText::new(state.target_platform.label())
                        .strong()
                        .color(ThemeTokens::GOLD_5090),
                )
                .width(235.0)
                .show_ui(ui, |ui| {
                    ui.label(egui::RichText::new("--- Smartphones & Mobiles ---").color(ThemeTokens::TEXT_MUTED).size(10.5));
                    let mobiles = [
                        HardwarePlatform::SmartphoneMid,
                        HardwarePlatform::SmartphoneHigh,
                        HardwarePlatform::SmartphonePro,
                    ];
                    for hw in mobiles {
                        if ui.selectable_label(state.target_platform == hw, hw.label()).clicked() {
                            state.target_platform = hw;
                            state.rig = HardwareRigConfig::from_platform(hw);
                        }
                    }

                    ui.separator();
                    ui.label(egui::RichText::new("--- Cartes Graphiques PC ---").color(ThemeTokens::TEXT_MUTED).size(10.5));
                    let gpus = [
                        HardwarePlatform::Gpu8Gb,
                        HardwarePlatform::Gpu12Gb,
                        HardwarePlatform::Gpu16Gb,
                        HardwarePlatform::Gpu24Gb,
                        HardwarePlatform::Gpu32Gb,
                        HardwarePlatform::Workstation,
                    ];
                    for hw in gpus {
                        if ui.selectable_label(state.target_platform == hw, hw.label()).clicked() {
                            state.target_platform = hw;
                            state.rig = HardwareRigConfig::from_platform(hw);
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
                        if ui.selectable_label(state.target_platform == hw, hw.label()).clicked() {
                            state.target_platform = hw;
                            state.rig = HardwareRigConfig::from_platform(hw);
                        }
                    }
                });
            ui.label(egui::RichText::new(t.comp.target_rig_label).strong());
        });
    }).response;
    ui.ctx().memory_mut(|m| m.data.insert_temp(egui::Id::new("tour_target_comp_header"), header_resp.rect));
}
