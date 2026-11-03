use eframe::egui;
use crate::domain::model::ModelBenchmark;
use crate::domain::specialty::DomainSpecialty;
use crate::presentation::components::top_bar::AppTab;
use crate::presentation::filtering::{FilterState, SpecialtyFilter};
use crate::presentation::i18n::Language;
use crate::presentation::theme::ThemeTokens;

pub fn render_domains_view(
    ui: &mut egui::Ui,
    all_models: &[ModelBenchmark],
    filters: &mut FilterState,
    active_tab: &mut AppTab,
    selected_model_id: &mut Option<String>,
    lang: Language,
) {
    let is_fr = lang == Language::French;

    egui::ScrollArea::vertical().show(ui, |ui| {
        ui.add_space(6.0);

        // --- 1. Header Hero Banner ---
        egui::Frame::group(ui.style())
            .fill(egui::Color32::from_rgb(16, 22, 34))
            .stroke(egui::Stroke::new(1.2, ThemeTokens::CYAN_EDGE))
            .inner_margin(14.0)
            .rounding(8.0)
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new("[*]").size(16.0).color(ThemeTokens::GOLD_5090).strong());
                    ui.heading(
                        egui::RichText::new(if is_fr {
                            "Hub des Modèles Spécialisés & Domaines Métiers"
                        } else {
                            "Specialized Foundation Models & Industry Verticals"
                        })
                        .strong()
                        .color(ThemeTokens::TEXT_PRIMARY)
                        .size(16.5),
                    );
                });

                ui.add_space(4.0);
                ui.label(
                    egui::RichText::new(if is_fr {
                        "Panorama des modèles par verticalité industrielle (ADN, Vision, 3D, Voix, Agents, Code). Détection 100% dynamique."
                    } else {
                        "Overview of foundation models by industry vertical (DNA, Vision, 3D, Voice, Agents, Code). 100% dynamic detection."
                    })
                    .color(ThemeTokens::TEXT_SECONDARY)
                    .size(12.0),
                );
            });

        ui.add_space(12.0);

        // --- 2. Responsive 2-Column Domain Cards ---
        let avail_w = ui.available_width();
        let card_w = if avail_w < 780.0 {
            (avail_w - 24.0).max(240.0)
        } else {
            ((avail_w - 36.0) * 0.5 - 24.0).max(240.0)
        };

        let domains = [
            (
                DomainSpecialty::SoftwareDev,
                if is_fr { "Génie Logiciel & Programmation" } else { "Software Engineering & Deep Coding" },
                if is_fr {
                    "Modèles experts en programmation, résolution autonome de bugs (SWE), refactoring d'architectures et génération de suites de tests."
                } else {
                    "Specialized models for coding, autonomous issue resolution (SWE), architectural refactoring, and test generation."
                },
                "SWE-Bench Verified, HumanEval, BigCodeBench",
            ),
            (
                DomainSpecialty::VisionCreative,
                if is_fr { "Vision, Graphisme & Image" } else { "Vision, Graphics & Image" },
                if is_fr {
                    "Modèles de retouche, inpainting, segmentation et transformation d'images guidés par instructions textuelles en langage naturel."
                } else {
                    "Models specialized in image retouching, inpainting, segmentation, and natural language instruction editing."
                },
                "GenEval, ImageReward, Aesthetic Score",
            ),
            (
                DomainSpecialty::VoiceSpeech,
                if is_fr { "Voix & Synthèse Vocale" } else { "Voice & Speech Synthesis" },
                if is_fr {
                    "Modèles audio et vocaux à très faible latence avec intonations réalistes, transcription bidirectionnelle et conversion vocale."
                } else {
                    "Ultra-low latency audio and speech models with lifelike intonations, bidirectional transcription, and voice conversion."
                },
                "WER (Word Error Rate), MOS Authenticity, Latency",
            ),
            (
                DomainSpecialty::MultiAgent,
                if is_fr { "Agents & Recherche Autonome" } else { "Autonomous Agents & Discovery" },
                if is_fr {
                    "Équipes et essaims d'agents collaboratifs orchestrés pour exécuter des workflows analytiques et scientifiques complexes."
                } else {
                    "Collaborative agent teams orchestrated to solve complex multi-step analytical and scientific workflows."
                },
                "AgentBench, GAIA, Multi-Agent Collaboration Bench",
            ),
            (
                DomainSpecialty::GenomicsBio,
                if is_fr { "ADN & Génomique (Bio-Santé)" } else { "DNA & Genomics (Bio-Health)" },
                if is_fr {
                    "Modèles spécialisés dans le séquençage génomique, l'analyse d'ADN, le repliement de protéines et les interactions moléculaires."
                } else {
                    "Specialized models for genomic sequencing, DNA analysis, protein folding, and molecular biology interactions."
                },
                "Genomic-Bench, ProteinGym, Caduceus DNA-Seq",
            ),
            (
                DomainSpecialty::WorldModel3D,
                if is_fr { "3D, CAO & World Models" } else { "3D, CAD & World Models" },
                if is_fr {
                    "Architectures spatiales et world models capables de générer, simuler et concevoir en 3D des objets physiques et maillages CAO."
                } else {
                    "Spatial architectures and world models generating, simulating, and designing 3D physical objects and CAD meshes."
                },
                "CAD-Rec, Omni3D, Physical Stress Simulator",
            ),
        ];

        if avail_w < 780.0 {
            for (spec, title, desc, bench) in &domains {
                render_domain_card(ui, card_w, *spec, title, desc, bench, all_models, filters, active_tab, selected_model_id, is_fr);
                ui.add_space(8.0);
            }
        } else {
            for chunk in domains.chunks(2) {
                ui.horizontal(|ui| {
                    let (spec0, title0, desc0, bench0) = chunk[0];
                    render_domain_card(ui, card_w, spec0, title0, desc0, bench0, all_models, filters, active_tab, selected_model_id, is_fr);
                    ui.add_space(12.0);
                    if let Some(&(spec1, title1, desc1, bench1)) = chunk.get(1) {
                        render_domain_card(ui, card_w, spec1, title1, desc1, bench1, all_models, filters, active_tab, selected_model_id, is_fr);
                    }
                });
                ui.add_space(10.0);
            }
        }

        ui.add_space(16.0);
    });
}

