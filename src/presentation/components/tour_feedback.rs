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
                    ui.label(egui::RichText::new("[OK]").strong().color(ThemeTokens::SUCCESS).size(14.0));
                    let success_lbl = match state.current_step {
                        0 => "Périmètre sélectionné avec succès !",
                        1 => "Matériel détecté avec succès !",
                        2 => "Filtre matériel / VRAM appliqué !",
                        3 => "Colonnes et filtres avancés configurés !",
                        4 => "Mode d'affichage basculé !",
                        5 => "Modèle inspecté avec succès !",
                        6 => "Graphe recentré avec succès !",
                        7 => "Comparateur Face-à-Face activé !",
                        _ => "Etape validée avec succès !",
                    };
                    ui.label(egui::RichText::new(success_lbl).color(ThemeTokens::SUCCESS).strong().size(13.0));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button(egui::RichText::new(t.tour_skip_transition).size(11.5).strong().color(ThemeTokens::CYAN_EDGE)).clicked() {
                            state.next(total_steps);
                        }
                        if state.is_paused {
                            if ui.button(egui::RichText::new("> Reprendre").size(11.5).strong().color(ThemeTokens::SUCCESS)).clicked() {
                                state.is_paused = false;
                            }
                            ui.label(egui::RichText::new("(En Pause)").color(ThemeTokens::GOLD_5090).size(11.0));
                        } else {
                            if ui.button(egui::RichText::new("|| Pause").size(11.5).color(ThemeTokens::TEXT_SECONDARY)).clicked() {
                                state.is_paused = true;
                            }
                            ui.label(egui::RichText::new(format!("({:.1}s)", remaining)).color(ThemeTokens::TEXT_SECONDARY).size(11.0));
                        }
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
                        ui.label(egui::RichText::new(format!("Périmètre actif : {scope_name}")).color(ThemeTokens::TEXT_PRIMARY).size(11.5));
                        ui.label(egui::RichText::new(format!("Modèles filtrés : {matching_count} visibles sur {total_count} modèles au total.")).color(ThemeTokens::CYAN_EDGE).size(11.5));
                    }
                    1 => {
                        if let Some(hw) = detected_hardware {
                            let gpu_str = hw.gpu_name.as_deref().unwrap_or("CPU / GPU Intégré");
                            let vram_str = hw.gpu_vram_gb.map(|v| format!("{v:.1} Go VRAM")).unwrap_or_else(|| "Mémoire partagée".into());
                            ui.label(egui::RichText::new(format!("GPU : {gpu_str} ({vram_str})")).color(ThemeTokens::GOLD_5090).strong().size(11.5));
                            ui.label(egui::RichText::new(format!("CPU / RAM : {} | {:.1} Go RAM", hw.cpu_name.trim(), hw.total_ram_gb)).color(ThemeTokens::TEXT_PRIMARY).size(11.5));
                            ui.label(egui::RichText::new(format!("Profil assigné : {} ({matching_count} modèles compatibles)", hw.matched_platform.label())).color(ThemeTokens::SUCCESS).size(11.5));
                        } else {
                            ui.label(egui::RichText::new(format!("Profil détecté : {} ({matching_count} modèles compatibles)", filters.hardware.label())).color(ThemeTokens::SUCCESS).size(11.5));
                        }
                    }
                    2 => {
                        ui.label(egui::RichText::new(format!("Profil matériel sélectionné : {}", filters.hardware.label())).color(ThemeTokens::GOLD_5090).strong().size(11.5));
                        ui.label(egui::RichText::new(format!("Modèles affichés : {matching_count} modèles correspondent exactement à cette enveloppe mémoire.")).color(ThemeTokens::CYAN_EDGE).size(11.5));
                    }
                    3 => {
                        let adv_str = if filters.show_advanced_columns { "Activé" } else { "Masqué" };
                        let raw_str = if filters.show_raw_sources { "Activé" } else { "Masqué" };
                        ui.label(egui::RichText::new(format!("Colonnes : Avancé [{adv_str}] | Sources Brutes [{raw_str}]")).color(ThemeTokens::CYAN_EDGE).strong().size(11.5));
                        let rel_str = match filters.confidence_level {
                            crate::presentation::filtering::ConfidenceFilter::All => "Tous les niveaux",
                            crate::presentation::filtering::ConfidenceFilter::MediumPlus => "Moyenne & + (Consolidé)",
                            crate::presentation::filtering::ConfidenceFilter::HighOnly => "Haute (3+ sources)",
                        };
                        ui.label(egui::RichText::new(format!("Filtre de fiabilité : {rel_str}")).color(ThemeTokens::GOLD_5090).size(11.5));
                    }
                    4 => {
                        let mode_str = if filters.group_by_root {
                            "Groupé par Modèle Racine (1 ligne par famille condensée)"
                        } else {
                            "Variantes dépliées (toutes les quantifications affichées)"
                        };
                        ui.label(egui::RichText::new(format!("Mode d'affichage : {mode_str}")).color(ThemeTokens::CYAN_EDGE).strong().size(11.5));
                        ui.label(egui::RichText::new(format!("Lignes visibles dans le tableau : {matching_count} entrées.")).color(ThemeTokens::TEXT_PRIMARY).size(11.5));
                    }
                    5 => {
                        if let Some(id) = selected_model_id {
                            if let Some(model) = all_models.iter().find(|m| &m.id == id) {
                                ui.label(egui::RichText::new(format!("Modèle : {} ({:.1} Go)", model.name, model.size_gb)).color(ThemeTokens::GOLD_5090).strong().size(11.5));
                                let intel_score = model.intelligence().unwrap_or(0.0);
                                let eff = model.efficiency_ratio();
                                ui.label(egui::RichText::new(format!("Score : {intel_score:.1} pts | Efficacité : {eff:.1} pts/Go")).color(ThemeTokens::TEXT_PRIMARY).size(11.5));
                                ui.label(egui::RichText::new("Panneau inférieur déployé avec métriques détaillées et calcul de coût.").color(ThemeTokens::SUCCESS).size(11.5));
                            }
                        }
                    }
                    6 => {
                        ui.label(egui::RichText::new("Graphe recentré : la frontière de Pareto met en valeur les meilleurs rapports qualité/prix.").color(ThemeTokens::CYAN_EDGE).size(11.5));
                    }
                    7 => {
                        ui.label(egui::RichText::new("Comparaison Face-à-Face active : observez les barres côte-à-côte pour chaque pilier de test.").color(ThemeTokens::GOLD_5090).size(11.5));
                    }
                    _ => {
                        ui.label(egui::RichText::new("Simulation actualisée : l'occupation VRAM et la marge système sont recalculées en temps réel.").color(ThemeTokens::SUCCESS).size(11.5));
                    }
                }
            });
        });
}

