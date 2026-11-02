use crate::domain::hardware::HardwarePlatform;
use crate::domain::license::LicenseCategory;
use crate::domain::model::{ModelBenchmark, SortField, SortOrder};
use crate::presentation::filtering::{
    ConfidenceFilter, FilterState, ModalityFilter, ModelScope, RootModelGroup, TableColumnMode,
};

fn cmp_f32(a: f32, b: f32, order: SortOrder) -> std::cmp::Ordering {
    match order {
        SortOrder::Descending => b.partial_cmp(&a).unwrap_or(std::cmp::Ordering::Equal),
        SortOrder::Ascending => a.partial_cmp(&b).unwrap_or(std::cmp::Ordering::Equal),
    }
}

fn cmp_ord<T: Ord>(a: T, b: T, order: SortOrder) -> std::cmp::Ordering {
    match order {
        SortOrder::Descending => b.cmp(&a),
        SortOrder::Ascending => a.cmp(&b),
    }
}

pub struct FilterService;

impl FilterService {
    pub fn compute_filtered_indices(models: &[ModelBenchmark], filters: &FilterState) -> Vec<usize> {
        let mut indices: Vec<usize> = models.iter().enumerate().filter_map(|(idx, m)| {
            if !filters.search_text.is_empty() {
                let q = filters.search_text.to_lowercase();
                let matches_name = m.name.to_lowercase().contains(&q);
                let matches_id = m.id.to_lowercase().contains(&q);
                let matches_fam = m.family.to_lowercase().contains(&q);
                if !matches_name && !matches_id && !matches_fam {
                    return None;
                }
            }

            match filters.scope {
                ModelScope::LocalOnly if m.is_cloud => return None,
                ModelScope::CloudOnly if !m.is_cloud => return None,
                _ => {}
            }

            if let ModalityFilter::Specific(target_mod) = filters.modality {
                if m.modality != target_mod {
                    return None;
                }
            }

            if filters.license != LicenseCategory::All {
                if m.license().category() != filters.license {
                    return None;
                }
            }

            if !m.is_cloud && filters.hardware != HardwarePlatform::All {
                if !filters.hardware.fits_model(m.size_gb, m.vram_rec_gb) {
                    return None;
                }
            }

            if !m.is_cloud && (m.size_gb < filters.min_size_gb || m.size_gb > filters.max_size_gb) {
                return None;
            }

            if filters.min_speed_tps > 0.0 && m.tokens_per_sec.unwrap_or(0.0) < filters.min_speed_tps {
                return None;
            }

            if filters.min_intelligence > 0.0 && m.score_for_sources(&filters.sources).unwrap_or(0.0) < filters.min_intelligence {
                return None;
            }

            if filters.selected_family != "All" && filters.selected_family != "Tous" && m.family != filters.selected_family {
                return None;
            }

            match filters.confidence_level {
                ConfidenceFilter::All => {},
                ConfidenceFilter::MediumPlus => {
                    if let Some(breakdown) = m.score_breakdown(&filters.sources) {
                        if breakdown.confidence == crate::domain::scoring::ScoreConfidence::Low { return None; }
                    } else { return None; }
                }
                ConfidenceFilter::HighOnly => {
                    if let Some(breakdown) = m.score_breakdown(&filters.sources) {
                        if breakdown.confidence != crate::domain::scoring::ScoreConfidence::High { return None; }
                    } else { return None; }
                }
            }
            if filters.column_mode == TableColumnMode::StrictVerified {
                if let Some(breakdown) = m.score_breakdown(&filters.sources) {
                    if breakdown.confidence == crate::domain::scoring::ScoreConfidence::Low { return None; }
                } else { return None; }
            }

            Some(idx)
        }).collect();

        let order = filters.sort_order;

        match filters.sort_field {
            SortField::Intelligence => indices.sort_by(|&a, &b| cmp_f32(models[a].score_for_sources(&filters.sources).unwrap_or(0.0), models[b].score_for_sources(&filters.sources).unwrap_or(0.0), order)),
            SortField::Efficiency => indices.sort_by(|&a, &b| cmp_f32(models[a].efficiency_ratio_with_sources(&filters.sources), models[b].efficiency_ratio_with_sources(&filters.sources), order)),
            SortField::Speed => indices.sort_by(|&a, &b| cmp_f32(models[a].tokens_per_sec.unwrap_or(0.0), models[b].tokens_per_sec.unwrap_or(0.0), order)),
            SortField::Size => indices.sort_by(|&a, &b| cmp_f32(models[a].size_gb, models[b].size_gb, order)),
            SortField::Memory => indices.sort_by(|&a, &b| cmp_f32(models[a].vram_rec_gb, models[b].vram_rec_gb, order)),
            SortField::Downloads => indices.sort_by(|&a, &b| cmp_ord(models[a].downloads, models[b].downloads, order)),
            SortField::Name => indices.sort_by(|&a, &b| cmp_ord(models[a].name.to_lowercase(), models[b].name.to_lowercase(), order)),
            SortField::Price => indices.sort_by(|&a, &b| cmp_f32(models[a].pricing_input_usd.unwrap_or(0.0), models[b].pricing_input_usd.unwrap_or(0.0), order)),
            SortField::Year => indices.sort_by(|&a, &b| cmp_ord(models[a].release_year.unwrap_or(0), models[b].release_year.unwrap_or(0), order)),
            SortField::RealWorld => indices.sort_by(|&a, &b| cmp_f32(
                models[a].economic.as_ref().and_then(|e| e.composite_score()).or(models[a].scores.real_world).unwrap_or(0.0),
                models[b].economic.as_ref().and_then(|e| e.composite_score()).or(models[b].scores.real_world).unwrap_or(0.0),
                order
            )),
            SortField::Coding => indices.sort_by(|&a, &b| cmp_f32(models[a].scores.coding.unwrap_or(0.0), models[b].scores.coding.unwrap_or(0.0), order)),
            SortField::Agentic => indices.sort_by(|&a, &b| cmp_f32(models[a].scores.agentic.unwrap_or(0.0), models[b].scores.agentic.unwrap_or(0.0), order)),
        }

        indices
    }

    pub fn group_into_roots(models: &[ModelBenchmark], indices: &[usize]) -> Vec<RootModelGroup> {
        let mut groups: Vec<RootModelGroup> = Vec::new();
        let mut name_to_group: std::collections::HashMap<String, usize> = std::collections::HashMap::new();

        for &idx in indices {
            let root_name = models[idx].root_model_name();
            if let Some(&group_idx) = name_to_group.get(&root_name) {
                groups[group_idx].variant_indices.push(idx);
            } else {
                let new_idx = groups.len();
                name_to_group.insert(root_name.clone(), new_idx);
                groups.push(RootModelGroup {
                    root_name,
                    family: models[idx].family.clone(),
                    champion_idx: idx,
                    variant_indices: vec![idx],
                });
            }
        }

        groups
    }
}
