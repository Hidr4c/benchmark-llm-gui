use eframe::egui;
use crate::domain::hardware_detector::DetectedHardware;
use crate::domain::model::ModelBenchmark;
use crate::presentation::components::tour_state::TourState;
use crate::presentation::filtering::FilterState;
use crate::presentation::i18n::Language;
use crate::presentation::theme::ThemeTokens;

pub fn render_transition_feedback(
    ui: &mut egui::Ui,
    state: &mut TourState,
    filters: &FilterState,
    selected_model_id: Option<&str>,
    detected_hardware: Option<&DetectedHardware>,
    all_models: &[ModelBenchmark],
    matching_count: usize,
    total_count: usize,
    lang: Language,
    total_steps: usize,
) {
    let t = lang.t();
    let remaining = state.transition_timer.unwrap_or(0.0);

    egui::Frame::none()
        .fill(egui::Color32::from_rgb(12, 32, 22))
        .stroke(egui::Stroke::new(1.5, ThemeTokens::SUCCESS))
        .inner_margin(10.0)
        .rounding(8.0)
        .show(ui, |ui| {
            ui.vertical(|ui| {
                // Top line: Success title, remaining timer, and immediate Skip button
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("✓").strong().color(ThemeTokens::SUCCESS).size(14.0));
                    let success_lbl = match state.current_step {
                        0 => "Périmètre sélectionné avec succès !",
                        1 => "Matériel détecté avec succès !",
                        2 => "Filtre matériel / VRAM appliqué !",
                        3 => "Mode d'affichage basculé !",
                        4 => "Modèle inspecté avec succès !",
                        _ => "Étape validée !",
                    };
                    ui.label(egui::RichText::new(success_lbl).color(ThemeTokens::SUCCESS).strong().size(13.0));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(egui::RichText::new(t.tour_skip_transition).size(11.5).strong().color(ThemeTokens::CYAN_EDGE)).clicked() {
                            state.next(total_steps);
                        }
                        ui.label(egui::RichText::new(format!("({:.1}s)", remaining)).color(ThemeTokens::TEXT_SECONDARY).size(11.0));
                    });
                });

                ui.add_space(4.0);
                ui.separator();
                ui.add_space(4.0);

                // Concrete details of what changed in the application
                match state.current_step {
                    0 => {
                        let scope_name = match filters.scope {
                            crate::presentation::filtering::ModelScope::LocalOnly => t.scope_local,
                            crate::presentation::filtering::ModelScope::CloudOnly => t.scope_cloud,
                            crate::presentation::filtering::ModelScope::All => t.scope_all,
                        };
                        ui.label(
                            egui::RichText::new(format!("• Périmètre actif : {scope_name}"))
                                .color(ThemeTokens::TEXT_PRIMARY)
                                .size(11.5),
                        );
                        ui.label(
                            egui::RichText::new(format!("• Modèles filtrés : {matching_count} visibles sur {total_count} modèles au total."))
                                .color(ThemeTokens::CYAN_EDGE)
                                .size(11.5),
                        );
                    }
                    1 => {
                        if let Some(hw) = detected_hardware {
                            let gpu_str = hw.gpu_name.as_deref().unwrap_or("CPU / GPU Intégré");
                            let vram_str = hw.gpu_vram_gb.map(|v| format!("{v:.1} Go VRAM")).unwrap_or_else(|| "Mémoire partagée".into());
                            ui.label(
                                egui::RichText::new(format!("• GPU : {gpu_str} ({vram_str})"))
                                    .color(ThemeTokens::GOLD_5090)
                                    .strong()
                                    .size(11.5),
                            );
                            ui.label(
                                egui::RichText::new(format!("• CPU / RAM : {} | {:.1} Go RAM", hw.cpu_name.trim(), hw.total_ram_gb))
                                    .color(ThemeTokens::TEXT_PRIMARY)
                                    .size(11.5),
                            );
                            ui.label(
                                egui::RichText::new(format!("• Profil assigné : {} ({matching_count} modèles compatibles)", hw.matched_platform.label()))
                                    .color(ThemeTokens::SUCCESS)
                                    .size(11.5),
                            );
                        } else {
                            ui.label(
                                egui::RichText::new(format!("• Profil détecté : {} ({matching_count} modèles compatibles)", filters.hardware.label()))
                                    .color(ThemeTokens::SUCCESS)
                                    .size(11.5),
                            );
                        }
                    }
                    2 => {
                        ui.label(
                            egui::RichText::new(format!("• Profil matériel sélectionné : {}", filters.hardware.label()))
                                .color(ThemeTokens::GOLD_5090)
                                .strong()
                                .size(11.5),
                        );
                        ui.label(
                            egui::RichText::new(format!("• Modèles affichés : {matching_count} modèles correspondent exactement à cette enveloppe mémoire."))
                                .color(ThemeTokens::CYAN_EDGE)
                                .size(11.5),
                        );
                    }
                    3 => {
                        let mode_str = if filters.group_by_root {
                            "Groupé par Modèle Racine (1 ligne par famille condensée)"
                        } else {
                            "Variantes dépliées (toutes les quantifications affichées)"
                        };
                        ui.label(
                            egui::RichText::new(format!("• Mode d'affichage : {mode_str}"))
                                .color(ThemeTokens::CYAN_EDGE)
                                .strong()
                                .size(11.5),
                        );
                        ui.label(
                            egui::RichText::new(format!("• Lignes visibles dans le tableau : {matching_count} entrées."))
                                .color(ThemeTokens::TEXT_PRIMARY)
                                .size(11.5),
                        );
                    }
                    4 => {
                        if let Some(id) = selected_model_id {
                            if let Some(model) = all_models.iter().find(|m| &m.id == id) {
                                ui.label(
                                    egui::RichText::new(format!("• Modèle : {} ({:.1} Go)", model.name, model.size_gb))
                                        .color(ThemeTokens::GOLD_5090)
                                        .strong()
                                        .size(11.5),
                                );
                                let intel_score = model.intelligence().unwrap_or(0.0);
                                let eff = model.efficiency_ratio();
                                ui.label(
                                    egui::RichText::new(format!("• Score : {intel_score:.1} pts | Efficacité : {eff:.1} pts/Go"))
                                        .color(ThemeTokens::TEXT_PRIMARY)
                                        .size(11.5),
                                );
                                ui.label(
                                    egui::RichText::new("• Panneau inférieur déployé avec métriques détaillées et calcul de coût.")
                                        .color(ThemeTokens::SUCCESS)
                                        .size(11.5),
                                );
                            }
                        }
                    }
                    _ => {
                        ui.label(
                            egui::RichText::new("• Navigation effectuée avec succès.")
                                .color(ThemeTokens::TEXT_PRIMARY)
                                .size(11.5),
                        );
                    }
                }
            });
        });
}