pub fn get_callout_label(step: usize, is_result: bool, lang: Language) -> &'static str {
    match lang {
        Language::French => match (step, is_result) {
            (0, false) => "[Action] Choisissez 'Modèles Locaux'",
            (0, true) => "[OK] Modèles locaux affichés dans le tableau !",
            (1, false) => "[Action] Cliquez sur [ Détecter mon PC ]",
            (1, true) => "[OK] CPU, RAM et GPU profilés !",
            (2, false) => "[Action] Sélectionnez votre palier VRAM",
            (2, true) => "[OK] Modèles adaptés à votre mémoire !",
            (3, false) => "[Action] Cochez [Avancé] ou [Sources Brutes]",
            (3, true) => "[OK] Colonnes et filtres avancés configurés !",
            (4, false) => "[Action] Cliquez sur [+] pour regrouper",
            (4, true) => "[OK] Variantes repliées par famille !",
            (5, false) => "[Action] Cliquez sur le nom d'un modèle",
            (5, true) => "[OK] Fiche technique détaillée ouverte !",
            (6, false) => "[Action] Cliquez sur [ Recentrer ]",
            (6, true) => "[OK] Courbe de Pareto des champions !",
            (7, false) => "[Action] Cliquez sur un Duo Recommandé",
            (7, true) => "[OK] Comparateur Face-à-Face sur 5 piliers !",
            (8, false) => "[Action] Changez de GPU dans la liste",
            (8, true) => "[OK] Risque de crash OOM calculé en direct !",
            _ => "Regardez ici",
        },
        Language::English => match (step, is_result) {
            (0, false) => "[Action] Select 'Local Models'",
            (0, true) => "[OK] Local models displayed in table!",
            (1, false) => "[Action] Click [ Auto-Detect PC ]",
            (1, true) => "[OK] Hardware detected: CPU, RAM & GPU!",
            (2, false) => "[Action] Select your VRAM budget tier",
            (2, true) => "[OK] Filtered models fitting your memory!",
            (3, false) => "[Action] Check [Advanced] or [Raw Sources]",
            (3, true) => "[OK] Advanced columns and filters configured!",
            (4, false) => "[Action] Click [+] icon to group variants",
            (4, true) => "[OK] Collapsed variants by family!",
            (5, false) => "[Action] Click on any model name",
            (5, true) => "[OK] Detail panel opened below!",
            (6, false) => "[Action] Click [ Recenter ]",
            (6, true) => "[OK] Pareto efficiency curve framed!",
            (7, false) => "[Action] Click a Recommended Duo",
            (7, true) => "[OK] 2-Model Side-by-Side comparison across 5 pillars!",
            (8, false) => "[Action] Switch GPU tier in dropdown",
            (8, true) => "[OK] Simulator verdict: real-time OOM risk!",
            _ => "Look here",
        },
    }
}