fn render_domain_card(
    ui: &mut egui::Ui,
    width: f32,
    spec: DomainSpecialty,
    title: &str,
    desc: &str,
    bench_name: &str,
    all_models: &[ModelBenchmark],
    filters: &mut FilterState,
    active_tab: &mut AppTab,
    selected_model_id: &mut Option<String>,
    is_fr: bool,
) {
    let (r, g, b) = spec.badge_color_rgb();
    let accent = egui::Color32::from_rgb(r, g, b);

    // Count matching models in catalog dynamically
    let matching_models: Vec<&ModelBenchmark> = all_models.iter().filter(|m| m.specialty == spec).collect();
    let matching_count = matching_models.len();

    let sample_text = if matching_models.is_empty() {
        if is_fr { "Aucun modèle dans ce snapshot" } else { "No model in current snapshot" }.to_string()
    } else {
        matching_models.iter().take(3).map(|m| m.name.as_str()).collect::<Vec<_>>().join(", ")
    };

    egui::Frame::group(ui.style())
        .fill(egui::Color32::from_rgb(15, 20, 30))
        .stroke(egui::Stroke::new(1.2, accent))
        .inner_margin(12.0)
        .rounding(8.0)
        .show(ui, |ui| {
            ui.set_width(width);
            ui.vertical(|ui| {
                ui.horizontal(|ui| {
                    ui.label(egui::RichText::new(format!("[{}]", spec.badge_text())).color(accent).strong().size(12.0));
                    ui.label(egui::RichText::new(title).strong().color(ThemeTokens::TEXT_PRIMARY).size(13.5));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(
                            egui::RichText::new(format!("({matching_count} {})", if is_fr { "modèles" } else { "models" }))
                                .size(11.0)
                                .color(ThemeTokens::TEXT_MUTED),
                        );
                    });
                });

                ui.add_space(4.0);
                ui.label(egui::RichText::new(desc).color(ThemeTokens::TEXT_SECONDARY).size(11.5));
                ui.add_space(6.0);

                ui.horizontal_wrapped(|ui| {
                    ui.label(egui::RichText::new(if is_fr { "Bancs Clés :" } else { "Key Benchmarks:" }).size(11.0).color(ThemeTokens::GOLD_5090).strong());
                    ui.label(egui::RichText::new(bench_name).size(11.0).color(ThemeTokens::TEXT_MUTED));
                });

                ui.horizontal_wrapped(|ui| {
                    ui.label(egui::RichText::new(if is_fr { "Modèles Détectés :" } else { "Detected Models:" }).size(11.0).color(ThemeTokens::CYAN_EDGE).strong());
                    ui.label(egui::RichText::new(sample_text).size(11.0).color(ThemeTokens::TEXT_MUTED));
                });

                ui.add_space(8.0);
                ui.horizontal(|ui| {
                    let btn_lbl = if is_fr {
                        format!("Explorer {} ({matching_count}) >", spec.badge_text())
                    } else {
                        format!("Explore {} ({matching_count}) >", spec.badge_text())
                    };

                    let btn = egui::Button::new(egui::RichText::new(btn_lbl).strong().color(accent).size(11.5))
                        .fill(egui::Color32::from_rgb(22, 30, 46))
                        .stroke(egui::Stroke::new(1.0, accent))
                        .rounding(5.0);

                    if ui.add(btn).clicked() {
                        filters.specialty = SpecialtyFilter::Specific(spec);
                        filters.group_by_root = false;
                        if let Some(first_match) = matching_models.first() {
                            *selected_model_id = Some(first_match.id.clone());
                        }
                        *active_tab = AppTab::Table;
                    }
                });
            });
        });
}
